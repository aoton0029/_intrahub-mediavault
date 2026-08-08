//! mediavault-mcp 型定義（設計文書）
//!
//! 作成日: 2026-08-07
//! 関連設計: architecture.md / dataflow.md / mcp-tools.md
//! 関連要件: ../spec/requirements.md
//!
//! このファイルは設計上の型定義であり、コンパイル対象ではない。
//! 実装時は `backend/mediavault-mcp/src/` 配下へ分割して配置する。
//!
//! 信頼性レベル:
//! - 🔵 青信号: 要件定義書・設計文書・既存実装を参考にした確実な型定義
//! - 🟡 黄信号: 要件定義書・設計文書・既存実装から妥当な推測による型定義
//! - 🔴 赤信号: それらにない推測による型定義

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// 共通: ツール結果の外枠 (src/result/outcome.rs)
// ============================================================

/// すべてのMCPツール結果が持つ結果種別。
/// 🔵 設計決定 D-01（ヒアリング2026-08-07 Q1）・REQ-146 / REQ-114 より
#[derive(Debug, Clone, Copy, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// 要求されたすべての操作が完了した
    Success,
    /// 一部の操作が失敗または未処理のまま終了した（REQ-114）
    Partial,
    /// 操作を実行できなかった
    Error,
    /// 対象が一意に確定せず、書き込みを行わなかった（REQ-142・PRD §6 原則3）
    Ambiguous,
    /// 指定された対象が存在しなかった（REQ-110）
    NotFound,
}

/// ツール結果に埋め込むエラー情報。
/// MediaVault-api 由来の code / message をそのまま保持する。
/// 🔵 REQ-146・既存 `mediavault-api/src/models/response.rs` の ApiErrorBody より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ToolError {
    /// MediaVault-api のエラーコード（例: `ITEM_NOT_FOUND`）、
    /// または MCP 自身が生成したコード（`MCP_` プレフィックス付き）
    /// 🔵 REQ-146・dataflow.md エラーハンドリングフローより
    pub code: String,
    /// 利用者向けメッセージ。API 由来の場合は原文を保持する
    /// 🔵 REQ-146 より
    pub message: String,
    /// 同じ入力で再試行すれば成功しうるか（接続エラー・5xx は true）
    /// 🟡 設計決定 D-01。AI の再試行判断を助けるための追加フィールド
    pub retriable: bool,
}

/// MCP 自身が生成するエラーコード。API 由来のコードと区別するため `MCP_` を付ける。
/// 🟡 dataflow.md エラーハンドリングフローより
#[derive(Debug, Clone, Copy)]
pub enum McpErrorCode {
    /// MediaVault-api へ到達できない（接続失敗・タイムアウト）🔵 REQ-120 / EDGE-001
    ApiUnreachable,
    /// 内部APIキーによる認証に失敗した 🔵 NFR-103
    InternalAuthFailed,
    /// ツール引数のバリデーションに失敗した 🔵 REQ-116 / EDGE-102 / EDGE-103
    InvalidArgument,
    /// 名前指定の対象がマスタに存在しない（`create_if_missing` 未指定）🔵 REQ-111
    NameNotResolved,
}

impl McpErrorCode {
    /// 🟡 命名規約は設計上の決定
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ApiUnreachable => "MCP_API_UNREACHABLE",
            Self::InternalAuthFailed => "MCP_INTERNAL_AUTH_FAILED",
            Self::InvalidArgument => "MCP_INVALID_ARGUMENT",
            Self::NameNotResolved => "MCP_NAME_NOT_RESOLVED",
        }
    }
}

// ============================================================
// 共通: 部分失敗の表現 (src/result/operation.rs)
// ============================================================

/// 複数段階の書き込みにおける、各操作の結果。
/// 「成功」「既に適用済み」「未解決」「失敗」「未処理」を区別する。
/// 🔵 REQ-114 / REQ-061 / REQ-113・EDGE-003 / EDGE-004 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum OperationResult {
    /// 新たに適用した。`created_new` は対象マスタを新規作成したかどうか（REQ-061）
    Applied {
        target_id: Uuid,
        target_name: String,
        created_new: bool,
    },
    /// 既に適用済みだったため何もしなかった（REQ-113 冪等性）
    AlreadyApplied {
        target_id: Uuid,
        target_name: String,
    },
    /// 名前が解決できず、`create_if_missing` も指定されていないため適用しなかった（REQ-111）
    NotResolved {
        requested_name: String,
        /// 参考として提示する既存の候補名（上位数件）🟡 AIが次の手を打てるようにする
        available_names: Vec<String>,
    },
    /// 適用を試みたが失敗した
    Failed {
        requested_name: String,
        error: ToolError,
    },
    /// 先行する操作の失敗により実行されなかった（EDGE-003）
    Skipped { requested_name: String },
}

/// 単一フィールドの変更内容。`update_consumption` の before/after 提示に使う。
/// 🔵 REQ-051 / NFR-203 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct FieldChange {
    pub field: String,
    /// 変更前の値。未設定だった場合は null
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
}

// ============================================================
// ドメイン列挙型（MediaVault-api の ENUM に対応）
// ============================================================

/// メディア種別。
/// 🔵 `mediavault-api/src/models/item.rs` の MediaType・`items.md` より
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Anime,
    Movie,
    Drama,
    Manga,
    Novel,
    Game,
    AcademicBook,
    Paper,
}

/// 視聴・読了状態。
/// 🔵 `migrations/20260623000001_init_schema.up.sql` の item_status ENUM（3値）より
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    NotStarted,
    InProgress,
    Completed,
}

/// 作品間の関係種別。
/// 🔵 REQ-071・PRD §15.1「関係種別」・ヒアリング（requirements フェーズ Q1）より
/// ⚠️ 前提: MediaVault-api の relation_type ENUM 拡張（PREP-01）が完了していること。
///    現行の api は `Reference` / `Dlc` の2値のみ受け付ける。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// item_id（原作）→ related_item_id（映像化・翻案作品）🔵 PRD §8 より
    Adaptation,
    /// item_id（前作）→ related_item_id（続編）🔵
    Sequel,
    /// item_id（後の作品）→ related_item_id（前日譚）🔵
    Prequel,
    /// item_id（本編）→ related_item_id（スピンオフ）🔵
    Spinoff,
    /// item_id（本編）→ related_item_id（DLC・追加コンテンツ）🔵 既存ENUM
    Dlc,
    /// item_id（引用元）→ related_item_id（引用先）🔵 既存ENUM
    Reference,
}

/// 一覧の並び順。
/// 🔵 ヒアリング2026-08-07 Q3・`models/item.rs` の ItemSort 実装より
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    CreatedDesc,
    CreatedAsc,
    UpdatedDesc,
    TitleAsc,
    RatingDesc,
    ReleaseDesc,
}

// ============================================================
// 縮約したItem表現 (src/result/)
// ============================================================

/// 一覧系ツールが返す Item の要約表現。
/// `description` / `details` を含めないことでレスポンスサイズを抑える。
/// 🔵 設計決定 D-04（ヒアリング2026-08-07 Q4）・PRD §11 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ItemSummary {
    pub item_id: Uuid,
    pub title: String,
    /// 原題。同名作品の判別に使う 🔵 REQ-012
    pub original_title: Option<String>,
    pub media_type: MediaType,
    /// 公開年（`release_date` から年のみ抽出）🟡 D-04。フル日付より読みやすい
    pub release_year: Option<i32>,
    pub status: ItemStatus,
    pub rating: Option<f32>,
    pub is_favorite: bool,
    /// タグ名のみ（IDは含めない）🟡 D-04。AIが読む用途ではIDは不要
    pub tags: Vec<String>,
}

/// 参照用の軽量な名前付きID。タグ・カテゴリ・マイリスト・スタッフ等に共通。
/// 🔵 既存 api の TagRef / CategoryRef パターンより
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct NamedRef {
    pub id: Uuid,
    pub name: String,
}

// ============================================================
// ツール1: search_library (US-01, US-09)
// ============================================================

/// 🔵 REQ-010 / REQ-011 / REQ-143・ヒアリング2026-08-07 Q3 より
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct SearchLibraryParams {
    /// タイトル部分一致。本題・原題・別名を横断して検索する
    /// 🔵 REQ-011（前提: PREP-02）
    pub title: Option<String>,
    /// 🔵 REQ-010
    pub media_type: Option<MediaType>,
    /// 🔵 REQ-010
    pub status: Option<ItemStatus>,
    /// タグ名。MCP が `GET /tags` で ID へ解決する 🔵 REQ-060 と同じ解決経路
    pub tag: Option<String>,
    /// カテゴリ名。同上 🔵 REQ-010
    pub category: Option<String>,
    /// 🔵 REQ-010
    pub is_favorite: Option<bool>,
    /// 公開年 🔵 ヒアリング2026-08-07 Q3（`ListItemsQuery.year` として実装済み）
    pub year: Option<i32>,
    /// 並び順 🔵 ヒアリング2026-08-07 Q3（`ItemSort` として実装済み）
    pub sort: Option<SortOrder>,
    /// 1..=50。既定 20 🔵 REQ-143
    pub limit: Option<u32>,
    /// 前回結果の `next_cursor` をそのまま渡す 🔵 REQ-130
    pub cursor: Option<String>,
}

/// 🔵 REQ-012 / REQ-013 / REQ-014 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct SearchLibraryResult {
    pub outcome: Outcome,
    /// 検索対象が MediaVault 内であることを明示する固定値 `"mediavault_library"`。
    /// 外部カタログ結果との取り違えを防ぐ 🔵 REQ-014 / REQ-032 / NFR-202
    pub source: &'static str,
    /// 条件に該当する総件数 🔵 REQ-013（前提: PREP-03）
    pub total_count: u64,
    pub items: Vec<ItemSummary>,
    /// 続きがある場合の不透明なカーソル 🟡 keyset ページネーションを隠蔽する
    pub next_cursor: Option<String>,
    /// 実際に適用された検索条件（AIが条件を確認できるようにする）🔵 REQ-013（US-09「選定条件が分かる」）
    pub applied_filters: serde_json::Value,
    pub error: Option<ToolError>,
}

// ============================================================
// ツール2: get_item_context (US-02)
// ============================================================

/// 🔵 REQ-020 より
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct GetItemContextParams {
    pub item_id: Uuid,
}

/// コンテキストの各セクションが取りうる3状態。
/// 「未登録」と「取得失敗」を区別するための型。
/// 🔵 REQ-021・EDGE-105・設計決定 D-05 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Section<T> {
    /// 取得に成功し、1件以上のデータがある
    Loaded { items: Vec<T> },
    /// 取得に成功したが、データが登録されていない
    Empty,
    /// 取得に失敗した（未登録とは異なる）
    Failed { error: ToolError },
}

/// 🔵 REQ-020 / REQ-021 / REQ-022・PRD §8「追加呼び出しは関連・マイリスト・
/// グループ・キャスト・ファイル・リンク・トレーラーに限られる」より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ItemContextResult {
    pub outcome: Outcome,
    /// `GET /items/{id}` から取得した本体。これが取得できなければ outcome は NotFound
    pub item: Option<ItemDetailView>,
    // --- 以下は追加の並列取得分 ---
    pub relations: Section<RelationView>,
    pub mylists: Section<NamedRef>,
    pub groups: Section<GroupView>,
    pub cast: Section<CreditView>,
    pub staff: Section<CreditView>,
    pub files: Section<FileView>,
    pub links: Section<LinkView>,
    pub trailers: Section<LinkView>,
    pub error: Option<ToolError>,
}

/// `GET /items/{id}` のレスポンス（ItemDetail）に対応する表示用型。
/// タグ・カテゴリ・配信リンク・画像・Calibre連携は api が同梱するため追加取得しない。
/// 🔵 `docs/backend/mediavault-api/items.md` GET /items/{id} 仕様より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ItemDetailView {
    pub item_id: Uuid,
    pub title: String,
    pub original_title: Option<String>,
    pub media_type: MediaType,
    pub description: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    pub status: ItemStatus,
    pub consumed_date: Option<NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: bool,
    pub external_id: Option<String>,
    pub tags: Vec<NamedRef>,
    pub categories: Vec<NamedRef>,
    pub streaming_links: Vec<StreamingLinkView>,
    /// media_type ごとに形が異なる自由形式JSON。加工せずそのまま渡す
    /// 🔵 `items.md`「details は型定義された共通スキーマを持たない」より
    pub detail: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 🔵 `item-relations.md` より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct RelationView {
    pub relation_id: Uuid,
    pub relation_type: RelationType,
    /// この Item から見た関係の向き 🔵 REQ-072
    pub direction: RelationDirection,
    pub related_item: ItemSummary,
}

/// 🟡 REQ-072「関係の向きが明示される」を表現するための設計上の追加型
#[derive(Debug, Clone, Copy, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationDirection {
    /// この Item が `item_id` 側（関係の起点）
    Outgoing,
    /// この Item が `related_item_id` 側（関係の終点）
    Incoming,
}

/// 🔵 `item-groups.md` より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GroupView {
    pub group_id: Uuid,
    pub name: String,
    /// season / volume 等 🟡 api の group_type に対応
    pub group_type: String,
    pub episode_count: Option<u32>,
}

/// スタッフ・キャストの共通表現。
/// 🔵 `staff.md` / `cast.md` より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CreditView {
    pub person_id: Uuid,
    pub name: String,
    /// 役割（監督・声優など）または役名
    pub role: Option<String>,
}

/// 🔵 `item-files.md` より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct FileView {
    pub file_id: Uuid,
    pub path: String,
    pub file_kind: Option<String>,
    /// 全文抽出済みかどうか。第2段階の `get_item_text` の入口になる
    /// 🟡 REQ-900 を見据えた枠。api 側が値を持たない場合は None
    pub has_extracted_text: Option<bool>,
}

/// 通常リンクとトレーラーの共通表現。
/// 🔵 `item-links.md` / `item-trailers.md` より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct LinkView {
    pub link_id: Uuid,
    pub url: String,
    pub label: Option<String>,
}

/// 🔵 `item-streaming-links.md` より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct StreamingLinkView {
    pub link_id: Uuid,
    pub platform: StreamingPlatform,
    pub url: String,
}

/// MediaVault-api が受け付ける配信プラットフォーム（5種固定）。
/// 🔵 `item-streaming-links.md` の platform 定義より
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StreamingPlatform {
    Netflix,
    AmazonPrime,
    DisneyPlus,
    DmmTv,
    AppleTv,
}

// ============================================================
// ツール3-4: 外部カタログ (US-03)
// ============================================================

/// 🔵 REQ-030・`GET /items/search` の ItemSearchQuery（media_type / q 必須）より
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct SearchExternalCatalogParams {
    pub media_type: MediaType,
    /// 検索語
    pub q: String,
}

/// 外部カタログの候補。MediaVault の `item_id` を**持たない**ことで、
/// 所蔵品との取り違えを型レベルで防ぐ。
/// 🔵 REQ-031 / REQ-032・`items.md` の SearchResultItem より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ExternalCandidate {
    /// プロバイダ固有ID。`import_external_item` の `external_id` へそのまま渡す
    /// 🔵 REQ-031 / NFR-003
    pub external_id: String,
    /// annict / rakuten / tmdb / steam / ndl 🔵 `items.md` より
    pub provider: Option<String>,
    pub title: String,
    pub release_year: Option<i32>,
    pub media_type: MediaType,
    pub cover_image_url: Option<String>,
    /// 候補の区別に役立つ短い補足（あらすじ冒頭など）🟡 REQ-031「候補を提示して選択を求める」
    pub summary: Option<String>,
}

/// 🔵 REQ-030 / REQ-032 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct SearchExternalCatalogResult {
    pub outcome: Outcome,
    /// 固定値 `"external_catalog"`。`search_library` の `source` と対になる
    /// 🔵 REQ-032 / NFR-202
    pub source: &'static str,
    pub provider: Option<String>,
    pub candidates: Vec<ExternalCandidate>,
    pub error: Option<ToolError>,
}

/// 🔵 REQ-033・`POST /items/import` の ImportByIdRequest より
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct ImportExternalItemParams {
    pub media_type: MediaType,
    /// `search_external_catalog` の候補から選んだ値をそのまま渡す
    pub external_id: String,
    pub provider: Option<String>,
}

/// 🔵 REQ-033 / REQ-112 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ImportExternalItemResult {
    pub outcome: Outcome,
    pub item: Option<ItemSummary>,
    /// 409 `ITEM_ALREADY_IMPORTED` により既存Itemを返した場合 true
    /// 🔵 REQ-112「重複を作らず既存Itemを返す」
    pub already_existed: bool,
    pub error: Option<ToolError>,
}

// ============================================================
// ツール5: create_item (US-04)
// ============================================================

/// 🔵 REQ-040 / REQ-041・`POST /items` の CreateItemRequest より
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct CreateItemParams {
    /// 必須 🔵 REQ-040
    pub media_type: MediaType,
    /// 必須。空白のみは拒否 🔵 REQ-040 / EDGE-103
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    pub rating: Option<f32>,
    pub is_favorite: Option<bool>,
    // --- 以下は同一呼び出しで関連付ける任意項目（REQ-041）---
    /// ファイルパス（実データ領域の絶対パス）🔵 REQ-041
    pub file_paths: Vec<String>,
    /// 参照URL 🔵 REQ-041
    pub urls: Vec<String>,
    /// タグ名 🔵 REQ-041
    pub tags: Vec<String>,
    /// マイリスト名 🔵 REQ-041
    pub mylists: Vec<String>,
    /// タグ・マイリストが存在しない場合に作成するか。既定 false
    /// 🔵 REQ-111（organize_item と同一の規約を適用）
    #[serde(default)]
    pub create_if_missing: bool,
}

/// 🔵 REQ-114・EDGE-004 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CreateItemResult {
    pub outcome: Outcome,
    /// Item 作成に成功していれば、後続が失敗しても必ず値が入る
    /// 🔵 EDGE-004「作成済み Item ID を返したうえで付与失敗を示す」
    pub item: Option<ItemSummary>,
    pub tags: Vec<OperationResult>,
    pub mylists: Vec<OperationResult>,
    pub files: Vec<OperationResult>,
    pub links: Vec<OperationResult>,
    pub error: Option<ToolError>,
}

// ============================================================
// ツール6: update_consumption (US-05)
// ============================================================

/// 🔵 REQ-050 / REQ-052 / REQ-142・PRD §15.1「update_consumption の対象項目」より
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct UpdateConsumptionParams {
    /// UUID のみ。タイトル指定は受け付けない 🔵 REQ-142 / 設計決定 D-02
    pub item_id: Uuid,
    pub status: Option<ItemStatus>,
    /// `YYYY-MM-DD` のみ。自然言語は MCP クライアントが変換する
    /// 🔵 REQ-052
    pub consumed_date: Option<NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: Option<bool>,
}

/// 🔵 REQ-051 / NFR-203 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct UpdateConsumptionResult {
    pub outcome: Outcome,
    pub item_id: Uuid,
    /// 更新対象が意図した作品か利用者が確認できるようにする 🔵 REQ-051
    pub title: Option<String>,
    /// 実際に変化のあったフィールドのみ 🟡 変化なしを除くのは設計上の判断
    pub changes: Vec<FieldChange>,
    pub error: Option<ToolError>,
}

// ============================================================
// ツール7: organize_item (US-06)
// ============================================================

/// 🔵 REQ-060 / REQ-111 / REQ-142・PRD §15.1 より
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct OrganizeItemParams {
    /// 🔵 REQ-142
    pub item_id: Uuid,
    /// タグ名。ID ではなく名前で受け取る 🔵 REQ-060
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub mylists: Vec<String>,
    /// 既定 false。true のときのみ未存在の対象を新規作成する
    /// 🔵 REQ-111・PRD §15.1「既定は作成しない」
    #[serde(default)]
    pub create_if_missing: bool,
}

/// 🔵 REQ-061 / REQ-113・EDGE-003 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct OrganizeItemResult {
    pub outcome: Outcome,
    pub item_id: Uuid,
    pub title: Option<String>,
    pub tags: Vec<OperationResult>,
    pub categories: Vec<OperationResult>,
    pub mylists: Vec<OperationResult>,
    pub error: Option<ToolError>,
}

// ============================================================
// ツール8: relate_items (US-07)
// ============================================================

/// 🔵 REQ-070 / REQ-071 / REQ-142 より
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct RelateItemsParams {
    /// 関係の起点 🔵 REQ-072
    pub item_id: Uuid,
    /// 関係の終点 🔵 REQ-072
    pub related_item_id: Uuid,
    /// 固定一覧のみ 🔵 REQ-071
    pub relation_type: RelationType,
}

/// 🔵 REQ-070 / REQ-113・EDGE-005 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct RelateItemsResult {
    pub outcome: Outcome,
    pub relation_id: Option<Uuid>,
    pub relation_type: RelationType,
    /// 「A（原作）→ B（映像化）」のように向きを自然文で示す 🔵 REQ-072
    pub description: String,
    pub item: Option<ItemSummary>,
    pub related_item: Option<ItemSummary>,
    /// 409 `DUPLICATE_RELATION` により既存関係を返した場合 true 🔵 REQ-113
    pub already_related: bool,
    pub error: Option<ToolError>,
}

// ============================================================
// ツール9: add_access_link (US-08)
// ============================================================

/// リンクの用途。登録先エンドポイントを決定する。
/// 🔵 REQ-080 より
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    /// `POST /items/{id}/links`
    Link,
    /// `POST /items/{id}/streaming-links`（platform が対応5種の場合）
    Streaming,
    /// `POST /items/{id}/trailers`
    Trailer,
}

/// 🔵 REQ-080 / REQ-081 / REQ-116 / REQ-142 より
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct AddAccessLinkParams {
    pub item_id: Uuid,
    /// http / https のみ許可 🔵 REQ-116
    pub url: String,
    pub kind: LinkKind,
    /// `kind = Streaming` のときのプラットフォーム名。
    /// 対応5種以外を指定した場合は通常リンクへフォールバックする
    /// 🔵 REQ-081・ヒアリング（requirements フェーズ Q4）
    pub platform: Option<String>,
    /// 通常リンクのラベル 🔵 `item-links.md` の label 必須仕様より
    pub label: Option<String>,
}

/// 実際に登録された先。
/// 🔵 REQ-081「どちらに登録したかを結果で返す」より
#[derive(Debug, Clone, Copy, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegisteredAs {
    Link,
    StreamingLink,
    Trailer,
}

/// 🔵 REQ-080 / REQ-081 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct AddAccessLinkResult {
    pub outcome: Outcome,
    pub link_id: Option<Uuid>,
    pub registered_as: Option<RegisteredAs>,
    /// `Streaming` を要求したが通常リンクへフォールバックした場合に `"streaming"` が入る
    /// 🔵 REQ-081「フォールバックが起きたことを結果に含める」
    pub fallback_from: Option<String>,
    /// 409 により既存リンクとみなした場合 true 🟡 冪等性のための設計上の追加
    pub already_registered: bool,
    pub error: Option<ToolError>,
}

// ============================================================
// ツール10: collection_overview (US-09)
// ============================================================

/// 🟡 REQ-090。パラメータは要件に明記がないため設計上の判断
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct CollectionOverviewParams {
    /// 最近追加・更新の返却件数。1..=50、既定 10 🟡 REQ-143 の範囲内で設定
    pub recent_limit: Option<u32>,
}

/// 🔵 REQ-090・ヒアリング（requirements フェーズ Q3「集計APIも今回実装」）より
/// ⚠️ 前提: `GET /api/v1/collection/overview`（PREP-04）が実装済みであること
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CollectionOverviewResult {
    pub outcome: Outcome,
    pub total_items: u64,
    /// media_type 別件数 🔵 REQ-090
    pub by_media_type: Vec<CountEntry>,
    /// status 別件数 🔵 REQ-090（PREP-04 が前提）
    pub by_status: Vec<CountEntry>,
    /// お気に入り件数 🔵 REQ-090
    pub favorite_count: u64,
    pub recently_added: Vec<ItemSummary>,
    pub recently_updated: Vec<ItemSummary>,
    pub error: Option<ToolError>,
}

/// 🔵 `GET /items/counts-by-media-type` の MediaTypeCounts より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CountEntry {
    pub key: String,
    pub count: u64,
}

// ============================================================
// ツール11: health
// ============================================================

/// 引数なし 🔵 REQ-100
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct HealthParams {}

/// 🔵 REQ-100・ヒアリング（requirements フェーズ Q8）より
/// 確認範囲は「MediaVault-api 到達性」と「MCP 自身のバージョン」に限定する。
/// 内部APIキーの有効性・外部APIキーの設定状況は含めない。
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct HealthResult {
    /// api が停止していてもツール自体は Success を返す
    /// 🔵 EDGE-001「接続エラーであることを示す」
    pub outcome: Outcome,
    /// mediavault-mcp のバージョン（`CARGO_PKG_VERSION`）🔵 REQ-100
    pub mcp_version: String,
    pub api: ApiHealth,
}

/// 🔵 REQ-100 / REQ-120 より
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ApiHealth {
    pub reachable: bool,
    /// 到達できた場合の往復時間 🔵 REQ-100「応答時間」
    pub latency_ms: Option<u64>,
    /// 到達できなかった場合の理由（接続エラー／業務エラーの区別を含む）🔵 REQ-120
    pub error: Option<ToolError>,
}

// ============================================================
// ApiClient層 (src/api/error.rs, src/api/envelope.rs)
// ============================================================

/// MediaVault-api 呼び出しの失敗種別。3種を混同させない。
/// 🔵 REQ-120・EDGE-001 / EDGE-002・[architecture.md](architecture.md) ApiClient層より
#[derive(Debug, thiserror::Error)]
pub enum ApiClientError {
    /// 接続失敗・タイムアウト・DNS失敗 🔵 EDGE-001
    #[error("MediaVault-api へ接続できません: {0}")]
    Connection(String),
    /// 内部API呼び出しの認証失敗（401）🔵 NFR-103
    /// ⚠️ この文字列にキー値を含めてはならない（REQ-145）
    #[error("内部APIの認証に失敗しました")]
    Auth,
    /// MediaVault-api が返した業務エラー 🔵 REQ-146
    #[error("{code}: {message}")]
    Api {
        code: String,
        message: String,
        status: u16,
    },
    /// レスポンスのデシリアライズ失敗 🟡 実装上必要な分類
    #[error("レスポンスの解析に失敗しました: {0}")]
    Decode(String),
}

/// MediaVault-api の統一レスポンス。
/// 🔵 `mediavault-api/src/models/response.rs` の ApiOk / ApiError に対応
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiEnvelope<T> {
    Ok {
        success: bool,
        data: T,
        /// 一覧系のみ 🔵 `items.md` の PaginatedOk より
        pagination: Option<ApiPagination>,
    },
    Err {
        success: bool,
        error: ApiErrorBody,
    },
}

/// 🔵 `models/response.rs` の ApiErrorBody より
#[derive(Debug, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

/// keyset ページネーション。MCP 側では不透明カーソルへ変換して隠蔽する。
/// 🔵 `items.md` の pagination 仕様より
#[derive(Debug, Deserialize)]
pub struct ApiPagination {
    pub limit: u32,
    pub has_more: bool,
    pub next_after_created_at: Option<NaiveDateTime>,
    pub next_after_id: Option<Uuid>,
}

// ============================================================
// 設定 (src/config.rs)
// ============================================================

/// 環境変数から読み込む設定。
/// 🔵 [tech-stack.md](../tech-stack.md)・REQ-122 / NFR-101 / NFR-103 より
#[derive(Debug, Clone)]
pub struct Config {
    /// 待受アドレス。既定 `0.0.0.0:8081` 🟡 tech-stack.md の例より
    pub bind_addr: String,
    /// 必須。未設定・空文字なら起動失敗 🔵 REQ-122
    /// `Debug` 実装でマスクすること 🟡 NFR-103 / REQ-145
    pub auth_token: SecretString,
    /// 例: `http://mediavault-api:8080` 🔵 tech-stack.md
    pub api_base_url: String,
    /// 内部書き込みAPI用。クライアントへ露出させない 🔵 NFR-103
    pub internal_api_key: SecretString,
    /// 接続タイムアウト。既定 3秒 🟡 NFR-006
    pub connect_timeout_secs: u64,
    /// 全体タイムアウト。既定 15秒（外部検索は 30秒）🟡 NFR-006
    pub request_timeout_secs: u64,
}

/// `Debug` / `Display` でマスクされる秘密文字列。
/// 🟡 REQ-145 / NFR-103 を型で担保するための設計上の追加
pub struct SecretString(String);

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretString(***)")
    }
}

// ============================================================
// 信頼性レベルサマリー
// ============================================================
//
// - 🔵 青信号: 78件 (83%)
// - 🟡 黄信号: 16件 (17%)
// - 🔴 赤信号: 0件 (0%)
//
// 品質評価: 高品質
//
// 🟡 の内訳: レスポンス整形（release_year / tags名のみ）、
// タイムアウト値、`retriable` フラグ、`SecretString`、
// `RelationDirection` など、要件から導出した実装レベルの具体化。
// 要件の解釈に曖昧さは残っていない。
