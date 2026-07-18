//! itemsモデル・リクエストDTO・バリデーション
//!
//! TASK-0008: itemsモデル・リクエストDTO・バリデーション実装

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// メディア種別
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

/// 視聴・読了ステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "item_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    NotStarted,
    InProgress,
    Completed,
}

/// アイテムの登録元
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "item_source", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ItemSource {
    Api,
    Manual,
}

/// items 共通テーブルに対応するモデル
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

/// アイテム新規作成リクエスト
///
/// `source`はサーバー側で`manual`固定、`external_id`はNULL固定とするため
/// 入力対象外（TASK-0009のハンドラ側で付与する）。
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemRequest {
    pub media_type: MediaType,
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    pub rating: Option<f32>,
    pub is_favorite: Option<bool>,
    /// メディア別詳細（anime_details等）。ハンドラ側でmedia_typeに応じて振り分ける。
    #[serde(default)]
    pub details: Option<serde_json::Value>,
    /// 消費（読了・視聴）日
    #[serde(default)]
    pub consumed_date: Option<NaiveDate>,
    /// item_imagesへ紐づけて保存する画像一覧（url + kind）。
    /// 手動作成時は任意入力（source=manual固定）、インポート時はサーバー側で
    /// プロバイダ別に明示抽出した値（kind/source付き）が入る。
    #[serde(default)]
    pub additional_images: Vec<crate::models::item_image::NewItemImage>,
}

/// `GET /items` クエリパラメータDTO
///
/// 【機能概要】: 一覧取得・絞り込み用のクエリパラメータをAxumの`Query`抽出器で受け取るための構造体
/// 【実装方針】: すべてのフィールドをOptionalとし、未指定時はNoneになるようDeserializeを実装する。
/// enum値（media_type/status）が不正な文字列の場合はAxumのデシリアライズエラーとして400へつながる
/// 【テスト対応】: TC-0010-Q01〜Q06（SQL生成）、TC-0010-N01〜N08（統合テスト）で利用する
/// 🔵 信頼性レベル: 要件定義書 2.1 入力仕様表に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct ListItemsQuery {
    pub media_type: Option<MediaType>,
    pub tag_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub is_favorite: Option<bool>,
    pub status: Option<ItemStatus>,
    /// 【TASK-0029】: titleの部分一致（ILIKE）検索条件。`/internal/items/search`で
    /// タイトル検索を行うために追加。GET /itemsからも利用可能（後方互換、未指定時は影響なし）
    /// 🔵 信頼性レベル: TASK-0029要件定義2.2表#3「title部分一致」に直接対応
    #[serde(default)]
    pub title: Option<String>,
    /// 年フィルタ（年別コレクションページ用）。`date_field`で指定した日付カラムの
    /// 年が一致するアイテムのみ返す。未指定時は影響なし（後方互換）
    #[serde(default)]
    pub year: Option<i32>,
    /// `year`フィルタの対象日付カラム（未指定時はany＝両カラムのOR条件扱い）
    #[serde(default)]
    pub date_field: Option<DateField>,
    pub limit: Option<u32>,
    /// keysetページネーションのカーソル（前回レスポンスのnext_after_created_atを渡す）。
    /// after_idと両方指定された場合のみ有効（片方のみの場合は先頭ページとして扱う）
    #[serde(default)]
    pub after_created_at: Option<NaiveDateTime>,
    /// keysetページネーションのカーソル（前回レスポンスのnext_after_idを渡す）
    #[serde(default)]
    pub after_id: Option<Uuid>,
    /// 並び順（未指定時はcreated_at＝登録日時の降順）
    #[serde(default)]
    pub sort: Option<ItemSort>,
    /// sortがcreated_at以外の場合のkeysetカーソル（前回レスポンスのnext_after_valueを渡す）。
    /// after_created_at/after_idと合わせて指定された場合のみ有効
    #[serde(default)]
    pub after_value: Option<String>,
}

/// `GET /items` の並び順
///
/// SQL組み立て時は`order_expr`が返す静的文字列のみを埋め込むため、
/// ユーザー入力がSQLへ直接混入することはない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemSort {
    CreatedAt,
    UpdatedAt,
    Rating,
    Title,
    ReleaseDate,
}

impl ItemSort {
    /// ORDER BY・カーソル比較に使うソート式（静的文字列）。
    /// NULL可のカラムは番兵値でCOALESCEし、keysetのタプル比較を成立させる
    pub fn order_expr(self) -> &'static str {
        match self {
            ItemSort::CreatedAt => "created_at",
            ItemSort::UpdatedAt => "updated_at",
            ItemSort::Rating => "COALESCE(rating, -1)",
            ItemSort::Title => "title",
            ItemSort::ReleaseDate => "COALESCE(release_date, DATE '0001-01-01')",
        }
    }

    /// 降順ソートかどうか（titleのみ昇順）
    pub fn is_descending(self) -> bool {
        !matches!(self, ItemSort::Title)
    }
}

/// 年グルーピングの基準日付カラム（`GET /items` の`year`フィルタおよび`GET /items/years`用）
///
/// SQL組み立て時は`column_name`が返す静的文字列のみを埋め込むため、
/// ユーザー入力がSQLへ直接混入することはない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateField {
    Release,
    Consumed,
    /// release_date / consumed_date のどちらかが一致すれば対象とする（OR条件）
    Any,
}

impl DateField {
    /// 対応するitemsテーブルのカラム名（静的文字列）。
    /// `Any` は単一カラムに対応しないため`None`（呼び出し側でOR条件を組み立てる）
    pub fn column_name(self) -> Option<&'static str> {
        match self {
            DateField::Release => Some("release_date"),
            DateField::Consumed => Some("consumed_date"),
            DateField::Any => None,
        }
    }
}

/// `GET /items/years` クエリパラメータDTO
#[derive(Debug, Clone, Deserialize)]
pub struct ListItemYearsQuery {
    /// 集計対象の日付カラム（未指定時はany＝両カラムのOR条件扱い）
    #[serde(default)]
    pub date_field: Option<DateField>,
    #[serde(default)]
    pub media_type: Option<MediaType>,
}

/// `GET /items/years` レスポンス要素（年と合計件数、メディア種別ごとの内訳）
#[derive(Debug, Clone, Serialize)]
pub struct YearCount {
    pub year: i32,
    pub count: i64,
    /// メディア種別ごとの件数内訳（count降順、0件の種別は含まない）
    pub media_types: Vec<MediaTypeCount>,
}

/// 年別集計内のメディア種別ごとの件数
#[derive(Debug, Clone, Copy, Serialize)]
pub struct MediaTypeCount {
    pub media_type: MediaType,
    pub count: i64,
}

/// アイテム部分更新リクエスト（PATCH semantics）
///
/// `media_type`, `source`, `external_id`は変更不可フィールドのため除外する。
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateItemRequest {
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    pub status: Option<ItemStatus>,
    pub consumed_date: Option<NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: Option<bool>,
}

/// `PATCH /items/:id/status` 専用リクエスト
///
/// 【機能概要】: 視聴・読了状況（`status`）および消費日（`consumed_date`）のみを
/// 更新するための専用DTO。`status`は必須、`consumed_date`はoptional
/// 【実装方針】: `status`が不正な文字列の場合はserdeデシリアライズ自体が失敗し、
/// `deserialize_request`により`VALIDATION_ERROR`へ変換される
/// 🔵 信頼性レベル: TASK-0014 api-endpoints.md PATCH /items/:id/status リクエスト例より
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: ItemStatus,
    pub consumed_date: Option<NaiveDate>,
}

/// JSON値を任意のリクエストDTOへデシリアライズし、失敗時は`VALIDATION_ERROR`へ変換する。
///
/// `parse_create_item_request`・`update_item_handler`のいずれもserdeデシリアライズ失敗を
/// 同一形式（`VALIDATION_ERROR`・同一メッセージフォーマット）のApiErrorへ変換する必要があり、
/// その共通処理をここに集約する。
pub fn deserialize_request<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, ApiError> {
    serde_json::from_value(value).map_err(|err| {
        ApiError::new(
            ApiErrorCode::ValidationError,
            format!("リクエストの形式が不正です: {err}"),
        )
    })
}

/// JSON値を`CreateItemRequest`へデシリアライズし、バリデーションを行う。
///
/// `media_type`が不正な値の場合（デシリアライズ失敗）や`title`が空白のみの場合に
/// `VALIDATION_ERROR`を返す。
pub fn parse_create_item_request(value: serde_json::Value) -> Result<CreateItemRequest, ApiError> {
    let request: CreateItemRequest = deserialize_request(value)?;
    validate_title(&request.title)?;
    Ok(request)
}

/// `title`が空白のみでないことを検証する。
pub fn validate_title(title: &str) -> Result<(), ApiError> {
    if title.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "titleは空にできません",
        ));
    }
    Ok(())
}

/// `UpdateItemRequest.title`が`Some("")`（空文字、または空白のみ）でないことを検証する。
///
/// 【機能概要】: PATCH /items/:id 用のtitle部分更新バリデーション。`title`が`None`の場合は
/// 更新対象外のためバリデーションをスキップし、`Some(値)`の場合のみ既存`validate_title`と
/// 同じtrim().is_empty()基準でチェックする
/// 【実装方針】: 既存`validate_title`（CreateItemRequest用）をtitleの非空判定ロジックとして
/// 流用し、Option分岐のみ追加する形でPATCH特有の「Noneはスキップ」要件を満たす
/// 【テスト対応】: validate_update_title_rejects_empty_string,
/// validate_update_title_distinguishes_none_empty_and_nonempty,
/// validate_update_title_rejects_blank_only_string を通すための実装
/// 🔵 信頼性レベル: 要件定義書REQ-0012-102・EDGE-0012-101・EDGE-0012-102、既存validate_title実装より
pub fn validate_update_title(title: &Option<String>) -> Result<(), ApiError> {
    // 【Noneスキップ】: titleが更新対象外（None）の場合はバリデーション自体を行わない 🔵
    if let Some(value) = title {
        // 【既存ロジック流用】: trim().is_empty()による空文字・空白のみ判定（validate_titleと同基準） 🔵
        validate_title(value)?;
    }
    Ok(())
}

/// `UpdateItemRequest`の全フィールドが`None`かどうかを判定する。
///
/// 【機能概要】: SET対象フィールドが1つも存在しない（更新リクエストが実質no-op）かを判定する
/// 【実装方針】: 全フィールドのOptionをORで結合し、いずれか1つでもSomeならtrueを返す
/// 【テスト対応】: has_any_update_field_returns_false_when_all_fields_none を通すための実装
/// 🔵 信頼性レベル: 要件定義書REQ-0012-101・EDGE-0012-01、note.md「SET句が0件のときはUPDATE文を
/// 実行せず取得のみ」より
pub fn has_any_update_field(request: &UpdateItemRequest) -> bool {
    // 【全フィールド判定】: いずれか1つでもSomeであれば更新対象ありと判定する 🔵
    request.title.is_some()
        || request.original_title.is_some()
        || request.description.is_some()
        || request.cover_image_url.is_some()
        || request.release_date.is_some()
        || request.homepage_url.is_some()
        || request.status.is_some()
        || request.consumed_date.is_some()
        || request.rating.is_some()
        || request.is_favorite.is_some()
}

/// タグ参照（GET /items/:id レスポンス用の簡易表現）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TagRef {
    pub id: Uuid,
    pub name: String,
}

/// カテゴリ参照（GET /items/:id レスポンス用の簡易表現）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CategoryRef {
    pub id: Uuid,
    pub name: String,
}

/// `GET /items/:id` レスポンス用の詳細表現
///
/// 【機能概要】: items本体 + メディア別詳細テーブル + タグ・カテゴリ一覧を1つにまとめたDTO
/// 🟡 信頼性レベル: api-endpoints.md「メディア別詳細テーブル・タグ・関連付け等を含む」からの妥当な推測
#[derive(Debug, Clone, Serialize)]
pub struct ItemDetail {
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
    pub detail: Option<serde_json::Value>,
    pub tags: Vec<TagRef>,
    pub categories: Vec<CategoryRef>,
    /// Calibre-Web遷移情報（calibre_book_id設定済みのPDF item_filesのみ付加）
    /// 🟡 信頼性レベル: TASK-0028 calibre-link-requirements.md 2.3 L77・タスク定義書 L58-64より新規追加
    #[serde(default)]
    pub calibre_links: Vec<crate::models::item_file::CalibreWebLinkInfo>,
    /// 配信サービスURL一覧(Netflix/AmazonPrime/DisneyPlus/DmmTv/AppleTv)
    #[serde(default)]
    pub streaming_links: Vec<crate::models::item_streaming_link::ItemStreamingLink>,
    /// 画像URL一覧（手動追加分 + 外部APIレスポンスから自動収集した分）
    #[serde(default)]
    pub images: Vec<crate::models::item_image::ItemImage>,
}

impl ItemDetail {
    /// itemの基本情報 + 詳細/タグ/カテゴリを合成してItemDetailを構築する
    pub fn from_parts(
        item: Item,
        detail: Option<serde_json::Value>,
        tags: Vec<TagRef>,
        categories: Vec<CategoryRef>,
    ) -> Self {
        Self::from_parts_with_calibre_links(item, detail, tags, categories, Vec::new())
    }

    /// itemの基本情報 + 詳細/タグ/カテゴリ + Calibre-Web遷移情報を合成してItemDetailを構築する
    /// 🟡 信頼性レベル: TASK-0028 calibre-link-testcases.md TC-020-02/N03/B05より新規追加
    pub fn from_parts_with_calibre_links(
        item: Item,
        detail: Option<serde_json::Value>,
        tags: Vec<TagRef>,
        categories: Vec<CategoryRef>,
        calibre_links: Vec<crate::models::item_file::CalibreWebLinkInfo>,
    ) -> Self {
        Self::from_parts_full(
            item,
            detail,
            tags,
            categories,
            calibre_links,
            Vec::new(),
            Vec::new(),
        )
    }

    /// itemの基本情報 + 詳細/タグ/カテゴリ + Calibre-Web遷移情報 + 配信URL一覧 + 画像URL一覧を
    /// 合成してItemDetailを構築する
    pub fn from_parts_full(
        item: Item,
        detail: Option<serde_json::Value>,
        tags: Vec<TagRef>,
        categories: Vec<CategoryRef>,
        calibre_links: Vec<crate::models::item_file::CalibreWebLinkInfo>,
        streaming_links: Vec<crate::models::item_streaming_link::ItemStreamingLink>,
        images: Vec<crate::models::item_image::ItemImage>,
    ) -> Self {
        Self {
            id: item.id,
            media_type: item.media_type,
            title: item.title,
            original_title: item.original_title,
            description: item.description,
            cover_image_url: item.cover_image_url,
            release_date: item.release_date,
            homepage_url: item.homepage_url,
            status: item.status,
            consumed_date: item.consumed_date,
            rating: item.rating,
            is_favorite: item.is_favorite,
            source: item.source,
            external_id: item.external_id,
            created_at: item.created_at,
            updated_at: item.updated_at,
            detail,
            tags,
            categories,
            calibre_links,
            streaming_links,
            images,
        }
    }
}

/// `GET /items` 一覧レスポンス用の表現（Item本体 + tags/categories）
///
/// フロントエンドのカードUI（タグピル表示）のため、一覧取得時にも
/// `ItemDetail`と同様のtags/categoriesを付加する。DB行マッピング用の
/// `Item`構造体自体は変更せず、ハンドラ側でバッチ取得した結果を合成する。
#[derive(Debug, Clone, Serialize)]
pub struct ItemWithRefs {
    #[serde(flatten)]
    pub item: Item,
    pub tags: Vec<TagRef>,
    pub categories: Vec<CategoryRef>,
}

/// メディア種別ごとのアイテム件数（サイドバー表示用、`GET /items/counts-by-media-type`）
///
/// 8種のMediaType値それぞれの件数を固定形状のフィールドとして持つ。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct MediaTypeCounts {
    pub anime: i64,
    pub movie: i64,
    pub drama: i64,
    pub manga: i64,
    pub novel: i64,
    pub game: i64,
    pub academic_book: i64,
    pub paper: i64,
    pub total: i64,
}

/// パスパラメータの文字列をUUIDへパースする。
///
/// 【機能概要】: `GET /items/:id`のパスパラメータを検証し、不正な形式の場合は
/// `VALIDATION_ERROR`（400）を返す
/// 【テスト対応】: TC-0011-U01, TC-0011-U02, TC-0011-E02 に対応
/// 🟡 信頼性レベル: タスクファイル完了条件「不正なUUID形式の場合、適切なエラー（400）を返す」からの妥当な推測
pub fn parse_item_id(raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw).map_err(|_| {
        ApiError::new(
            ApiErrorCode::ValidationError,
            "idはUUID形式である必要があります",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    /// テストケース1: CreateItemRequestの正常デシリアライズ
    #[test]
    fn create_item_request_deserializes_successfully() {
        let value = serde_json::json!({ "media_type": "anime", "title": "作品A" });
        let request = parse_create_item_request(value).unwrap();
        assert_eq!(request.media_type, MediaType::Anime);
        assert_eq!(request.title, "作品A");
    }

    /// テストケース2 (TC-001-E01): media_type不正値での検証エラー
    #[test]
    fn invalid_media_type_returns_validation_error() {
        let value = serde_json::json!({ "media_type": "invalid_type", "title": "作品A" });
        let err = parse_create_item_request(value).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    /// テストケース3 (TC-001-B01): title空文字での検証エラー
    #[test]
    fn empty_title_returns_validation_error() {
        let value = serde_json::json!({ "media_type": "anime", "title": "" });
        let err = parse_create_item_request(value).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
    }

    /// title空白のみでも検証エラーとなることの確認
    #[test]
    fn blank_title_returns_validation_error() {
        let value = serde_json::json!({ "media_type": "anime", "title": "   " });
        let err = parse_create_item_request(value).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
    }

    /// テストケース4: UpdateItemRequestの部分指定
    #[test]
    fn update_item_request_deserializes_partial_fields() {
        let value = serde_json::json!({ "rating": 4.5 });
        let request: UpdateItemRequest = serde_json::from_value(value).unwrap();
        assert_eq!(request.rating, Some(4.5));
        assert_eq!(request.title, None);
        assert_eq!(request.is_favorite, None);
    }

    /// TC-NEW-03: 更新不可フィールド（media_type/source/external_id）が混入してもデシリアライズが成功し無視される
    /// 【テスト目的】: UpdateItemRequestのDTOに存在しないフィールドが含まれていてもエラーにならず、
    /// 対象フィールド（title）のみが反映されることを確認する
    /// 【テスト内容】: media_type/source/external_idを含むJSONをUpdateItemRequestへデシリアライズする
    /// 【期待される動作】: デシリアライズが成功し、titleのみSome("新タイトル")になる
    /// 🔵 信頼性レベル: 要件定義書シナリオ5・REQ-0012-05より
    #[test]
    fn update_item_request_ignores_unknown_immutable_fields() {
        // 【テストデータ準備】: 更新不可フィールドを混在させたリクエストボディ
        // 【初期条件設定】: クライアントが誤ってmedia_type等を送信したケースを再現する
        let value = serde_json::json!({
            "title": "新タイトル",
            "media_type": "movie",
            "source": "manual",
            "external_id": "999"
        });

        // 【実際の処理実行】: serdeの標準デシリアライズ動作を確認する
        // 【処理内容】: UpdateItemRequestには該当フィールドが存在しないため自動的に無視される想定
        let request: UpdateItemRequest = serde_json::from_value(value).unwrap();

        // 【結果検証】: titleのみ反映され、構造体に存在しないフィールドは型レベルで保証される
        assert_eq!(request.title, Some("新タイトル".to_string())); // 【確認内容】: titleのみが正しく反映されることを確認 🔵
    }

    /// TC-001-B01-A: title空文字バリデーション関数がSome("")に対してエラーを返す
    /// 【テスト目的】: PATCH用の新規バリデーション関数（validate_update_title）が
    /// title=Some("")の場合にVALIDATION_ERRORを返すことを確認する
    /// 【テスト内容】: validate_update_title(&Some(String::new()))を呼び出す
    /// 【期待される動作】: Err(ApiError { code: VALIDATION_ERROR, .. })が返る
    /// 🔵 信頼性レベル: 要件定義書REQ-0012-102・EDGE-0012-101、タスクファイルテストケース3より
    #[test]
    fn validate_update_title_rejects_empty_string() {
        // 【テストデータ準備】: title=Some("")（長さ0文字列）を用意する
        // 【初期条件設定】: EDGE-0012-101の具体例（リクエストボディ{"title": ""}相当）
        let title: Option<String> = Some(String::from(""));

        // 【実際の処理実行】: まだ実装されていないvalidate_update_title関数を呼び出す
        // 【処理内容】: title空文字検証ロジックの単体動作を確認する
        let result = validate_update_title(&title);

        // 【結果検証】: バリデーションエラーが返ることを確認
        let err = result.unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: title空文字がVALIDATION_ERRORになることを確認 🔵
        assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🔵
    }

    /// TC-001-EDGE101: titleがNoneの場合は空文字バリデーションを実行しない
    /// 【テスト目的】: Option<String>の3状態（None / Some("") / Some(非空)）すべてに対する
    /// 分岐の一貫性を保証する
    /// 【テスト内容】: ケースA: None, ケースB: Some(""), ケースC: Some("正常タイトル")の3パターンを検証する
    /// 【期待される動作】: ケースA・Cは成功（Ok）、ケースBはエラー（Err）
    /// 🔵 信頼性レベル: 要件定義書EDGE-0012-101より直接対応
    #[test]
    fn validate_update_title_distinguishes_none_empty_and_nonempty() {
        // 【テストデータ準備】: None / Some("") / Some(非空文字)の3パターンを用意する
        // 【初期条件設定】: 部分更新の基本契約（更新対象外フィールドにはバリデーションをかけない）を確認するため
        let case_a: Option<String> = None;
        let case_b: Option<String> = Some(String::from(""));
        let case_c: Option<String> = Some(String::from("正常タイトル"));

        // 【実際の処理実行】: それぞれのケースでvalidate_update_titleを呼び出す
        // 【処理内容】: Noneはスキップ、Some("")はエラー、Some(非空)は成功という分岐を確認する
        let result_a = validate_update_title(&case_a);
        let result_b = validate_update_title(&case_b);
        let result_c = validate_update_title(&case_c);

        // 【結果検証】: ケースA・Cは成功、ケースBはエラーであることを確認
        assert!(result_a.is_ok()); // 【確認内容】: titleがNoneの場合はバリデーション対象外で成功することを確認 🔵
        assert!(result_b.is_err()); // 【確認内容】: title=Some("")はバリデーションエラーになることを確認 🔵
        assert!(result_c.is_ok()); // 【確認内容】: title=Some(非空文字)はバリデーション成功することを確認 🔵
    }

    /// TC-NEW-06: title=Some(" ")（空白のみ文字列）の扱い確認
    /// 【テスト目的】: 既存validate_title（CreateItemRequest用）がtrim()による空白判定を行っている
    /// ことから、UpdateItemRequest用のvalidate_update_titleも同様に空白のみを拒否する仕様に揃えることを
    /// 確認する（note.md・models/item.rs L137-145のvalidate_title実装を確認した結果、trim().is_empty()で
    /// 判定しているため、空白のみの文字列も拒否される仕様とする）
    /// 【テスト内容】: validate_update_title(&Some(" "))を呼び出す
    /// 【期待される動作】: Err（既存validate_titleと同様、空白のみはエラーとする）
    /// 🟡 信頼性レベル: 要件定義書EDGE-0012-102（仕様未確定の旨を明記）に基づき、既存validate_title実装の
    /// trim().is_empty()方式に揃える妥当な推測
    #[test]
    fn validate_update_title_rejects_blank_only_string() {
        // 【テストデータ準備】: 半角スペース1文字のtitleを用意する
        // 【初期条件設定】: 既存CreateItemRequest用validate_titleの挙動（trim().is_empty()）に揃える前提
        let title: Option<String> = Some(String::from(" "));

        // 【実際の処理実行】: validate_update_titleを呼び出す
        let result = validate_update_title(&title);

        // 【結果検証】: 既存validate_titleと同様に空白のみはエラーになることを確認
        assert!(result.is_err()); // 【確認内容】: 空白のみのtitleが既存validate_titleと同じ基準で拒否されることを確認 🟡
    }

    /// TC-001-EDGE01-A: 全フィールドNoneのUpdateItemRequestはno-op判定対象となる
    /// 【テスト目的】: 全フィールドNoneの場合にSET対象が0件であることを判定するための補助関数
    /// （has_any_update_field等）が正しくfalseを返すことを確認する
    /// 【テスト内容】: 全フィールドNoneのUpdateItemRequestを構築し、has_any_update_fieldを呼び出す
    /// 【期待される動作】: falseが返る（SET対象なし＝UPDATE文を実行しない分岐に入るべきことを示す）
    /// 🔵 信頼性レベル: 要件定義書REQ-0012-101・EDGE-0012-01、note.md「SET句が0件のときはUPDATE文を
    /// 実行せず取得のみ」より
    #[test]
    fn has_any_update_field_returns_false_when_all_fields_none() {
        // 【テストデータ準備】: 全フィールドNoneのUpdateItemRequest（リクエストボディ{}相当）
        // 【初期条件設定】: EDGE-0012-01の具体例
        let request = UpdateItemRequest {
            title: None,
            original_title: None,
            description: None,
            cover_image_url: None,
            release_date: None,
            homepage_url: None,
            status: None,
            consumed_date: None,
            rating: None,
            is_favorite: None,
        };

        // 【実際の処理実行】: まだ実装されていないhas_any_update_field関数を呼び出す
        // 【処理内容】: 「更新対象なし」を呼び出し元（リポジトリ関数）に伝える契約になっているかを確認する
        let result = has_any_update_field(&request);

        // 【結果検証】: SET対象フィールド数が0であることを示すfalseが返ることを確認
        assert!(!result); // 【確認内容】: 全フィールドNoneの場合にfalse（更新対象なし）が返ることを確認 🔵
    }

    /// TC-0011-U01: 正しい形式のUUID文字列がパースできる
    /// 🟡 信頼性レベル: タスク完了条件「不正なUUID形式の場合、適切なエラー（400）を返す」からの妥当な推測
    #[test]
    fn parse_item_id_succeeds_for_valid_uuid() {
        let id = Uuid::new_v4();
        let parsed = parse_item_id(&id.to_string()).unwrap();
        assert_eq!(parsed, id);
    }

    /// TC-0011-U02: 不正な文字列はVALIDATION_ERROR(400)になる
    /// 🟡 信頼性レベル: タスクファイル テストケース3（不正なUUID形式で400）に対応
    #[test]
    fn parse_item_id_returns_validation_error_for_invalid_string() {
        let err = parse_item_id("abc").unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    /// テストケース1: UpdateStatusRequestの正常デシリアライズ（status・consumed_date両方）
    /// 🔵 信頼性レベル: TASK-0014 テストケース1（statusとconsumed_dateの正常更新）に対応
    #[test]
    fn update_status_request_deserializes_status_and_consumed_date() {
        let value = serde_json::json!({ "status": "completed", "consumed_date": "2026-06-20" });
        let request: UpdateStatusRequest = deserialize_request(value).unwrap();
        assert_eq!(request.status, ItemStatus::Completed);
        assert_eq!(
            request.consumed_date,
            Some(NaiveDate::from_ymd_opt(2026, 6, 20).unwrap())
        );
    }

    /// UpdateStatusRequestはconsumed_date省略時はNoneになる
    /// 🔵 信頼性レベル: タスクファイル注意事項「consumed_date未指定時は既存値を維持する」に対応
    #[test]
    fn update_status_request_allows_omitted_consumed_date() {
        let value = serde_json::json!({ "status": "in_progress" });
        let request: UpdateStatusRequest = deserialize_request(value).unwrap();
        assert_eq!(request.status, ItemStatus::InProgress);
        assert_eq!(request.consumed_date, None);
    }

    /// TC-N-04 (a): consumed_dateを含むJSONがCreateItemRequestへ正しくデシリアライズされる
    /// 【テスト目的】: ブクログCSVインポート（TASK-0030）の「読了日」をDB永続化するために追加した
    /// `consumed_date: Option<chrono::NaiveDate>`フィールドが、JSONに値が含まれる場合に正しく
    /// パースされることを確認する
    /// 【テスト内容】: {"media_type":"novel","title":"x","consumed_date":"2024-01-15"}を
    /// parse_create_item_requestへ渡す
    /// 【期待される動作】: consumed_date == Some(NaiveDate(2024,1,15))
    /// 🔵 信頼性レベル: テストケース定義書TC-N-04・設計判断#1（ユーザー確定）に基づく
    #[test]
    fn create_item_request_deserializes_consumed_date_when_present() {
        // 【テストデータ準備】: 読了日2024-01-15を含む正常なリクエストボディを用意する
        // 【初期条件設定】: ブクログCSVの「読了日」列がCreateItemRequestへ変換される想定の最小ケース
        let value = serde_json::json!({
            "media_type": "novel",
            "title": "ノルウェイの森",
            "consumed_date": "2024-01-15"
        });

        // 【実際の処理実行】: parse_create_item_requestを呼び出す
        // 【処理内容】: consumed_dateフィールドを含むJSONのデシリアライズ＋バリデーションを行う
        let request = parse_create_item_request(value).unwrap();

        // 【結果検証】: consumed_dateがCSV値どおりにパースされていることを確認
        assert_eq!(
            request.consumed_date,
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        ); // 【確認内容】: consumed_dateが"2024-01-15"から正しくNaiveDateへ変換されることを確認 🔵
    }

    /// TC-N-04 (b) / TC-REG-01: consumed_dateを省略したJSON（既存の手動作成形）はNoneになる
    /// 【テスト目的】: `#[serde(default)]`付与により、consumed_dateキー省略時に既存の手動作成
    /// （TASK-0009）リクエストの後方互換が保たれることを確認する
    /// 【テスト内容】: consumed_dateキーを含まないJSON（既存形）をparse_create_item_requestへ渡す
    /// 【期待される動作】: デシリアライズ成功、consumed_date == None
    /// 🔵 信頼性レベル: テストケース定義書TC-N-04・TC-REG-01、設計判断#1の回帰確認に基づく
    #[test]
    fn create_item_request_consumed_date_defaults_to_none_when_omitted() {
        // 【テストデータ準備】: consumed_dateを含まない既存の手動作成相当のリクエストボディ
        // 【初期条件設定】: TASK-0009時点の既存呼び出し（フィールド追加前）と同形のJSONを再現する
        let value = serde_json::json!({ "media_type": "anime", "title": "作品A" });

        // 【実際の処理実行】: parse_create_item_requestを呼び出す
        // 【処理内容】: consumed_date省略時のデフォルト値解決を確認する
        let request = parse_create_item_request(value).unwrap();

        // 【結果検証】: consumed_dateがNone（既存後方互換）であることを確認
        assert_eq!(request.consumed_date, None); // 【確認内容】: consumed_date省略時に#[serde(default)]でNoneになることを確認 🔵
    }

    /// テストケース2: status不正値はVALIDATION_ERROR(400)になる
    /// 🟡 信頼性レベル: TASK-0014 テストケース2（status不正値で400）に対応
    #[test]
    fn update_status_request_rejects_invalid_status_value() {
        let value = serde_json::json!({ "status": "invalid_status" });
        let err = deserialize_request::<UpdateStatusRequest>(value).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    /// statusが未指定の場合もVALIDATION_ERROR(400)になる（必須フィールド）
    /// 🟡 信頼性レベル: タスクファイル「statusは必須」より
    #[test]
    fn update_status_request_rejects_missing_status_field() {
        let value = serde_json::json!({ "consumed_date": "2026-06-20" });
        let err = deserialize_request::<UpdateStatusRequest>(value).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }
}
