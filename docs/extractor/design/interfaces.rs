//! MediaVault Extractor 型定義（mediavault-api 側 / Rust）
//!
//! 作成日: 2026-08-14
//! 関連設計: architecture.md, api-endpoints.md, database-schema.sql
//!
//! 配置先:
//!   backend/mediavault-api/src/models/item_extraction.rs
//!   backend/mediavault-api/src/models/item_file_text.rs
//!   backend/mediavault-api/src/models/response.rs（ApiErrorCode への追加分）
//!
//! 本ファイルは設計ドキュメントであり、そのままコンパイルはされない。
//! 実装時は上記3ファイルへ分割して配置する。
//!
//! 既存パターンの踏襲元: backend/mediavault-api/src/models/item_file.rs
//!   * ENUM は #[derive(sqlx::Type)] + #[sqlx(type_name = "...", rename_all = "lowercase")]
//!   * DB行は #[derive(sqlx::FromRow)]
//!   * 日時は chrono::NaiveDateTime（TIMESTAMPTZ ではなく TIMESTAMP のため）
//!   * バリデーションは parse_xxx_request(req) -> Result<Validated, ApiError> 関数として分離
//!
//! 信頼性レベル:
//! - 🔵 青信号: EARS要件定義書・設計文書・既存実装を参考にした確実な型定義
//! - 🟡 黄信号: EARS要件定義書・設計文書・既存実装から妥当な推測による型定義
//! - 🔴 赤信号: EARS要件定義書・設計文書・既存実装にない推測による型定義

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::models::item_file::FileType;
use crate::models::response::{ApiError, ApiErrorCode};

// ========================================
// ENUM 定義
// ========================================

/// 抽出の状態
/// 🔵 信頼性: database-schema.sql の extraction_state・REQ-201〜203 に直接対応
/// 既存 FileType（item_file.rs:13）と同じ sqlx::Type パターン
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "extraction_state", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ExtractionState {
    Queued,     // 🔵 REQ-201
    Running,    // 🔵 REQ-201
    Cancelling, // 🔵 REQ-202
    Succeeded,  // 🔵 REQ-203（終端）
    Failed,     // 🔵 REQ-203（終端）
    Cancelled,  // 🔵 REQ-203（終端）
}

impl ExtractionState {
    /// 終端状態かどうか
    /// 🔵 信頼性: REQ-203・REQ-205 に直接対応。cancel/complete/fail の可否判定で使う
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            ExtractionState::Succeeded | ExtractionState::Failed | ExtractionState::Cancelled
        )
    }

    /// 未完了（部分UNIQUE index の対象）かどうか
    /// 🔵 信頼性: database-schema.sql uq_item_file_extractions_active の WHERE 句に対応
    pub fn is_active(self) -> bool {
        !self.is_terminal()
    }
}

/// ファイル参照のルート種別
/// 🔵 信頼性: 設計ヒアリングQ4（要件定義フェーズ）・architecture.md D-3 に直接対応
///
/// DBカラムではなく、内部APIレスポンスの型としてのみ使う。
/// item_files.path の2経路（リンク=絶対パス / アップロード=STORAGE_ROOT 相対）を吸収し、
/// worker のマウントパスに依存しない形式で参照を渡す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileRefRoot {
    /// MediaVault専用領域（アップロード経路）。worker 側 EXTRACTOR_STORAGE_ROOT
    Storage, // 🔵 item-files.md §2つの登録経路より
    /// 実データ領域（リンク経路）。worker 側 EXTRACTOR_LIBRARY_ROOT
    Library, // 🔵 同上
}

// ========================================
// エンティティ定義
// ========================================

/// item_file_extractions のDB行
/// 🔵 信頼性: database-schema.sql の item_file_extractions・REQ-040 に直接対応
///
/// 既存 ItemFile（item_file.rs:53）と同じ sqlx::FromRow パターン。
/// Serialize は付けない。公開レスポンスへは ExtractionResponse へ変換して返す
/// （lease_token 等の内部情報を漏らさないため）。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemFileExtraction {
    pub id: Uuid,                            // 🔵 共通パターン
    pub item_file_id: Uuid,                  // 🔵 REQ-040
    pub state: ExtractionState,              // 🔵 REQ-040
    pub attempts: i32,                       // 🔵 REQ-040・REQ-112
    pub max_attempts: i32,                   // 🔵 REQ-040・REQ-111
    pub progress_current: i32,               // 🔵 REQ-023
    pub progress_total: Option<i32>,         // 🔵 NULL = 総数未確定
    pub claimed_by: Option<String>,          // 🔵 REQ-040（観測用）
    pub lease_token: Option<Uuid>,           // 🔵 REQ-021・REQ-407
    pub lease_expires_at: Option<NaiveDateTime>, // 🔵 REQ-118
    pub error: Option<JsonValue>,            // 🔵 REQ-026（ExtractionError の JSON）
    pub created_at: NaiveDateTime,           // 🔵 共通パターン
    pub updated_at: NaiveDateTime,           // 🔵 共通パターン
}

/// item_file_texts のDB行
/// 🔵 信頼性: database-schema.sql の item_file_texts・REQ-041〜043 に直接対応
///
/// content は巨大になりうるため、この型を丸ごと SELECT するのは避ける。
/// GET /items/{id}/text は SUBSTRING で必要な部分だけ取る（REQ-008・NFR-001）。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemFileText {
    pub id: Uuid,                       // 🔵 共通パターン
    pub item_file_id: Uuid,             // 🔵 REQ-041（UNIQUE）
    pub content: String,                // 🔵 REQ-041
    pub boundaries: JsonValue,          // 🔵 REQ-042（Vec<TextBoundary> の JSON）
    pub extraction_version: String,     // 🔵 REQ-041・REQ-104
    pub extractor: JsonValue,           // 🔵 REQ-043（ExtractorMetadata の JSON）
    pub extracted_at: NaiveDateTime,    // 🔵 REQ-041
    pub created_at: NaiveDateTime,      // 🔵 共通パターン
    pub updated_at: NaiveDateTime,      // 🔵 共通パターン
}

/// ページ・章の境界（jsonb 配列の1要素）
/// 🔵 信頼性: REQ-042・設計ヒアリングQ2・architecture.md D-5 に直接対応
///
/// start は含む / end は含まない（half-open）。いずれも**文字**オフセット（バイトではない）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBoundary {
    pub start: i64,     // 🔵 REQ-042
    pub end: i64,       // 🔵 REQ-042
    pub label: String,  // 🔵 例: "p.1" / "第3章"
}

/// 抽出方式のメタデータ（extractor jsonb）
/// 🟡 信頼性: REQ-043・FR-007 から architecture.md D-8 として具体化した推測
///
/// PDF は「一部ページのみOCR」がありうるため method を単一値にせず、
/// 方式ごとのページ数を併記する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorMetadata {
    pub method: ExtractionMethod,          // 🟡 D-8
    pub embedded_text_pages: u32,          // 🟡 D-8
    pub ocr_pages: u32,                    // 🟡 D-8
    /// OCRを一度も使わなかった場合は None
    pub ocr: Option<OcrMetadata>,          // 🔵 REQ-043（FR-007「OCRを使用した場合の」）
}

/// 🟡 信頼性: REQ-043・architecture.md D-8 より
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMethod {
    EmbeddedText, // 🔵 FR-007「embedded text」
    Ocr,          // 🔵 FR-007「OCR」
    Mixed,        // 🟡 D-8（PDFの一部ページのみOCRのケース）
}

/// 🔵 信頼性: REQ-043・PRD FR-007「実行方式（cpu / gpu）、エンジン、モデル識別子」に直接対応
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrMetadata {
    pub engine: String,     // 🔵 例: "yomitoku"
    pub device: OcrDevice,  // 🔵 REQ-043・FR-011
    pub model: String,      // 🔵 モデル識別子
}

/// OCR実行方式。**外部へ報告する値**であり、設定値（cpu / cuda）とは異なる
/// 🔵 信頼性: PRD FR-011「外部へ報告する実行方式はそれぞれ cpu、gpu とする」に直接対応
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OcrDevice {
    Cpu, // 🔵 設定値 cpu に対応
    Gpu, // 🔵 設定値 cuda に対応（"cuda" ではなく "gpu" と報告する）
}

/// 構造化エラー（error jsonb）
/// 🔵 信頼性: REQ-026・NFR-503 に直接対応（kind の値域は 🟡）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionError {
    pub kind: ExtractionErrorKind, // 🟡 値域は api-endpoints.md §fail より
    pub message: String,           // 🔵 REQ-026
    /// 再試行可能か。false なら attempts に関わらず即 failed（REQ-110）
    pub retryable: bool,           // 🔵 REQ-109・REQ-110
}

/// 🟡 信頼性: PRD FR-009・tech-stack.md の TransientError / PermanentError 分類から妥当な推測
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionErrorKind {
    UnsupportedFormat,  // 🔵 FR-003（非retryable）
    CorruptFile,        // 🔵 FR-009（非retryable）
    FileNotFound,       // 🟡 EDGE-004（非retryable）
    SizeLimitExceeded,  // 🔵 EDGE-106・REQ-408（非retryable）
    OcrFailed,          // 🔵 FR-009（非retryable）
    ApiUnreachable,     // 🔵 REQ-109（retryable）
    LeaseExpired,       // 🟡 database-schema.sql §sweeper より
    Internal,           // 🟡 分類不能な失敗の受け皿
}

// ========================================
// 公開APIレスポンス
// ========================================

/// 公開API（POST/GET/cancel .../extraction）のレスポンス表現
/// 🔵 信頼性: api-endpoints.md §公開API に直接対応
///
/// **lease_token / lease_expires_at / claimed_by を含めない。**
/// 内部の排他制御情報であり、外部へ出す必要がない 🟡。
#[derive(Debug, Clone, Serialize)]
pub struct ExtractionResponse {
    pub id: Uuid,                              // 🔵
    pub item_file_id: Uuid,                    // 🔵
    pub state: ExtractionState,                // 🔵 REQ-002
    pub attempts: i32,                         // 🔵 REQ-301
    pub max_attempts: i32,                     // 🔵 REQ-301
    pub progress_current: i32,                 // 🔵 REQ-002
    pub progress_total: Option<i32>,           // 🔵 REQ-002
    pub error: Option<ExtractionError>,        // 🔵 REQ-301
    pub created_at: NaiveDateTime,             // 🔵
    pub updated_at: NaiveDateTime,             // 🔵
}

impl From<ItemFileExtraction> for ExtractionResponse {
    /// 🔵 信頼性: api-endpoints.md §GET .../extraction「lease_token を公開レスポンスに含めない」より
    fn from(row: ItemFileExtraction) -> Self {
        Self {
            id: row.id,
            item_file_id: row.item_file_id,
            state: row.state,
            attempts: row.attempts,
            max_attempts: row.max_attempts,
            progress_current: row.progress_current,
            progress_total: row.progress_total,
            // 【設計判断】: error の JSON がパース不能でもレスポンス全体を失敗させない。
            // 観測性のためのフィールドであり、必須情報ではない 🟡
            error: row.error.and_then(|v| serde_json::from_value(v).ok()),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

// ========================================
// Item Text API（GET /items/{id}/text）
// ========================================

/// 🔵 信頼性: item-text.md §データ型 ItemText に直接対応
#[derive(Debug, Clone, Serialize)]
pub struct ItemTextResponse {
    pub item_id: Uuid,                  // 🔵 item-text.md
    pub file_id: Uuid,                  // 🔵 item-text.md
    pub extracted_at: NaiveDateTime,    // 🔵 item-text.md
    pub extraction_version: String,     // 🔵 item-text.md・REQ-006
    pub chunk: TextChunk,               // 🔵 item-text.md
}

/// 🔵 信頼性: item-text.md §データ型 TextChunk に直接対応
#[derive(Debug, Clone, Serialize)]
pub struct TextChunk {
    /// 形式非依存の0起点連番。ページ番号ではない（REQ-413）
    pub index: i64,                 // 🔵 item-text.md 最重要規約
    pub size: i64,                  // 🔵 item-text.md（要求値と一致）
    pub total_chunks: i64,          // 🔵 REQ-006・EDGE-103
    /// 範囲表記（"p.1-3"）。境界情報がなければ None（architecture.md D-5）
    pub label: Option<String>,      // 🔵 設計ヒアリングQ2
    /// 末尾チャンクは size 未満になりうる（EDGE-104）
    pub text: String,               // 🔵 item-text.md
}

/// GET /items/{id}/text のクエリパラメータ
/// 🔵 信頼性: item-text.md §クエリパラメータ・EDGE-101/102 に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct ItemTextQuery {
    pub file_id: Option<Uuid>,      // 🔵 省略時は主ファイルを解決（REQ-115）
    pub chunk_index: Option<i64>,   // 🔵 既定 0
    pub chunk_size: Option<i64>,    // 🔵 既定 4000、最大 20000
}

/// 検証済みのチャンク要求
/// 🔵 信頼性: 既存 ValidatedItemFileRequest（item_file.rs:71）と同じパターン
#[derive(Debug, Clone)]
pub struct ValidatedItemTextQuery {
    pub file_id: Option<Uuid>,
    pub chunk_index: i64,
    pub chunk_size: i64,
}

pub const DEFAULT_CHUNK_SIZE: i64 = 4000; // 🔵 item-text.md
pub const MAX_CHUNK_SIZE: i64 = 20000;    // 🔵 item-text.md・EDGE-102

/// 【機能概要】: chunk_index / chunk_size の範囲検証と既定値の適用
/// 【実装方針】: 既存 parse_create_item_file_request（item_file.rs:82）と同じ、
/// リクエストDTO → 検証済み型への変換関数として分離する
/// 【テスト対応】: TC-005-B01（chunk_index >= total_chunks）は total_chunks 算出後の
/// 判定になるため本関数の対象外。ここでは負値と chunk_size 範囲のみを見る
/// 🔵 信頼性: EDGE-101・EDGE-102・item-text.md §エラーの使い分けに直接対応
pub fn parse_item_text_query(query: ItemTextQuery) -> Result<ValidatedItemTextQuery, ApiError> {
    let chunk_index = query.chunk_index.unwrap_or(0);
    if chunk_index < 0 {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "chunk_indexは0以上である必要があります",
        ));
    }

    let chunk_size = query.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
    if chunk_size < 1 || chunk_size > MAX_CHUNK_SIZE {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "chunk_sizeは1以上20000以下である必要があります",
        ));
    }

    Ok(ValidatedItemTextQuery {
        file_id: query.file_id,
        chunk_index,
        chunk_size,
    })
}

/// AMBIGUOUS_FILE の候補
/// 🔵 信頼性: item-text.md §主ファイルの解決 candidates・NFR-502 に直接対応
///
/// 注: 共通の ApiErrorBody { code, message } は変更しない。
/// candidates を持つのは本エンドポイントのみの拡張であり、専用のレスポンス型を用意する。
#[derive(Debug, Clone, Serialize)]
pub struct AmbiguousFileCandidate {
    pub file_id: Uuid,           // 🔵 item-text.md
    pub label: Option<String>,   // 🔵 item-text.md
    pub file_type: FileType,     // 🔵 既存 FileType を再利用
}

// ========================================
// 内部API（worker 専用）
// ========================================

/// POST /internal/extractions/claim リクエスト
/// 🔵 信頼性: api-endpoints.md §claim に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct ClaimRequest {
    pub worker_id: String,      // 🔵 REQ-040 claimed_by（観測用）
    pub lease_seconds: i64,     // 🔵 REQ-021
}

/// claim レスポンス（取得できた場合）
/// 🔵 信頼性: REQ-021・REQ-022・api-endpoints.md §claim に直接対応
///
/// 取得できなかった場合は ApiOk<Option<ClaimResponse>> の None として返す 🟡
/// （204 ではなく data: null。worker 側のデシリアライズ経路を1本にするため）
#[derive(Debug, Clone, Serialize)]
pub struct ClaimResponse {
    pub extraction_id: Uuid,                // 🔵 REQ-020
    pub item_file_id: Uuid,                 // 🔵 REQ-022
    pub item_id: Uuid,                      // 🟡 ログ・観測用（NFR-401）
    pub file_type: FileType,                // 🔵 REQ-022
    pub size_bytes: i64,                    // 🔵 REQ-022（EDGE-106 の事前検証に使う）
    pub attempts: i32,                      // 🟡 worker のログ用
    pub lease_token: Uuid,                  // 🔵 REQ-021・REQ-407
    pub lease_expires_at: NaiveDateTime,    // 🔵 REQ-021
    pub file_ref: FileRef,                  // 🔵 REQ-022・architecture.md D-3
}

/// worker のマウントパスに依存しないファイル参照
/// 🔵 信頼性: 設計ヒアリングQ4（要件定義フェーズ）・architecture.md D-3 に直接対応
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    pub root: FileRefRoot,       // 🔵 storage / library
    /// 許可ルートからの相対パス。".." を含む値は api 側でも拒否する（REQ-402）
    pub relative_path: String,   // 🔵 REQ-402・REQ-403
}

/// POST /internal/extractions/{id}/heartbeat リクエスト
/// 🔵 信頼性: REQ-023・api-endpoints.md §heartbeat に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct HeartbeatRequest {
    pub lease_token: Uuid,              // 🔵 REQ-407
    pub progress_current: Option<i32>,  // 🔵 REQ-023
    pub progress_total: Option<i32>,    // 🔵 REQ-023
    pub lease_seconds: Option<i64>,     // 🔵 REQ-023（省略時は既定値で延長）
}

/// 🔵 信頼性: REQ-023・REQ-202・TC-023-03 に直接対応
#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatResponse {
    pub state: ExtractionState,             // 🔵 REQ-023
    /// state == Cancelling のときのみ true。worker はページ境界で中断する（REQ-207）
    pub cancel_requested: bool,             // 🔵 REQ-023・REQ-202
    pub lease_expires_at: NaiveDateTime,    // 🔵 REQ-023
}

/// POST /internal/extractions/{id}/complete リクエスト
/// 🔵 信頼性: REQ-024・REQ-065・FR-007・api-endpoints.md §complete に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CompleteRequest {
    pub lease_token: Uuid,                  // 🔵 REQ-407
    pub content: String,                    // 🔵 REQ-065（サイズ上限あり: REQ-408）
    pub boundaries: Vec<TextBoundary>,      // 🔵 REQ-042・REQ-068
    pub extraction_version: String,         // 🔵 REQ-065・REQ-104
    pub extracted_at: NaiveDateTime,        // 🔵 REQ-065
    pub extractor: ExtractorMetadata,       // 🔵 REQ-043・REQ-065
}

/// POST /internal/extractions/{id}/fail リクエスト
/// 🔵 信頼性: REQ-026・api-endpoints.md §fail に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct FailRequest {
    pub lease_token: Uuid,          // 🔵 REQ-407
    pub error: ExtractionError,     // 🔵 REQ-026
}

/// POST /internal/extractions/{id}/cancelled リクエスト
/// 🔵 信頼性: REQ-027・api-endpoints.md §cancelled に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CancelledRequest {
    pub lease_token: Uuid, // 🔵 REQ-407
}

// ========================================
// バリデーション
// ========================================

/// 抽出本文の保存サイズ上限（文字数）
/// 🟡 信頼性: REQ-408・EDGE-009 は上限の存在を要求しているが、値は prep.md §確認事項で判断待ち。
/// 500万文字は文庫本 約20冊分に相当し、単一書籍としては十分な余裕がある
pub const MAX_CONTENT_CHARS: usize = 5_000_000;

/// エラーメッセージの保存サイズ上限（文字数）
/// 🟡 信頼性: REQ-408 から妥当な推測
pub const MAX_ERROR_MESSAGE_CHARS: usize = 4_000;

/// 【機能概要】: complete リクエストの本文サイズと boundaries の整合性を検証する
/// 【実装方針】: DB の CHECK 制約では jsonb 配列の各要素を検証できないため、
/// アプリ側で行う（database-schema.sql §chk_item_file_texts_boundaries_is_array 参照）
/// 【テスト対応】: TC-024-E03（サイズ上限超過 → 422）, TC-024-B01（boundaries 整合性 → 400）
/// 🔵 信頼性: REQ-408・EDGE-009・EDGE-107 に直接対応（上限値そのものは 🟡）
pub fn validate_complete_request(req: &CompleteRequest) -> Result<(), ApiError> {
    // 【サイズ上限】: 文字数で数える。バイト長ではない（EDGE-103 と同じ方針）🔵
    let content_chars = req.content.chars().count();
    if content_chars > MAX_CONTENT_CHARS {
        return Err(ApiError::new(
            ApiErrorCode::UnprocessableEntity,
            "抽出本文が保存サイズ上限を超えています",
        ));
    }

    // 【boundaries 整合性】: start <= end、かつ end が本文長を超えない 🔵 EDGE-107
    let content_len = content_chars as i64;
    for b in &req.boundaries {
        if b.start < 0 || b.start > b.end || b.end > content_len {
            return Err(ApiError::new(
                ApiErrorCode::ValidationError,
                "boundariesの範囲が本文と整合しません",
            ));
        }
    }

    Ok(())
}

/// 【機能概要】: 抽出対象として許可された file_type かを判定する
/// 【実装方針】: MVPは pdf / image のみ。archive（cbz等）を含めるかは prep.md §確認事項
/// 【テスト対応】: TC-001-E03（video → 422 UNSUPPORTED_FILE_TYPE）
/// 🟡 信頼性: REQ-410（PRD §4.2「動画・音声の文字起こしは対象外」からの推測）
pub fn is_extractable(file_type: FileType) -> bool {
    matches!(file_type, FileType::Pdf | FileType::Image)
}

// ========================================
// label 合成
// ========================================

/// 【機能概要】: チャンクの文字範囲と交差する境界から表示用ラベルを合成する
/// 【実装方針】: chunk_size=4000 は通常数ページ分に相当するため、複数境界と交差するのが
/// 常態である。先頭ページのみを返すとチャンク末尾の引用でページがずれるため、範囲表記にする
/// 【テスト対応】: TC-005-02（label にページ番号）, TC-005-03（境界に対応しない → null）
/// 🔵 信頼性: 設計ヒアリングQ2・architecture.md D-5 に直接対応
///
/// ```text
/// 交差0件  → None
/// 交差1件  → Some("p.9")
/// 交差2件+ → Some("p.1-3")
/// ```
pub fn compose_chunk_label(
    boundaries: &[TextBoundary],
    chunk_start: i64,
    chunk_end: i64,
) -> Option<String> {
    // 【交差判定】: half-open 区間同士の重なり。b.end == chunk_start は交差しない 🔵
    let overlapping: Vec<&TextBoundary> = boundaries
        .iter()
        .filter(|b| b.start < chunk_end && b.end > chunk_start)
        .collect();

    match overlapping.as_slice() {
        [] => None,
        [single] => Some(single.label.clone()),
        [first, .., last] => {
            // 【範囲表記】: 末尾ラベルからは数値部のみを取り、"p.1-p.3" ではなく "p.1-3" とする 🔵
            let last_suffix = numeric_suffix(&last.label).unwrap_or(&last.label);
            Some(format!("{}-{}", first.label, last_suffix))
        }
    }
}

/// 【ヘルパー関数】: ラベル末尾の連続する数字部分を取り出す（"p.42" → "42"、"第3章" → None）
/// 🟡 信頼性: compose_chunk_label の範囲表記のために必要な補助。要件に明記はない。
/// 数値で終わらないラベル（"第3章"）は範囲表記に向かないためラベル全体を使う
fn numeric_suffix(label: &str) -> Option<&str> {
    let start = label.len() - label.chars().rev().take_while(|c| c.is_ascii_digit()).count();
    if start == label.len() {
        None // 数字で終わらない
    } else {
        Some(&label[start..])
    }
}

// ========================================
// ApiErrorCode への追加分
// ========================================
//
// backend/mediavault-api/src/models/response.rs の ApiErrorCode enum と
// code_and_status() へ以下を追加する。既存の variant・文字列は変更しない。
// 🔵 信頼性: requirements.md §追加・削除するエラーコード・api-endpoints.md に直接対応
//
//   /// 指定ファイルに抽出が1件も存在しない（GET/cancel .../extraction 時404）
//   ExtractionNotFound,
//   /// 終端状態の抽出をキャンセルしようとした（REQ-205）
//   ExtractionAlreadyFinished,
//   /// 抽出非対応の file_type（REQ-410）
//   UnsupportedFileType,
//   /// lease token 不一致・失効後の complete/fail（REQ-407・EDGE-002）
//   InvalidLeaseToken,
//   /// ファイルは存在するが抽出結果がない（REQ-116。index.md に定義済み・未実装だった）
//   TextNotExtracted,
//   /// file_id 省略時に抽出済みファイルが2件以上（REQ-115。同上）
//   AmbiguousFile,
//
//   ApiErrorCode::ExtractionNotFound => ("EXTRACTION_NOT_FOUND", StatusCode::NOT_FOUND),
//   ApiErrorCode::ExtractionAlreadyFinished => ("EXTRACTION_ALREADY_FINISHED", StatusCode::CONFLICT),
//   ApiErrorCode::UnsupportedFileType => ("UNSUPPORTED_FILE_TYPE", StatusCode::UNPROCESSABLE_ENTITY),
//   ApiErrorCode::InvalidLeaseToken => ("INVALID_LEASE_TOKEN", StatusCode::CONFLICT),
//   ApiErrorCode::TextNotExtracted => ("TEXT_NOT_EXTRACTED", StatusCode::UNPROCESSABLE_ENTITY),
//   ApiErrorCode::AmbiguousFile => ("AMBIGUOUS_FILE", StatusCode::CONFLICT),
//
// JobNotFound / JobAlreadyFinished は追加しない（実体がないため削除ではなく「追加しない」）。

// ========================================
// 信頼性レベルサマリー
// ========================================
/// 型定義・関数・定数 42件の内訳:
/// - 🔵 青信号: 34件 (81%)
/// - 🟡 黄信号: 8件 (19%)
/// - 🔴 赤信号: 0件 (0%)
///
/// 品質評価: ✅ 高品質
///
/// 🟡 の内訳: ExtractorMetadata の記録粒度（D-8）、ExtractionErrorKind の値域、
/// MAX_CONTENT_CHARS / MAX_ERROR_MESSAGE_CHARS の値（prep.md 判断待ち）、
/// is_extractable の対象形式（archive の扱いが未確定）、numeric_suffix、
/// claim レスポンスの item_id / attempts、error パース失敗時の握りつぶし。
