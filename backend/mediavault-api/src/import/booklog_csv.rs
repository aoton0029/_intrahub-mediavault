//! ブクログCSVのカラムマッピング・行単位パーサ・バリデーション
//!
//! TASK-0030: ブクログCSVインポート実装
//!
//! 【機能概要】: ブクログのエクスポートCSVの仮カラムフォーマット（作品名/感想レビュー/読了日/評価/ISBN）を
//! `BooklogCsvRow`構造体にマッピングし、`csv`crateで1行ずつデシリアライズ・バリデーションした結果を
//! `CreateItemRequest`相当へ変換する
//! 【実装方針】: カラム名・型変換ロジックを本ファイルに閉じ込め、実サンプル確認後は`#[serde(rename = "...")]`の
//! 値のみを差分修正できる構造とする（要件定義書3章「カラムマッピングの分離」最重要設計制約）
//! 【テスト対応】: TC-N-02, TC-N-03, TC-N-06, TC-N-07, TC-N-08, TC-E-01〜TC-E-05, TC-B-04, TC-B-06
//! 🔵🟡 信頼性レベル: 仮カラムフォーマットは🟡（実サンプル未確認）、行単位スキップ・必須/型変換ロジックは🔵（EDGE-002直結）

use chrono::NaiveDate;
use serde::Deserialize;

use crate::models::import::ImportFailure;
use crate::models::item::{CreateItemRequest, MediaType};

/// ブクログCSVの仮カラムフォーマットに対応するデシリアライズ用の生データ構造体
///
/// 【機能概要】: `csv`crateのヘッダーベースデシリアライズ対象。各フィールドは文字列のまま受け取り、
/// 型変換（日付・数値）は`parse_booklog_csv_row`側で行う（デシリアライズ自体は失敗させず、
/// バリデーション層で理由を分けて記録するため）
/// 【実装方針】: 必須カラム`作品名`のみ`String`、他は`Option<String>`として「空文字列」と「列自体が無い」を
/// 区別しすぎない方針とする（仮カラム定義は固定5列を前提とするため）
/// 🟡 信頼性レベル: prep.md「ブクログCSVサンプル準備」より実物未確認のため仮定義
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BooklogCsvRow {
    #[serde(rename = "作品名")]
    pub title: String,
    #[serde(rename = "感想/レビュー")]
    pub description: Option<String>,
    #[serde(rename = "読了日")]
    pub consumed_date_raw: Option<String>,
    #[serde(rename = "評価")]
    pub rating_raw: Option<String>,
    #[serde(rename = "ISBN")]
    pub external_id: Option<String>,
}

/// ブクログCSVヘッダー行の固定文字列（仮定義）
///
/// 【機能概要】: テストコード内で共通利用するヘッダー定数。実サンプル確認後はこの1箇所のみ
/// 修正すればよい構造とする
/// 【dead_code対応】: 本番ロジック（`parse_booklog_csv`等）はcsv crateのヘッダーベース
/// デシリアライズ（`#[serde(rename)]`）に依存するため、この定数自体は本番コードパスでは
/// 参照されない。テストヘルパー専用の定数であるため`#[cfg(test)]`を付与し、
/// dead_code警告を解消する 🟡
/// 🟡 信頼性レベル: 仮カラムフォーマット定義（要件定義書2章）より
#[cfg(test)]
pub const BOOKLOG_CSV_HEADER: &str = "作品名,感想/レビュー,読了日,評価,ISBN";

/// `BooklogCsvRow`をバリデーション・型変換し、`CreateItemRequest`へ変換する。
///
/// 【機能概要】: 必須項目（作品名）の空チェック、読了日・評価の型変換を行い、いずれかに失敗した場合は
/// `ImportFailure`（row_number, reason）を返す。すべて成功した場合は`CreateItemRequest`を返す
/// 【実装方針】:
///   1. titleをtrim().is_empty()で判定（空・空白のみは不正、TC-E-01・TC-B-06）
///   2. consumed_date_rawが空文字でなければ`YYYY-MM-DD`としてパース（失敗時は"invalid date format"、TC-E-02）
///   3. rating_rawが空文字でなければ`f32`としてパース（失敗時は"invalid rating"、TC-E-03）
///   4. media_typeは固定でNovel（設計判断#2、TC-N-06）
///   5. external_idは空文字をNoneへ正規化（TC-N-03・TC-N-07）
/// 【テスト対応】: TC-N-02, TC-N-03, TC-N-06, TC-N-07, TC-E-01, TC-E-02, TC-E-03, TC-B-06
/// 🔵 信頼性レベル: 要件定義書3章・4章（型変換失敗時のスキップ＋reason記録）に直接対応
pub fn parse_booklog_csv_row(
    row: BooklogCsvRow,
    row_number: u32,
) -> Result<CreateItemRequest, ImportFailure> {
    // 【必須項目チェック】: title（作品名）がtrim後に空ならば不正行として扱う（TC-E-01・TC-B-06） 🔵
    if row.title.trim().is_empty() {
        return Err(ImportFailure::new(row_number, "title is empty"));
    }

    // 【任意列の空文字→None正規化】: descriptionは空文字を「未入力」とみなしNoneにする。
    // ISBN(external_id)は別途extract_external_id()で抽出する（create_item_with_sourceの
    // 専用引数として渡すため、CreateItemRequest自体にはexternal_idフィールドが存在しない） 🟡
    let description = normalize_optional_field(row.description);

    // 【読了日パース】: "YYYY-MM-DD"形式以外はinvalid date formatとしてスキップする（TC-E-02） 🔵
    let consumed_date = match normalize_optional_field(row.consumed_date_raw) {
        None => None,
        Some(raw) => match NaiveDate::parse_from_str(&raw, "%Y-%m-%d") {
            Ok(date) => Some(date),
            Err(_) => return Err(ImportFailure::new(row_number, "invalid date format")),
        },
    };

    // 【評価パース】: f32にパースできない場合はinvalid ratingとしてスキップする（TC-E-03） 🔵
    let rating = match normalize_optional_field(row.rating_raw) {
        None => None,
        Some(raw) => match raw.parse::<f32>() {
            Ok(value) => Some(value),
            Err(_) => return Err(ImportFailure::new(row_number, "invalid rating")),
        },
    };

    // 【CreateItemRequestへの変換】: media_typeはブクログ＝書籍前提でNovel固定（設計判断#2、TC-N-06） 🔵
    Ok(CreateItemRequest {
        media_type: MediaType::Novel,
        title: row.title,
        original_title: None,
        description,
        cover_image_url: None,
        release_date: None,
        homepage_url: None,
        rating,
        is_favorite: None,
        details: None,
        consumed_date,
    })
}

/// `BooklogCsvRow`のISBN列から`external_id`相当の文字列を抽出する。
///
/// 【機能概要】: ハンドラ層（`handlers/import_booklog.rs`、Greenフェーズで実装）が
/// `item_repository::create_item_with_source`へ渡す`external_id`引数を組み立てる際に使う、
/// ISBN列専用の抽出ヘルパー。空文字列は`None`へ正規化する
/// 【実装方針】: 本関数は意図的に未実装（Redフェーズ）とし、Greenフェーズで
/// `normalize_optional_field`を呼び出すだけの薄い実装へ差し替える
/// 【テスト対応】: TC-N-07
/// 🔵 信頼性レベル: 要件定義書3章「ISBN→external_id」マッピングに基づく
pub fn extract_external_id(row: &BooklogCsvRow) -> Option<String> {
    // 【Green実装】: ISBN列の値をtrimし、空文字列ならNoneへ正規化する（TC-N-07・TC-N-03整合） 🔵
    normalize_optional_field(row.external_id.clone())
}

/// 空文字列（trim後に空）を`None`へ正規化する。
///
/// 【機能概要】: CSVの任意列は「空文字列」として渡ってくることが多いため、`Some("")`のまま
/// `CreateItemRequest`へ流し込まないよう正規化する
/// 【テスト対応】: TC-N-03（任意列空→None）
/// 🟡 信頼性レベル: 要件定義書4章「任意列空文字を不正扱いしない」の境界明確化からの妥当な推測
fn normalize_optional_field(value: Option<String>) -> Option<String> {
    match value {
        Some(text) if !text.trim().is_empty() => Some(text),
        _ => None,
    }
}

/// パース成功行1件分のデータ。DB登録時まで`row_number`・`external_id`(ISBN)を保持するために
/// `CreateItemRequest`と一緒に保持する。
///
/// 【機能概要】: ハンドラ層が`item_repository::create_item_with_source`へ渡す`external_id`引数と、
/// DB登録失敗時にfailureへ記録する`row_number`を、パース時点から登録時点まで保持するための運搬用構造体
/// 【実装方針】: Redフェーズでは`row_number=0`固定・`extract_external_id_placeholder()`が
/// `unimplemented!()`のままだったため、Greenフェーズで`parse_booklog_csv`の戻り値構造をこの形へ変更する
/// 🔵 信頼性レベル: Red記録「次のフェーズへの要求事項1」に基づく
#[derive(Debug, Clone)]
pub struct ParsedBooklogRow {
    pub row_number: u32,
    pub request: CreateItemRequest,
    pub external_id: Option<String>,
}

/// CSVバイト列を行単位でパースし、成功行・失敗行を分離して返す。
///
/// 【機能概要】: `csv`crateでヘッダー行を読み、データ行を1行ずつ`BooklogCsvRow`へデシリアライズする。
/// デシリアライズ自体が失敗する行（カラム数不足等）も`ImportFailure`としてスキップ扱いにする
/// 【実装方針】: イテレータ処理中にパニックしないよう、各行の処理結果を`Result`として個別にハンドリングし、
/// 1行の失敗が他行の処理を止めないようにする（EDGE-002・TC-016-E01）。row_numberはヘッダーを除いた
/// データ行基準で1始まりとする（TC-B-04）。成功行は`ParsedBooklogRow`として`row_number`・`external_id`
/// （ISBN）も併せて保持し、ハンドラ側でDB登録時まで失わないようにする
/// 【テスト対応】: TC-N-08, TC-E-04, TC-E-05, TC-B-01, TC-B-02, TC-B-04
/// 🔵 信頼性レベル: 要件定義書2章データフロー・EDGE-002・TC-016-E01に直接対応
pub fn parse_booklog_csv(bytes: &[u8]) -> (Vec<ParsedBooklogRow>, Vec<ImportFailure>) {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    let mut reader = csv::ReaderBuilder::new().has_headers(true).from_reader(bytes);

    // 【行単位処理】: csv crateのレコードイテレータで1行ずつデシリアライズ・バリデーションする 🔵
    for (idx, result) in reader.deserialize::<BooklogCsvRow>().enumerate() {
        // 【row_number算出】: ヘッダー行を除いたデータ行基準で1始まり（idxは0始まりのため+1する） 🔵
        let row_number = (idx + 1) as u32;

        match result {
            // 【デシリアライズ成功】: バリデーション・型変換へ進む 🔵
            Ok(row) => {
                // 【external_id抽出】: バリデーション前にISBN列を抽出しておく（rowはparse_booklog_csv_rowに
                // 所有権を渡すため、先に複製しておく必要がある） 🔵
                let external_id = extract_external_id(&row);
                match parse_booklog_csv_row(row, row_number) {
                    Ok(request) => successes.push(ParsedBooklogRow {
                        row_number,
                        request,
                        external_id,
                    }),
                    Err(failure) => failures.push(failure),
                }
            }
            // 【デシリアライズ失敗】: カラム数不足等。パニックせずImportFailureとして記録する（TC-E-04） 🔵
            Err(err) => {
                tracing::error!("booklog csv row deserialize error: {err}");
                failures.push(ImportFailure::new(row_number, "invalid row format"));
            }
        }
    }

    (successes, failures)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用ヘルパー: ヘッダー + 1データ行のCSVバイト列を構築する
    fn csv_bytes(data_rows: &[&str]) -> Vec<u8> {
        let mut content = String::from(BOOKLOG_CSV_HEADER);
        content.push_str("\r\n");
        for row in data_rows {
            content.push_str(row);
            content.push_str("\r\n");
        }
        content.into_bytes()
    }

    /// TC-N-02: 正常行デシリアライズ（BooklogCsvRow単体）
    /// 【テスト目的】: csv crateのヘッダーベースデシリアライズが`#[serde(rename)]`を介して
    /// 日本語ヘッダー（スラッシュ含む「感想/レビュー」）を正しく解釈することを確認する
    /// 【テスト内容】: ヘッダー＋1データ行（全列値あり）をBooklogCsvRowへデシリアライズする
    /// 【期待される動作】: title="吾輩は猫である", description=Some("面白い"),
    /// consumed_date_raw=Some("2024-01-15"), rating_raw=Some("4.5"), external_id=Some("9784...")
    /// 🟡 信頼性レベル: 仮カラム定義（実サンプル未確認）に基づくため黄信号
    #[test]
    fn booklog_csv_row_deserializes_from_japanese_header() {
        // 【テストデータ準備】: 5列すべてに値がある正常な1データ行を用意する
        // 【初期条件設定】: カラムマッピング分離設計（最重要制約）の検証用最小ケース
        let bytes = csv_bytes(&["吾輩は猫である,面白い,2024-01-15,4.5,9784101010014"]);

        // 【実際の処理実行】: csv crateで1行デシリアライズする
        // 【処理内容】: BooklogCsvRowへのヘッダーベースマッピングを確認する
        let mut reader = csv::ReaderBuilder::new().has_headers(true).from_reader(bytes.as_slice());
        let row: BooklogCsvRow = reader.deserialize().next().unwrap().unwrap();

        // 【結果検証】: 各フィールドが期待値どおりであることを確認
        assert_eq!(row.title, "吾輩は猫である"); // 【確認内容】: "作品名"列がtitleへマッピングされることを確認 🟡
        assert_eq!(row.description, Some("面白い".to_string())); // 【確認内容】: "感想/レビュー"列がdescriptionへマッピングされることを確認 🟡
        assert_eq!(row.consumed_date_raw, Some("2024-01-15".to_string())); // 【確認内容】: "読了日"列がconsumed_date_rawへマッピングされることを確認 🟡
        assert_eq!(row.rating_raw, Some("4.5".to_string())); // 【確認内容】: "評価"列がrating_rawへマッピングされることを確認 🟡
        assert_eq!(row.external_id, Some("9784101010014".to_string())); // 【確認内容】: "ISBN"列がexternal_idへマッピングされることを確認 🟡
    }

    /// TC-N-03: 任意列が空でも正常登録される（description/読了日/評価/ISBN空文字→None）
    /// 【テスト目的】: 必須の作品名のみ値があり、他4列が空文字の行が有効行として変換されることを確認する
    /// 【テスト内容】: 作品名=星の王子さま、他4列すべて空文字のBooklogCsvRowをparse_booklog_csv_rowへ渡す
    /// 【期待される動作】: Ok(CreateItemRequest)が返り、description/consumed_date/rating/external_idが
    /// すべてNone
    /// 🟡 信頼性レベル: 仮カラム定義の「任意」属性からの妥当な推測
    #[test]
    fn parse_booklog_csv_row_treats_blank_optional_columns_as_none() {
        // 【テストデータ準備】: 必須列のみ値があり、他4列が空文字の行を用意する
        // 【初期条件設定】: 任意列の空は不正ではないという仕様（要件定義2章「任意」）の境界明確化
        let row = BooklogCsvRow {
            title: "星の王子さま".to_string(),
            description: Some(String::new()),
            consumed_date_raw: Some(String::new()),
            rating_raw: Some(String::new()),
            external_id: Some(String::new()),
        };

        // 【実際の処理実行】: parse_booklog_csv_rowを呼び出す
        // 【処理内容】: 必須/任意の判定・空文字→None正規化ロジックを確認する
        let request = parse_booklog_csv_row(row, 1).unwrap();

        // 【結果検証】: 任意列がすべてNoneへ正規化されていることを確認
        assert_eq!(request.title, "星の王子さま"); // 【確認内容】: titleが正しく保持されることを確認 🟡
        assert_eq!(request.description, None); // 【確認内容】: 空文字のdescriptionがNoneへ正規化されることを確認 🟡
        assert_eq!(request.consumed_date, None); // 【確認内容】: 空文字のconsumed_dateがNoneへ正規化されることを確認 🟡
        assert_eq!(request.rating, None); // 【確認内容】: 空文字のratingがNoneへ正規化されることを確認 🟡
    }

    /// TC-N-06: media_typeがNovel固定で変換される（設計判断#2）
    /// 【テスト目的】: CSVにmedia_type列が無くても、変換後のCreateItemRequest.media_typeが常に
    /// MediaType::Novelになることを確認する
    /// 【テスト内容】: 正常な1行をparse_booklog_csv_rowへ渡す
    /// 【期待される動作】: request.media_type == MediaType::Novel
    /// 🔵 信頼性レベル: 設計判断#2（ユーザー確定）に基づく
    #[test]
    fn parse_booklog_csv_row_assigns_fixed_novel_media_type() {
        // 【テストデータ準備】: media_type列が存在しない仮カラム定義に従った正常行を用意する
        let row = BooklogCsvRow {
            title: "ノルウェイの森".to_string(),
            description: None,
            consumed_date_raw: None,
            rating_raw: None,
            external_id: None,
        };

        // 【実際の処理実行】: parse_booklog_csv_rowを呼び出す
        // 【処理内容】: 固定media_type付与ロジックを確認する
        let request = parse_booklog_csv_row(row, 1).unwrap();

        // 【結果検証】: media_typeが常にNovelであることを確認
        assert_eq!(request.media_type, MediaType::Novel); // 【確認内容】: media_type列が無くてもNovel固定で付与されることを確認 🔵
    }

    /// TC-N-07: ISBN列の抽出関数がISBN空時にNoneを返すことを確認する
    /// （source=Manual固定・external_idへの実際の引き渡しはhandlers/import_booklog.rs側の責務であり、
    /// 本パーサ層では「ISBN文字列の抽出・空文字正規化」までを検証する）
    /// 【テスト目的】: ISBN列の値が抽出関数で正しく取得でき、ISBN空文字時はNoneになることを確認する
    /// 【テスト内容】: ISBN="9784101010014"の行とISBN=Some("")の行をそれぞれextract_external_idへ渡す
    /// 【期待される動作】: 前者はSome("9784101010014")、後者はNone
    /// 🔵 信頼性レベル: 要件定義3章「既存コードとの整合制約」・TC-N-03との整合に基づく
    #[test]
    fn extract_external_id_maps_isbn_and_normalizes_blank_to_none() {
        // 【テストデータ準備】: ISBN値あり/ISBN空文字の2パターンを用意する
        let row_with_isbn = BooklogCsvRow {
            title: "斜陽".to_string(),
            description: None,
            consumed_date_raw: None,
            rating_raw: None,
            external_id: Some("9784101010014".to_string()),
        };
        let row_without_isbn = BooklogCsvRow {
            title: "斜陽".to_string(),
            description: None,
            consumed_date_raw: None,
            rating_raw: None,
            external_id: Some(String::new()),
        };

        // 【実際の処理実行】: それぞれextract_external_idを呼び出す
        // 【処理内容】: ISBN→external_id抽出のマッピング経路を確認する
        let external_id_with_isbn = extract_external_id(&row_with_isbn);
        let external_id_without_isbn = extract_external_id(&row_without_isbn);

        // 【結果検証】: ISBN値の有無に応じてexternal_idが正しく設定されることを確認
        assert_eq!(
            external_id_with_isbn,
            Some("9784101010014".to_string())
        ); // 【確認内容】: ISBN値ありの場合external_idへ反映されることを確認 🔵
        assert_eq!(external_id_without_isbn, None); // 【確認内容】: ISBN空文字の場合external_idがNoneになることを確認（TC-N-03と整合） 🔵
    }

    /// TC-E-01: 作品名(title)空行のスキップ＋failure記録（"title is empty"）
    /// 【テスト目的】: 必須カラム「作品名」が空文字の行を不正行として扱い、reasonが
    /// "title is empty"で記録されることを確認する
    /// 【テスト内容】: title=""・他列妥当な値のBooklogCsvRowをparse_booklog_csv_rowへ渡す
    /// 【期待される動作】: Err(ImportFailure{row_number, reason: "title is empty"})
    /// 🔵 信頼性レベル: 要件定義書4章・note.md「title is empty」の例示文言に直接対応
    #[test]
    fn parse_booklog_csv_row_skips_empty_title() {
        // 【テストデータ準備】: 必須title空・他列妥当の1行を用意する理由＝空title排除仕様の検証
        let row = BooklogCsvRow {
            title: String::new(),
            description: Some("面白い".to_string()),
            consumed_date_raw: Some("2024-01-15".to_string()),
            rating_raw: Some("4.5".to_string()),
            external_id: Some("9784101010014".to_string()),
        };

        // 【実際の処理実行】: 行単位パーサ/バリデーションを呼び出す
        let result = parse_booklog_csv_row(row, 1);

        // 【結果検証】: スキップ理由が"title is empty"であることを確認
        let failure = result.unwrap_err();
        assert_eq!(failure.row_number, 1); // 【確認内容】: row_numberが渡した値どおりであることを確認 🔵
        assert_eq!(failure.reason, "title is empty"); // 【確認内容】: reason文言が"title is empty"であることを確認 🔵
    }

    /// TC-E-02: 読了日が不正形式の行スキップ（invalid date format）
    /// 【テスト目的】: 読了日がYYYY-MM-DD形式でない行を不正行として扱い、reasonが
    /// "invalid date format"で記録されることを確認する
    /// 【テスト内容】: 読了日="2024/13/40"（不正日付）の行をparse_booklog_csv_rowへ渡す
    /// 【期待される動作】: Err(ImportFailure{reason: "invalid date format"})
    /// 🔵 信頼性レベル: 要件定義書4章「日付/数値の型変換失敗」に対応
    #[test]
    fn parse_booklog_csv_row_skips_invalid_date_format() {
        // 【テストデータ準備】: 月13・日40という不正な日付を持つ行を用意する
        // 【初期条件設定】: ロケール差異・手編集による日付フォーマット崩れを再現する
        let row = BooklogCsvRow {
            title: "x".to_string(),
            description: None,
            consumed_date_raw: Some("2024/13/40".to_string()),
            rating_raw: None,
            external_id: None,
        };

        // 【実際の処理実行】: parse_booklog_csv_rowを呼び出す
        let result = parse_booklog_csv_row(row, 5);

        // 【結果検証】: スキップ理由が"invalid date format"であることを確認
        let failure = result.unwrap_err();
        assert_eq!(failure.row_number, 5); // 【確認内容】: row_numberが渡した値どおりであることを確認 🔵
        assert_eq!(failure.reason, "invalid date format"); // 【確認内容】: reason文言が"invalid date format"であることを確認 🔵
    }

    /// TC-E-03: 評価が数値でない行スキップ（invalid rating）
    /// 【テスト目的】: 評価がf32にパースできない行を不正行として扱い、reasonが
    /// "invalid rating"で記録されることを確認する
    /// 【テスト内容】: 評価="とても良い"（非数値）の行をparse_booklog_csv_rowへ渡す
    /// 【期待される動作】: Err(ImportFailure{reason: "invalid rating"})
    /// 🔵 信頼性レベル: 要件定義書4章「日付/数値の型変換失敗」に対応
    #[test]
    fn parse_booklog_csv_row_skips_invalid_rating() {
        // 【テストデータ準備】: ブクログの星表記やコメント混入を模した非数値の評価を用意する
        let row = BooklogCsvRow {
            title: "x".to_string(),
            description: None,
            consumed_date_raw: None,
            rating_raw: Some("とても良い".to_string()),
            external_id: None,
        };

        // 【実際の処理実行】: parse_booklog_csv_rowを呼び出す
        let result = parse_booklog_csv_row(row, 3);

        // 【結果検証】: スキップ理由が"invalid rating"であることを確認
        let failure = result.unwrap_err();
        assert_eq!(failure.row_number, 3); // 【確認内容】: row_numberが渡した値どおりであることを確認 🔵
        assert_eq!(failure.reason, "invalid rating"); // 【確認内容】: reason文言が"invalid rating"であることを確認 🔵
    }

    /// TC-E-04: カラム数不足行のスキップ（deserialize失敗、reason="invalid row format"）
    /// 【テスト目的】: ヘッダー5列に対し2列しかないデータ行が、csv crateのデシリアライズ失敗として
    /// 検知され、パニックせずImportFailureとしてスキップされることを確認する
    /// 【テスト内容】: ヘッダー5列・データ行「斜陽,面白い」（2列のみ）のCSVバイト列をparse_booklog_csvへ渡す
    /// 【期待される動作】: successesが空、failuresに1件（reason="invalid row format"）
    /// 🔵 信頼性レベル: 要件定義書4章「カラム数不足行」に対応（reason文言は実装確定のため分類は青・文言は黄寄り）
    #[test]
    fn parse_booklog_csv_skips_row_with_insufficient_columns() {
        // 【テストデータ準備】: ヘッダー5列に対しデータ行が2列のみの破損CSVを用意する
        // 【初期条件設定】: CSV破損・改行混入・エクスポート不具合を再現する
        let bytes = csv_bytes(&["斜陽,面白い"]);

        // 【実際の処理実行】: parse_booklog_csvを呼び出す
        // 【処理内容】: csv crateのデシリアライズ失敗を検知し、行スキップで処理継続するロジックを確認する
        let (successes, failures) = parse_booklog_csv(&bytes);

        // 【結果検証】: 成功0件・失敗1件であり、パニックせず処理が完走することを確認
        assert_eq!(successes.len(), 0); // 【確認内容】: カラム数不足行が成功扱いにならないことを確認 🔵
        assert_eq!(failures.len(), 1); // 【確認内容】: カラム数不足行が1件のImportFailureとして記録されることを確認 🔵
        assert_eq!(failures[0].row_number, 1); // 【確認内容】: row_numberが1（ヘッダー除く先頭データ行）であることを確認 🔵
    }

    /// TC-E-05: 複数の不正行が独立したImportFailureとして記録される
    /// 【テスト目的】: 複数の不正行（理由がそれぞれ異なる）があるとき、各行が独立した
    /// ImportFailureとして正確なrow_number・reasonで記録されることを確認する
    /// 【テスト内容】: 行1=正常、行2=title空、行3=正常、行4=日付不正 の4行CSVをparse_booklog_csvへ渡す
    /// 【期待される動作】: failures.len()==2、{row_number:2, reason:"title is empty"}と
    /// {row_number:4, reason:"invalid date format"}を含む。success_count相当のsuccesses.len()==2
    /// 🔵 信頼性レベル: 要件定義2章 ImportFailure 仕様・EDGE-002 に直接対応
    #[test]
    fn parse_booklog_csv_records_multiple_independent_failures() {
        // 【テストデータ準備】: 正常・不正(title空)・正常・不正(日付不正)の4行を用意する
        // 【初期条件設定】: 大規模CSVで複数種の不正が混在する典型シナリオを再現する
        let bytes = csv_bytes(&[
            "吾輩は猫である,面白い,2024-01-15,4.5,9784101010014",
            ",面白い,2024-01-15,4.5,9784101010014",
            "星の王子さま,,, ,",
            "破戒,,2024/13/40,,",
        ]);

        // 【実際の処理実行】: parse_booklog_csvを呼び出す
        // 【処理内容】: 複数行の独立した失敗記録・継続処理を確認する
        let (successes, failures) = parse_booklog_csv(&bytes);

        // 【結果検証】: success/failureの件数・各failureのrow_number・reasonが正確であることを確認
        assert_eq!(successes.len(), 2); // 【確認内容】: 正常2行（行1・行3）が登録対象になることを確認 🔵
        assert_eq!(failures.len(), 2); // 【確認内容】: 不正2行（行2・行4）がそれぞれ独立記録されることを確認 🔵
        assert_eq!(failures[0].row_number, 2); // 【確認内容】: 1件目の不正がrow_number=2であることを確認 🔵
        assert_eq!(failures[0].reason, "title is empty"); // 【確認内容】: 1件目の理由が"title is empty"であることを確認 🔵
        assert_eq!(failures[1].row_number, 4); // 【確認内容】: 2件目の不正がrow_number=4であることを確認 🔵
        assert_eq!(failures[1].reason, "invalid date format"); // 【確認内容】: 2件目の理由が"invalid date format"であることを確認 🔵
    }

    /// TC-N-08: 部分不正混在CSVで正常行のみ登録される（EDGE-002継続性、パーサ単体での確認）
    /// 【テスト目的】: 不正行（title空）の後続行が処理を止められず、正しく登録対象になることを確認する
    /// 【テスト内容】: 行1=正常、行2=title空（不正）、行3=正常 のCSVをparse_booklog_csvへ渡す
    /// 【期待される動作】: successes.len()==2、failures.len()==1（row_number=2）
    /// 🔵 信頼性レベル: EDGE-002 / TC-016-02 に直接対応
    #[test]
    fn parse_booklog_csv_continues_processing_after_invalid_row() {
        // 【テストデータ準備】: 正常→不正→正常の3行を用意する
        // 【初期条件設定】: EDGE-002「一部行が形式不正の場合、不正行のみスキップし正常行の取込を継続」の中核シナリオ
        let bytes = csv_bytes(&[
            "吾輩は猫である,面白い,2024-01-15,4.5,9784101010014",
            ",,,,",
            "星の王子さま,,,,",
        ]);

        // 【実際の処理実行】: parse_booklog_csvを呼び出す
        // 【処理内容】: 不正行の後ろの正常行が確実に処理されることを確認する
        let (successes, failures) = parse_booklog_csv(&bytes);

        // 【結果検証】: 不正行の後の正常行（行3）が登録対象に含まれることを確認
        assert_eq!(successes.len(), 2); // 【確認内容】: 不正行を挟んでも正常行2件が登録対象になることを確認 🔵
        assert_eq!(failures.len(), 1); // 【確認内容】: 不正行が1件記録されることを確認 🔵
        assert_eq!(failures[0].row_number, 2); // 【確認内容】: 不正行のrow_numberが正しく2であることを確認 🔵
    }

    /// TC-B-04: row_number採番（ヘッダー除外・1始まり、先頭行・末尾行の境界確認）
    /// 【テスト目的】: failureのrow_numberがヘッダーを除いたデータ行基準で1始まりであり、
    /// 先頭データ行・末尾データ行のいずれでも正確に採番されることを確認する
    /// 【テスト内容】: 行1=不正(title空)、行2=正常、行3=不正(title空) のCSVをparse_booklog_csvへ渡す
    /// 【期待される動作】: failuresの先頭row_number==1、末尾row_number==3（データ行数と一致）。
    /// ヘッダーは加算されない
    /// 🔵 信頼性レベル: 要件定義2章 ImportFailure.row_number 定義に直接対応
    #[test]
    fn parse_booklog_csv_row_number_starts_at_one_and_excludes_header() {
        // 【テストデータ準備】: 先頭データ不正・中間正常・末尾データ不正の3行を用意する
        // 【初期条件設定】: オフバイワン（ヘッダーを誤って1行目として数える等）の防止確認
        let bytes = csv_bytes(&[
            ",,,,",
            "吾輩は猫である,,,,",
            ",,,,",
        ]);

        // 【実際の処理実行】: parse_booklog_csvを呼び出す
        // 【処理内容】: 行番号採番ロジック（ヘッダー非カウント・1始まり）を確認する
        let (_successes, failures) = parse_booklog_csv(&bytes);

        // 【結果検証】: 先頭不正行がrow_number=1、末尾不正行がrow_number=3（データ行数と一致）であることを確認
        assert_eq!(failures.len(), 2); // 【確認内容】: 不正行が2件（先頭・末尾）記録されることを確認 🔵
        assert_eq!(failures[0].row_number, 1); // 【確認内容】: 先頭データ行の不正がrow_number=1（ヘッダーが0扱いされない）ことを確認 🔵
        assert_eq!(failures[1].row_number, 3); // 【確認内容】: 末尾データ行の不正がrow_number=3（データ行数と一致）であることを確認 🔵
    }

    /// TC-B-06: title前後空白のみ（trim後空）→ title is empty扱い
    /// 【テスト目的】: 作品名が半角/全角スペースのみの行を、視覚的に空のtitleとして不正扱いすることを確認する
    /// 【テスト内容】: title="   "（半角スペースのみ）の行をparse_booklog_csv_rowへ渡す
    /// 【期待される動作】: Err(ImportFailure{reason: "title is empty"})（TC-E-01と同じreason）
    /// 🟡 信頼性レベル: 要件定義3章入力検証からの妥当な推測（trim方針を採用する実装を前提とする）
    #[test]
    fn parse_booklog_csv_row_treats_blank_only_title_as_empty() {
        // 【テストデータ準備】: 半角スペース3文字のみのtitleを持つ行を用意する
        // 【初期条件設定】: コピペ事故による空白titleを再現する
        let row = BooklogCsvRow {
            title: "   ".to_string(),
            description: None,
            consumed_date_raw: None,
            rating_raw: None,
            external_id: None,
        };

        // 【実際の処理実行】: parse_booklog_csv_rowを呼び出す
        let result = parse_booklog_csv_row(row, 1);

        // 【結果検証】: 空白のみのtitleがTC-E-01と同じ"title is empty"理由でスキップされることを確認
        let failure = result.unwrap_err();
        assert_eq!(failure.reason, "title is empty"); // 【確認内容】: 空白のみのtitleがTC-E-01と同一reasonで拒否されることを確認 🟡
    }

    /// TC-B-01関連: ヘッダーのみ（データ0行）はsuccesses/failuresともに空になる
    /// 【テスト目的】: データ行が1行も存在しないCSV（ヘッダーのみ）を処理した際、例外を起こさず
    /// 成功・失敗ともに0件の結果を返すことを確認する
    /// 【テスト内容】: ヘッダー行のみ（データ行なし）のCSVバイト列をparse_booklog_csvへ渡す
    /// 【期待される動作】: successes.len()==0、failures.len()==0
    /// 🟡 信頼性レベル: 要件定義の「0バイト=400」「全行処理後にサマリ返却」からの妥当な推測
    #[test]
    fn parse_booklog_csv_with_header_only_returns_empty_results() {
        // 【テストデータ準備】: ヘッダー行のみ、データ行なしのCSV（非0バイト）を用意する
        // 【初期条件設定】: 蔵書が空の利用者のエクスポートを再現する
        let bytes = csv_bytes(&[]);

        // 【実際の処理実行】: parse_booklog_csvを呼び出す
        // 【処理内容】: データ0行時の空サマリ生成ロジックを確認する
        let (successes, failures) = parse_booklog_csv(&bytes);

        // 【結果検証】: 成功・失敗ともに0件であることを確認
        assert_eq!(successes.len(), 0); // 【確認内容】: データ0行時にsuccessesが空であることを確認 🟡
        assert_eq!(failures.len(), 0); // 【確認内容】: データ0行時にfailuresも空であることを確認 🟡
    }

    /// TC-B-02関連: 全行不正でもパニックせず全件failureとして処理完走する
    /// 【テスト目的】: 全データ行が不正（title空）でも例外を起こさず、全行がfailuresに記録されることを確認する
    /// 【テスト内容】: title空の3行を持つCSVをparse_booklog_csvへ渡す
    /// 【期待される動作】: successes.len()==0、failures.len()==3
    /// 🔵 信頼性レベル: TC-016-E01「全行不正でも例外にならない」に直接対応
    #[test]
    fn parse_booklog_csv_with_all_invalid_rows_completes_without_panic() {
        // 【テストデータ準備】: 全データ行（3行）がtitle空の不正なCSVを用意する
        // 【初期条件設定】: フォーマット完全不一致のCSV誤アップロードを再現する
        let bytes = csv_bytes(&[",,,,", ",,,,", ",,,,"]);

        // 【実際の処理実行】: parse_booklog_csvを呼び出す
        // 【処理内容】: イテレータ処理中にパニックしないことを確認する
        let (successes, failures) = parse_booklog_csv(&bytes);

        // 【結果検証】: 全行が失敗として記録され、成功は0件であることを確認
        assert_eq!(successes.len(), 0); // 【確認内容】: 全行不正時にsuccessesが0件であることを確認 🔵
        assert_eq!(failures.len(), 3); // 【確認内容】: 全行不正時にfailuresがデータ行数3件と一致することを確認 🔵
    }
}
