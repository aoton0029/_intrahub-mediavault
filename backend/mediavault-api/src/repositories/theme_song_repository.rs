//! theme_songs / theme_song_links / item_theme_songs のDB操作
//!
//! staff_repository.rs（マスタ + 紐付けテーブル）と対称な構造。
//! item_id / theme_song_id という2つの異なるテーブルへのFK参照を持つ紐付け作成では、
//! FK制約違反だけでは原因を区別できないため、事前存在確認でITEM_NOT_FOUND /
//! THEME_SONG_NOT_FOUNDを明確に分ける（staff_repository::link_staffと同一方針）。

use std::collections::HashMap;

use chrono::NaiveDateTime;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};
use crate::models::theme_song::{
    CreateThemeSongLinkRequest, CreateThemeSongRequest, ItemThemeSong, ThemeSong, ThemeSongDetail,
    ThemeSongItemRef, ThemeSongLink, ThemeSongType, ThemeSongWithLinks, UpdateThemeSongRequest,
    has_any_update_field,
};
use crate::repositories::db_error_utils::is_unique_violation;

const SONG_COLUMNS: &str =
    "id, title, artist, composer, lyricist, arranger, note, created_at, updated_at";

const LINK_COLUMNS: &str = "id, theme_song_id, link_type, url, label, sort_order, created_at";

/// sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("theme_songs repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "テーマソングの処理に失敗しました",
    )
}

/// 一意制約違反（同一曲内のURL重複）をDUPLICATE_THEME_SONG_LINK（409）へ変換する
fn link_db_error(err: sqlx::Error) -> ApiError {
    if is_unique_violation(&err) {
        ApiError::new(
            ApiErrorCode::DuplicateThemeSongLink,
            "このURLは既にこの曲へ登録されています",
        )
    } else {
        db_error(err)
    }
}

fn theme_song_not_found() -> ApiError {
    ApiError::new(
        ApiErrorCode::ThemeSongNotFound,
        "指定された曲が見つかりません",
    )
}

fn item_not_found() -> ApiError {
    ApiError::new(
        ApiErrorCode::ItemNotFound,
        "指定されたアイテムが見つかりません",
    )
}

/// 指定したitem_idがitemsテーブルに存在するか確認する
async fn item_exists(pool: &PgPool, item_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 指定したtheme_song_idがtheme_songsテーブルに存在するか確認する
pub async fn theme_song_exists(pool: &PgPool, theme_song_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM theme_songs WHERE id = $1")
        .bind(theme_song_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 複数曲のリンクを1クエリでまとめて取得し、theme_song_idごとに束ねる（N+1回避）
async fn fetch_links_by_song_ids(
    pool: &PgPool,
    song_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<ThemeSongLink>>, ApiError> {
    if song_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let query = format!(
        "SELECT {LINK_COLUMNS} FROM theme_song_links \
         WHERE theme_song_id = ANY($1) \
         ORDER BY sort_order, created_at"
    );

    let links: Vec<ThemeSongLink> = sqlx::query_as(&query)
        .bind(song_ids)
        .fetch_all(pool)
        .await
        .map_err(db_error)?;

    let mut grouped: HashMap<Uuid, Vec<ThemeSongLink>> = HashMap::new();
    for link in links {
        grouped.entry(link.theme_song_id).or_default().push(link);
    }

    Ok(grouped)
}

/// 曲一覧にリンクを合成する（リンクが無い曲は空配列）
async fn attach_links(
    pool: &PgPool,
    songs: Vec<ThemeSong>,
) -> Result<Vec<ThemeSongWithLinks>, ApiError> {
    let song_ids: Vec<Uuid> = songs.iter().map(|song| song.id).collect();
    let mut grouped = fetch_links_by_song_ids(pool, &song_ids).await?;

    Ok(songs
        .into_iter()
        .map(|song| {
            let links = grouped.remove(&song.id).unwrap_or_default();
            ThemeSongWithLinks::from_parts(song, links)
        })
        .collect())
}

/// 曲名・artistの部分一致（ILIKE、大文字小文字を区別しない）で曲を一覧取得する（`GET /theme-songs`）
pub async fn list_theme_songs(
    pool: &PgPool,
    q: Option<&str>,
    limit: i64,
) -> Result<Vec<ThemeSongWithLinks>, ApiError> {
    let pattern = q.map(|raw| format!("%{raw}%"));

    let query = format!(
        "SELECT {SONG_COLUMNS} FROM theme_songs \
         WHERE ($1::text IS NULL OR title ILIKE $1 OR artist ILIKE $1) \
         ORDER BY title, created_at \
         LIMIT $2"
    );

    let songs: Vec<ThemeSong> = sqlx::query_as(&query)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(db_error)?;

    attach_links(pool, songs).await
}

/// 曲と（指定があれば）そのリンクを単一トランザクションで作成する（`POST /theme-songs`）
pub async fn create_theme_song(
    pool: &PgPool,
    request: CreateThemeSongRequest,
) -> Result<ThemeSongWithLinks, ApiError> {
    let mut tx = pool.begin().await.map_err(db_error)?;

    let insert_song = format!(
        "INSERT INTO theme_songs (title, artist, composer, lyricist, arranger, note) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING {SONG_COLUMNS}"
    );

    let song: ThemeSong = sqlx::query_as(&insert_song)
        .bind(&request.title)
        .bind(&request.artist)
        .bind(&request.composer)
        .bind(&request.lyricist)
        .bind(&request.arranger)
        .bind(&request.note)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;

    let insert_link = format!(
        "INSERT INTO theme_song_links (theme_song_id, link_type, url, label, sort_order) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING {LINK_COLUMNS}"
    );

    let mut links = Vec::with_capacity(request.links.len());
    for link in &request.links {
        let created: ThemeSongLink = sqlx::query_as(&insert_link)
            .bind(song.id)
            .bind(link.link_type)
            .bind(&link.url)
            .bind(&link.label)
            .bind(link.sort_order.unwrap_or(0))
            .fetch_one(&mut *tx)
            .await
            .map_err(link_db_error)?;
        links.push(created);
    }

    tx.commit().await.map_err(db_error)?;

    // 【並び順】: 一覧取得（GET /theme-songs/:id/links）と同じsort_order昇順で返す
    links.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });

    Ok(ThemeSongWithLinks::from_parts(song, links))
}

/// 曲1件をリンク付きで取得する
async fn get_theme_song_with_links(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ThemeSongWithLinks>, ApiError> {
    let query = format!("SELECT {SONG_COLUMNS} FROM theme_songs WHERE id = $1");

    let song: Option<ThemeSong> = sqlx::query_as(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    let Some(song) = song else {
        return Ok(None);
    };

    let links = list_theme_song_links_unchecked(pool, id).await?;
    Ok(Some(ThemeSongWithLinks::from_parts(song, links)))
}

/// 曲の詳細（リンク + その曲が使われているアイテム一覧）を取得する（`GET /theme-songs/{id}`）
pub async fn get_theme_song_detail(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ThemeSongDetail>, ApiError> {
    let Some(song) = get_theme_song_with_links(pool, id).await? else {
        return Ok(None);
    };

    let items: Vec<ThemeSongItemRef> = sqlx::query_as(
        "SELECT items.id AS item_id, items.title, items.media_type, item_theme_songs.theme_type
         FROM item_theme_songs
         JOIN items ON items.id = item_theme_songs.item_id
         WHERE item_theme_songs.theme_song_id = $1
         ORDER BY item_theme_songs.theme_type, items.title",
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(db_error)?;

    Ok(Some(ThemeSongDetail::from_parts(song, items)))
}

/// UpdateThemeSongRequestからSET句を動的に構築する（SET対象が1件も無い場合はNoneを返す）
#[allow(unused_assignments)]
fn build_update_theme_song_query(
    request: &UpdateThemeSongRequest,
) -> Option<QueryBuilder<'_, Postgres>> {
    if !has_any_update_field(request) {
        return None;
    }

    let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new("UPDATE theme_songs SET ");
    let mut has_condition = false;

    macro_rules! push_set_separator {
        () => {
            if has_condition {
                builder.push(", ");
            } else {
                has_condition = true;
            }
        };
    }

    macro_rules! push_optional_field {
        ($field:ident, $column:literal) => {
            if let Some(value) = &request.$field {
                push_set_separator!();
                builder.push(concat!($column, " = "));
                builder.push_bind(value.clone());
            }
        };
    }

    push_optional_field!(title, "title");
    push_optional_field!(artist, "artist");
    push_optional_field!(composer, "composer");
    push_optional_field!(lyricist, "lyricist");
    push_optional_field!(arranger, "arranger");
    push_optional_field!(note, "note");

    Some(builder)
}

/// 指定idの曲を部分更新する（`PATCH /theme-songs/{id}`）
pub async fn update_theme_song(
    pool: &PgPool,
    id: Uuid,
    request: UpdateThemeSongRequest,
) -> Result<Option<ThemeSongWithLinks>, ApiError> {
    let Some(mut builder) = build_update_theme_song_query(&request) else {
        // 【更新対象なし】: 全フィールド未指定のPATCHは現在値をそのまま返す（citationsと同一方針）
        return get_theme_song_with_links(pool, id).await;
    };

    builder.push(" WHERE id = ");
    builder.push_bind(id);
    builder.push(" RETURNING ");
    builder.push(SONG_COLUMNS);

    let song: Option<ThemeSong> = builder
        .build_query_as()
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    let Some(song) = song else {
        return Ok(None);
    };

    let links = list_theme_song_links_unchecked(pool, id).await?;
    Ok(Some(ThemeSongWithLinks::from_parts(song, links)))
}

/// 曲を削除する（`DELETE /theme-songs/{id}`）。
/// theme_song_links・item_theme_songsはCASCADEで削除され、items自体は削除されない
pub async fn delete_theme_song(pool: &PgPool, id: Uuid) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM theme_songs WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

/// 曲の存在確認をせずにリンクをsort_order昇順（同順位は作成日時昇順）で取得する
async fn list_theme_song_links_unchecked(
    pool: &PgPool,
    theme_song_id: Uuid,
) -> Result<Vec<ThemeSongLink>, ApiError> {
    let query = format!(
        "SELECT {LINK_COLUMNS} FROM theme_song_links \
         WHERE theme_song_id = $1 \
         ORDER BY sort_order, created_at"
    );

    sqlx::query_as(&query)
        .bind(theme_song_id)
        .fetch_all(pool)
        .await
        .map_err(db_error)
}

/// 指定曲のリンクを一覧取得する（`GET /theme-songs/{id}/links`）。曲が無ければ404
pub async fn list_theme_song_links(
    pool: &PgPool,
    theme_song_id: Uuid,
) -> Result<Vec<ThemeSongLink>, ApiError> {
    if !theme_song_exists(pool, theme_song_id).await? {
        return Err(theme_song_not_found());
    }

    list_theme_song_links_unchecked(pool, theme_song_id).await
}

/// 指定曲へリンクを追加する（`POST /theme-songs/{id}/links`）
pub async fn create_theme_song_link(
    pool: &PgPool,
    theme_song_id: Uuid,
    request: CreateThemeSongLinkRequest,
) -> Result<ThemeSongLink, ApiError> {
    if !theme_song_exists(pool, theme_song_id).await? {
        return Err(theme_song_not_found());
    }

    let query = format!(
        "INSERT INTO theme_song_links (theme_song_id, link_type, url, label, sort_order) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING {LINK_COLUMNS}"
    );

    sqlx::query_as(&query)
        .bind(theme_song_id)
        .bind(request.link_type)
        .bind(&request.url)
        .bind(&request.label)
        .bind(request.sort_order.unwrap_or(0))
        .fetch_one(pool)
        .await
        .map_err(link_db_error)
}

/// 指定曲のリンクを削除する（`DELETE /theme-songs/{id}/links/{link_id}`）
pub async fn delete_theme_song_link(
    pool: &PgPool,
    theme_song_id: Uuid,
    link_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM theme_song_links WHERE id = $1 AND theme_song_id = $2")
        .bind(link_id)
        .bind(theme_song_id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

/// item_theme_songs + theme_songs のJOIN行
#[derive(Debug, Clone, sqlx::FromRow)]
struct ItemThemeSongRow {
    id: Uuid,
    item_id: Uuid,
    theme_type: ThemeSongType,
    display_order: i32,
    created_at: NaiveDateTime,
    song_id: Uuid,
    song_title: String,
    song_artist: Option<String>,
    song_composer: Option<String>,
    song_lyricist: Option<String>,
    song_arranger: Option<String>,
    song_note: Option<String>,
    song_created_at: NaiveDateTime,
    song_updated_at: NaiveDateTime,
}

impl ItemThemeSongRow {
    fn into_item_theme_song(self, links: Vec<ThemeSongLink>) -> ItemThemeSong {
        let song = ThemeSong {
            id: self.song_id,
            title: self.song_title,
            artist: self.song_artist,
            composer: self.song_composer,
            lyricist: self.song_lyricist,
            arranger: self.song_arranger,
            note: self.song_note,
            created_at: self.song_created_at,
            updated_at: self.song_updated_at,
        };

        ItemThemeSong {
            id: self.id,
            item_id: self.item_id,
            theme_type: self.theme_type,
            display_order: self.display_order,
            created_at: self.created_at,
            theme_song: ThemeSongWithLinks::from_parts(song, links),
        }
    }
}

const ITEM_THEME_SONG_SELECT: &str = "SELECT item_theme_songs.id, item_theme_songs.item_id, \
     item_theme_songs.theme_type, item_theme_songs.display_order, item_theme_songs.created_at, \
     theme_songs.id AS song_id, theme_songs.title AS song_title, \
     theme_songs.artist AS song_artist, theme_songs.composer AS song_composer, \
     theme_songs.lyricist AS song_lyricist, theme_songs.arranger AS song_arranger, \
     theme_songs.note AS song_note, theme_songs.created_at AS song_created_at, \
     theme_songs.updated_at AS song_updated_at \
     FROM item_theme_songs \
     JOIN theme_songs ON theme_songs.id = item_theme_songs.theme_song_id";

/// 指定アイテムのテーマソングを取得する（存在確認なし。ItemDetail合成用）
///
/// 並び順はtheme_typeのenum順（op → ed → insert → image → character → theme → other）、
/// 次にdisplay_order昇順、次に作成日時昇順。
pub async fn list_item_theme_songs_unchecked(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<ItemThemeSong>, ApiError> {
    let query = format!(
        "{ITEM_THEME_SONG_SELECT} WHERE item_theme_songs.item_id = $1 \
         ORDER BY item_theme_songs.theme_type, item_theme_songs.display_order, \
         item_theme_songs.created_at"
    );

    let rows: Vec<ItemThemeSongRow> = sqlx::query_as(&query)
        .bind(item_id)
        .fetch_all(pool)
        .await
        .map_err(db_error)?;

    let song_ids: Vec<Uuid> = rows.iter().map(|row| row.song_id).collect();
    let grouped = fetch_links_by_song_ids(pool, &song_ids).await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            // 【リンクの共有】: 同じ曲が複数のtheme_typeで紐づく場合に備えcloneして渡す
            let links = grouped.get(&row.song_id).cloned().unwrap_or_default();
            row.into_item_theme_song(links)
        })
        .collect())
}

/// 指定アイテムのテーマソングを一覧取得する（`GET /items/{id}/theme-songs`）。アイテムが無ければ404
pub async fn list_item_theme_songs(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<ItemThemeSong>, ApiError> {
    if !item_exists(pool, item_id).await? {
        return Err(item_not_found());
    }

    list_item_theme_songs_unchecked(pool, item_id).await
}

/// 既存の曲をアイテムへ紐づける（`POST /items/{id}/theme-songs`）
pub async fn create_item_theme_song(
    pool: &PgPool,
    item_id: Uuid,
    theme_song_id: Uuid,
    theme_type: ThemeSongType,
    display_order: i32,
) -> Result<ItemThemeSong, ApiError> {
    // 【存在確認】: FK制約違反ではitem/theme_songのどちらが原因か区別できないため事前に判定する
    if !item_exists(pool, item_id).await? {
        return Err(item_not_found());
    }
    if !theme_song_exists(pool, theme_song_id).await? {
        return Err(theme_song_not_found());
    }

    let inserted_id: Uuid = sqlx::query_scalar(
        "INSERT INTO item_theme_songs (item_id, theme_song_id, theme_type, display_order)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(item_id)
    .bind(theme_song_id)
    .bind(theme_type)
    .bind(display_order)
    .fetch_one(pool)
    .await
    .map_err(|err| {
        if is_unique_violation(&err) {
            ApiError::new(
                ApiErrorCode::DuplicateItemThemeSong,
                "この曲は同じ種別で既にこのアイテムへ紐づいています",
            )
        } else {
            db_error(err)
        }
    })?;

    let query = format!("{ITEM_THEME_SONG_SELECT} WHERE item_theme_songs.id = $1");
    let row: ItemThemeSongRow = sqlx::query_as(&query)
        .bind(inserted_id)
        .fetch_one(pool)
        .await
        .map_err(db_error)?;

    let links = list_theme_song_links_unchecked(pool, theme_song_id).await?;
    Ok(row.into_item_theme_song(links))
}

/// アイテムと曲の紐付けのみを解除する（`DELETE /items/{id}/theme-songs/{item_theme_song_id}`）。
/// 曲レコード自体は削除されない
pub async fn delete_item_theme_song(
    pool: &PgPool,
    item_id: Uuid,
    item_theme_song_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM item_theme_songs WHERE id = $1 AND item_id = $2")
        .bind(item_theme_song_id)
        .bind(item_id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::theme_song::ThemeSongLinkType;

    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("theme-songs統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    #[test]
    fn build_update_theme_song_query_returns_none_for_empty_request() {
        let request = UpdateThemeSongRequest::default();
        assert!(build_update_theme_song_query(&request).is_none());
    }

    #[test]
    fn build_update_theme_song_query_builds_sql_for_specified_fields_only() {
        let request = UpdateThemeSongRequest {
            artist: Some("高橋洋子".to_string()),
            ..UpdateThemeSongRequest::default()
        };

        let builder = build_update_theme_song_query(&request).expect("SET句が構築されるはず");
        let sql = builder.sql();
        assert!(sql.contains("artist = "));
        assert!(!sql.contains("title = "));
    }

    #[tokio::test]
    #[ignore]
    async fn create_theme_song_persists_song_with_links() {
        let pool = test_pool().await;

        let created = create_theme_song(
            &pool,
            CreateThemeSongRequest {
                title: "統合テスト用テーマ".to_string(),
                artist: Some("テスト歌手".to_string()),
                composer: None,
                lyricist: None,
                arranger: None,
                note: None,
                links: vec![CreateThemeSongLinkRequest {
                    link_type: ThemeSongLinkType::Youtube,
                    url: format!("https://example.com/{}", Uuid::new_v4()),
                    label: Some("MV".to_string()),
                    sort_order: Some(0),
                }],
            },
        )
        .await
        .expect("create_theme_songは成功するはず");

        assert_eq!(created.title, "統合テスト用テーマ");
        assert_eq!(created.links.len(), 1);

        delete_theme_song(&pool, created.id)
            .await
            .expect("後片付けの削除に失敗");
    }

    #[tokio::test]
    #[ignore]
    async fn create_theme_song_with_duplicate_urls_rolls_back() {
        let pool = test_pool().await;
        let url = format!("https://example.com/{}", Uuid::new_v4());

        let result = create_theme_song(
            &pool,
            CreateThemeSongRequest {
                title: "ロールバック確認用".to_string(),
                artist: None,
                composer: None,
                lyricist: None,
                arranger: None,
                note: None,
                links: vec![
                    CreateThemeSongLinkRequest {
                        link_type: ThemeSongLinkType::Youtube,
                        url: url.clone(),
                        label: None,
                        sort_order: None,
                    },
                    CreateThemeSongLinkRequest {
                        link_type: ThemeSongLinkType::Official,
                        url: url.clone(),
                        label: None,
                        sort_order: None,
                    },
                ],
            },
        )
        .await;

        let err = result.expect_err("URL重複はエラーになるはず");
        assert_eq!(err.error.code, "DUPLICATE_THEME_SONG_LINK");

        // 【ロールバック確認】: トランザクションが巻き戻り、曲レコードも残らないこと
        let remaining: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM theme_songs WHERE title = $1")
                .bind("ロールバック確認用")
                .fetch_one(&pool)
                .await
                .expect("COUNT取得に失敗");
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn create_item_theme_song_with_nonexistent_item_returns_item_not_found() {
        let pool = test_pool().await;

        let err =
            create_item_theme_song(&pool, Uuid::new_v4(), Uuid::new_v4(), ThemeSongType::Op, 0)
                .await
                .expect_err("不存在item_idはエラーになるはず");

        assert_eq!(err.error.code, "ITEM_NOT_FOUND");
    }

    #[tokio::test]
    #[ignore]
    async fn list_theme_song_links_with_nonexistent_song_returns_theme_song_not_found() {
        let pool = test_pool().await;

        let err = list_theme_song_links(&pool, Uuid::new_v4())
            .await
            .expect_err("不存在theme_song_idはエラーになるはず");

        assert_eq!(err.error.code, "THEME_SONG_NOT_FOUND");
    }

    #[tokio::test]
    #[ignore]
    async fn delete_theme_song_link_returns_false_for_nonexistent_id() {
        let pool = test_pool().await;

        let deleted = delete_theme_song_link(&pool, Uuid::new_v4(), Uuid::new_v4())
            .await
            .expect("削除処理自体はエラーにならないはず");

        assert!(!deleted);
    }

    #[tokio::test]
    #[ignore]
    async fn delete_item_theme_song_returns_false_for_mismatched_item_id() {
        let pool = test_pool().await;

        let deleted = delete_item_theme_song(&pool, Uuid::new_v4(), Uuid::new_v4())
            .await
            .expect("削除処理自体はエラーにならないはず");

        assert!(!deleted);
    }
}
