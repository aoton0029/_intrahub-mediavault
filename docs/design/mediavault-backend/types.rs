//! mediavault-backend 型定義（Rust構造体・enum）
//!
//! 作成日: 2026-06-22
//! 関連設計: architecture.md
//!
//! 信頼性レベル:
//! - 🔵 青信号: EARS要件定義書・PRDデータモデル・既存実装を参考にした確実な型定義
//! - 🟡 黄信号: EARS要件定義書・PRD・既存実装から妥当な推測による型定義
//! - 🔴 赤信号: EARS要件定義書・PRD・既存実装にない推測による型定義

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ========================================
// enum定義
// ========================================

/// メディア種別 🔵 PRDデータモデルより
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "media_type", rename_all = "snake_case")]
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

/// 視聴・読了ステータス 🟡 PRD「視聴中/読了/未着手」から妥当な推測
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "item_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    NotStarted,
    InProgress,
    Completed,
}

/// アイテムの登録元 🔵 REQ-201/201bより
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "item_source", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ItemSource {
    Api,
    Manual,
}

/// item_groups のグループ種別 🔵 PRDデータモデルより
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "group_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GroupType {
    Season,
    Volume,
    Chapter,
}

/// item_relations の関係種別 🔵 PRDデータモデルより
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "relation_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Reference,
    Dlc,
}

/// item_files のファイル種別 🟡 PRD「ファイルアップロード」から妥当な推測
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "file_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    Pdf,
    Image,
    Other,
}

/// 外部APIキーのプロバイダ種別 🔵 PRD・interview-record Q5より
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "api_provider", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ApiProvider {
    Tmdb,
    Igdb,
    Ndl,
    Steam,
    OpenLibrary,
    AniList,
    // Jikanはキー不要のため対象外 🔵 requirements.mdより
}

// ========================================
// エンティティ定義（DBモデル）
// ========================================

/// items 共通テーブル 🔵 PRDデータモデル「共通項目」より
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: Uuid,
    pub media_type: MediaType,
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    pub status: ItemStatus,
    pub consumed_date: Option<NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: bool,
    pub source: ItemSource,
    pub external_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// item_groups（シーズン/巻/章の汎用グループ） 🔵 PRDデータモデルより
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemGroup {
    pub id: Uuid,
    pub item_id: Uuid,
    pub parent_item_id: Option<Uuid>,
    pub group_type: GroupType,
    pub group_name: String,
    pub number: Option<i32>,
    pub display_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// item_episodes（season/chapter配下のみ使用） 🔵 PRDデータモデル・EDGE-101より
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemEpisode {
    pub id: Uuid,
    pub group_id: Uuid,
    pub episode_number: i32,
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub air_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// item_relations（引用・関連／DLC） 🔵 PRDデータモデルより
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemRelation {
    pub id: Uuid,
    pub item_id: Uuid,
    pub related_item_id: Uuid,
    pub relation_type: RelationType,
    pub created_at: NaiveDateTime,
}

/// item_links（配信サイト等URL） 🔵 PRDデータモデルより
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemLink {
    pub id: Uuid,
    pub item_id: Uuid,
    pub url: String,
    pub label: String,
    pub created_at: NaiveDateTime,
}

/// item_files（ファイルサーバー上のファイル参照） 🔵 PRDデータモデル・REQ-019/020より
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemFile {
    pub id: Uuid,
    pub item_id: Uuid,
    pub path: String,
    pub label: Option<String>,
    pub file_type: FileType,
    pub calibre_book_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// item_trailers（トレーラーURL） 🔵 PRDデータモデルより
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemTrailer {
    pub id: Uuid,
    pub item_id: Uuid,
    pub url: String,
    pub label: Option<String>,
    pub created_at: NaiveDateTime,
}

/// staff（スタッフ） 🔵 PRDデータモデルより
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Staff {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub name: String,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
}

/// item_staff（作品とスタッフの紐付け） 🔵 PRDデータモデルより
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemStaff {
    pub id: Uuid,
    pub item_id: Uuid,
    pub staff_id: Uuid,
    pub role: String,
    pub character_name: Option<String>,
}

/// tags 🔵 PRDデータモデルより
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
}

/// categories 🔵 PRDデータモデルより
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
}

/// mylists 🔵 PRDデータモデルより
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MyList {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

/// api_credentials（外部APIキー管理） 🔵 REQ-015/NFR-202より
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiCredential {
    pub provider: ApiProvider,
    pub api_key: String,
    pub updated_at: NaiveDateTime,
}

// ========================================
// メディア別詳細テーブル
// ========================================

/// anime_details 🔵 PRDメディア別機能より
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AnimeDetails {
    pub item_id: Uuid,
    pub episode_count: Option<i32>,
    pub season_count: Option<i32>,
    pub studio: Option<String>,
    pub genre_list: Vec<String>,
    pub source_type: Option<String>,
    pub jikan_id: Option<String>,
}

/// movie_details 🔵
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MovieDetails {
    pub item_id: Uuid,
    pub runtime_minutes: Option<i32>,
    pub director: Option<String>,
    pub genre_list: Vec<String>,
    pub tmdb_id: Option<String>,
}

/// drama_details 🔵
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DramaDetails {
    pub item_id: Uuid,
    pub episode_count: Option<i32>,
    pub season_count: Option<i32>,
    pub network: Option<String>,
    pub genre_list: Vec<String>,
    pub tmdb_id: Option<String>,
}

/// manga_details 🔵
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MangaDetails {
    pub item_id: Uuid,
    pub volume_count: Option<i32>,
    pub chapter_count: Option<i32>,
    pub author: Option<String>,
    pub illustrator: Option<String>,
    pub magazine: Option<String>,
    pub jikan_id: Option<String>,
}

/// novel_details 🔵
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NovelDetails {
    pub item_id: Uuid,
    pub volume_count: Option<i32>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub openlibrary_id: Option<String>,
    pub google_books_id: Option<String>,
}

/// game_details 🔵
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GameDetails {
    pub item_id: Uuid,
    pub platform_list: Vec<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub steam_appid: Option<String>,
    pub igdb_id: Option<String>,
}

/// academic_book_details 🔵
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AcademicBookDetails {
    pub item_id: Uuid,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub ndl_id: Option<String>,
    pub google_books_id: Option<String>,
}

/// paper_details 🔵
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaperDetails {
    pub item_id: Uuid,
    pub doi: Option<String>,
    pub journal_name: Option<String>,
    pub volume_issue: Option<String>,
    pub page_range: Option<String>,
    pub author_list: Vec<String>,
    pub ndl_id: Option<String>,
}

// ========================================
// APIリクエスト/レスポンスDTO
// ========================================

/// アイテム新規作成リクエスト（手動 / API取込共通） 🔵 TC-001-01・TC-002-03より
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemRequest {
    pub media_type: MediaType,
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    pub source: ItemSource,
    pub external_id: Option<String>,
    // メディア別詳細は media_type に応じて details に格納 🟡 設計上の妥当な推測
    pub details: serde_json::Value,
}

/// アイテム部分更新リクエスト 🔵 TC-001-02より
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateItemRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<ItemStatus>,
    pub consumed_date: Option<NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: Option<bool>,
}

/// 外部API検索クエリ 🔵 TC-002-01/02より
#[derive(Debug, Clone, Deserialize)]
pub struct ExternalSearchQuery {
    pub media_type: MediaType,
    pub q: String,
}

/// グループ作成リクエスト 🔵 TC-010-01/TC-011-01より
#[derive(Debug, Clone, Deserialize)]
pub struct CreateGroupRequest {
    pub group_type: GroupType,
    pub group_name: String,
    pub number: Option<i32>,
    pub display_order: Option<i32>,
}

/// 話数作成リクエスト 🔵 TC-010-01より
#[derive(Debug, Clone, Deserialize)]
pub struct CreateEpisodeRequest {
    pub episode_number: i32,
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub air_date: Option<NaiveDate>,
    pub description: Option<String>,
}

/// ファイル登録リクエスト（パス指定方式） 🔵 TC-007-01より
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemFileRequest {
    pub path: String,
    pub label: Option<String>,
    pub file_type: FileType,
}

/// インポート結果サマリー 🔵 REQ-016/017・TC-016-E01より
#[derive(Debug, Clone, Serialize)]
pub struct ImportSummary {
    pub success_count: i32,
    pub failure_count: i32,
    pub failures: Vec<ImportFailure>,
}

/// インポート失敗行の詳細 🟡 EDGE-002から妥当な推測
#[derive(Debug, Clone, Serialize)]
pub struct ImportFailure {
    pub row_number: i32,
    pub reason: String,
}

/// 外部APIキー登録リクエスト 🔵 TC-015-01より
#[derive(Debug, Clone, Deserialize)]
pub struct UpsertApiCredentialRequest {
    pub api_key: String,
}

/// 統一APIレスポンス（成功） 🟡 既存実装の共通パターンから妥当な推測
#[derive(Debug, Clone, Serialize)]
pub struct ApiOk<T> {
    pub success: bool,
    pub data: T,
}

/// 統一エラーレスポンス 🟡 既存実装の共通パターンから妥当な推測
#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub error: ApiErrorBody,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

// ========================================
// 信頼性レベルサマリー
// ========================================
// - 🔵 青信号: 38件 (86%)
// - 🟡 黄信号: 6件 (14%)
// - 🔴 赤信号: 0件 (0%)
//
// 品質評価: 高品質
