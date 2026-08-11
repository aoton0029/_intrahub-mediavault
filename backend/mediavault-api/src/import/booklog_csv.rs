//! ブクログCSVのカラムマッピング・行単位パーサ・バリデーション
//!
//! 【機能概要】: ブクログの実エクスポートCSV（ヘッダー行なし・Shift-JISエンコーディング・
//! 17列固定フォーマット）を1行ずつパースし、`CreateItemRequest`相当へ変換する。
//! 列の並びは`docs/booklog-import-sample/README.md`に記載の
//! 「サービスID, アイテムID, 13桁ISBN, カテゴリ, 評価, 読書状況, レビュー, タグ,
//! 読書メモ(非公開), 登録日時, 読了日, タイトル, 作者名, 出版社名, 発行年, タイプ, ページ数」
//! に対応する。
//! 【ジャンル判定】: 「タイプ」列だけでは小説と学術書・専門書を区別できないため、
//! `マンガ`はManga固定、`雑誌`はNovel固定、それ以外（本/電子書籍/洋書/未知値）はISBNがあれば
//! `needs_classification=true`を立てて暫定Novelとし、ハンドラ層でOpenBDのCコードによる
//! 再分類に委ねる（設計判断: OpenBD Cコード方式、ユーザー確定）。

use chrono::NaiveDate;

use crate::models::import::ImportFailure;
use crate::models::item::{CreateItemRequest, MediaType};

/// ブクログCSVの列インデックス（0始まり、ヘッダー行なし・17列固定）
const COL_ISBN: usize = 2;
const COL_RATING: usize = 4;
const COL_REVIEW: usize = 6;
const COL_CONSUMED_DATE: usize = 10;
const COL_TITLE: usize = 11;
const COL_FORMAT: usize = 15;
const EXPECTED_COLUMN_COUNT: usize = 17;

const FORMAT_MANGA: &str = "マンガ";
const FORMAT_MAGAZINE: &str = "雑誌";

/// ブクログCSV 1行分の生データ（列インデックスベース）
#[derive(Debug, Clone, PartialEq)]
struct BooklogCsvRow {
    isbn: Option<String>,
    review: Option<String>,
    consumed_date_raw: Option<String>,
    rating_raw: Option<String>,
    title: String,
    format: Option<String>,
}

/// `csv::StringRecord`（位置ベース）から`BooklogCsvRow`を組み立てる。
/// 列数が17未満の行は「invalid row format」として扱う。
fn row_from_record(record: &csv::StringRecord) -> Result<BooklogCsvRow, &'static str> {
    if record.len() < EXPECTED_COLUMN_COUNT {
        return Err("invalid row format");
    }

    let field = |idx: usize| {
        record
            .get(idx)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };

    Ok(BooklogCsvRow {
        isbn: field(COL_ISBN),
        review: field(COL_REVIEW),
        consumed_date_raw: field(COL_CONSUMED_DATE),
        rating_raw: field(COL_RATING),
        title: record.get(COL_TITLE).unwrap_or("").to_string(),
        format: field(COL_FORMAT),
    })
}

/// 「タイプ」列から暫定`MediaType`と、OpenBDでの再分類が必要かどうかを判定する。
/// `マンガ`→Manga確定、`雑誌`→Novel確定。それ以外は暫定Novel＋再分類要（呼び出し側でISBN有無を加味）。
fn determine_media_type(format: Option<&str>) -> (MediaType, bool) {
    match format.map(str::trim) {
        Some(FORMAT_MANGA) => (MediaType::Manga, false),
        Some(FORMAT_MAGAZINE) => (MediaType::Novel, false),
        _ => (MediaType::Novel, true),
    }
}

/// `BooklogCsvRow`をバリデーション・型変換し、`CreateItemRequest`と再分類要否へ変換する。
///
/// 1. titleをtrim().is_empty()で判定（空・空白のみは不正）
/// 2. consumed_date_rawが空でなければ`YYYY-MM-DD`としてパース（失敗時は"invalid date format"）
/// 3. rating_rawが空でなければ`f32`としてパース（失敗時は"invalid rating"）
/// 4. media_typeは「タイプ」列から暫定決定し、ISBNがある場合のみ再分類フラグを立てる
fn parse_booklog_csv_row(
    row: BooklogCsvRow,
    row_number: u32,
) -> Result<(CreateItemRequest, bool), ImportFailure> {
    if row.title.trim().is_empty() {
        return Err(ImportFailure::new(row_number, "title is empty"));
    }

    let description = row.review;

    let consumed_date = match row.consumed_date_raw {
        None => None,
        Some(raw) => match NaiveDate::parse_from_str(&raw, "%Y-%m-%d") {
            Ok(date) => Some(date),
            Err(_) => return Err(ImportFailure::new(row_number, "invalid date format")),
        },
    };

    let rating = match row.rating_raw {
        None => None,
        Some(raw) => match raw.parse::<f32>() {
            Ok(value) => Some(value),
            Err(_) => return Err(ImportFailure::new(row_number, "invalid rating")),
        },
    };

    let (media_type, tentative_needs_classification) = determine_media_type(row.format.as_deref());
    let needs_classification = tentative_needs_classification && row.isbn.is_some();

    Ok((
        CreateItemRequest {
            media_type,
            title: row.title,
            original_title: None,
            description,
            cover_image_url: None,
            release_date: None,
            homepage_url: None,
            rating,
            is_favorite: None,
            authors: None,
            publication_year: None,
            journal: None,
            doi: None,
            details: None,
            consumed_date,
            additional_images: Vec::new(),
        },
        needs_classification,
    ))
}

/// パース成功行1件分のデータ。DB登録時まで`row_number`・`external_id`(ISBN)・
/// OpenBD再分類要否を保持するために`CreateItemRequest`と一緒に保持する。
#[derive(Debug, Clone)]
pub struct ParsedBooklogRow {
    pub row_number: u32,
    pub request: CreateItemRequest,
    pub external_id: Option<String>,
    /// trueの場合、ハンドラ層でOpenBDのCコードにより`request.media_type`を
    /// Novel/AcademicBookへ再分類する必要がある（タイプ列が本/電子書籍/洋書などでISBNがある行）。
    pub needs_classification: bool,
}

/// ブクログCSVのバイト列（Shift-JIS）をUTF-8文字列へデコードする。
/// 不正なバイト列は置換文字に変換され、処理は継続する。
fn decode_booklog_csv_bytes(bytes: &[u8]) -> String {
    let (decoded, _encoding, had_errors) = encoding_rs::SHIFT_JIS.decode(bytes);
    if had_errors {
        tracing::warn!("booklog csv contains bytes that are not valid Shift-JIS");
    }
    decoded.into_owned()
}

/// CSVバイト列（Shift-JIS・ヘッダー行なし・17列固定）を行単位でパースし、
/// 成功行・失敗行を分離して返す。row_numberは1始まり。
pub fn parse_booklog_csv(bytes: &[u8]) -> (Vec<ParsedBooklogRow>, Vec<ImportFailure>) {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    let text = decode_booklog_csv_bytes(bytes);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    for (idx, result) in reader.records().enumerate() {
        let row_number = (idx + 1) as u32;

        match result {
            Ok(record) => match row_from_record(&record) {
                Ok(row) => {
                    let external_id = row.isbn.clone();
                    match parse_booklog_csv_row(row, row_number) {
                        Ok((request, needs_classification)) => successes.push(ParsedBooklogRow {
                            row_number,
                            request,
                            external_id,
                            needs_classification,
                        }),
                        Err(failure) => failures.push(failure),
                    }
                }
                Err(reason) => failures.push(ImportFailure::new(row_number, reason)),
            },
            Err(err) => {
                tracing::error!("booklog csv row read error: {err}");
                failures.push(ImportFailure::new(row_number, "invalid row format"));
            }
        }
    }

    (successes, failures)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用ヘルパー: 17列固定のブクログCSV1行を組み立てる（未指定列は空欄）
    #[allow(clippy::too_many_arguments)]
    fn booklog_row(
        isbn: &str,
        rating: &str,
        status: &str,
        review: &str,
        consumed_date: &str,
        title: &str,
        format: &str,
    ) -> String {
        format!(
            "\"1\",\"0000000000\",\"{isbn}\",\"\",\"{rating}\",\"{status}\",\"{review}\",\"\",\"\",\"2022-06-16 12:32:57\",\"{consumed_date}\",\"{title}\",\"著者\",\"出版社\",\"2020\",\"{format}\",\"300\""
        )
    }

    /// テスト用ヘルパー: 行文字列群をShift-JISエンコード済みバイト列へ変換する
    /// （実ブクログCSVはShift-JISのため、`parse_booklog_csv`の入力を実データに合わせる）
    fn csv_bytes_from_rows(rows: &[String]) -> Vec<u8> {
        let mut content = String::new();
        for row in rows {
            content.push_str(row);
            content.push_str("\r\n");
        }
        let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(&content);
        encoded.into_owned()
    }

    /// 正常行（タイプ=本）は暫定Novel・要再分類として登録される
    #[test]
    fn parses_book_type_row_as_tentative_novel_needing_classification() {
        let bytes = csv_bytes_from_rows(&[booklog_row(
            "9784101010014",
            "4.5",
            "読み終わった",
            "面白い",
            "2024-01-15",
            "吾輩は猫である",
            "本",
        )]);

        let (successes, failures) = parse_booklog_csv(&bytes);

        assert_eq!(failures.len(), 0);
        assert_eq!(successes.len(), 1);
        let row = &successes[0];
        assert_eq!(row.request.title, "吾輩は猫である");
        assert_eq!(row.request.media_type, MediaType::Novel);
        assert_eq!(row.request.description, Some("面白い".to_string()));
        assert_eq!(
            row.request.consumed_date,
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
        assert_eq!(row.request.rating, Some(4.5));
        assert_eq!(row.external_id, Some("9784101010014".to_string()));
        assert!(row.needs_classification);
    }

    /// タイプ=マンガはManga確定・再分類不要
    #[test]
    fn parses_manga_type_row_as_manga_without_classification() {
        let bytes = csv_bytes_from_rows(&[booklog_row(
            "9784063876500",
            "",
            "積読",
            "",
            "",
            "進撃の巨人",
            "マンガ",
        )]);

        let (successes, _failures) = parse_booklog_csv(&bytes);

        assert_eq!(successes.len(), 1);
        assert_eq!(successes[0].request.media_type, MediaType::Manga);
        assert!(!successes[0].needs_classification);
    }

    /// タイプ=雑誌はNovel固定・再分類不要
    #[test]
    fn parses_magazine_type_row_as_fixed_novel_without_classification() {
        let bytes = csv_bytes_from_rows(&[booklog_row(
            "9784910000000",
            "",
            "積読",
            "",
            "",
            "月刊なにか",
            "雑誌",
        )]);

        let (successes, _failures) = parse_booklog_csv(&bytes);

        assert_eq!(successes.len(), 1);
        assert_eq!(successes[0].request.media_type, MediaType::Novel);
        assert!(!successes[0].needs_classification);
    }

    /// ISBNが空の行は本タイプでも再分類フラグが立たない（OpenBDへ問い合わせようがないため）
    #[test]
    fn book_type_row_without_isbn_does_not_need_classification() {
        let bytes = csv_bytes_from_rows(&[booklog_row("", "", "読みたい", "", "", "無題", "本")]);

        let (successes, _failures) = parse_booklog_csv(&bytes);

        assert_eq!(successes.len(), 1);
        assert_eq!(successes[0].external_id, None);
        assert!(!successes[0].needs_classification);
    }

    /// 任意列（レビュー・読了日・評価・ISBN）が空でも正常登録される
    #[test]
    fn treats_blank_optional_columns_as_none() {
        let bytes =
            csv_bytes_from_rows(&[booklog_row("", "", "積読", "", "", "星の王子さま", "本")]);

        let (successes, failures) = parse_booklog_csv(&bytes);

        assert_eq!(failures.len(), 0);
        let row = &successes[0];
        assert_eq!(row.request.title, "星の王子さま");
        assert_eq!(row.request.description, None);
        assert_eq!(row.request.consumed_date, None);
        assert_eq!(row.request.rating, None);
        assert_eq!(row.external_id, None);
    }

    /// タイトル空行はスキップされる（reason="title is empty"）
    #[test]
    fn skips_empty_title() {
        let bytes = csv_bytes_from_rows(&[booklog_row(
            "9784101010014",
            "4.5",
            "読み終わった",
            "面白い",
            "2024-01-15",
            "",
            "本",
        )]);

        let (successes, failures) = parse_booklog_csv(&bytes);

        assert_eq!(successes.len(), 0);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].row_number, 1);
        assert_eq!(failures[0].reason, "title is empty");
    }

    /// 空白のみのタイトルも空として扱われる
    #[test]
    fn treats_blank_only_title_as_empty() {
        let bytes = csv_bytes_from_rows(&[booklog_row("", "", "積読", "", "", "   ", "本")]);

        let (_successes, failures) = parse_booklog_csv(&bytes);

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].reason, "title is empty");
    }

    /// 読了日が不正形式の行はスキップされる（reason="invalid date format"）
    #[test]
    fn skips_invalid_date_format() {
        let bytes = csv_bytes_from_rows(&[booklog_row(
            "",
            "",
            "読み終わった",
            "",
            "2024/13/40",
            "x",
            "本",
        )]);

        let (_successes, failures) = parse_booklog_csv(&bytes);

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].reason, "invalid date format");
    }

    /// 評価が数値でない行はスキップされる（reason="invalid rating"）
    #[test]
    fn skips_invalid_rating() {
        let bytes =
            csv_bytes_from_rows(&[booklog_row("", "とても良い", "積読", "", "", "x", "本")]);

        let (_successes, failures) = parse_booklog_csv(&bytes);

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].reason, "invalid rating");
    }

    /// 列数不足行はデシリアライズ失敗としてスキップされる（reason="invalid row format"）
    #[test]
    fn skips_row_with_insufficient_columns() {
        let bytes = b"\"1\",\"123\",\"9784101010014\"\r\n".to_vec();

        let (successes, failures) = parse_booklog_csv(&bytes);

        assert_eq!(successes.len(), 0);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].row_number, 1);
        assert_eq!(failures[0].reason, "invalid row format");
    }

    /// 複数の不正行が独立したImportFailureとして記録され、正常行の処理は継続する
    #[test]
    fn continues_processing_after_invalid_row_and_records_independent_failures() {
        let bytes = csv_bytes_from_rows(&[
            booklog_row(
                "9784101010014",
                "4.5",
                "読み終わった",
                "面白い",
                "2024-01-15",
                "吾輩は猫である",
                "本",
            ),
            booklog_row("", "", "積読", "", "", "", "本"),
            booklog_row("", "", "積読", "", "", "星の王子さま", "本"),
            booklog_row("", "", "読み終わった", "", "2024/13/40", "破戒", "本"),
        ]);

        let (successes, failures) = parse_booklog_csv(&bytes);

        assert_eq!(successes.len(), 2);
        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0].row_number, 2);
        assert_eq!(failures[0].reason, "title is empty");
        assert_eq!(failures[1].row_number, 4);
        assert_eq!(failures[1].reason, "invalid date format");
    }

    /// 空バイト列（データなし）はsuccesses/failuresともに空になる
    #[test]
    fn empty_bytes_return_empty_results() {
        let (successes, failures) = parse_booklog_csv(&[]);

        assert_eq!(successes.len(), 0);
        assert_eq!(failures.len(), 0);
    }

    /// 全行不正でもパニックせず全件failureとして処理完走する
    #[test]
    fn all_invalid_rows_complete_without_panic() {
        let bytes = csv_bytes_from_rows(&[
            booklog_row("", "", "積読", "", "", "", "本"),
            booklog_row("", "", "積読", "", "", "", "本"),
            booklog_row("", "", "積読", "", "", "", "本"),
        ]);

        let (successes, failures) = parse_booklog_csv(&bytes);

        assert_eq!(successes.len(), 0);
        assert_eq!(failures.len(), 3);
    }

    /// Shift-JISでエンコードされたバイト列も正しくデコードされる
    #[test]
    fn decodes_shift_jis_encoded_bytes() {
        let row = booklog_row(
            "9784101010014",
            "4.5",
            "読み終わった",
            "面白い",
            "2024-01-15",
            "吾輩は猫である",
            "本",
        );
        let content = format!("{row}\r\n");
        let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&content);
        assert!(!had_errors);

        let (successes, failures) = parse_booklog_csv(&encoded);

        assert_eq!(failures.len(), 0);
        assert_eq!(successes.len(), 1);
        assert_eq!(successes[0].request.title, "吾輩は猫である");
    }
}
