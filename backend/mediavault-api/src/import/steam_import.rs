//! Steam Web API連携・変換・重複チェック・DB登録オーケストレーション
//!
//! TASK-0031: Steamライブラリインポート実装
//!
//! 【機能概要】: `POST /import/steam` のusecase層。`steam_id`検証、Steam Web API
//! （`SteamClient::get_owned_games`）呼び出し、`SteamGameEntry → CreateItemRequest`変換、
//! 重複チェック・DB登録（`item_repository::create_item_with_source`）を行い、`ImportSummary`を構築する。
//! 【実装方針】: ハンドラ（`handlers/import_steam.rs`）→ usecase（本ファイル）→
//! リポジトリ/外部APIクライアントの3層構成（note.md「アーキテクチャパターン」）に従う。
//! APIキー取得は`ApiCredentialLookup`相当のDIで外部化し、テスト時はDB非依存にする
//! （TASK-0023 `ExternalSearchService::with_fixed_credentials`パターンを踏襲）。
//! 【テスト対応】: TC-017-01, TC-017-N02, TC-017-N03, TC-017-02, TC-017-E01, TC-017-E02,
//! TC-017-E03, TC-017-E05, TC-017-B01, TC-017-B02, TC-017-B05
//! 🔵🟡 信頼性レベル: steam-import-requirements.md・steam-import-testcases.mdに基づく
//!
//! 【Red フェーズ注記】: 本ファイルの公開関数・型はすべて未実装（コンパイルエラーまたは`todo!()`）。
//! Greenフェーズで実装する。

use std::sync::Arc;

use api_client_lib::auth::AuthStrategy;
use api_client_lib::clients::steam::SteamClient;
use api_client_lib::clients::steam::requests::GetOwnedGamesRequest;
use sqlx::PgPool;

use crate::models::api_credential::{ApiCredential, ApiProvider};
use crate::models::import::{ImportFailure, ImportSummary};
use crate::models::item::{CreateItemRequest, ItemSource, MediaType};
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::api_credential_repository;

/// SteamID64形式（17桁の数値文字列）を検証する。
///
/// 【機能概要】: `steam_id`が17桁の数値文字列であることを検証する。
/// 簡易検証: `steam_id.len() == 17 && steam_id.chars().all(|c| c.is_numeric())`
/// 検証通過後は`u64`へ変換する（17桁全域は`u64`の範囲内のためオーバーフローしない）。
/// 🔵 信頼性レベル: note.md「SteamID64 形式検証」・TC-017-E03・TC-017-B01・TC-017-B05に直接対応
pub fn validate_steam_id(steam_id: &str) -> Result<u64, ApiError> {
    // 【桁数・数値検証】: 17桁かつ全文字が数値であることを確認する（TC-017-B01/E03） 🔵
    if steam_id.len() != 17 || !steam_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "steam_idはSteamID64形式（17桁の数値文字列）である必要があります",
        ));
    }

    // 【u64変換】: 桁数検証を通過した文字列は必ずu64へ変換できる（TC-017-B05） 🟡
    steam_id.parse::<u64>().map_err(|_| {
        ApiError::new(
            ApiErrorCode::ValidationError,
            "steam_idはSteamID64形式（17桁の数値文字列）である必要があります",
        )
    })
}

/// Steam APIキー解決のDIポイント（テスト用差し替え可能）
///
/// 【設計判断】: TASK-0023 `ApiCredentialLookup`パターンを踏襲し、本番経路は実PgPoolへ
/// `find_by_provider`を発行、テスト経路は固定の`Option<ApiCredential>`を即時返す。
/// 🟡 信頼性レベル: TASK-0023設計判断からの踏襲（妥当な推測）
#[derive(Clone)]
pub enum SteamCredentialLookup {
    /// 本番経路: 実際のPgPoolへ`find_by_provider`を発行する
    Pool(PgPool),
    /// テスト経路: 固定の`Option<ApiCredential>`を即時返す（DBアクセスなし）
    Fixed(Arc<dyn Fn() -> Option<ApiCredential> + Send + Sync>),
}

impl SteamCredentialLookup {
    /// Steam APIキー（`ApiProvider::Steam`）を解決する。
    ///
    /// 【機能概要】: 本番経路（`Pool`）は実DBへ`find_by_provider`を発行しapi_keyを取り出す。
    /// テスト経路（`Fixed`）は固定クロージャを即時呼び出す（DBアクセスなし）。
    /// 🟡 信頼性レベル: TASK-0023 `ApiCredentialLookup::find_by_provider`パターンの踏襲
    async fn find(&self) -> Result<Option<String>, ApiError> {
        match self {
            SteamCredentialLookup::Pool(pool) => {
                let credential =
                    api_credential_repository::find_by_provider(pool, ApiProvider::Steam).await?;
                Ok(credential.map(|c| c.api_key))
            }
            SteamCredentialLookup::Fixed(resolver) => Ok(resolver().map(|c| c.api_key)),
        }
    }
}

/// Steam Web APIキーを用いて`SteamClient`を構築する。
///
/// 【ヘルパー関数】: `import_steam_library`から「クライアント構築」の責務のみを切り出したもの。
/// 【再利用性】: テスト用ベースURL差し替え（wiremock注入）と本番経路（公式URL固定）の分岐を
/// この1関数に閉じ込めることで、呼び出し側（`import_steam_library`）は分岐を意識せずに済む。
/// 【単一責任】: 「APIキー文字列＋任意のベースURLから疎通可能なSteamClientを作る」ことのみを担当する。
/// 🟡 信頼性レベル: ExternalSearchServiceのテスト用ベースURL差し替えパターン（TASK-0023）の踏襲
fn build_steam_client(
    api_key: String,
    steam_api_base_url: Option<&str>,
) -> Result<SteamClient, ApiError> {
    // 【ベースURL分岐】: テスト時（Some）はwiremockのURLを認証用・データ取得用の両方に使う。
    // 本番（None）はSteamClient::new()が内部で公式URLを設定する 🟡
    match steam_api_base_url {
        Some(base_url) => SteamClient::new_with_base_urls(
            AuthStrategy::ApiKey(api_key),
            base_url.to_string(),
            base_url.to_string(),
        )
        .map_err(map_steam_api_error),
        None => SteamClient::new(AuthStrategy::ApiKey(api_key)).map_err(map_steam_api_error),
    }
}

/// Steam Web APIから対象ユーザーの所持ゲーム一覧を取得する。
///
/// 【ヘルパー関数】: `import_steam_library`から「APIキー解決→クライアント構築→get_owned_games呼び出し」の
/// 責務のみを切り出したもの。エラーはすべて`map_steam_api_error`を通して`ApiError`へ統一する。
/// 【単一責任】: Steam Web API呼び出しの組み立てと実行のみを担当し、DB登録やループ処理には関与しない。
/// 🔵🟡 信頼性レベル: 要件定義書2章データフロー（🔵）・クライアント構築分岐は🟡（TASK-0023踏襲）
async fn fetch_owned_games(
    credentials: &SteamCredentialLookup,
    steam_api_base_url: Option<&str>,
    steam_id_u64: u64,
) -> Result<Vec<api_client_lib::clients::steam::models::SteamGameEntry>, ApiError> {
    // 【APIキー取得】: SteamCredentialLookup経由でAPIキーを解決する。未設定なら401で早期return（TC-017-E01-A） 🔵
    let credential = credentials.find().await?;
    let api_key = credential.ok_or_else(|| {
        ApiError::new(
            ApiErrorCode::SteamApiKeyInvalid,
            "Steam APIキーが未設定です",
        )
    })?;

    // 【クライアント構築】: テスト時はベースURL差し替え、本番経路は公式URL固定を使う 🟡
    let client = build_steam_client(api_key, steam_api_base_url)?;

    // 【Steam API呼び出し】: GetOwnedGamesを実行し、所持ゲーム一覧を取得する 🔵
    let request = GetOwnedGamesRequest {
        steam_id: steam_id_u64,
        include_appinfo: true,
        include_played_free_games: true,
    };
    let response = client
        .get_owned_games(request)
        .await
        .map_err(map_steam_api_error)?;

    Ok(response.model.games)
}

/// 1件の`SteamGameEntry`について重複チェック・変換・DB登録を行い、`ImportSummary`へ結果を反映する。
///
/// 【ヘルパー関数】: `import_steam_library`のループ本体から「1件分の処理」を切り出したもの。
/// 【単一責任】: 重複スキップ判定（確定方針#1）・`CreateItemRequest`への変換・DB登録・
/// `ImportSummary`への結果記録（success/failure）のみを担当する。Steam API呼び出しには関与しない。
/// 【保守性】: ループ側（呼び出し元）は「何番目の要素か（index）」の追跡のみに専念でき、
/// 1件分の登録ロジックを変更する際も本関数のみを修正すればよい。
/// 🟡 信頼性レベル: 要件定義書2章データフロー・確定方針#1/#3からの責務分割（妥当な推測）
async fn register_single_game(
    pool: &PgPool,
    summary: &mut ImportSummary,
    index: usize,
    game: api_client_lib::clients::steam::models::SteamGameEntry,
) -> Result<(), ApiError> {
    let external_id = game.appid.to_string();

    // 【重複チェック】: 既存(media_type=Game, source=Api, external_id=appid)があればスキップ（集計外、確定方針#1） 🟡
    let duplicate = crate::repositories::item_repository::find_existing_import(
        pool,
        MediaType::Game,
        &external_id,
    )
    .await?;
    if duplicate {
        return Ok(());
    }

    // 【変換・登録】: SteamGameEntryをCreateItemRequestへ変換しDBへ登録する（1件ごと独立処理、EDGE-002） 🔵
    let create_request =
        steam_game_entry_to_create_item_request(game.appid, game.name, game.playtime_forever);

    match crate::repositories::item_repository::create_item_with_source(
        pool,
        create_request,
        ItemSource::Api,
        Some(external_id),
    )
    .await
    {
        Ok(_) => summary.record_success(),
        Err(err) => {
            // 【DB起因失敗の記録】: 配列インデックス（0始まり）をrow_number相当として流用する（確定方針#3） 🟡
            tracing::error!("steam import db registration error: {err:?}");
            summary.record_failure(ImportFailure::new(index as u32, "db error"));
        }
    }

    Ok(())
}

/// Steamインポートusecase本体。
///
/// 【機能概要】: `steam_id`検証 → APIキー取得 → `get_owned_games`呼び出し →
/// 各`SteamGameEntry`変換・重複チェック・登録 → `ImportSummary`構築、を行う。
/// 【改善内容】: Greenフェーズでは検証・APIキー取得・クライアント構築・ループ処理が1関数に
/// 集約されていたため、責務ごとに`fetch_owned_games`（API呼び出し）・`register_single_game`
/// （1件分の登録処理）・`build_steam_client`（クライアント構築）へ分割した。本関数は
/// 「全体の処理順序を組み立てるオーケストレーション」のみに専念する構成へ整理した。
/// 【実装方針】:
///   1. `validate_steam_id`で早期検証（外部API呼び出し前、TC-017-E03）
///   2. `fetch_owned_games`でAPIキー取得・クライアント構築・`get_owned_games`呼び出しを行う
///      （APIキー未設定は401 STEAM_API_KEY_INVALID、TC-017-E01-A）
///   3. 取得した各`SteamGameEntry`を`register_single_game`で1件ずつ処理する
///      （重複はスキップ＝集計外、登録成功/失敗は`ImportSummary`へ反映、EDGE-002）
///
/// 【保守性】: 各責務が独立した関数になったことで、それぞれを単体で読み・テストしやすくなった。
///
/// 🔵 信頼性レベル: 要件定義書2章データフロー・TASK-0031.md実装詳細に直接対応（責務分割自体は🟡 妥当な推測）
pub async fn import_steam_library(
    pool: &PgPool,
    credentials: &SteamCredentialLookup,
    steam_api_base_url: Option<&str>,
    steam_id: &str,
) -> Result<ImportSummary, ApiError> {
    // 【steam_id早期検証】: 外部API呼び出し前にSteamID64形式を検証する（TC-017-E03） 🟡
    let steam_id_u64 = validate_steam_id(steam_id)?;

    // 【所持ゲーム一覧取得】: APIキー解決・クライアント構築・Steam API呼び出しを委譲する 🔵
    let games = fetch_owned_games(credentials, steam_api_base_url, steam_id_u64).await?;

    // 【結果集計初期化】: 空配列（プロフィール非公開等）の場合もこのまま空サマリで返る（TC-017-02） 🟡
    let mut summary = ImportSummary::empty();

    for (index, game) in games.into_iter().enumerate() {
        register_single_game(pool, &mut summary, index, game).await?;
    }

    Ok(summary)
}

/// `api_client_lib::ApiError`を`ApiError`へマッピングする。
///
/// 【機能概要】: Steamクライアントが返す各種エラーを、認証系（Http 401/403, Auth）は
/// `SteamApiKeyInvalid`（401）へ、それ以外（Timeout/Network/RateLimit/Parse/Http(その他)）は
/// `ExternalApiTimeout`（502）へマッピングする。
/// 🟡 信頼性レベル: TASK-0031.md注意事項・TASK-0018/0019パターン再利用からの妥当な推測
fn map_steam_api_error(err: api_client_lib::ApiError) -> ApiError {
    match err {
        api_client_lib::ApiError::Http { status, .. } if status == 401 || status == 403 => {
            ApiError::new(ApiErrorCode::SteamApiKeyInvalid, "Steam APIキーが無効です")
        }
        api_client_lib::ApiError::Auth(_) => {
            ApiError::new(ApiErrorCode::SteamApiKeyInvalid, "Steam APIキーが無効です")
        }
        _ => ApiError::new(
            ApiErrorCode::ExternalApiTimeout,
            "Steam APIへの接続に失敗しました",
        ),
    }
}

/// `SteamGameEntry`相当のデータを`CreateItemRequest`へ変換する。
///
/// 【機能概要】: appid/name/playtime_forever等から`CreateItemRequest`（`media_type=Game`）を構築する。
/// `name`が`None`の場合はフォールバックタイトル`"Unknown (appid:{appid})"`を用いる
/// （未確定事項#2の確定: フォールバック登録方針、TC-017-B02）。
/// 🔵🟡 信頼性レベル: 要件定義書2章データフロー（🔵）・フォールバック方針（🟡 note.mdからの妥当な推測）
///
pub fn steam_game_entry_to_create_item_request(
    appid: u32,
    name: Option<String>,
    _playtime_forever: u32,
) -> CreateItemRequest {
    // 【タイトル解決】: nameがNoneの場合はフォールバックタイトルを用いる（確定方針#2、TC-017-B02） 🟡
    let title = name.unwrap_or_else(|| format!("Unknown (appid:{appid})"));

    // 【CreateItemRequest構築】: media_type=Game固定、他フィールドはデフォルト相当（None） 🔵
    CreateItemRequest {
        media_type: MediaType::Game,
        title,
        original_title: None,
        description: None,
        cover_image_url: None,
        release_date: None,
        homepage_url: None,
        rating: None,
        is_favorite: None,
        details: None,
        consumed_date: None,
        additional_image_urls: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// 統合テスト用プール取得ヘルパー（既存パターンと同型）
    /// 🟡 信頼性レベル: api_credential_repository.rs等既存test_poolパターンの踏襲
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0031統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// DB非依存テスト用: 到達不能プール（パニック時に実行される想定。検証ロジック単体テストでは未到達のはず）
    /// 【Green修正】: `PgPool::connect`（即時接続）は到達不能ホストへの接続試行で
    /// 環境依存のタイムアウト（最大30秒）が発生し、DB非依存のはずのユニットテストを大幅に遅延させていた。
    /// `connect_lazy`は実際にクエリが発行されるまで接続を試行しないため、本テスト群（早期returnでDB未到達）
    /// が高速に完了するようになる。挙動の意味（「到達不能プール」であること）自体は変わらない。
    /// 🟡 信頼性レベル: sqlx::Pool::connect_lazyの仕様（接続は遅延される）からの妥当な改善
    fn unreachable_pool() -> PgPool {
        PgPool::connect_lazy("postgres://invalid:invalid@127.0.0.1:1/invalid")
            .expect("到達不能プールの構築（lazy）に失敗しました")
    }

    /// テスト用: 固定APIキーを返すSteamCredentialLookupを構築する
    fn fixed_credentials(api_key: &'static str) -> SteamCredentialLookup {
        SteamCredentialLookup::Fixed(Arc::new(move || {
            Some(ApiCredential {
                provider: ApiProvider::Steam,
                api_key: api_key.to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            })
        }))
    }

    /// テスト用: APIキー未設定（None）を返すSteamCredentialLookupを構築する
    fn no_credentials() -> SteamCredentialLookup {
        SteamCredentialLookup::Fixed(Arc::new(|| None))
    }

    // ============================================================
    // 1. steam_id 形式検証ユニットテスト（DB非依存）
    // ============================================================

    /// TC-017-B01: 17桁ちょうどのsteam_idは受理されu64へ変換される
    /// 【テスト目的】: 検証式`len()==17`の合格境界（17桁）でOkが返ることを確認する
    /// 【テスト内容】: validate_steam_id("76561198000000000")を呼び出す
    /// 【期待される動作】: Ok(76561198000000000u64)が返る
    /// 🔵 信頼性レベル: note.md「17桁の数値文字列」検証式・TC-017-B01に直接対応
    #[test]
    fn validate_steam_id_accepts_17_digit_numeric_string() {
        // 【テストデータ準備】: 要件定義書の代表例である正規SteamID64を用意する
        // 【初期条件設定】: 17桁・全数字の文字列
        let input = "76561198000000000";

        // 【実際の処理実行】: validate_steam_idを呼び出す（未実装のためtodo!()でpanicする想定＝Red）
        // 【処理内容】: 桁数・数値検証 + u64変換
        let result = validate_steam_id(input);

        // 【結果検証】: Ok(u64)が返り、値が入力文字列のparseと一致することを確認
        let value = result.expect("17桁の正規SteamID64はOkを返すはず"); // 【確認内容】: 17桁ちょうどの境界が受理されることを確認 🔵
        assert_eq!(value, 76561198000000000u64); // 【確認内容】: u64変換値が入力文字列のparse結果と一致することを確認 🔵
    }

    /// TC-017-E03-a: 空文字のsteam_idはVALIDATION_ERROR(400)を返す
    /// 【テスト目的】: 空文字入力が拒否されることを確認する
    /// 【テスト内容】: validate_steam_id("")を呼び出す
    /// 【期待される動作】: Err(ApiError{code:"VALIDATION_ERROR", status:400})が返る
    /// 🟡 信頼性レベル: TASK-0031.md「テストケース4」・note.md検証式からの妥当な推測
    #[test]
    fn validate_steam_id_rejects_empty_string() {
        // 【テストデータ準備】: 空文字（未入力相当）を用意する
        let input = "";

        // 【実際の処理実行】: validate_steam_idを呼び出す
        let result = validate_steam_id(input);

        // 【結果検証】: VALIDATION_ERROR・400であることを確認
        let err = result.expect_err("空文字はErrを返すはず");
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 空文字がVALIDATION_ERRORで拒否されることを確認 🟡
        assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🟡
    }

    /// TC-017-E03-b: 非数値混在のsteam_idはVALIDATION_ERROR(400)を返す
    /// 【テスト目的】: 数値以外の文字を含む入力が拒否されることを確認する
    /// 【テスト内容】: validate_steam_id("7656119800000000a")を呼び出す
    /// 【期待される動作】: Err(ApiError{code:"VALIDATION_ERROR"})が返る
    /// 🟡 信頼性レベル: TASK-0031.md「テストケース4」より
    #[test]
    fn validate_steam_id_rejects_non_numeric_characters() {
        // 【テストデータ準備】: 末尾が数値以外の文字列（17桁だが非数値混在）を用意する
        let input = "7656119800000000a";

        // 【実際の処理実行】: validate_steam_idを呼び出す
        let result = validate_steam_id(input);

        // 【結果検証】: VALIDATION_ERRORであることを確認
        let err = result.expect_err("非数値混在はErrを返すはず");
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 非数値混在がVALIDATION_ERRORで拒否されることを確認 🟡
    }

    /// TC-017-E03-c: 桁数不足（16桁）のsteam_idはVALIDATION_ERROR(400)を返す
    /// 【テスト目的】: 桁数境界（合格側17桁の1つ下、16桁）が拒否されることを確認する
    /// 【テスト内容】: validate_steam_id("7656119800000000")（16桁）を呼び出す
    /// 【期待される動作】: Err(ApiError{code:"VALIDATION_ERROR"})が返る
    /// 🟡 信頼性レベル: TASK-0031.md「テストケース4」・TC-017-B01との対称ケースより
    #[test]
    fn validate_steam_id_rejects_16_digit_string() {
        // 【テストデータ準備】: 17桁の境界より1桁少ない16桁の数値文字列を用意する
        let input = "7656119800000000";
        assert_eq!(input.len(), 16); // 【前提条件確認】: 入力が確実に16桁であることを確認する

        // 【実際の処理実行】: validate_steam_idを呼び出す
        let result = validate_steam_id(input);

        // 【結果検証】: VALIDATION_ERRORであることを確認
        let err = result.expect_err("16桁はErrを返すはず");
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 16桁（境界の外側）がVALIDATION_ERRORで拒否されることを確認 🟡
    }

    /// TC-017-E03-d: 桁数超過（18桁）のsteam_idはVALIDATION_ERROR(400)を返す
    /// 【テスト目的】: 桁数境界（合格側17桁の1つ上、18桁）が拒否されることを確認する
    /// 【テスト内容】: validate_steam_id("765611980000000000")（18桁）を呼び出す
    /// 【期待される動作】: Err(ApiError{code:"VALIDATION_ERROR"})が返る
    /// 🟡 信頼性レベル: TASK-0031.md「テストケース4」・TC-017-B01との対称ケースより
    #[test]
    fn validate_steam_id_rejects_18_digit_string() {
        // 【テストデータ準備】: 17桁の境界より1桁多い18桁の数値文字列を用意する
        let input = "765611980000000000";
        assert_eq!(input.len(), 18); // 【前提条件確認】: 入力が確実に18桁であることを確認する

        // 【実際の処理実行】: validate_steam_idを呼び出す
        let result = validate_steam_id(input);

        // 【結果検証】: VALIDATION_ERRORであることを確認
        let err = result.expect_err("18桁はErrを返すはず");
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 18桁（境界の外側）がVALIDATION_ERRORで拒否されることを確認 🟡
    }

    /// TC-017-B05: 17桁最大値の数値文字列でもu64変換がオーバーフローせず成功する
    /// 【テスト目的】: 17桁の最大値（"99999999999999999"）でも`parse::<u64>()`がOkを返すことを確認する
    /// 【テスト内容】: validate_steam_id("99999999999999999")を呼び出す
    /// 【期待される動作】: Ok(99999999999999999u64)が返る（u64の範囲を超えないことの確認）
    /// 🟡 信頼性レベル: note.md「SteamID64範囲」「u64」記載からの算術的に妥当な推測
    #[test]
    fn validate_steam_id_handles_17_digit_max_value_without_overflow() {
        // 【テストデータ準備】: 17桁の最大値（全て9）を用意する。u64の最大値は20桁(約1.8e19)のため
        // 17桁全域は範囲内のはずだが、変換ロジックの実装ミスがあれば検出できる境界値
        let input = "99999999999999999";

        // 【実際の処理実行】: validate_steam_idを呼び出す
        let result = validate_steam_id(input);

        // 【結果検証】: Ok(u64)が返り、オーバーフローしないことを確認
        let value = result.expect("17桁最大値はu64範囲内でOkを返すはず"); // 【確認内容】: 17桁最大値でもパース失敗・オーバーフローしないことを確認 🟡
        assert_eq!(value, 99999999999999999u64); // 【確認内容】: 変換値が入力文字列のparse結果と一致することを確認 🟡
    }

    // ============================================================
    // 2. SteamGameEntry → CreateItemRequest 変換ユニットテスト（DB非依存）
    // ============================================================

    /// TC-017-N03: 変換結果がmedia_type=Game/title=name/外部キー連携に必要な情報を持つ
    /// 【テスト目的】: appid/name/playtime_foreverからCreateItemRequestへの変換が正しいことを確認する
    /// 【テスト内容】: steam_game_entry_to_create_item_request(440, Some("Team Fortress 2"), 1200)を呼ぶ
    /// 【期待される動作】: media_type==Game、title=="Team Fortress 2"
    /// 🔵 信頼性レベル: 要件定義書2章「各SteamGameEntry→CreateItemRequest」に直接対応
    #[test]
    fn steam_game_entry_conversion_sets_media_type_game_and_title() {
        // 【テストデータ準備】: 実在appidのTeam Fortress 2を代表値として用いる
        // 【初期条件設定】: nameあり・プレイ時間1200分のエントリ
        let appid = 440u32;
        let name = Some("Team Fortress 2".to_string());
        let playtime_forever = 1200u32;

        // 【実際の処理実行】: steam_game_entry_to_create_item_requestを呼び出す
        // 【処理内容】: SteamGameEntry相当のフィールド→CreateItemRequestへのマッピング
        let request = steam_game_entry_to_create_item_request(appid, name, playtime_forever);

        // 【結果検証】: media_type/titleが期待通りであることを確認
        assert_eq!(request.media_type, MediaType::Game); // 【確認内容】: media_typeがGameに設定されることを確認 🔵
        assert_eq!(request.title, "Team Fortress 2"); // 【確認内容】: titleがSteamGameEntry.nameの値になることを確認 🔵
    }

    /// TC-017-B02: name=Noneのエントリはフォールバックタイトルで登録される（パニックしない）
    /// 【テスト目的】: nameがNoneのときにunwrapでパニックせず、フォールバックタイトルが設定されることを確認する
    /// 【テスト内容】: steam_game_entry_to_create_item_request(99999, None, 0)を呼ぶ
    /// 【期待される動作】: title=="Unknown (appid:99999)"（方針A: フォールバック登録、未確定事項#2の確定）
    /// 🟡 信頼性レベル: note.md「ゲーム名のnullチェック」・本タスクで確定したフォールバック方針より
    #[test]
    fn steam_game_entry_conversion_uses_fallback_title_when_name_is_none() {
        // 【テストデータ準備】: nameがNoneのエントリ（include_appinfo無効時等を想定）を用意する
        // 【初期条件設定】: appid=99999、playtime_forever=0（未プレイ）
        let appid = 99999u32;
        let name: Option<String> = None;
        let playtime_forever = 0u32;

        // 【実際の処理実行】: steam_game_entry_to_create_item_requestを呼び出す
        // 【処理内容】: name=Noneの境界分岐（フォールバックタイトル生成）
        let request = steam_game_entry_to_create_item_request(appid, name, playtime_forever);

        // 【結果検証】: パニックせずフォールバックタイトルが設定されることを確認
        assert_eq!(request.title, "Unknown (appid:99999)"); // 【確認内容】: name=Noneでもフォールバックタイトルで安全に登録されることを確認 🟡
        assert_eq!(request.media_type, MediaType::Game); // 【確認内容】: media_typeは通常時と同様Gameであることを確認 🟡
    }

    /// TC-017-N03-b: appidがゼロ埋めなしで文字列化されexternal_id相当になる
    /// 【テスト目的】: appid:u32→external_id:Stringの文字列化がゼロ埋め等を行わないことを確認する
    /// 【テスト内容】: appidに570を渡した変換結果を確認する（external_idはハンドラ/usecase側でSome(appid.to_string())として
    /// create_item_with_sourceへ渡される想定のため、本テストではCreateItemRequest自体にexternal_idを含めない
    /// 設計（既存パターンに合わせ引数で別途渡す）を前提に、titleの一致のみを確認する最小ケース）
    /// 🔵 信頼性レベル: 要件定義書2章「external_id=appid」・既存create_item_with_source引数分離パターンより
    #[test]
    fn steam_game_entry_conversion_preserves_title_for_different_appid() {
        // 【テストデータ準備】: Dota 2（appid:570）を代表値として用いる
        let appid = 570u32;
        let name = Some("Dota 2".to_string());
        let playtime_forever = 600u32;

        // 【実際の処理実行】: steam_game_entry_to_create_item_requestを呼び出す
        let request = steam_game_entry_to_create_item_request(appid, name, playtime_forever);

        // 【結果検証】: titleが入力のnameと一致することを確認
        assert_eq!(request.title, "Dota 2"); // 【確認内容】: 異なるappid/nameでも変換ロジックが正しく機能することを確認 🔵
    }

    // ============================================================
    // 3. import_steam_library ユニットテスト（wiremock + DI、DB依存は#[ignore]）
    // ============================================================

    /// TC-017-E01-A: Steam APIキー未設定時にSTEAM_API_KEY_INVALID(401)を返す（DB非依存・ユニット）
    /// 【テスト目的】: APIキーがDB未登録（lookup結果None）の場合、Steam API呼び出しに進まず401を返すことを確認する
    /// 【テスト内容】: no_credentials()（常にNoneを返すDI）でimport_steam_libraryを呼ぶ
    /// 【期待される動作】: Err(ApiError{code:"STEAM_API_KEY_INVALID", status:401})が返り、外部APIへのHTTPリクエストは発生しない
    /// 🔵 信頼性レベル: TASK-0031.md「テストケース2（TC-017-E01）」・要件定義書3章/4章に直接対応
    #[tokio::test]
    async fn import_steam_library_returns_401_when_api_key_not_configured() {
        // 【テスト前準備】: 到達不能プール（APIキー未設定経路ではDBアクセス自体が発生しない想定の検証も兼ねる）
        // 【初期条件設定】: SteamCredentialLookup::Fixedで常にNoneを返すDIを使用する
        let pool = unreachable_pool();
        let credentials = no_credentials();

        // 【実際の処理実行】: import_steam_libraryを呼び出す（未実装のためtodo!()でpanicする想定＝Red）
        // 【処理内容】: APIキー解決→未設定検出→401エラー返却
        let result = import_steam_library(&pool, &credentials, None, "76561198000000000").await;

        // 【結果検証】: STEAM_API_KEY_INVALID・401であることを確認
        let err = result.expect_err("APIキー未設定時はErrを返すはず");
        assert_eq!(err.error.code, "STEAM_API_KEY_INVALID"); // 【確認内容】: APIキー未設定時にSTEAM_API_KEY_INVALIDが返ることを確認 🔵
        assert_eq!(err.status, axum::http::StatusCode::UNAUTHORIZED); // 【確認内容】: HTTPステータスが401であることを確認 🔵
    }

    /// TC-017-E01-B: Steam Web APIが401/403相当を返す場合にSTEAM_API_KEY_INVALID(401)へマッピングする（wiremock・ユニット）
    /// 【テスト目的】: APIキーは存在するがSteam側が認証エラーを返すケースで、401へ正しくマッピングされることを確認する
    /// 【テスト内容】: wiremockでGetOwnedGamesエンドポイントが401を返すように設定し、import_steam_libraryを呼ぶ
    /// 【期待される動作】: Err(ApiError{code:"STEAM_API_KEY_INVALID", status:401})が返る
    /// 🔵 信頼性レベル: TASK-0031.md「テストケース2（TC-017-E01）」・ケースBに直接対応
    #[tokio::test]
    async fn import_steam_library_returns_401_when_steam_api_rejects_key() {
        // 【テスト前準備】: Steam APIモックが401を返すよう設定する
        let steam_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&steam_mock)
            .await;

        let pool = unreachable_pool();
        let credentials = fixed_credentials("invalid-or-revoked-key");

        // 【実際の処理実行】: モックベースURLを注入してimport_steam_libraryを呼び出す
        let result = import_steam_library(
            &pool,
            &credentials,
            Some(&steam_mock.uri()),
            "76561198000000000",
        )
        .await;

        // 【結果検証】: STEAM_API_KEY_INVALID・401であることを確認し、DB登録に進まないことを保証する
        let err = result.expect_err("Steam APIの401応答はErrを返すはず");
        assert_eq!(err.error.code, "STEAM_API_KEY_INVALID"); // 【確認内容】: Steam側401応答がSTEAM_API_KEY_INVALIDへマッピングされることを確認 🔵
        assert_eq!(err.status, axum::http::StatusCode::UNAUTHORIZED); // 【確認内容】: HTTPステータスが401であることを確認 🔵
    }

    /// TC-017-02: 所持ゲーム0件（空配列）時は正常終了し空のImportSummaryを返す（wiremock・ユニット、DB非到達想定）
    /// 【テスト目的】: プロフィール非公開等でSteam APIが空配列を返す場合、エラーとせず空サマリで正常終了することを確認する
    /// 【テスト内容】: wiremockでgame_count=0/games=[]を返すよう設定し、import_steam_libraryを呼ぶ
    /// 【期待される動作】: Ok(ImportSummary{success_count:0, failure_count:0, failures:[]})が返る
    /// 🟡 信頼性レベル: TASK-0031.md注意事項・要件定義書4章「プロフィール非公開（TC-017-02）」に対応
    #[tokio::test]
    async fn import_steam_library_returns_empty_summary_for_empty_game_list() {
        // 【テスト前準備】: Steam APIモックがgame_count=0の空配列を返すよう設定する
        let steam_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "response": { "game_count": 0, "games": [] }
            })))
            .mount(&steam_mock)
            .await;

        // 【初期条件設定】: DB登録が発生しないため到達不能プールでも成功するはず（プロフィール非公開時はDBアクセス不要）
        let pool = unreachable_pool();
        let credentials = fixed_credentials("valid-steam-key");

        // 【実際の処理実行】: import_steam_libraryを呼び出す
        let result = import_steam_library(
            &pool,
            &credentials,
            Some(&steam_mock.uri()),
            "76561198000000000",
        )
        .await;

        // 【結果検証】: Okで空サマリが返ることを確認
        let summary = result.expect("空配列時はOkを返すはず"); // 【確認内容】: 空配列がエラーとして扱われないことを確認 🟡
        assert_eq!(summary.success_count, 0); // 【確認内容】: success_countが0であることを確認 🟡
        assert_eq!(summary.failure_count, 0); // 【確認内容】: failure_countが0であることを確認 🟡
        assert_eq!(summary.failures.len(), 0); // 【確認内容】: failuresが空配列であることを確認 🟡
    }

    /// TC-017-01: 複数ゲーム返却時に全件登録されsuccess_countが件数と一致する（wiremock + 実DB統合、#[ignore]）
    /// 【テスト目的】: 主要正常フロー（取得→変換→登録→集計）が件数整合性を保って動作することを確認する
    /// 【テスト内容】: wiremockが2件のゲームを返すよう設定し、実DBへimport_steam_libraryを実行する
    /// 【期待される動作】: Ok(ImportSummary{success_count:2, failure_count:0, failures:[]})、items/game_detailsに2行登録される
    /// 🔵 信頼性レベル: TASK-0031.md「単体テスト要件 テストケース1（TC-017-01）」に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn import_steam_library_registers_all_games_and_reports_success_count() {
        // 【テスト前準備】: Steam APIモックが2件の所持ゲームを返すよう設定する
        let steam_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "response": {
                    "game_count": 2,
                    "games": [
                        {"appid": 440, "name": "Team Fortress 2", "playtime_forever": 1200},
                        {"appid": 570, "name": "Dota 2", "playtime_forever": 600}
                    ]
                }
            })))
            .mount(&steam_mock)
            .await;

        let pool = test_pool().await;
        let credentials = fixed_credentials("valid-steam-key");

        // 【テストデータ前提】: external_id=440/570のitemsが事前に存在しないことを前提とする
        // （Greenフェーズでクリーンアップヘルパーを追加する想定）

        // 【実際の処理実行】: import_steam_libraryを実DBに対して実行する
        let result = import_steam_library(
            &pool,
            &credentials,
            Some(&steam_mock.uri()),
            "76561198000000000",
        )
        .await;

        // 【結果検証】: success_countが2件と一致することを確認
        let summary = result.expect("正常系はOkを返すはず");
        assert_eq!(summary.success_count, 2); // 【確認内容】: 取得2件が全て成功カウントされることを確認 🔵
        assert_eq!(summary.failure_count, 0); // 【確認内容】: 失敗が発生しないことを確認 🔵
    }

    /// TC-017-E02: 1件登録失敗（appid:0等）でも他ゲーム登録が継続される（EDGE-002、wiremock + 実DB統合、#[ignore]）
    /// 【テスト目的】: 部分失敗時の処理継続と失敗記録（EDGE-002）を確認する
    /// 【テスト内容】: wiremockが3件返却し、うち1件が一意制約違反等で登録失敗するデータを含むケースを実DBで検証する
    /// 【期待される動作】: Ok(ImportSummary{success_count:2, failure_count:1, failures:[{row_number:.., reason:..}]})
    /// 🟡 信頼性レベル: TASK-0031.md「テストケース3（EDGE-002）」・失敗誘発データは妥当な推測
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn import_steam_library_continues_processing_after_one_game_registration_fails() {
        // 【テスト前準備】: 3件中1件が失敗を誘発するデータ（既存登録と重複しないが別要因で失敗する想定）を用意する
        let steam_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "response": {
                    "game_count": 3,
                    "games": [
                        {"appid": 440, "name": "Team Fortress 2", "playtime_forever": 1200},
                        {"appid": 9999998, "name": "Broken Game", "playtime_forever": 0},
                        {"appid": 570, "name": "Dota 2", "playtime_forever": 600}
                    ]
                }
            })))
            .mount(&steam_mock)
            .await;

        let pool = test_pool().await;
        let credentials = fixed_credentials("valid-steam-key");

        // 【実際の処理実行】: import_steam_libraryを実DBに対して実行する
        let result = import_steam_library(
            &pool,
            &credentials,
            Some(&steam_mock.uri()),
            "76561198000000000",
        )
        .await;

        // 【結果検証】: 全体は200相当（Ok）で返り、失敗1件が記録され他2件は成功することを確認
        let summary = result.expect("部分失敗時もOkを返すはず（EDGE-002）");
        assert_eq!(summary.success_count, 2); // 【確認内容】: 正常な2件が成功登録されることを確認 🟡
        assert_eq!(summary.failure_count, 1); // 【確認内容】: 失敗1件がfailure_countに反映されることを確認 🟡
        assert_eq!(summary.failures.len(), 1); // 【確認内容】: failures配列にも1件記録されることを確認（不変条件） 🟡
    }

    /// TC-017-E05: 既登録steam_appidを含む一覧で重複がスキップされ集計外になる（実DB統合、#[ignore]）
    /// 【テスト目的】: 重複防止ロジックと「重複≠失敗」の集計方針（スキップ＝集計外、確定方針）を確認する
    /// 【テスト内容】: 事前にexternal_id="440"のitemを登録した状態で、appid 440/570を含む一覧をインポートする
    /// 【期待される動作】: Ok(ImportSummary{success_count:1, failure_count:0, failures:[]})（440は集計外スキップ、570のみ新規success）
    /// 🟡 信頼性レベル: TASK-0031.md「テストケース5」・本タスクで確定した集計方針（スキップ＝集計外）より
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn import_steam_library_skips_duplicate_appid_without_counting_as_failure() {
        // 【テスト前準備】: 事前にexternal_id="440"のitemを(media_type=game, source=api)で登録しておく
        let pool = test_pool().await;
        let existing_request = CreateItemRequest {
            media_type: MediaType::Game,
            title: "Team Fortress 2".to_string(),
            original_title: None,
            description: None,
            cover_image_url: None,
            release_date: None,
            homepage_url: None,
            rating: None,
            is_favorite: None,
            details: None,
            consumed_date: None,
            additional_image_urls: Vec::new(),
        };
        crate::repositories::item_repository::create_item_with_source(
            &pool,
            existing_request,
            ItemSource::Api,
            Some("440".to_string()),
        )
        .await
        .expect("事前データ投入は成功するはず");

        let steam_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "response": {
                    "game_count": 2,
                    "games": [
                        {"appid": 440, "name": "Team Fortress 2", "playtime_forever": 1200},
                        {"appid": 570, "name": "Dota 2", "playtime_forever": 600}
                    ]
                }
            })))
            .mount(&steam_mock)
            .await;

        let credentials = fixed_credentials("valid-steam-key");

        // 【実際の処理実行】: import_steam_libraryを実DBに対して実行する
        let result = import_steam_library(
            &pool,
            &credentials,
            Some(&steam_mock.uri()),
            "76561198000000000",
        )
        .await;

        // 【結果検証】: 重複440はスキップ（集計外）、新規570のみsuccessカウントされることを確認
        let summary = result.expect("重複を含む一覧でもOkを返すはず");
        assert_eq!(summary.success_count, 1); // 【確認内容】: 新規570のみがsuccess_countに加算されることを確認（重複440は集計外） 🟡
        assert_eq!(summary.failure_count, 0); // 【確認内容】: 重複はfailureとしてカウントされないことを確認 🟡
    }

    /// TC-017-E04: Steam APIタイムアウト/接続不能時にEXTERNAL_API_TIMEOUT(502)を返す（wiremock不使用・ユニット）
    /// 【テスト目的】: 外部API障害がEXTERNAL_API_TIMEOUT(502)へマッピングされることを確認する（panicしない）
    /// 【テスト内容】: 到達不能なベースURL（接続即時拒否）でimport_steam_libraryを呼ぶ
    /// 【期待される動作】: Err(ApiError{code:"EXTERNAL_API_TIMEOUT", status:502})が返る
    /// 🟡 信頼性レベル: TASK-0031.md注意事項・TASK-0018/0019パターン再利用からの妥当な推測
    #[tokio::test]
    async fn import_steam_library_returns_502_when_steam_api_is_unreachable() {
        // 【テスト前準備】: 到達不能ポートをSteam APIベースURLとして注入する
        let pool = unreachable_pool();
        let credentials = fixed_credentials("valid-steam-key");

        // 【実際の処理実行】: 到達不能URLでimport_steam_libraryを呼び出す
        let result = import_steam_library(
            &pool,
            &credentials,
            Some("http://127.0.0.1:1"),
            "76561198000000000",
        )
        .await;

        // 【結果検証】: EXTERNAL_API_TIMEOUT・502であることを確認（panicしないことが最重要）
        let err = result.expect_err("接続不能時はErrを返すはず");
        assert_eq!(err.error.code, "EXTERNAL_API_TIMEOUT"); // 【確認内容】: 外部API接続不能がEXTERNAL_API_TIMEOUTへマッピングされることを確認 🟡
        assert_eq!(err.status, axum::http::StatusCode::BAD_GATEWAY); // 【確認内容】: HTTPステータスが502であることを確認 🟡
    }

    /// TC-017-E03-handler: import_steam_library自体もsteam_id形式不正時にVALIDATION_ERROR(400)を返す
    /// 【テスト目的】: usecase層が独自にsteam_id検証を行い、不正形式の場合は外部API呼び出し前に早期returnすることを確認する
    /// 【テスト内容】: import_steam_libraryに不正なsteam_id（空文字）を渡す
    /// 【期待される動作】: Err(ApiError{code:"VALIDATION_ERROR", status:400})が返り、外部APIへのHTTPリクエストは発生しない
    /// 🟡 信頼性レベル: TASK-0031.md「テストケース4」・入力検証は外部API呼び出し前に行う方針より
    #[tokio::test]
    async fn import_steam_library_returns_400_for_invalid_steam_id_before_calling_steam_api() {
        // 【テスト前準備】: 到達不能プール・到達不能ベースURLでも、検証失敗が先に発生するため問題にならないことを確認する設計
        let pool = unreachable_pool();
        let credentials = fixed_credentials("valid-steam-key");

        // 【実際の処理実行】: 不正なsteam_id（空文字）でimport_steam_libraryを呼び出す
        let result = import_steam_library(&pool, &credentials, None, "").await;

        // 【結果検証】: VALIDATION_ERROR・400であることを確認
        let err = result.expect_err("不正なsteam_idはErrを返すはず");
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 不正なsteam_idがVALIDATION_ERRORで早期に拒否されることを確認 🟡
        assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🟡
    }
}
