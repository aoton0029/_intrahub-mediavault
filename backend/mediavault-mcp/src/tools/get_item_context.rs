//! Tool層: `get_item_context`
//!
//! TASK-0014: get_item_context ツールの実装

use uuid::Uuid;

use crate::api::models::{ItemStatus, MediaType, RelationType, StreamingPlatform};
use crate::result::outcome::{Outcome, ToolError};
use crate::result::summary::NamedRef;
use crate::result::{CountSection, Section, SeriesSection};

/// `get_item_context` ツールの引数。
///
/// 🔵 Intent: interfaces.rs `GetItemContextParams` より。`item_id` のみ。
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct GetItemContextParams {
    pub item_id: Uuid,
}

/// この Item から見た関係の向き。
///
/// 🔵 Intent: REQ-072・interfaces.rs `RelationDirection` より。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationDirection {
    /// この Item が `item_id` 側（関係の起点）
    Outgoing,
    /// この Item が `related_item_id` 側（関係の終点）
    Incoming,
}

/// 🟡 Intent: `GET /items/{id}/relations` は関連先 Item の詳細（title 等）を含まないため、
///    追加取得はせず `related_item_id` のみ返す縮退案を採用する
///    （タスクファイル実装項目7・NFR-002 を優先）。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct RelationView {
    pub relation_id: Uuid,
    pub relation_type: RelationType,
    pub direction: RelationDirection,
    pub related_item_id: Uuid,
}

/// 🟡 Intent: `item-groups.md` `ItemGroup` より。`episode_count` はエピソード一覧の
///    追加取得が必要になるため含めない（NFR-002 を優先）。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct GroupView {
    pub group_id: Uuid,
    pub name: String,
    pub group_type: String,
    pub number: Option<f64>,
}

/// スタッフ・キャストの共通表現。
///
/// 🟡 Intent: `staff.md` / `cast.md` の `ItemStaff` / `ItemCast` は人物名を含まない
///    （`staff_id` / `cast_id` のみ）。人物名解決には `GET /staff` 等の追加取得が
///    必要になるため、NFR-002（1回で完結）を優先し `role` のみ返す縮退案を採用する。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct CreditView {
    pub person_id: Uuid,
    /// スタッフの場合は `role`、キャストの場合は `character_name`
    pub role: Option<String>,
}

/// 🔵 Intent: `item-files.md` より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct FileView {
    pub file_id: Uuid,
    pub path: String,
    pub file_kind: String,
}

/// 通常リンクとトレーラーの共通表現。
///
/// 🔵 Intent: `item-links.md` / `item-trailers.md` より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct LinkView {
    pub link_id: Uuid,
    pub url: String,
    pub label: Option<String>,
}

/// 🔵 Intent: `item-streaming-links.md`「ItemDetail拡張」より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct StreamingLinkView {
    pub link_id: Uuid,
    pub platform: StreamingPlatform,
    pub url: String,
}

/// `GET /items/{id}` のレスポンス（`ItemDetail`）に対応する表示用型。
///
/// 🔵 Intent: interfaces.rs `ItemDetailView` より。タグ・カテゴリ・配信リンクは
///    api が同梱するため追加取得しない。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct ItemDetailView {
    pub item_id: Uuid,
    pub title: String,
    pub original_title: Option<String>,
    pub media_type: MediaType,
    pub description: Option<String>,
    pub release_date: Option<chrono::NaiveDate>,
    pub homepage_url: Option<String>,
    pub status: ItemStatus,
    pub consumed_date: Option<chrono::NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: bool,
    pub external_id: Option<String>,
    pub tags: Vec<NamedRef>,
    pub categories: Vec<NamedRef>,
    pub streaming_links: Vec<StreamingLinkView>,
    /// media_type ごとに形が異なる自由形式JSON。加工せずそのまま渡す
    /// 🔵 `items.md`「details は型定義された共通スキーマを持たない」より
    pub detail: Option<serde_json::Value>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

/// 🔵 Intent: interfaces.rs `ItemContextResult`・PRD §8 より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct ItemContextResult {
    pub outcome: Outcome,
    /// `GET /items/{id}` から取得した本体。取得できなければ outcome は NotFound
    pub item: Option<ItemDetailView>,
    /// シリーズ（親作品）。`groups` の `parent_item_id` から解決する。
    ///
    /// 🔵 Intent: 設計決定 D-07・intrahub-mastra REQ-016a より。**推測で埋めない**。
    ///    解決できない場合は `empty` を返し、利用側の未分類処理に委ねる。
    pub series: SeriesSection,
    pub relations: Section<RelationView>,
    pub mylists: Section<NamedRef>,
    pub groups: Section<GroupView>,
    pub cast: Section<CreditView>,
    pub staff: Section<CreditView>,
    pub files: Section<FileView>,
    pub links: Section<LinkView>,
    pub trailers: Section<LinkView>,
    /// 引用は**件数のみ**。本文は `list_citations` で取得する。
    ///
    /// 🟡 Intent: 設計決定 D-12・NFR-002 より。`quote_text` は長さ・件数とも上限がなく、
    ///    本文を含めると 1回あたりのレスポンスサイズが Item 依存で予測不能になる。
    pub citations: CountSection,
    pub error: Option<ToolError>,
}

impl ItemContextResult {
    /// `GET /items/{id}` が失敗し、他のセクションを取得しなかった場合の結果を組み立てる。
    ///
    /// 🟡 Intent: タスクファイル完了条件「404なら他を返さず not_found」より。
    ///    `Section` に「未実行」を表す状態がないため `Empty` で代替する
    ///    （未登録ではなく単に取得していない旨は `item: None` と `outcome` から判別できる）。
    pub fn early_return(outcome: Outcome, error: Option<ToolError>) -> Self {
        ItemContextResult {
            outcome,
            item: None,
            series: SeriesSection::Empty,
            relations: Section::Empty,
            mylists: Section::Empty,
            groups: Section::Empty,
            cast: Section::Empty,
            staff: Section::Empty,
            files: Section::Empty,
            links: Section::Empty,
            trailers: Section::Empty,
            citations: CountSection::Empty,
            error,
        }
    }
}
