//! GET /items/search 用クエリDTO
//!
//! TASK-0024: GET /items/search 外部API検索エンドポイント実装
//!
//! 【信頼性レベル】: 🔵 要件定義書 第2章・テストケース定義書 TC-0024-N04/B03より

use serde::Deserialize;

use crate::models::item::MediaType;

/// `GET /items/search` クエリパラメータDTO
///
/// 【機能概要】: media_type（必須）・q（必須・検索語）を受け取るためのAxum `Query` extractor用DTO
/// 【実装方針】: 両フィールドとも必須（Option化しない）。Axumのデシリアライズ失敗時は
/// extractorレベルで400 Bad Requestとなる
/// 【テスト対応】: TC-0024-N04（デシリアライズ成功）, TC-0024-B03（MediaType全variant網羅）
/// 🔵 信頼性レベル: 要件定義書 2.1 入力仕様表・DTO定義に直接対応
#[derive(Debug, Deserialize)]
pub struct ItemSearchQuery {
    pub media_type: MediaType,
    pub q: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-0024-N04: ItemSearchQuery クエリ文字列デシリアライズ成功
    /// 🔵 信頼性レベル: 要件 2.1 DTO 定義・models/item.rs MediaType（snake_case Deserialize 実装済み）より
    #[test]
    fn item_search_query_deserializes_media_type_and_q() {
        // 【テスト目的】: media_type=anime&q=fooのクエリ文字列がItemSearchQuery { media_type: Anime, q: "foo" }
        // にデシリアライズされることを確認する
        // 【テスト内容】: serde_urlencodedによる相当のデシリアライズ（serde_json::from_valueで模擬）を実行する
        // 【期待される動作】: media_type==MediaType::Anime、q=="foo"
        // 🔵 信頼性レベル: 要件 2.1 DTO 定義（pub media_type: MediaType, pub q: String）に対応
        // 【Red期待】: ItemSearchQueryがまだ存在しないため、本ファイル自体がコンパイルエラーとなる想定

        // 【テストデータ準備】: DTOが必須2フィールドを正しく受理する最小正常ケース
        // 【初期条件設定】: クエリ文字列相当のJSON値を用意する
        let value = serde_json::json!({ "media_type": "anime", "q": "foo" });

        // 【実際の処理実行】: ItemSearchQueryへのデシリアライズを実行する
        // 【処理内容】: serde::Deserialize実装によるJSON→構造体変換
        let query: ItemSearchQuery =
            serde_json::from_value(value).expect("デシリアライズは成功するはず");

        // 【結果検証】: media_type/qが期待値どおりであることを確認
        // 【期待値確認】: snake_caseの"anime"文字列がMediaType::Animeへ正しく変換されることを確認
        assert_eq!(query.media_type, MediaType::Anime); // 【確認内容】: media_typeがAnimeへ変換されることを確認 🔵
        assert_eq!(query.q, "foo"); // 【確認内容】: qがそのまま保持されることを確認 🔵
    }

    /// TC-0024-B03: MediaType 全8variantが media_type として受理される（境界・ユニット）
    /// 🔵 信頼性レベル: 要件 2.1 許容値・models/item.rs L15-24（既存 snake_case Deserialize）より
    #[test]
    fn item_search_query_accepts_all_eight_media_type_variants() {
        // 【テスト目的】: anime/movie/drama/manga/novel/game/academic_book/paperの8値すべてが
        // ItemSearchQueryにデシリアライズ可能であることを確認する
        // 【テスト内容】: 8パターンのmedia_type文字列をそれぞれデシリアライズする
        // 【期待される動作】: 8 variantすべてが対応するenum値へ変換され、デシリアライズが成功する
        // 🔵 信頼性レベル: 要件 2.1 許容値8種・models/item.rs L15-24に直接対応

        // 【テストデータ準備】: 許容される8つのsnake_case文字列と対応する期待MediaTypeのペア
        // 【初期条件設定】: 全variant網羅による取りこぼし検出を意図する
        let cases = [
            ("anime", MediaType::Anime),
            ("movie", MediaType::Movie),
            ("drama", MediaType::Drama),
            ("manga", MediaType::Manga),
            ("novel", MediaType::Novel),
            ("game", MediaType::Game),
            ("academic_book", MediaType::AcademicBook),
            ("paper", MediaType::Paper),
        ];

        for (raw, expected) in cases {
            // 【実際の処理実行】: 各media_type文字列でItemSearchQueryへデシリアライズする
            let value = serde_json::json!({ "media_type": raw, "q": "x" });
            let query: ItemSearchQuery = serde_json::from_value(value)
                .unwrap_or_else(|e| panic!("{raw}のデシリアライズに失敗: {e}"));

            // 【結果検証】: 各variantが正しい対応enum値になることを確認
            assert_eq!(query.media_type, expected); // 【確認内容】: snake_case文字列と enum の1対1対応を確認 🔵
        }
    }

    /// TC-0024-B03(集合外): media_type=unknown は受理されず拒否される（境界・ユニット）
    /// 🔵 信頼性レベル: 要件 2.1 許容値8種からの境界（集合外1件）の妥当な推測
    #[test]
    fn item_search_query_rejects_media_type_outside_enum_set() {
        // 【テスト目的】: MediaType enumに存在しない文字列がデシリアライズ失敗（Err）になることを確認する
        // 【テスト内容】: media_type="unknown"でデシリアライズを試みる
        // 【期待される動作】: Errが返る
        // 🔵 信頼性レベル: 要件 2.1 許容値8種・受理集合の内側/外側の一貫動作より

        // 【テストデータ準備】: 8 variantに含まれない不正値
        let value = serde_json::json!({ "media_type": "unknown", "q": "x" });

        // 【実際の処理実行】: ItemSearchQueryへのデシリアライズを試みる
        let result: Result<ItemSearchQuery, _> = serde_json::from_value(value);

        // 【結果検証】: 集合外の値はデシリアライズに失敗することを確認
        assert!(result.is_err()); // 【確認内容】: 許容8値に含まれないmedia_typeが拒否されることを確認 🔵
    }
}
