//! MediaVault-api のレスポンス型（必要分のみ）
//!
//! TASK-0012: ItemSummary 縮約と名前→ID解決サービス

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// メディア種別。
///
/// 🔵 Intent: `docs/backend/mediavault-api/items.md`・interfaces.rs `MediaType` より。
///    api からの受信（Deserialize）と `ItemSummary` としての送信（Serialize）の
///    両方に使うため両方 derive する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
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
///
/// 🔵 Intent: interfaces.rs `ItemStatus`（3値）より。MediaType 同様、受信・送信の両方で使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    NotStarted,
    InProgress,
    Completed,
}

/// 一覧の並び順。
///
/// 🔵 Intent: interfaces.rs `SortOrder`・`items.md` GET /items の `sort` パラメータより。
///    api 側は `sort` の昇降順を固定で決めるため、MCP 側の値も `sort` クエリへそのまま渡せる
///    ように api の値（`created_at`/`updated_at`/`rating`/`title`/`release_date`）へ対応させる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    CreatedDesc,
    CreatedAsc,
    UpdatedDesc,
    TitleAsc,
    RatingDesc,
    ReleaseDesc,
}

impl SortOrder {
    /// api の `sort` クエリ値へ変換する。
    ///
    /// 🟡 Intent: api 側は `created_at` の降順/昇順を明示的なパラメータで区別しないため
    ///    （`items.md` は `created_at`は常に降順と記載）、`CreatedAsc` は api に存在しない値。
    ///    MVP では昇順指定の需要が薄いため `created_at` にフォールバックする設計判断。
    pub fn as_api_sort(self) -> &'static str {
        match self {
            SortOrder::CreatedDesc | SortOrder::CreatedAsc => "created_at",
            SortOrder::UpdatedDesc => "updated_at",
            SortOrder::TitleAsc => "title",
            SortOrder::RatingDesc => "rating",
            SortOrder::ReleaseDesc => "release_date",
        }
    }
}

/// `POST /items` / `PATCH /items/{id}` 等が返す `Item`（tags/categories なし）。
///
/// 🔵 Intent: `items.md` の `Item` レスポンス形より。`ItemSummary` への縮約元。
#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub media_type: MediaType,
    pub title: String,
    pub original_title: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub status: ItemStatus,
    pub rating: Option<f32>,
    pub is_favorite: bool,
}

/// タグ・カテゴリの参照表現（api レスポンス上の `{id, name}`）。
///
/// 🔵 Intent: `items.md` の `TagRef` / `CategoryRef` より。
#[derive(Debug, Clone, Deserialize)]
pub struct ApiNamedRef {
    pub id: Uuid,
    pub name: String,
}

/// `GET /items` が返す `ItemWithRefs`（`Item` の全フィールド + tags + categories）。
///
/// 🔵 Intent: `items.md` `GET /items` レスポンス形より。
#[derive(Debug, Clone, Deserialize)]
pub struct ItemWithRefs {
    pub id: Uuid,
    pub media_type: MediaType,
    pub title: String,
    pub original_title: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub status: ItemStatus,
    pub rating: Option<f32>,
    pub is_favorite: bool,
    /// 409 冪等化での既存Item特定に使う（`import_external_item` のみ利用）🔵
    #[serde(default)]
    pub external_id: Option<String>,
    pub tags: Vec<ApiNamedRef>,
    #[serde(default)]
    pub categories: Vec<ApiNamedRef>,
}

/// `GET /tags` が返す `TagWithCount`。名前解決には `id` / `name` のみ使う。
///
/// 🔵 Intent: `tags.md`「`TagWithCount = { id, name, item_count }`」より。
#[derive(Debug, Clone, Deserialize)]
pub struct TagWithCount {
    pub id: Uuid,
    pub name: String,
    #[allow(
        dead_code,
        reason = "名前解決では未使用。api 形状との対応を明示するため保持"
    )]
    pub item_count: u64,
}

/// `GET /categories` が返す `CategoryWithCount`。タグと同形。
///
/// 🔵 Intent: `categories.md`「タグと構造は同一」より。
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryWithCount {
    pub id: Uuid,
    pub name: String,
    #[allow(
        dead_code,
        reason = "名前解決では未使用。api 形状との対応を明示するため保持"
    )]
    pub item_count: u64,
}

/// `GET /mylists` が返す `Mylist`。
///
/// 🟡 Intent: `mylists.md` にレスポンス項目の詳細がなく、名前解決に必要な `id` / `name`
///    のみを取り出す。他フィールドは `serde` が既定で無視する。
#[derive(Debug, Clone, Deserialize)]
pub struct Mylist {
    pub id: Uuid,
    pub name: String,
}

/// 作品間の関係種別。
///
/// 🔵 Intent: `item-relations.md`「relation_type と向きの意味」の6値より。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Adaptation,
    Sequel,
    Prequel,
    Spinoff,
    Dlc,
    Reference,
}

/// `GET /items/{id}/relations` が返す `ItemRelation`。
///
/// 🔵 Intent: `data-model.md` ItemRelation より。
#[derive(Debug, Clone, Deserialize)]
pub struct ItemRelation {
    pub id: Uuid,
    pub item_id: Uuid,
    pub related_item_id: Uuid,
    pub relation_type: RelationType,
}

/// `GET /items/{id}/groups` が返す `ItemGroup`。
///
/// 🔵 Intent: `data-model.md` ItemGroup より。
#[derive(Debug, Clone, Deserialize)]
pub struct ItemGroup {
    pub id: Uuid,
    pub group_type: String,
    pub group_name: String,
    pub number: Option<f64>,
    /// 親作品（シリーズ本体）の item_id。
    ///
    /// 🔵 Intent: 設計決定 D-07 より。`mediavault-api/src/models/item_group.rs` の
    ///    `parent_item_id: Option<Uuid>` に対応する。シリーズ名解決の**唯一の一次情報**であり、
    ///    `group_name`（"Season 1" 等）を代用してはならない。
    #[serde(default)]
    pub parent_item_id: Option<Uuid>,
}

/// `GET /items/{id}/staff` が返す `ItemStaff`。
///
/// 🔵 Intent: `data-model.md` ItemStaff より。**人物名は含まれない**（`staff_id` のみ）。
#[derive(Debug, Clone, Deserialize)]
pub struct ItemStaff {
    pub staff_id: Uuid,
    pub role: String,
}

/// `GET /items/{id}/cast` が返す `ItemCast`。
///
/// 🔵 Intent: `data-model.md` ItemCast より。**人物名は含まれない**（`cast_id` のみ）。
#[derive(Debug, Clone, Deserialize)]
pub struct ItemCast {
    pub cast_id: Uuid,
    pub character_name: Option<String>,
}

/// `GET /items/{id}/files` が返す `ItemFile`。
///
/// 🔵 Intent: `data-model.md` ItemFile より。
#[derive(Debug, Clone, Deserialize)]
pub struct ItemFile {
    pub id: Uuid,
    pub path: String,
    pub file_type: String,
}

/// `GET /items/{id}/links` が返す `ItemLink`。
///
/// 🔵 Intent: `data-model.md` ItemLink より。
#[derive(Debug, Clone, Deserialize)]
pub struct ItemLink {
    pub id: Uuid,
    pub url: String,
    pub label: String,
}

/// `GET /items/{id}/trailers` が返す `ItemTrailer`。
///
/// 🔵 Intent: `data-model.md` ItemTrailer より。
#[derive(Debug, Clone, Deserialize)]
pub struct ItemTrailer {
    pub id: Uuid,
    pub url: String,
    pub label: Option<String>,
}

/// `GET /items/search` が返す `SearchResultItem`。
///
/// 🔵 Intent: `items.md` GET /items/search レスポンス形より。`year` は api 側で
///    プロバイダのレスポンスから抽出済みの整数として返るため、MCP 側で日付文字列から
///    再抽出する処理は不要（`ItemWithRefs.release_date` とは異なる形）。
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResultItem {
    /// プロバイダ固有ID。`media_type` により意味が異なる 🔵
    pub id: String,
    pub media_type: MediaType,
    pub provider: Option<String>,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub year: Option<i32>,
}

/// MediaVault-api が受け付ける配信プラットフォーム（5種固定）。
///
/// 🔵 Intent: `item-streaming-links.md` の platform 定義より。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StreamingPlatform {
    Netflix,
    AmazonPrime,
    DisneyPlus,
    DmmTv,
    AppleTv,
}

/// `ItemDetail.streaming_links` の要素。
///
/// 🔵 Intent: `item-streaming-links.md`「ItemDetail拡張」より。
#[derive(Debug, Clone, Deserialize)]
pub struct ApiStreamingLink {
    pub id: Uuid,
    pub platform: StreamingPlatform,
    pub url: String,
}

/// `GET /collection/overview` が返す `key`/`count` の集計エントリ。
///
/// 🔵 Intent: `docs/backend/mediavault-api/collection.md`・TASK-0004 `CountEntry` より。
#[derive(Debug, Clone, Deserialize)]
pub struct CountEntry {
    pub key: String,
    pub count: i64,
}

/// `GET /collection/overview` が返す `CollectionOverview`。
///
/// 🔵 Intent: TASK-0004 `models::collection::CollectionOverview` より。
#[derive(Debug, Clone, Deserialize)]
pub struct CollectionOverview {
    pub total_items: i64,
    pub favorite_count: i64,
    pub by_media_type: Vec<CountEntry>,
    pub by_status: Vec<CountEntry>,
    pub recently_added: Vec<ItemWithRefs>,
    pub recently_updated: Vec<ItemWithRefs>,
}

/// `GET /items/{id}` が返す `ItemDetail`。
///
/// 🔵 Intent: `items.md` GET /items/{id} レスポンス形より。`calibre_links` / `images` /
///    `cover_image_url` / `source` は `ItemDetailView` に含めないため定義しない
///    （serde は構造体に無いフィールドを既定で無視する）。
#[derive(Debug, Clone, Deserialize)]
pub struct ItemDetail {
    pub id: Uuid,
    pub media_type: MediaType,
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    pub status: ItemStatus,
    pub consumed_date: Option<NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: bool,
    pub external_id: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    /// media_type ごとに形が異なる自由形式JSON。加工せずそのまま渡す 🔵 `items.md` より
    pub detail: Option<serde_json::Value>,
    pub tags: Vec<ApiNamedRef>,
    #[serde(default)]
    pub categories: Vec<ApiNamedRef>,
    #[serde(default)]
    pub streaming_links: Vec<ApiStreamingLink>,
}

/// 引用の位置情報の種別。
///
/// 🔵 Intent: `mediavault-api/src/models/citation.rs` の `LocatorType`・`citations.md` より。
///    api への送信（Serialize）と受信（Deserialize）の両方で使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LocatorType {
    /// ページ番号（書籍・論文）
    Page,
    /// 再生秒数（映像作品）
    Timestamp,
    /// 電子書籍の位置No.
    Location,
    /// 章
    Chapter,
    /// 位置情報なし
    None,
}

/// `GET /items/{id}/citations` が返す `Citation`。
///
/// 🔵 Intent: `mediavault-api/src/models/citation.rs` の `Citation` より。位置情報は
///    api の値をそのまま保持し、「p.128」のような表示文字列へ整形しない（REQ-146）。
#[derive(Debug, Clone, Deserialize)]
pub struct Citation {
    pub id: Uuid,
    pub quote_text: String,
    pub note: Option<String>,
    pub locator_type: LocatorType,
    pub page_number: Option<i32>,
    pub timestamp_seconds: Option<i32>,
    pub location_number: Option<i32>,
    pub chapter: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

/// `POST /items/{id}/citations` のリクエストボディ。
///
/// 🔵 Intent: `citations.md` `CreateCitationRequest` より。api 側は `locator_type` と
///    位置フィールドの整合を**必須バリデーションしない**ため、送信前に MCP 側で検証する
///    （設計決定 D-11 / REQ-904）。
#[derive(Debug, Clone, Serialize)]
pub struct CreateCitationRequest {
    pub quote_text: String,
    pub locator_type: LocatorType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter: Option<String>,
}
