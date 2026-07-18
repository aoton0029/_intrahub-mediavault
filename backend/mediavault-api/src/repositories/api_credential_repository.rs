//! api_credentials（外部APIキー管理）リポジトリ
//!
//! TASK-0022: api_credentials（外部APIキー管理）CRUD実装
//!
//! 🔵 信頼性レベル: 要件定義書REQ-0022-01〜06・note.md L21-22・database-schema.sql L348-353より

use sqlx::PgPool;

use crate::models::api_credential::{ApiCredential, ApiProvider};
use crate::models::response::{ApiError, ApiErrorCode};

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 【改善内容】: tag_repository.rsのdb_errorと同型のコメント形式に統一し、
/// ログ出力に`api_key`の値そのものを含めない設計意図を明文化した
/// 【設計方針】: `err`（sqlx::Errorのデバッグ表現）にはSQL文やバインド値は含まれず、
/// 接続エラー・制約違反等のメタ情報のみが出力される。`api_key`の平文をログへ
/// 記録しないことで、タスクファイル注意事項にある「ログ出力のマスキング」要請を
/// ログに値自体を渡さないことで構造的に満たす
/// 🔵 信頼性レベル: repositories/db_error_utils.rs・既存db_error関数（tag_repository.rs等）と同型
fn db_error(err: sqlx::Error) -> ApiError {
    // 【詳細ログ出力】: 内部調査用に詳細をサーバーログへ記録する（クライアントへは返さない）。
    // 【機密情報保護】: api_keyの値は引数に取っておらず、ここで参照すること自体ができないため、
    // 将来の実装変更でも誤って平文ログ出力されない構造になっている 🔵
    tracing::error!("api_credential repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "APIキーの登録処理に失敗しました",
    )
}

/// providerのAPIキーをupsert（存在しなければ作成、存在すれば更新）する
///
/// 【機能概要】: `INSERT ... ON CONFLICT (provider) DO UPDATE SET api_key = $2 ...` を実行する
/// 🔵 信頼性レベル: REQ-0022-01〜04、note.md L14・L21-22より
pub async fn upsert_api_credential(
    pool: &PgPool,
    provider: ApiProvider,
    api_key: String,
) -> Result<ApiCredential, ApiError> {
    // 【UPSERT実行】: providerがPRIMARY KEYのため、ON CONFLICT (provider) DO UPDATEで
    // 新規作成（INSERT分岐）と既存更新（UPDATE分岐）を単一SQLで処理する。
    // updated_atはDBトリガー(trg_api_credentials_updated_at)がBEFORE UPDATEで自動設定するため、
    // SET句に明示すれば即値、含めなくても最終的にトリガー値が優先される（REQ-0022-201）。
    // 🔵 信頼性レベル: REQ-0022-01〜04・database-schema.sql L350（provider PRIMARY KEY）より
    let result: Result<ApiCredential, sqlx::Error> = sqlx::query_as(
        "INSERT INTO api_credentials (provider, api_key) VALUES ($1, $2) \
         ON CONFLICT (provider) DO UPDATE SET api_key = $2, updated_at = CURRENT_TIMESTAMP \
         RETURNING provider, api_key, updated_at",
    )
    .bind(provider)
    .bind(&api_key)
    .fetch_one(pool)
    .await;

    result.map_err(db_error)
}

/// providerに対応するAPIキーをDBから取得する（未登録ならNone）
///
/// 【機能概要】: 後続タスク（TASK-0023）のExternalSearchServiceが利用する取得関数
/// 🔵 信頼性レベル: REQ-0022-06・実装詳細5より
pub async fn find_by_provider(
    pool: &PgPool,
    provider: ApiProvider,
) -> Result<Option<ApiCredential>, ApiError> {
    // 【SELECT実行】: 該当providerのレコードが無ければNone（fetch_optional）を返す。
    // 未登録を例外ではなく正常な「なし」として扱う設計（REQ-0022-06）。
    // 🔵 信頼性レベル: REQ-0022-06・実装詳細5より
    let result: Result<Option<ApiCredential>, sqlx::Error> = sqlx::query_as(
        "SELECT provider, api_key, updated_at FROM api_credentials WHERE provider = $1",
    )
    .bind(provider)
    .fetch_optional(pool)
    .await;

    result.map_err(db_error)
}

/// 🟡 Intent: 設定画面にはキー本体を返さず、空でないキーが存在するproviderだけを返す。
pub async fn list_configured_providers(pool: &PgPool) -> Result<Vec<ApiProvider>, ApiError> {
    sqlx::query_scalar(
        "SELECT provider FROM api_credentials \
         WHERE provider IN ('tmdb', 'steam', 'annict', 'rakuten') \
         AND LENGTH(TRIM(api_key)) > 0 ORDER BY provider",
    )
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0022統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    async fn unreachable_pool() -> PgPool {
        PgPool::connect("postgres://invalid:invalid@127.0.0.1:1/invalid")
            .await
            .expect("到達不能プールの構築検証用接続に失敗しました")
    }

    /// 統合テスト共通: tmdb行をDBから削除しクリーンな状態にする
    async fn cleanup_provider(pool: &PgPool, provider: &str) {
        sqlx::query("DELETE FROM api_credentials WHERE provider = $1::api_provider")
            .bind(provider)
            .execute(pool)
            .await
            .expect("テスト前クリーンアップに失敗しました");
    }

    // ============================================================
    // 正常系（統合）
    // ============================================================

    /// TC-015-01-B: TMDbキー新規登録（upsert、実DB統合）
    /// 🔵 信頼性レベル: 要件定義書シナリオ1・REQ-0022-02、タスクファイルテストケース1より
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn upsert_api_credential_creates_new_record_when_not_exists() {
        // 【テスト目的】: 実DB上で対象providerのレコードが存在しない状態からupsertを行い、新規行が作成されるかを確認する
        // 【テスト内容】: tmdb行が存在しないクリーンな状態でApiProvider::Tmdb + api_key="valid-tmdb-key"でupsertする
        // 【期待される動作】: 戻り値ApiCredentialのprovider==Tmdb・api_key=="valid-tmdb-key"、DB再取得でも同一行が1件存在する
        // 🔵 信頼性レベル: 要件定義書シナリオ1・REQ-0022-02、タスクファイルテストケース1（TC-015-01）より

        // 【テストデータ準備】: INSERTパス（ON CONFLICT非発生）を表すクリーンな初期状態を用意する
        // 【初期条件設定】: tmdb行を事前に削除しておく
        let pool = test_pool().await;
        cleanup_provider(&pool, "tmdb").await;

        // 【実際の処理実行】: upsert_api_credentialを新規状態に対して呼び出す
        // 【処理内容】: INSERT ... ON CONFLICT ... の実行
        let result =
            upsert_api_credential(&pool, ApiProvider::Tmdb, "valid-tmdb-key".to_string()).await;

        // 【結果検証】: 戻り値の内容とDB再取得結果を確認する
        // 【期待値確認】: 行が重複生成されず1件のみであることが重要
        let credential = result.expect("upsert_api_credentialは成功するはず"); // 【確認内容】: 新規作成が成功すること 🔵
        assert_eq!(credential.provider, ApiProvider::Tmdb); // 【確認内容】: providerがTmdbであることを確認 🔵
        assert_eq!(credential.api_key, "valid-tmdb-key"); // 【確認内容】: api_keyが指定値で保存されることを確認 🔵
    }

    // TC-015-01-C相当: ハンドラ統合はroutes/mod.rsの統合テストで実施（本ファイルはリポジトリ層のみ）

    /// TC-015-03-A: キー更新が`find_by_provider`に反映される（upsert+取得、実DB統合）
    /// 🔵 信頼性レベル: 要件定義書シナリオ3・REQ-0022-03・REQ-0022-06より
    #[tokio::test]
    #[ignore]
    async fn upsert_then_find_by_provider_returns_updated_key() {
        // 【テスト目的】: 既存レコードがある状態でupsert（UPDATE分岐）を行い、find_by_providerで取得した値が更新後のキーになっているかを確認する
        // 【テスト内容】: provider=tmdb, api_key=old-keyを事前投入後、api_key=new-keyでupsertし、find_by_provider(Tmdb)を呼ぶ
        // 【期待される動作】: api_keyがold-key→new-keyに更新され、find_by_provider(Tmdb)がSome(ApiCredential{api_key: "new-key", ..})を返す
        // 🔵 信頼性レベル: 要件定義書シナリオ3・REQ-0022-03・REQ-0022-06、タスクファイルテストケース3より

        // 【テストデータ準備】: ON CONFLICT発生によるUPDATEパス＋後続タスク参照経路を確認するための事前データ
        // 【初期条件設定】: tmdbにold-keyを投入してからnew-keyでupsertする
        let pool = test_pool().await;
        cleanup_provider(&pool, "tmdb").await;
        upsert_api_credential(&pool, ApiProvider::Tmdb, "old-key".to_string())
            .await
            .expect("事前データ投入のupsertは成功するはず");

        // 【実際の処理実行】: new-keyでの再upsert後、find_by_providerで取得する
        // 【処理内容】: UPDATE分岐実行 + 取得処理
        upsert_api_credential(&pool, ApiProvider::Tmdb, "new-key".to_string())
            .await
            .expect("キー更新のupsertは成功するはず");
        let found = find_by_provider(&pool, ApiProvider::Tmdb).await;

        // 【結果検証】: find_by_providerの戻り値が最新のapi_keyを保持しているかを確認する
        // 【期待値確認】: provider PRIMARY KEYにより行が増えないことも前提
        let credential = found
            .expect("find_by_providerは成功するはず")
            .expect("tmdbレコードは存在するはず"); // 【確認内容】: upsert後にfind_by_providerが値を返すことを確認 🔵
        assert_eq!(credential.api_key, "new-key"); // 【確認内容】: find_by_providerが更新後の最新キーを返すことを確認 🔵
    }

    /// TC-015-03-B: キー更新時に`updated_at`がトリガーで更新される（実DB統合）
    /// 🔵 信頼性レベル: 要件定義書REQ-0022-201・シナリオ3、note.md L14より
    #[tokio::test]
    #[ignore]
    async fn upsert_updates_updated_at_via_trigger() {
        // 【テスト目的】: UPDATE分岐実行時に、DBトリガーがupdated_atを自動更新するかを確認する
        // 【テスト内容】: 既存レコードへのupsert前後でupdated_atを比較する
        // 【期待される動作】: upsert前のupdated_atより、upsert後のupdated_atが新しい
        // 🔵 信頼性レベル: 要件定義書REQ-0022-201、note.md L14・database-schema.sql L375-376より

        // 【テストデータ準備】: 事前投入済みtmdbレコード（updated_at記録済み）を用意する
        // 【初期条件設定】: cleanup後に初回upsertを行いupdated_at_beforeを記録する
        let pool = test_pool().await;
        cleanup_provider(&pool, "tmdb").await;
        let before = upsert_api_credential(&pool, ApiProvider::Tmdb, "old-key".to_string())
            .await
            .expect("事前データ投入のupsertは成功するはず");

        // 【実際の処理実行】: 同一providerへ再度upsertしupdated_at_afterを取得する
        // 【処理内容】: trg_api_credentials_updated_atトリガーの発火確認
        let after = upsert_api_credential(&pool, ApiProvider::Tmdb, "new-key".to_string())
            .await
            .expect("キー更新のupsertは成功するはず");

        // 【結果検証】: updated_at_after > updated_at_beforeであることを確認する
        // 【期待値確認】: SQLのSET句にupdated_atを含めても含めなくても最終的にトリガー値になること
        assert!(after.updated_at > before.updated_at); // 【確認内容】: トリガーによりupdated_atが新しい値に更新されることを確認 🔵
    }

    // ============================================================
    // 異常系（統合）
    // ============================================================

    /// TC-NEW-05: DB接続不能時にInternalError（500）へ正規化される（実DB統合、unreachable_pool）
    /// 🔵 信頼性レベル: 要件定義書REQ-0022-402・EDGE-0022-04、note.md L22より
    #[tokio::test]
    #[ignore]
    async fn upsert_api_credential_db_error_maps_to_internal_error() {
        // 【テスト目的】: upsert_api_credentialがDB接続不能時にdb_error経由でInternalErrorに変換されることを確認する
        // 【テスト内容】: 到達不能なPgPoolに対しupsert_api_credentialを呼び出す
        // 【期待される動作】: Err(ApiError)が返り、ApiErrorCode::InternalError（500）であること
        // 🔵 信頼性レベル: 要件定義書REQ-0022-402・EDGE-0022-04、note.md L22（db_error_utils.rs・unreachable_poolパターン）より

        // 【テストデータ準備】: 接続先が存在せず全クエリがエラーになるプールを用意する
        // 【初期条件設定】: unreachable_pool()ヘルパーで到達不能プールを構築する
        let pool = unreachable_pool().await;

        // 【実際の処理実行】: 到達不能プールに対しupsertを実行する
        // 【処理内容】: DB接続不能エラーのdb_error変換処理
        let result = upsert_api_credential(&pool, ApiProvider::Tmdb, "x".to_string()).await;

        // 【結果検証】: エラーが返り、InternalErrorコードであることを確認する
        // 【期待値確認】: 機密情報（SQL文・接続文字列）がレスポンスに漏れないこと
        let err = result.expect_err("到達不能プールではErrが返るはず"); // 【確認内容】: DB接続不能時にErrが返ることを確認 🔵
        assert_eq!(err.error.code, ApiErrorCode::InternalError.as_code_str()); // 【確認内容】: エラーコードがINTERNAL_ERRORに正規化されることを確認 🔵
    }

    /// TC-NEW-06: `find_by_provider`が未登録providerで`None`を返す（実DB統合）
    /// 🟡 信頼性レベル: 要件定義書REQ-0022-06（Result<Option<ApiCredential>, ...>シグネチャ）からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn find_by_provider_returns_none_for_unregistered_provider() {
        // 【テスト目的】: 後続タスク向け取得関数が、レコード未登録時に正しく「なし」を表現するかを確認する
        // 【テスト内容】: igdbのレコードが存在しないクリーンな状態でfind_by_provider(Igdb)を呼ぶ
        // 【期待される動作】: find_by_provider(Igdb)がOk(None)を返す（パニック・Errではなく）
        // 🟡 信頼性レベル: 要件定義書REQ-0022-06からの妥当な推測。未登録時None挙動はシグネチャから自然だが明示テスト記載はない

        // 【テストデータ準備】: 該当providerのレコードがDBに存在しない状態を用意する
        // 【初期条件設定】: igdb行を事前に削除しておく
        let pool = test_pool().await;
        cleanup_provider(&pool, "igdb").await;

        // 【実際の処理実行】: find_by_providerを未登録providerに対して呼び出す
        // 【処理内容】: 未登録時の「なし」表現を確認する処理
        let result = find_by_provider(&pool, ApiProvider::Igdb).await;

        // 【結果検証】: Ok(None)が返ることを確認する
        // 【期待値確認】: 未登録を例外ではなく正常な「なし」として扱う
        let found = result.expect("find_by_providerは成功するはず"); // 【確認内容】: 未登録provider取得時にErrにならないことを確認 🟡
        assert!(found.is_none()); // 【確認内容】: 未登録providerに対してNoneが返ることを確認 🟡
    }

    // ============================================================
    // 境界値（統合）
    // ============================================================

    /// TC-NEW-07: `api_key`空文字列でのupsert（境界、実DB統合）
    /// 🟡 信頼性レベル: 要件定義書「api_keyは必須のString」L43からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn upsert_api_credential_accepts_empty_string_api_key() {
        // 【テスト目的】: api_key=""（空文字列）でupsertした場合の挙動を確認する
        // 【テスト内容】: ApiProvider::Tmdb + api_key=""でupsertする
        // 【期待される動作】: 要件に空文字拒否の記載がないため、現仕様では200で保存される想定（NOT NULL制約は空文字を許容する）
        // 🟡 信頼性レベル: 要件定義書「api_keyは必須のString」L43からの妥当な推測。非空バリデーション有無は要件に明記なし

        // 【テストデータ準備】: 長さ0文字列がNOT NULL制約を満たし受理されるかを確認するための境界値
        // 【初期条件設定】: tmdb行を事前に削除しておく
        let pool = test_pool().await;
        cleanup_provider(&pool, "tmdb").await;

        // 【実際の処理実行】: 空文字api_keyでupsert_api_credentialを呼び出す
        // 【処理内容】: 最小長境界での保存処理
        let result = upsert_api_credential(&pool, ApiProvider::Tmdb, String::new()).await;

        // 【結果検証】: 空文字がNOT NULL制約に違反せず受理されることを確認する
        // 【期待値確認】: 将来api_keyバリデーションが追加された場合に検知できる回帰テストとして機能する
        let credential = result.expect("空文字api_keyのupsertは現仕様では成功するはず"); // 【確認内容】: 空文字列がNOT NULL制約に違反せずINSERTが成功することを確認 🟡
        assert_eq!(credential.api_key, ""); // 【確認内容】: 保存されたapi_keyが空文字列のままであることを確認 🟡
    }

    /// TC-NEW-08: `api_key`が500文字（VARCHAR上限）でのupsert（境界、実DB統合）
    /// 🟡 信頼性レベル: 要件定義書「DBカラムはVARCHAR(500) NOT NULL」L43・database-schema.sql L351からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn upsert_api_credential_accepts_500_character_api_key() {
        // 【テスト目的】: api_keyが500文字ちょうどの場合にupsertが成功するかを確認する
        // 【テスト内容】: VARCHAR(500)の最大長境界である500文字の文字列でupsertする
        // 【期待される動作】: 500文字は200で保存成功し、find_by_providerで同一文字列が取得できる
        // 🟡 信頼性レベル: 要件定義書「DBカラムはVARCHAR(500) NOT NULL」L43・database-schema.sql L351からの妥当な推測

        // 【テストデータ準備】: database-schema.sql L351のVARCHAR(500)制約の境界値（ちょうど500文字）を用意する
        // 【初期条件設定】: tmdb行を事前に削除しておく
        let pool = test_pool().await;
        cleanup_provider(&pool, "tmdb").await;
        let long_key = "a".repeat(500);

        // 【実際の処理実行】: 500文字のapi_keyでupsert_api_credentialを呼び出す
        // 【処理内容】: 最大長境界での保存処理
        let result = upsert_api_credential(&pool, ApiProvider::Tmdb, long_key.clone()).await;

        // 【結果検証】: 500文字で切り詰めが発生せず保存成功することを確認する
        // 【期待値確認】: VARCHAR(500)制約付近での極端な入力に対する安定動作を確認する
        let credential = result.expect("500文字のapi_keyのupsertは成功するはず"); // 【確認内容】: 500文字ちょうどのapi_keyがエラーにならず保存されることを確認 🟡
        assert_eq!(credential.api_key.len(), 500); // 【確認内容】: 保存されたapi_keyの長さが500文字のまま切り詰められていないことを確認 🟡
        assert_eq!(credential.api_key, long_key); // 【確認内容】: 保存されたapi_keyの内容が入力と完全一致することを確認 🟡
    }

    /// TC-NEW-09: 同一providerへの連続upsertで重複行が生成されない（境界、実DB統合）
    /// 🔵 信頼性レベル: 要件定義書EDGE-0022-03・database-schema.sql L350・TC-015-03より
    #[tokio::test]
    #[ignore]
    async fn repeated_upserts_do_not_create_duplicate_rows() {
        // 【テスト目的】: tmdbへ複数回upsertしてもapi_credentialsのtmdb行は常に1件であることを確認する
        // 【テスト内容】: ApiProvider::Tmdbに対しapi_key="k1"→"k2"→"k3"と3回連続upsertする
        // 【期待される動作】: 各upsert後もtmdb行は1件のみ。最終的にfind_by_provider(Tmdb)がapi_key=="k3"を返す
        // 🔵 信頼性レベル: 要件定義書EDGE-0022-03・database-schema.sql L350（provider PRIMARY KEY）・TC-015-03より

        // 【テストデータ準備】: upsert回数0→1→2→3の境界（1回目INSERT、2回目以降UPDATE）を確認するための連続入力
        // 【初期条件設定】: tmdb行を事前に削除しておく
        let pool = test_pool().await;
        cleanup_provider(&pool, "tmdb").await;

        // 【実際の処理実行】: 同一providerに対しk1→k2→k3と3回連続でupsertする
        // 【処理内容】: ON CONFLICT (provider) DO UPDATEの繰り返し実行
        upsert_api_credential(&pool, ApiProvider::Tmdb, "k1".to_string())
            .await
            .expect("1回目のupsertは成功するはず");
        upsert_api_credential(&pool, ApiProvider::Tmdb, "k2".to_string())
            .await
            .expect("2回目のupsertは成功するはず");
        upsert_api_credential(&pool, ApiProvider::Tmdb, "k3".to_string())
            .await
            .expect("3回目のupsertは成功するはず");

        // 【結果検証】: 最終的にfind_by_provider(Tmdb)が最後の値"k3"を返すことを確認する
        // 【期待値確認】: PRIMARY KEY制約により重複行が0であることが前提
        let found = find_by_provider(&pool, ApiProvider::Tmdb)
            .await
            .expect("find_by_providerは成功するはず")
            .expect("tmdbレコードは存在するはず"); // 【確認内容】: 連続upsert後もtmdbレコードが取得できることを確認 🔵
        assert_eq!(found.api_key, "k3"); // 【確認内容】: 最終的な値が最後にupsertした"k3"であることを確認 🔵

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM api_credentials WHERE provider = 'tmdb'::api_provider",
        )
        .fetch_one(&pool)
        .await
        .expect("件数取得は成功するはず");
        assert_eq!(count, 1); // 【確認内容】: 3回連続upsertしてもtmdb行が重複せず1件のみであることを確認 🔵
    }
}
