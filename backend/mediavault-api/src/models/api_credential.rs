//! api_credentials（外部APIキー管理）モデル定義
//!
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 外部APIキーのプロバイダ種別
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "api_provider")]
#[serde(rename_all = "snake_case")]
pub enum ApiProvider {
    #[sqlx(rename = "tmdb")]
    Tmdb,
    #[sqlx(rename = "igdb")]
    Igdb,
    #[sqlx(rename = "ndl")]
    Ndl,
    #[sqlx(rename = "steam")]
    Steam,
    #[sqlx(rename = "openlibrary")]
    OpenLibrary,
    #[sqlx(rename = "anilist")]
    AniList,
    // Jikanはキー不要のため対象外 🔵 requirements.mdより
}

/// パスパラメータ文字列を `ApiProvider` に変換する
///
/// 【機能概要】: snake_case文字列（tmdb/igdb/ndl/steam/open_library/ani_list）のみを許可し、
/// それ以外（jikan・大文字TMDB等）は `None` を返す
/// 【実装方針】: TC-015-01-A・TC-015-02-A・TC-NEW-02・TC-NEW-03 を通すための変換関数
pub fn parse_api_provider(raw: &str) -> Option<ApiProvider> {
    // 【実装内容】: 許可された6つのsnake_case文字列のみを厳密一致でenumへ変換する。
    // match文による完全一致のため、大文字・末尾空白等のケース不一致は自動的にNoneとなる。
    // 【実装方針】: serde_json経由の変換も可能だが、match文の方が依存追加不要でシンプルなため採用。
    match raw {
        "tmdb" => Some(ApiProvider::Tmdb),
        "igdb" => Some(ApiProvider::Igdb),
        "ndl" => Some(ApiProvider::Ndl),
        "steam" => Some(ApiProvider::Steam),
        "open_library" => Some(ApiProvider::OpenLibrary),
        "ani_list" => Some(ApiProvider::AniList),
        // 【対象外provider】: jikanはキー不要のためenumに存在せず、ここでNoneとなる 🔵
        _ => None,
    }
}

/// api_credentials テーブル1行を表すモデル
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ApiCredential {
    pub provider: ApiProvider,
    pub api_key: String,
    pub updated_at: NaiveDateTime,
}

/// PUT /settings/api-keys/:provider のリクエストボディDTO
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub api_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // 正常系（ユニット）
    // ============================================================

    /// TC-015-01-A: 全6 provider文字列が`ApiProvider` enumに正しく変換される
    #[test]
    fn parse_api_provider_converts_all_six_allowed_strings() {
        // 【テスト目的】: provider文字列→enum変換ロジックが許可された全6種のsnake_case文字列を正しくマッピングするかを確認する
        // 【テスト内容】: tmdb/igdb/ndl/steam/open_library/ani_listの6文字列それぞれをparse_api_providerに渡す
        // 【期待される動作】: 各文字列が対応するenumバリアントに変換されSomeを返す
        // 🔵 信頼性レベル: 要件定義書L34・REQ-0022-01・note.md L15（ApiProvider定義）より

        // 【テストデータ準備】: types.rs L86-94のApiProvider全バリアントに対応する許可文字列を網羅する
        // 【初期条件設定】: 特になし（純粋関数のためDB等の前提条件は不要）
        let cases = [
            ("tmdb", ApiProvider::Tmdb),
            ("igdb", ApiProvider::Igdb),
            ("ndl", ApiProvider::Ndl),
            ("steam", ApiProvider::Steam),
            ("open_library", ApiProvider::OpenLibrary),
            ("ani_list", ApiProvider::AniList),
        ];

        for (input, expected) in cases {
            // 【実際の処理実行】: parse_api_providerにprovider文字列を渡す
            // 【処理内容】: 文字列→enumバリアントへの変換処理
            let result = parse_api_provider(input);

            // 【結果検証】: 変換結果が期待するenumバリアントと一致するかを確認する
            // 【期待値確認】: OpenLibrary→open_library、AniList→ani_listの複合語snake_case変換が正しいことが特に重要
            assert_eq!(result, Some(expected)); // 【確認内容】: 6 provider全てが正しいenumバリアントに変換されることを確認 🔵
        }
    }

    /// TC-NEW-01: `UpdateApiKeyRequest` DTOが`{ "api_key": "..." }`をデシリアライズする
    /// 🔵 信頼性レベル: 要件定義書L36-43（DTO定義）、タスクファイル実装詳細2 L37-45より
    #[test]
    fn update_api_key_request_deserializes_api_key_field() {
        // 【テスト目的】: リクエストボディDTOのserdeデシリアライズが期待通り機能するかを確認する
        // 【テスト内容】: { "api_key": "xxxxx" } というJSON値をUpdateApiKeyRequestにデシリアライズする
        // 【期待される動作】: api_key=="xxxxx"を保持するUpdateApiKeyRequestが得られる
        // 🔵 信頼性レベル: 要件定義書L36-43（DTO定義）、api-endpoints.md L384-386より

        // 【テストデータ準備】: api-endpoints.md記載のリクエスト例に対応するJSON値
        // 【初期条件設定】: 特になし
        let json = serde_json::json!({ "api_key": "xxxxx" });

        // 【実際の処理実行】: serde_json::from_valueでUpdateApiKeyRequestへデシリアライズする
        // 【処理内容】: serde Deserialize実装による変換処理
        let result: Result<UpdateApiKeyRequest, _> = serde_json::from_value(json);

        // 【結果検証】: デシリアライズが成功し、api_keyフィールドが期待値と一致するかを確認する
        // 【期待値確認】: フィールド名がapi_key（snake_case）であること
        let request = result.expect("UpdateApiKeyRequestのデシリアライズは成功するはず"); // 【確認内容】: 正しいJSONボディがエラーにならず変換できることを確認 🔵
        assert_eq!(request.api_key, "xxxxx"); // 【確認内容】: api_keyフィールドの値が期待した文字列と一致することを確認 🔵
    }

    // ============================================================
    // 異常系（ユニット）
    // ============================================================

    /// TC-015-02-A: 不明なprovider文字列が変換失敗する
    /// 🔵 信頼性レベル: 要件定義書REQ-0022-101・シナリオ2、タスクファイルテストケース2より
    #[test]
    fn parse_api_provider_returns_none_for_unknown_provider() {
        // 【テスト目的】: ApiProvider enumに存在しない文字列が指定された場合の変換ロジック単体の挙動を確認する
        // 【テスト内容】: "unknown_provider"という許可値のいずれにも一致しない文字列を変換する
        // 【期待される動作】: 変換関数がNoneを返す
        // 🔵 信頼性レベル: 要件定義書REQ-0022-101・シナリオ2（TC-015-02）、タスクファイルテストケース2より

        // 【テストデータ準備】: クライアントのタイプミスや未対応provider指定を想定した代表的な不正文字列
        // 【初期条件設定】: 特になし
        let input = "unknown_provider";

        // 【実際の処理実行】: parse_api_providerに不正provider文字列を渡す
        // 【処理内容】: 変換失敗時の挙動を確認する処理
        let result = parse_api_provider(input);

        // 【結果検証】: 変換失敗時にNoneが返ることを確認する
        // 【期待値確認】: 変換失敗時点でDBアクセスに進まないことを保証する設計の起点となる
        assert_eq!(result, None); // 【確認内容】: 許可値外の文字列がNoneとして変換失敗することを確認 🔵
    }

    /// TC-NEW-02: jikanは`ApiProvider`に含まれずINVALID_PROVIDERになる（ユニット部分）
    /// 🔵 信頼性レベル: 要件定義書EDGE-0022-01・「対象外」L24、note.md L6・タスクファイル注意事項L98より
    #[test]
    fn parse_api_provider_returns_none_for_jikan() {
        // 【テスト目的】: キー不要のためenum対象外とされたjikanが変換失敗することを明示的に確認する
        // 【テスト内容】: "jikan"という文字列を変換する
        // 【期待される動作】: 変換関数がNoneを返す（jikanはApiProvider enumに含まれない）
        // 🔵 信頼性レベル: 要件定義書EDGE-0022-01・「対象外」L24、note.md L6より

        // 【テストデータ準備】: 「キー不要provider」を誤って受理しないことが明示仕様であるための代表入力
        // 【初期条件設定】: 特になし
        let input = "jikan";

        // 【実際の処理実行】: parse_api_providerにjikanを渡す
        // 【処理内容】: 対象外provider判定の確認
        let result = parse_api_provider(input);

        // 【結果検証】: jikanがNoneとして変換失敗することを確認する
        // 【期待値確認】: 仕様上の「対象外provider」の境界を回帰テストで固定する
        assert_eq!(result, None); // 【確認内容】: jikanがApiProvider enumに含まれず変換失敗することを確認 🔵
    }

    /// TC-NEW-03: 大文字`TMDB`などsnake_case不一致はINVALID_PROVIDER
    /// 🔵 信頼性レベル: 要件定義書EDGE-0022-01（大文字TMDB例）より
    #[test]
    fn parse_api_provider_returns_none_for_case_mismatched_strings() {
        // 【テスト目的】: 正しいproviderの大文字/別ケース表記がenumにマッチしないことを確認する
        // 【テスト内容】: "TMDB"（大文字）、"Tmdb"（先頭大文字）、"tmdb "（末尾空白）の3パターンを変換する
        // 【期待される動作】: いずれも変換失敗（None）となる
        // 🔵 信頼性レベル: 要件定義書EDGE-0022-01（大文字TMDB例）より

        // 【テストデータ準備】: #[serde(rename_all = "snake_case")]は大文字小文字を厳密に区別するため、
        // 正しいproviderのケース違い・空白付き表記を代表的な不正入力として用意する
        // 【初期条件設定】: 特になし
        let inputs = ["TMDB", "Tmdb", "tmdb "];

        for input in inputs {
            // 【実際の処理実行】: parse_api_providerにケース不一致の文字列を渡す
            // 【処理内容】: 厳密一致照合の確認
            let result = parse_api_provider(input);

            // 【結果検証】: ケース不一致の文字列がNoneとして変換失敗することを確認する
            // 【期待値確認】: ケースセンシティブなprovider照合の堅牢性を確認する
            assert_eq!(result, None, "input={input:?} はNoneを返すはず"); // 【確認内容】: 大文字・空白付き等のケース不一致がいずれも変換失敗することを確認 🔵
        }
    }
}
