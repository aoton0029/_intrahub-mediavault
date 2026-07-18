//! items / メディア別詳細テーブルのDB操作
//!
//! TASK-0009: POST /items（手動作成）実装

use std::collections::HashMap;

use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::item::{
    CategoryRef, CreateItemRequest, DateField, Item, ItemSort, ItemSource, ListItemYearsQuery,
    ListItemsQuery, MediaType, MediaTypeCount, MediaTypeCounts, TagRef, UpdateItemRequest,
    UpdateStatusRequest,
    YearCount, has_any_update_field,
};
use crate::models::response::{ApiError, ApiErrorCode};

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 【改善内容】: 元のSQLエラー詳細（テーブル構造・制約名等の内部情報）をクライアントへの
/// レスポンスに含めず、サーバーログにのみ出力するよう変更した
/// 【設計方針】: DB内部実装の詳細が外部に漏洩するとスキーマ推測等の攻撃材料になり得るため、
/// クライアント向けメッセージは固定の汎用文言とし、詳細は`tracing::error!`でログ出力に留める
/// 🟡 信頼性レベル: エラーコード一覧（response.rs）から妥当な推測、漏洩対策はセキュリティレビューに基づく改善
fn db_error(err: sqlx::Error) -> ApiError {
    // 【詳細ログ出力】: 内部調査用に詳細をサーバーログへ記録する（クライアントへは返さない） 🟡
    tracing::error!("items repository db error: {err}");
    // 【汎用エラー返却】: クライアントにはDB内部情報を含まない固定メッセージを返す 🟡
    ApiError::new(
        ApiErrorCode::InternalError,
        "アイテムの登録処理に失敗しました",
    )
}

/// 【機能概要】: 手動作成アイテムをitemsテーブルとメディア別詳細テーブルへ
/// 同一トランザクション内でINSERTする
/// 【実装方針】: TASK-0025のリファクタにより、本関数は`create_item_with_source`の薄いラッパーへ
/// 変更された。`source=manual`・`external_id=None`を固定で渡すのみで、既存の挙動・テストへの
/// 影響は無い想定（回帰防止はTC-0025-N04で確認する）
/// 【テスト対応】: TC-001-01等の統合テスト（Green時点ではユニットテスト対象外）に向けた実装
/// 🔵 信頼性レベル: タスクファイル「トランザクションによるINSERT」に直接対応
pub async fn create_item(pool: &PgPool, request: CreateItemRequest) -> Result<Item, ApiError> {
    // 【薄いラッパー化】: source=manual, external_id=Noneを固定で内部関数へ委譲する 🔵
    create_item_with_source(pool, request, ItemSource::Manual, None).await
}

/// 【機能概要】: CreateItemRequest相当の入力に対し、`source`・`external_id`を引数で指定して
/// itemsテーブルとメディア別詳細テーブルへ同一トランザクション内でINSERTする
/// 【実装方針】: 既存`create_item`のSQLリテラルハードコード（'manual', NULL）を引数化したのみで、
/// トランザクション構造自体は変更しない。重複チェック（TC-0025-E06）はこの関数の責務外とし、
/// 呼び出し元（import_item_handler等）であらかじめ判定する想定（Greenフェーズで確定）
/// 【テスト対応】: TC-0025-N03（source=api作成）、TC-0025-N04（manualラッパー回帰）、
/// TC-0025-N05（パリティ）、TC-0025-N07（8種media_type網羅）に対応
/// 🔵 信頼性レベル: item-import-requirements.md 3.1（Option B再利用方針）、note.md L237-238より
pub async fn create_item_with_source(
    pool: &PgPool,
    request: CreateItemRequest,
    source: ItemSource,
    external_id: Option<String>,
) -> Result<Item, ApiError> {
    // 【トランザクション開始】: items・詳細テーブルへのINSERTを原子的に行う 🔵
    let mut tx = pool.begin().await.map_err(db_error)?;

    // 【items本体INSERT】: source/external_idを$10/$11としてbindし、ハードコードを撤廃する。
    // 【TASK-0030拡張】: consumed_dateを$12としてbindし、ブクログCSVの「読了日」をDB永続化可能にする
    // （設計判断#1：作成パスを拡張する。TC-N-05・TC-DB-01対応） 🔵
    // 【詳細情報】: detailsは正規化済みMediaDetails JSONをそのままJSONB($13)へ保存する
    let item: Item = sqlx::query_as(
        "INSERT INTO items (
            media_type, title, original_title, description, cover_image_url,
            release_date, homepage_url, rating, is_favorite, source, external_id, consumed_date,
            details
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id, media_type, title, original_title, description, cover_image_url,
            release_date, homepage_url, status, consumed_date, rating, is_favorite,
            source, external_id, created_at, updated_at",
    )
    .bind(request.media_type)
    .bind(&request.title)
    .bind(&request.original_title)
    .bind(&request.description)
    .bind(&request.cover_image_url)
    .bind(request.release_date)
    .bind(&request.homepage_url)
    .bind(request.rating)
    .bind(request.is_favorite.unwrap_or(false))
    .bind(source)
    .bind(&external_id)
    .bind(request.consumed_date)
    .bind(&request.details)
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;

    // 【画像URL一括登録】: 外部APIレスポンスから収集した画像URL群をitem_imagesへ紐づけて保存する
    // （手動作成時はadditional_imagesが空のことが多く、その場合は何もしない）
    if !request.additional_images.is_empty() {
        crate::repositories::item_image_repository::insert_item_images_bulk(
            &mut tx,
            item.id,
            &request.additional_images,
        )
        .await?;
    }

    tx.commit().await.map_err(db_error)?;

    Ok(item)
}

/// 【機能概要】: 同一(media_type, external_id)のitemが既に存在するかを判定する
/// 【実装方針】: `idx_items_external_id`は非UNIQUEのため、アプリ層で明示的にSELECTし重複を検知する。
/// 存在すれば`ItemAlreadyImported`（409）エラーを返す
/// 【テスト対応】: TC-0025-E06（重複検知）、TC-0025-B03（複合キー境界）に対応
/// 🟡 信頼性レベル: item-import-requirements.md 第6章6.3（重複チェックSQL方針）より
pub async fn find_existing_import(
    pool: &PgPool,
    media_type: MediaType,
    external_id: &str,
) -> Result<bool, ApiError> {
    // 【重複検知SELECT】: (media_type, external_id)の複合キーで既存行の有無を判定する。
    // idx_items_external_idは非UNIQUEのため、アプリ層で明示的にSELECTする必要がある 🟡
    let existing: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM items WHERE media_type = $1 AND external_id = $2 LIMIT 1",
    )
    .bind(media_type)
    .bind(external_id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)?;

    Ok(existing.is_some())
}

/// 【機能概要】: 外部APIインポート確定時に構築済みの`CreateItemRequest`をitems+詳細テーブルへ
/// 登録するためのインポート専用関数
/// 【実装方針】: 重複チェック（find_existing_import）→ 存在すれば`ItemAlreadyImported`（409）で
/// 早期return → 存在しなければ`create_item_with_source(pool, request, ItemSource::Api, Some(external_id))`
/// を呼び出す
/// 【models/domain全廃止に伴うリファクタ】: 旧`ImportItemRequest`（`MediaDetails`由来の中間DTO）は
/// 廃止され、`services::external_search`のプロバイダ別変換関数が`CreateItemRequest`を直接構築する
/// ようになったため、本関数は`media_type`・`external_id`・`CreateItemRequest`を個別の引数として
/// 受け取る形へ変更した
/// 【テスト対応】: TC-0025-N03、TC-0025-E06、TC-0025-E08に対応
/// 🟡 信頼性レベル: item-import-requirements.md 2.3データフロー・第6章6.3より
pub async fn import_item(
    pool: &PgPool,
    media_type: MediaType,
    external_id: String,
    request: CreateItemRequest,
) -> Result<Item, ApiError> {
    // 【重複チェック】: 同一(media_type, external_id)が既存の場合は409エラーで早期returnする 🟡
    let duplicate = find_existing_import(pool, media_type, &external_id).await?;
    if duplicate {
        // 【エラー処理】: 重複インポートは要件第6章の決定に従い409 ITEM_ALREADY_IMPORTEDを返す 🟡
        return Err(ApiError::new(
            ApiErrorCode::ItemAlreadyImported,
            "既にインポート済みです",
        ));
    }

    // 【DB登録】: source=Api・external_id=Some(...)でitems+詳細テーブルへ同一トランザクションでINSERTする 🔵
    create_item_with_source(pool, request, ItemSource::Api, Some(external_id)).await
}

/// 【機能概要】: ListItemsQueryの絞り込み条件をQueryBuilderのWHERE句として共通追加する
/// 【実装方針】: list用・count用の両クエリで同一のフィルタ条件を共有するための内部ヘルパー。
/// 1件目の条件追加時のみ "WHERE" を、以降は "AND" を付与する。tag_id/category_idは
/// 中間テーブルへのEXISTSサブクエリとして追加し、他のフィルタは通常カラム条件として追加する
/// 戻り値のboolは条件が1つ以上追加されたか（呼び出し元がその後カーソル条件をWHERE/ANDで
/// 繋ぐ判定に使う）
fn push_item_filters(builder: &mut QueryBuilder<'_, Postgres>, query: &ListItemsQuery) -> bool {
    // 【条件追加フラグ】: 2件目以降の条件にANDを付与するためのフラグ。呼び出し元へ返し、
    // カーソル条件をWHERE/ANDのどちらで繋ぐか判定できるようにする 🟡
    let mut has_condition = false;

    // 【WHERE/AND付与ヘルパー】: 1件目はWHERE、以降はANDを付与する。
    // 【マクロ採用理由】: builder への可変借用とフラグの可変参照を同時に必要とするため、
    // クロージャ化すると呼び出し毎に &mut QueryBuilder を引き渡す煩雑さが増す。
    // 各フィルタブロック内でのみ使う小さな構文糖としてマクロに留める 🟡
    macro_rules! push_clause_prefix {
        () => {
            if has_condition {
                builder.push(" AND ");
            } else {
                builder.push(" WHERE ");
                has_condition = true;
            }
        };
    }

    // 【media_typeフィルタ】: idx_items_media_typeインデックスを活用する通常カラム条件 🔵
    if let Some(media_type) = query.media_type {
        push_clause_prefix!();
        builder.push("media_type = ");
        builder.push_bind(media_type);
    }

    // 【statusフィルタ】: idx_items_statusインデックスを活用する通常カラム条件 🔵
    if let Some(status) = query.status {
        push_clause_prefix!();
        builder.push("status = ");
        builder.push_bind(status);
    }

    // 【is_favoriteフィルタ】: idx_items_is_favoriteインデックスを活用する通常カラム条件 🔵
    if let Some(is_favorite) = query.is_favorite {
        push_clause_prefix!();
        builder.push("is_favorite = ");
        builder.push_bind(is_favorite);
    }

    // 【tag_idフィルタ】: item_tags中間テーブルへのEXISTSサブクエリ（重複排除のためJOIN+DISTINCTではなくEXISTSを採用） 🟡
    if let Some(tag_id) = query.tag_id {
        push_clause_prefix!();
        builder.push(
            "EXISTS (SELECT 1 FROM item_tags it WHERE it.item_id = items.id AND it.tag_id = ",
        );
        builder.push_bind(tag_id);
        builder.push(")");
    }

    // 【category_idフィルタ】: item_categories中間テーブルへのEXISTSサブクエリ 🟡
    if let Some(category_id) = query.category_id {
        push_clause_prefix!();
        builder.push(
            "EXISTS (SELECT 1 FROM item_categories ic WHERE ic.item_id = items.id AND ic.category_id = ",
        );
        builder.push_bind(category_id);
        builder.push(")");
    }

    // 【yearフィルタ】: 年別コレクションページ用。date_fieldで対象カラム（release_date/
    // consumed_date）を選択し、EXTRACT(YEAR FROM <col>)で年一致を判定する。
    // 未指定・any時は両カラムのOR条件（どちらかの年が一致すれば対象）。
    // カラム名はDateField::column_name()が返す静的文字列のみを埋め込むためSQLインジェクションの余地はない
    if let Some(year) = query.year {
        push_clause_prefix!();
        match query.date_field.unwrap_or(DateField::Any).column_name() {
            Some(column) => {
                builder.push(format!("EXTRACT(YEAR FROM {column})::int = "));
                builder.push_bind(year);
            }
            None => {
                builder.push("(EXTRACT(YEAR FROM release_date)::int = ");
                builder.push_bind(year);
                builder.push(" OR EXTRACT(YEAR FROM consumed_date)::int = ");
                builder.push_bind(year);
                builder.push(")");
            }
        }
    }

    // 【TASK-0029】: titleフィルタ（部分一致・ILIKE）。/internal/items/search の検索条件として
    // list_items_handlerの検索ロジックを再利用するために追加した 🔵
    if let Some(title) = &query.title {
        push_clause_prefix!();
        builder.push("title ILIKE ");
        builder.push_bind(format!("%{title}%"));
    }

    has_condition
}

/// sortがcreated_at以外の場合のkeysetカーソル値（after_valueをソートキーの型へパースしたもの）
enum SortCursorValue {
    Float(f32),
    Date(chrono::NaiveDate),
    DateTime(chrono::NaiveDateTime),
    Text(String),
}

/// after_valueを`sort`に対応する型へパースする。パース不能な場合はNone（先頭ページ扱い）
fn parse_sort_cursor(sort: ItemSort, after_value: &str) -> Option<SortCursorValue> {
    match sort {
        ItemSort::CreatedAt => None,
        ItemSort::Rating => after_value.parse().ok().map(SortCursorValue::Float),
        ItemSort::ReleaseDate => after_value.parse().ok().map(SortCursorValue::Date),
        ItemSort::UpdatedAt => after_value.parse().ok().map(SortCursorValue::DateTime),
        ItemSort::Title => Some(SortCursorValue::Text(after_value.to_string())),
    }
}

/// 【機能概要】: GET /items 一覧取得用のSELECTクエリをQueryBuilderで構築する（keysetページネーション）
/// 【実装方針】: SELECT ... FROM items [WHERE ...] ORDER BY <ソートキー>, created_at, id LIMIT ...
/// の形でクエリを組み立てる。OFFSETは使わない。
/// - sort未指定/created_at: 従来どおり `ORDER BY created_at DESC, id` とし、
///   after_created_at/after_idが両方指定された場合のみ `(created_at, id) < (?, ?)` を追加する
/// - それ以外のsort: `ORDER BY <式> <向き>, created_at <向き>, id <向き>` とし、
///   after_value/after_created_at/after_idが全て指定された場合のみ3要素タプル比較を追加する。
///   NULL可のカラムは番兵値でCOALESCEするため、タプル比較が常に成立する。
///
/// LIMITはhas_more判定のため+1して発行する
pub fn build_list_items_query(query: &ListItemsQuery) -> QueryBuilder<'_, Postgres> {
    let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "SELECT id, media_type, title, original_title, description, cover_image_url, \
        release_date, homepage_url, status, consumed_date, rating, is_favorite, \
        source, external_id, created_at, updated_at FROM items",
    );

    // 【フィルタ条件追加】: WHERE句を構築する。has_conditionを受け取り、カーソル条件の
    // WHERE/AND判定に使う 🟡
    let mut has_condition = push_item_filters(&mut builder, query);

    let sort = query.sort.unwrap_or(ItemSort::CreatedAt);

    if sort == ItemSort::CreatedAt {
        // 【カーソル条件追加】: after_created_at/after_idが両方指定された場合のみkeyset条件を追加する。
        // ORDER BY created_at DESC, id と対応する複合キー比較で「前回最後の行より後」を表現する 🔵
        if let (Some(after_created_at), Some(after_id)) = (query.after_created_at, query.after_id) {
            if has_condition {
                builder.push(" AND (created_at, id) < (");
            } else {
                builder.push(" WHERE (created_at, id) < (");
                has_condition = true;
            }
            builder.push_bind(after_created_at);
            builder.push(", ");
            builder.push_bind(after_id);
            builder.push(")");
        }
    } else {
        let cursor = query
            .after_value
            .as_deref()
            .and_then(|value| parse_sort_cursor(sort, value));
        if let (Some(cursor), Some(after_created_at), Some(after_id)) =
            (cursor, query.after_created_at, query.after_id)
        {
            let operator = if sort.is_descending() { " < (" } else { " > (" };
            let prefix = if has_condition { " AND (" } else { " WHERE (" };
            builder.push(format!(
                "{prefix}{expr}, created_at, id){operator}",
                expr = sort.order_expr()
            ));
            has_condition = true;
            match cursor {
                SortCursorValue::Float(value) => builder.push_bind(value),
                SortCursorValue::Date(value) => builder.push_bind(value),
                SortCursorValue::DateTime(value) => builder.push_bind(value),
                SortCursorValue::Text(value) => builder.push_bind(value),
            };
            builder.push(", ");
            builder.push_bind(after_created_at);
            builder.push(", ");
            builder.push_bind(after_id);
            builder.push(")");
        }
    }
    let _ = has_condition;

    // 【LIMIT句追加】: has_more判定のため、要求されたlimitに+1した件数を取得する 🔵
    let limit = query.limit.unwrap_or(20);
    if sort == ItemSort::CreatedAt {
        builder.push(" ORDER BY created_at DESC, id LIMIT ");
    } else {
        let direction = if sort.is_descending() { "DESC" } else { "ASC" };
        builder.push(format!(
            " ORDER BY {expr} {direction}, created_at {direction}, id {direction} LIMIT ",
            expr = sort.order_expr(),
        ));
    }
    builder.push_bind(limit as i64 + 1);

    builder
}

/// 【機能概要】: 絞り込み条件・ページネーションに従いitems一覧を取得する
/// 【実装方針】: build_list_items_queryで構築したクエリをfetch_allし、DBエラーはdb_errorで変換する
/// 【テスト対応】: TC-0010-N01〜N08, B07/B08, E04（実DB統合テスト、#[ignore]）に対応
/// 🔵 信頼性レベル: 要件定義書 2.4 データフロー・既存db_errorパターンに直接対応
pub async fn list_items(pool: &PgPool, query: &ListItemsQuery) -> Result<Vec<Item>, ApiError> {
    let mut builder = build_list_items_query(query);
    let items: Vec<Item> = builder
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(db_error)?;
    Ok(items)
}

/// 【機能概要】: 年別コレクションページ用に、指定日付カラムの年ごとのアイテム件数を
/// メディア種別内訳付きで集計する
/// 【実装方針】: `GROUP BY year, media_type`で年×種別の件数を取得し、Rust側で年ごとに
/// 畳み込んで合計件数と種別内訳（count降順）を構築する。
/// date_fieldが未指定・anyの場合は両カラムの年をUNIONで展開してから集計する
/// （どちらかの日付が該当年のアイテムを、年ごとに1回だけカウント）。
/// NULL日付の行は集計対象外（WHERE <col> IS NOT NULL）。カラム名はDateField::column_name()の
/// 静的文字列のみを埋め込む。count_items_by_media_typeと同様にdb_errorでエラー変換する
pub async fn list_item_years(
    pool: &PgPool,
    query: &ListItemYearsQuery,
) -> Result<Vec<YearCount>, ApiError> {
    let mut builder: QueryBuilder<'_, Postgres> =
        match query.date_field.unwrap_or(DateField::Any).column_name() {
            Some(column) => {
                let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(format!(
                    "SELECT EXTRACT(YEAR FROM {column})::int AS year, media_type, COUNT(*) AS count \
                    FROM items WHERE {column} IS NOT NULL"
                ));
                if let Some(media_type) = query.media_type {
                    builder.push(" AND media_type = ");
                    builder.push_bind(media_type);
                }
                builder
            }
            None => {
                // any: 両カラムの年をUNIONで展開してから集計する。UNIONにより
                // (id, year, media_type) が重複排除されるため、release/consumedが
                // 同年のアイテムは1回だけカウントされる
                let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(
                    "SELECT year, media_type, COUNT(*) AS count FROM ( \
                    SELECT id, EXTRACT(YEAR FROM release_date)::int AS year, media_type \
                    FROM items WHERE release_date IS NOT NULL",
                );
                if let Some(media_type) = query.media_type {
                    builder.push(" AND media_type = ");
                    builder.push_bind(media_type);
                }
                builder.push(
                    " UNION SELECT id, EXTRACT(YEAR FROM consumed_date)::int AS year, media_type \
                    FROM items WHERE consumed_date IS NOT NULL",
                );
                if let Some(media_type) = query.media_type {
                    builder.push(" AND media_type = ");
                    builder.push_bind(media_type);
                }
                builder.push(") AS year_entries");
                builder
            }
        };
    builder.push(" GROUP BY year, media_type ORDER BY year DESC, count DESC");

    let rows: Vec<(i32, MediaType, i64)> = builder
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(db_error)?;

    // 年降順・種別count降順で並んでいるため、年の切り替わりごとに畳み込む
    let mut years: Vec<YearCount> = Vec::new();
    for (year, media_type, count) in rows {
        match years.last_mut() {
            Some(entry) if entry.year == year => {
                entry.count += count;
                entry.media_types.push(MediaTypeCount { media_type, count });
            }
            _ => years.push(YearCount {
                year,
                count,
                media_types: vec![MediaTypeCount { media_type, count }],
            }),
        }
    }
    Ok(years)
}

/// 【機能概要】: 指定UUIDのitemsレコードを1件取得する
/// 【実装方針】: 通常のSELECT + fetch_optionalで存在しない場合はNoneを返す（404判定はハンドラ側）
/// 【テスト対応】: TC-0011-N01〜N05, E01（実DB統合テスト）に対応
/// 🟡 信頼性レベル: api-endpoints.md GET /items/:id仕様からの妥当な推測
pub async fn get_item_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Item>, ApiError> {
    let item: Option<Item> = sqlx::query_as(
        "SELECT id, media_type, title, original_title, description, cover_image_url, \
        release_date, homepage_url, status, consumed_date, rating, is_favorite, \
        source, external_id, created_at, updated_at FROM items WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)?;
    Ok(item)
}

/// 【機能概要】: itemsテーブルのdetailsカラム（正規化済みMediaDetails JSON）を取得する
/// 【実装方針】: 保存時と同一のJSONをそのまま返す（GET /items/searchの要素と同形）。
/// detailsがNULL（手動作成・移行前インポート等）の場合はNoneを返す（エラーにしない）
pub async fn get_item_detail(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Option<serde_json::Value>, ApiError> {
    let details: Option<Option<serde_json::Value>> =
        sqlx::query_scalar("SELECT details FROM items WHERE id = $1")
            .bind(item_id)
            .fetch_optional(pool)
            .await
            .map_err(db_error)?;

    Ok(details.flatten())
}

/// 【機能概要】: item_idに紐づくタグ一覧をtagsテーブルとのJOINで取得する
/// 【実装方針】: item_tags中間テーブルをtagsへJOINし、id/nameを取得する
/// 【テスト対応】: TC-0011-N02, N04に対応
/// 🟡 信頼性レベル: database-schema.sqlのitem_tags/tagsテーブル定義からの妥当な推測
pub async fn get_item_tags(pool: &PgPool, item_id: Uuid) -> Result<Vec<TagRef>, ApiError> {
    let tags: Vec<TagRef> = sqlx::query_as(
        "SELECT t.id, t.name FROM tags t \
        INNER JOIN item_tags it ON it.tag_id = t.id \
        WHERE it.item_id = $1",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)?;
    Ok(tags)
}

/// 【機能概要】: item_idに紐づくカテゴリ一覧をcategoriesテーブルとのJOINで取得する
/// 【実装方針】: item_categories中間テーブルをcategoriesへJOINし、id/nameを取得する
/// 【テスト対応】: TC-0011-N02, N04に対応
/// 🟡 信頼性レベル: database-schema.sqlのitem_categories/categoriesテーブル定義からの妥当な推測
pub async fn get_item_categories(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<CategoryRef>, ApiError> {
    let categories: Vec<CategoryRef> = sqlx::query_as(
        "SELECT c.id, c.name FROM categories c \
        INNER JOIN item_categories ic ON ic.category_id = c.id \
        WHERE ic.item_id = $1",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)?;
    Ok(categories)
}

/// 【機能概要】: 複数item_idに紐づくタグをまとめて取得し、item_id単位でグルーピングする
/// 【実装方針】: `GET /items`一覧のN+1回避のため、`WHERE item_id = ANY($1)`で一括取得し、
/// Rust側でHashMapへグルーピングする。タグが1件も無いitem_idはキー自体が存在しない
/// （呼び出し側で`.get(id).cloned().unwrap_or_default()`のように空Vecへフォールバックする）
pub async fn get_items_tags_batch(
    pool: &PgPool,
    item_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<TagRef>>, ApiError> {
    if item_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows: Vec<(Uuid, Uuid, String)> = sqlx::query_as(
        "SELECT it.item_id, t.id, t.name FROM item_tags it \
        INNER JOIN tags t ON t.id = it.tag_id \
        WHERE it.item_id = ANY($1)",
    )
    .bind(item_ids)
    .fetch_all(pool)
    .await
    .map_err(db_error)?;

    let mut grouped: HashMap<Uuid, Vec<TagRef>> = HashMap::new();
    for (item_id, tag_id, name) in rows {
        grouped
            .entry(item_id)
            .or_default()
            .push(TagRef { id: tag_id, name });
    }
    Ok(grouped)
}

/// 【機能概要】: 複数item_idに紐づくカテゴリをまとめて取得し、item_id単位でグルーピングする
/// 🟡 信頼性レベル: get_items_tags_batchと完全に対称
pub async fn get_items_categories_batch(
    pool: &PgPool,
    item_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<CategoryRef>>, ApiError> {
    if item_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows: Vec<(Uuid, Uuid, String)> = sqlx::query_as(
        "SELECT ic.item_id, c.id, c.name FROM item_categories ic \
        INNER JOIN categories c ON c.id = ic.category_id \
        WHERE ic.item_id = ANY($1)",
    )
    .bind(item_ids)
    .fetch_all(pool)
    .await
    .map_err(db_error)?;

    let mut grouped: HashMap<Uuid, Vec<CategoryRef>> = HashMap::new();
    for (item_id, category_id, name) in rows {
        grouped.entry(item_id).or_default().push(CategoryRef {
            id: category_id,
            name,
        });
    }
    Ok(grouped)
}

/// 【機能概要】: メディア種別ごとのアイテム件数をサイドバー表示用に集計する
/// 【実装方針】: `GROUP BY media_type`で全種別の件数を取得し、8種固定形状の
/// `MediaTypeCounts`へマッピングする。該当0件の種別はデフォルト0のまま返す
pub async fn count_items_by_media_type(pool: &PgPool) -> Result<MediaTypeCounts, ApiError> {
    let rows: Vec<(MediaType, i64)> =
        sqlx::query_as("SELECT media_type, COUNT(*) FROM items GROUP BY media_type")
            .fetch_all(pool)
            .await
            .map_err(db_error)?;

    let mut counts = MediaTypeCounts {
        anime: 0,
        movie: 0,
        drama: 0,
        manga: 0,
        novel: 0,
        game: 0,
        academic_book: 0,
        paper: 0,
        total: 0,
    };

    for (media_type, count) in rows {
        match media_type {
            MediaType::Anime => counts.anime = count,
            MediaType::Movie => counts.movie = count,
            MediaType::Drama => counts.drama = count,
            MediaType::Manga => counts.manga = count,
            MediaType::Novel => counts.novel = count,
            MediaType::Game => counts.game = count,
            MediaType::AcademicBook => counts.academic_book = count,
            MediaType::Paper => counts.paper = count,
        }
        counts.total += count;
    }

    Ok(counts)
}

/// 【機能概要】: `UpdateItemRequest`のうちSomeであるフィールドのみを対象にUPDATE文のSET句を
/// `sqlx::QueryBuilder`で構築する。SET対象が0件の場合は`None`を返す
/// 【実装方針】: `push_item_filters`のカンマ区切り方式（has_conditionフラグで1件目はカンマなし、
/// 以降は", "を付与）を踏襲する。`updated_at`はDBトリガー(`trg_items_updated_at`)が自動更新する
/// ためSET句に含めない。`media_type`・`source`・`external_id`は`UpdateItemRequest`に
/// フィールド自体が存在しないため、型レベルで更新対象外であることが保証される
/// 【テスト対応】: build_update_item_query_contains_rating_and_is_favorite_only,
/// build_update_item_query_single_field_has_no_extra_comma,
/// build_update_item_query_returns_none_when_all_fields_none を通すための実装
/// 🔵 信頼性レベル: 要件定義書REQ-0012-01・REQ-0012-03・REQ-0012-04・REQ-0012-101、
/// note.md「push_item_filtersパターンの踏襲」より
#[allow(unused_assignments)]
pub fn build_update_item_query(request: &UpdateItemRequest) -> Option<QueryBuilder<'_, Postgres>> {
    // 【早期リターン】: SET対象フィールドが1つも無い場合はUPDATE文を構築しない 🔵
    if !has_any_update_field(request) {
        return None;
    }

    let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new("UPDATE items SET ");

    // 【カンマ付与フラグ】: 2件目以降のSET項目にのみカンマを付与するためのフラグ 🔵
    let mut has_condition = false;

    // 【SET句区切りヘルパー】: 1件目はカンマなし、以降は", "を付与する 🔵
    macro_rules! push_set_separator {
        () => {
            if has_condition {
                builder.push(", ");
            } else {
                has_condition = true;
            }
        };
    }

    if let Some(title) = &request.title {
        push_set_separator!();
        builder.push("title = ");
        builder.push_bind(title.clone());
    }
    if let Some(original_title) = &request.original_title {
        push_set_separator!();
        builder.push("original_title = ");
        builder.push_bind(original_title.clone());
    }
    if let Some(description) = &request.description {
        push_set_separator!();
        builder.push("description = ");
        builder.push_bind(description.clone());
    }
    if let Some(cover_image_url) = &request.cover_image_url {
        push_set_separator!();
        builder.push("cover_image_url = ");
        builder.push_bind(cover_image_url.clone());
    }
    if let Some(release_date) = request.release_date {
        push_set_separator!();
        builder.push("release_date = ");
        builder.push_bind(release_date);
    }
    if let Some(homepage_url) = &request.homepage_url {
        push_set_separator!();
        builder.push("homepage_url = ");
        builder.push_bind(homepage_url.clone());
    }
    if let Some(status) = request.status {
        push_set_separator!();
        builder.push("status = ");
        builder.push_bind(status);
    }
    if let Some(consumed_date) = request.consumed_date {
        push_set_separator!();
        builder.push("consumed_date = ");
        builder.push_bind(consumed_date);
    }
    if let Some(rating) = request.rating {
        push_set_separator!();
        builder.push("rating = ");
        builder.push_bind(rating);
    }
    if let Some(is_favorite) = request.is_favorite {
        push_set_separator!();
        builder.push("is_favorite = ");
        builder.push_bind(is_favorite);
    }

    Some(builder)
}

/// 【機能概要】: 指定IDのitemに対し`UpdateItemRequest`のSomeフィールドのみを部分更新する
/// 【実装方針】: `build_update_item_query`でSET句を構築し、Noneの場合（全フィールドNone）は
/// UPDATE文を実行せず`get_item_by_id`で現在の状態をそのまま返す（REQ-0012-101）。
/// SET対象がある場合は` WHERE id = `を追加し`RETURNING ...`で更新後の行を直接取得する
/// （`fetch_optional`）。影響0件（対象が存在しない）の場合は`Ok(None)`を返し、404判定は
/// ハンドラ層の責務とする（REQ-0012-201・REQ-0012-202）。単一テーブルのみの更新のため
/// トランザクションは必須としない（NFR-0012-03）
/// 【テスト対応】: TC-001-02-B, TC-NEW-01, TC-001-EDGE01-B, TC-001-E02-A, TC-NEW-05
/// （いずれも実DB統合テスト）に対応
/// 🔵 信頼性レベル: 要件定義書REQ-0012-01〜REQ-0012-202・REQ-0012-402、note.mdのdb_error/
/// RETURNING+fetch_optional方針より
pub async fn update_item(
    pool: &PgPool,
    id: Uuid,
    request: UpdateItemRequest,
) -> Result<Option<Item>, ApiError> {
    // 【no-op判定】: 全フィールドNoneの場合はUPDATE文を実行せず現在の状態を返す 🔵
    let Some(mut builder) = build_update_item_query(&request) else {
        return get_item_by_id(pool, id).await;
    };

    // 【WHERE句 + RETURNING追加】: 対象idを絞り込み、更新後の全カラムを直接取得する 🔵
    builder.push(" WHERE id = ");
    builder.push_bind(id);
    builder.push(
        " RETURNING id, media_type, title, original_title, description, cover_image_url, \
        release_date, homepage_url, status, consumed_date, rating, is_favorite, \
        source, external_id, created_at, updated_at",
    );

    // 【UPDATE実行】: RETURNINGの結果をfetch_optionalで受け、0行（対象不存在）ならNoneを返す 🔵
    let item: Option<Item> = builder
        .build_query_as()
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(item)
}

/// 【機能概要】: 指定IDのitemを削除する。`ON DELETE CASCADE`がDBスキーマで設定済みのため、
/// item_tags・item_categories・mylist_items・item_relations・item_links・item_files・
/// item_trailers・item_groups・item_staff・メディア別詳細テーブルの関連レコードは
/// アプリケーション側で個別削除せずDB制約に委ねる
/// 【実装方針】: `DELETE FROM items WHERE id = $1`を実行し、`rows_affected()`が0の場合は
/// 対象が存在しなかったことを示す`Ok(false)`を返す。404判定はハンドラ層の責務とする
/// 【テスト対応】: TC-001-03（正常削除で204）、存在しないitemで404 に対応
/// 🔵 信頼性レベル: タスクファイル「DELETE FROM items WHERE id = $1」「rows_affected()が0の場合404」に直接対応
pub async fn delete_item(pool: &PgPool, id: Uuid) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM items WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

/// 【機能概要】: 指定IDのitemに対し`status`・`consumed_date`のみを更新する
/// 【実装方針】: `status`は必須のため常にSET対象、`consumed_date`は未指定（None）の場合は
/// `COALESCE($2, consumed_date)`で既存値を維持する（注意事項「consumed_date未指定時は
/// 既存値を維持する」より）。`RETURNING`+`fetch_optional`で更新後の行を取得し、0行（対象不存在）
/// の場合は`Ok(None)`を返す（404判定はハンドラ層の責務）
/// 【テスト対応】: テストケース1（正常更新）・テストケース3（存在しないitemで404）に対応
/// 🔵 信頼性レベル: タスクファイル「items テーブルのstatus, consumed_dateカラムのみをUPDATE」に直接対応
pub async fn update_item_status(
    pool: &PgPool,
    id: Uuid,
    request: UpdateStatusRequest,
) -> Result<Option<Item>, ApiError> {
    let item: Option<Item> = sqlx::query_as(
        "UPDATE items SET status = $1, consumed_date = COALESCE($2, consumed_date) \
        WHERE id = $3 \
        RETURNING id, media_type, title, original_title, description, cover_image_url, \
        release_date, homepage_url, status, consumed_date, rating, is_favorite, \
        source, external_id, created_at, updated_at",
    )
    .bind(request.status)
    .bind(request.consumed_date)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)?;

    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::item::{ItemStatus, ListItemsQuery};

    /// TC-001-02-A: build_update_item_queryがrating・is_favoriteのみのSET句を生成する
    /// 【テスト目的】: UpdateItemRequestのうちrating・is_favoriteのみSomeの場合に、
    /// SET句ビルダー関数が両カラムのみをSET対象として含むSQLを構築するかを確認する
    /// 【テスト内容】: build_update_item_query(&request)でQueryBuilderを構築しSQL文字列を取得する
    /// 【期待される動作】: 生成SQLが"rating = "と"is_favorite = "を含み、"title = "は含まない
    /// 🔵 信頼性レベル: 要件定義書REQ-0012-01・REQ-0012-03、note.md「push_item_filtersパターンの踏襲」より
    #[test]
    fn build_update_item_query_contains_rating_and_is_favorite_only() {
        // 【テストデータ準備】: rating・is_favoriteのみSome、他は全てNoneのUpdateItemRequest
        // 【初期条件設定】: 要件定義書シナリオ1（TC-001-02）の代表入力
        let request = update_request_with(|r| {
            r.rating = Some(4.5);
            r.is_favorite = Some(true);
        });

        // 【実際の処理実行】: build_update_item_query関数を呼び出す（戻り値はOption<QueryBuilder>、
        // SET対象が1件以上ある場合はSomeになる契約のためunwrapする） 🔵
        // 【処理内容】: 動的SET句構築の基本動作を実DB不要で検証する
        let builder = build_update_item_query(&request).expect("SET対象がある場合はSomeを返す契約");
        let sql = builder.sql();

        // 【結果検証】: 対象2フィールドのみSET句に含まれ、他カラムは含まれないことを確認
        assert!(sql.contains("rating = ")); // 【確認内容】: ratingがSET対象に含まれることを確認 🔵
        assert!(sql.contains("is_favorite = ")); // 【確認内容】: is_favoriteがSET対象に含まれることを確認 🔵
        assert!(!sql.contains("title = ")); // 【確認内容】: 対象外のtitleがSET句に含まれないことを確認 🔵
    }

    /// TC-NEW-02: build_update_item_queryが単一フィールド指定時に余分なカンマを生成しない
    /// 【テスト目的】: TC-NEW-01のSQL構築部分のみを実DB無しで検証する。カンマ結合ロジックの
    /// 境界（0→1→2項目）を保証する
    /// 【テスト内容】: statusのみSomeのUpdateItemRequestでSQLを生成する
    /// 【期待される動作】: "SET status = "を含み、SET句部分にカンマが含まれない
    /// 🟡 信頼性レベル: note.mdのhas_condition方式記載からの妥当な推測
    #[test]
    fn build_update_item_query_single_field_has_no_extra_comma() {
        // 【テストデータ準備】: statusのみSome、他は全てNoneのUpdateItemRequest
        // 【初期条件設定】: SET句項目数=1の境界を表す代表値
        let request = update_request_with(|r| {
            r.status = Some(ItemStatus::Completed);
        });

        // 【実際の処理実行】: build_update_item_query関数を呼び出す（SET対象1件のためSomeを期待） 🟡
        let builder = build_update_item_query(&request).expect("SET対象がある場合はSomeを返す契約");
        let sql = builder.sql();

        // 【結果検証】: status条件が含まれ、SET句にカンマが残らないことを確認
        assert!(sql.contains("SET status = ")); // 【確認内容】: status条件がSET句に含まれることを確認 🟡
        assert_eq!(sql.matches(',').count(), 0); // 【確認内容】: 単一フィールド時に不要なカンマが残らないことを確認 🟡
    }

    /// TC-001-EDGE01-A: 全フィールドNoneのUpdateItemRequestに対してbuild_update_item_queryがNoneを返す
    /// 【テスト目的】: SET句生成関数が「更新対象なし」を呼び出し元（リポジトリ関数）に伝え、
    /// UPDATE文を組み立てない分岐に入れる契約になっているかを確認する
    /// 【テスト内容】: 全フィールドNoneのUpdateItemRequestでbuild_update_item_queryを呼び出す
    /// 【期待される動作】: 戻り値がNone（QueryBuilder構築不要を示す）
    /// 🔵 信頼性レベル: 要件定義書REQ-0012-101・EDGE-0012-01、note.md L36「SET句が0件のときはUPDATE文を
    /// 実行せず取得のみ」より
    #[test]
    fn build_update_item_query_returns_none_when_all_fields_none() {
        // 【テストデータ準備】: 全フィールドNoneのUpdateItemRequest（リクエストボディ{}相当）
        let request = update_request_with(|_| {});

        // 【実際の処理実行】: build_update_item_queryがOption<QueryBuilder>を返す契約になっているかを確認する
        let builder_opt = build_update_item_query(&request);

        // 【結果検証】: SET対象フィールド数が0であることを示すNoneが返ることを確認
        assert!(builder_opt.is_none()); // 【確認内容】: 全フィールドNoneの場合にNone（構築不要）が返ることを確認 🔵
    }

    /// 全フィールドNoneのUpdateItemRequestを生成し、クロージャで一部のみ上書きするテスト用ヘルパー
    /// 【テストデータ準備】: 各テストケースで対象フィールドのみを明示的に設定するための共通基盤
    fn update_request_with(
        f: impl FnOnce(&mut crate::models::item::UpdateItemRequest),
    ) -> crate::models::item::UpdateItemRequest {
        let mut request = crate::models::item::UpdateItemRequest {
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
        f(&mut request);
        request
    }

    /// 空のListItemsQueryを生成するヘルパー（テスト用）
    /// 【テストデータ準備】: 全フィルタ未指定の基準ケースを表現するため
    fn empty_query() -> ListItemsQuery {
        ListItemsQuery {
            media_type: None,
            tag_id: None,
            category_id: None,
            is_favorite: None,
            status: None,
            title: None,
            year: None,
            date_field: None,
            limit: None,
            after_created_at: None,
            after_id: None,
            sort: None,
            after_value: None,
        }
    }

    /// TC-0010-Q01: フィルタなし時のSQLにWHERE句が付かない
    /// 🟡 信頼性レベル: 確定3・QueryBuilder方針からの妥当な推測
    #[test]
    fn build_list_items_sql_has_no_where_clause_when_no_filters() {
        // 【テスト目的】: 全フィルタ未指定時、生成SQLにWHERE句が含まれないことを確認する
        // 【テスト内容】: build_list_items_query(&empty_query()) でQueryBuilderを構築しSQL文字列を取得する
        // 【期待される動作】: SQLが "SELECT ... FROM items" ベースでWHEREを含まず、LIMIT/OFFSETを含む
        // 🟡 信頼性レベル: テストケース定義書 TC-0010-Q01（確定3・QueryBuilder方針からの妥当な推測）

        // 【テストデータ準備】: 絞り込みなしの基準ケース
        // 【初期条件設定】: build_list_items_query はまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let query = empty_query();
        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        // 【結果検証】: 不要なWHERE句が付かないこと、LIMITが付き、OFFSETは付かないことを確認
        assert!(!sql.contains("WHERE")); // 【確認内容】: フィルタなし時にWHERE句が生成されないことを確認
        assert!(sql.contains("FROM items")); // 【確認内容】: itemsテーブルを対象としたクエリであることを確認
        assert!(sql.contains("LIMIT")); // 【確認内容】: keysetページネーション用LIMIT句が含まれることを確認
        assert!(!sql.contains("OFFSET")); // 【確認内容】: keysetページネーションではOFFSETを使わないことを確認
    }

    /// カーソル指定時にSQLへ (created_at, id) < (?, ?) 条件が追加される
    #[test]
    fn build_list_items_sql_contains_cursor_condition_when_after_specified() {
        let mut query = empty_query();
        query.after_created_at = Some(
            chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        );
        query.after_id = Some(Uuid::new_v4());

        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        assert!(sql.contains("WHERE (created_at, id) < (")); // 【確認内容】: 条件なし時はWHEREでカーソル条件が繋がることを確認
    }

    /// 他フィルタと併用時、カーソル条件はANDで繋がる
    #[test]
    fn build_list_items_sql_joins_cursor_condition_with_and_when_other_filters_present() {
        let mut query = empty_query();
        query.media_type = Some(MediaType::Anime);
        query.after_created_at = Some(
            chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        );
        query.after_id = Some(Uuid::new_v4());

        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        assert!(sql.contains("AND (created_at, id) < (")); // 【確認内容】: 他フィルタがある場合はANDでカーソル条件が繋がることを確認
    }

    /// yearフィルタ指定時（date_field未指定）は両日付カラムのOR条件が追加される
    #[test]
    fn build_list_items_sql_contains_either_date_year_condition_by_default() {
        let mut query = empty_query();
        query.year = Some(2026);

        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        assert!(sql.contains("WHERE (EXTRACT(YEAR FROM release_date)::int = ")); // 【確認内容】: date_field未指定時はrelease_dateも対象とすることを確認
        assert!(sql.contains(" OR EXTRACT(YEAR FROM consumed_date)::int = ")); // 【確認内容】: consumed_dateとのOR条件になることを確認
    }

    /// date_field=release指定時はrelease_date単独の年一致条件が追加される
    #[test]
    fn build_list_items_sql_uses_release_date_when_date_field_is_release() {
        let mut query = empty_query();
        query.year = Some(2026);
        query.date_field = Some(DateField::Release);

        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        assert!(sql.contains("WHERE EXTRACT(YEAR FROM release_date)::int = ")); // 【確認内容】: release_date単独条件になることを確認
        assert!(!sql.contains("consumed_date)::int")); // 【確認内容】: consumed_dateの年条件は付かないことを確認
    }

    /// date_field=consumed指定時はconsumed_dateの年一致条件が追加される
    #[test]
    fn build_list_items_sql_uses_consumed_date_when_date_field_is_consumed() {
        let mut query = empty_query();
        query.year = Some(2025);
        query.date_field = Some(DateField::Consumed);

        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        assert!(sql.contains("EXTRACT(YEAR FROM consumed_date)::int = ")); // 【確認内容】: date_field=consumed時はconsumed_dateを対象とすることを確認
        assert!(!sql.contains("release_date)::int")); // 【確認内容】: release_dateの年条件は付かないことを確認
    }

    /// date_fieldのみ指定（year未指定）の場合は年条件が追加されない（後方互換）
    #[test]
    fn build_list_items_sql_ignores_date_field_without_year() {
        let mut query = empty_query();
        query.date_field = Some(DateField::Consumed);

        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        assert!(!sql.contains("EXTRACT")); // 【確認内容】: year未指定時は年条件が生成されないことを確認
        assert!(!sql.contains("WHERE")); // 【確認内容】: 他フィルタも無いためWHERE自体が生成されないことを確認
    }

    /// after_created_at/after_idの片方のみ指定された場合はカーソル条件が無視される
    #[test]
    fn build_list_items_sql_ignores_cursor_when_only_one_field_specified() {
        let mut query = empty_query();
        query.after_created_at = Some(
            chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        );
        // after_idは未指定

        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        assert!(!sql.contains("created_at, id) <")); // 【確認内容】: 片方のみ指定時はカーソル条件が追加されないことを確認
        assert!(!sql.contains("WHERE")); // 【確認内容】: 他フィルタも無いため WHERE 自体が生成されないことを確認
    }

    /// TC-0010-Q02: media_type 指定時のSQLに `media_type = ` を含む
    /// 🔵 信頼性レベル: 完了条件「media_type 絞り込み」に対応
    #[test]
    fn build_list_items_sql_contains_media_type_filter() {
        // 【テスト目的】: media_typeフィルタ指定時、生成SQLにmedia_typeカラム条件が含まれることを確認する
        // 【テスト内容】: media_type=Some(Anime)のみ設定したクエリでSQLを生成する
        // 【期待される動作】: 生成SQLに "media_type = " を含む（値はバインドパラメータ化）
        // 🔵 信頼性レベル: タスク完了条件「media_type 絞り込み」に直接対応

        // 【テストデータ準備】: media_typeのみ指定する単一フィルタケース
        let mut query = empty_query();
        query.media_type = Some(MediaType::Anime);

        // 【実際の処理実行】: build_list_items_query呼び出し
        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        // 【結果検証】: media_typeカラムへの条件句が含まれることを確認
        assert!(sql.contains("media_type = ")); // 【確認内容】: 単一カラムフィルタの句が生成されることを確認 🔵
    }

    /// TC-0010-Q03: tag_id 指定時のSQLに item_tags の EXISTS を含む
    /// 🟡 信頼性レベル: 確定3に基づく
    #[test]
    fn build_list_items_sql_contains_item_tags_exists_subquery() {
        // 【テスト目的】: tag_idフィルタ指定時、item_tags中間テーブルへのEXISTSサブクエリが含まれることを確認する
        // 【テスト内容】: tag_id=Some(uuid)のみ設定したクエリでSQLを生成する
        // 【期待される動作】: 生成SQLに "EXISTS" と "item_tags" を含む
        // 🟡 信頼性レベル: テストケース定義書 確定3（EXISTSサブクエリパターン）に基づく

        // 【テストデータ準備】: tag_idのみ指定する単一フィルタケース
        let mut query = empty_query();
        query.tag_id = Some(Uuid::new_v4());

        // 【実際の処理実行】: build_list_items_query呼び出し
        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        // 【結果検証】: item_tagsへのEXISTSサブクエリが含まれることを確認
        assert!(sql.contains("EXISTS")); // 【確認内容】: EXISTSサブクエリ構文が使われていることを確認 🟡
        assert!(sql.contains("item_tags")); // 【確認内容】: item_tags中間テーブルが参照されていることを確認 🟡
    }

    /// TC-0010-Q04: category_id 指定時のSQLに item_categories の EXISTS を含む
    /// 🟡 信頼性レベル: 確定3に基づく
    #[test]
    fn build_list_items_sql_contains_item_categories_exists_subquery() {
        // 【テスト目的】: category_idフィルタ指定時、item_categories中間テーブルへのEXISTSサブクエリが含まれることを確認する
        // 【テスト内容】: category_id=Some(uuid)のみ設定したクエリでSQLを生成する
        // 【期待される動作】: 生成SQLに "EXISTS" と "item_categories" を含む
        // 🟡 信頼性レベル: テストケース定義書 確定3（EXISTSサブクエリパターン）に基づく

        // 【テストデータ準備】: category_idのみ指定する単一フィルタケース
        let mut query = empty_query();
        query.category_id = Some(Uuid::new_v4());

        // 【実際の処理実行】: build_list_items_query呼び出し
        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        // 【結果検証】: item_categoriesへのEXISTSサブクエリが含まれることを確認
        assert!(sql.contains("EXISTS")); // 【確認内容】: EXISTSサブクエリ構文が使われていることを確認 🟡
        assert!(sql.contains("item_categories")); // 【確認内容】: item_categories中間テーブルが参照されていることを確認 🟡
    }

    /// TC-0010-Q05: 複数フィルタ時にSQLが AND で結合される
    /// 🟡 信頼性レベル: 完了条件「AND結合」＋QueryBuilder方針からの妥当な推測
    #[test]
    fn build_list_items_sql_joins_multiple_filters_with_and() {
        // 【テスト目的】: 複数フィルタ指定時、各条件がANDで結合されることを確認する
        // 【テスト内容】: media_type=Some(Anime), is_favorite=Some(true) を設定したクエリでSQLを生成する
        // 【期待される動作】: 生成SQLに "AND" を含み、両カラム条件が連結される
        // 🟡 信頼性レベル: タスク完了条件「各フィルタはAND結合」＋QueryBuilder方針からの妥当な推測

        // 【テストデータ準備】: 2つのフィルタ（media_type, is_favorite）を同時指定
        let mut query = empty_query();
        query.media_type = Some(MediaType::Anime);
        query.is_favorite = Some(true);

        // 【実際の処理実行】: build_list_items_query呼び出し
        let builder = build_list_items_query(&query);
        let sql = builder.sql();

        // 【結果検証】: AND結合と両カラム条件が含まれることを確認
        assert!(sql.contains("AND")); // 【確認内容】: 複数フィルタがAND結合されることを確認 🟡
        assert!(sql.contains("media_type = ")); // 【確認内容】: media_type条件が含まれることを確認 🟡
        assert!(sql.contains("is_favorite = ")); // 【確認内容】: is_favorite条件が含まれることを確認 🟡
    }

    /// TC-0010-N01: 絞り込みなしの一覧取得（デフォルトページネーション、実DB必要）
    /// 🔵 信頼性レベル: TASK-0010 単体テスト要件TC-001・要件 UC-1 に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn list_items_returns_limit_plus_one_rows_for_has_more_detection() {
        // 【テスト目的】: items25件投入時、絞り込みなしのlist_itemsがlimit+1(21)件返すことを確認する
        // 【テスト内容】: テスト用DBへitemsを25件投入し、list_itemsをデフォルトlimitで呼ぶ
        // 【期待される動作】: items.len()==21（ハンドラ層でhas_more判定・truncateされる前提の生データ）
        // 🔵 信頼性レベル: タスク単体テスト要件TC-001・要件UC-1に直接対応（実装はGreenフェーズで行う）

        // 【テスト前準備】: 環境変数TEST_DATABASE_URL等からテスト用PgPoolを取得する想定（未実装のためここでは到達しない）
        // 【環境初期化】: マイグレーション適用済みDBに対し25件のitemsをINSERTする
        let pool = test_pool().await;
        seed_items(&pool, 25).await;

        // 【実際の処理実行】: list_itemsをデフォルトlimit(20)で呼ぶ（has_more判定用に+1件取得される）
        let query = empty_query();
        let items = list_items(&pool, &query).await.unwrap();

        // 【結果検証】: has_more判定用の+1件を含む21件が返ることを確認（ハンドラ層でtruncateされる）
        assert_eq!(items.len(), 21); // 【確認内容】: limit=20+1件のフェッチにより21件返ることを確認
    }

    /// TC-0010-N02: media_type による絞り込み（実DB必要）
    /// 🔵 信頼性レベル: TASK-0010 単体テスト要件TC-002・要件 UC-2 に直接対応
    #[tokio::test]
    #[ignore]
    async fn list_items_filters_by_media_type() {
        // 【テスト目的】: media_type=animeで絞り込んだ場合、anime以外が除外されることを確認する
        // 【テスト内容】: anime3件+movie2件を投入し、media_type=Some(Anime)で取得する
        // 【期待される動作】: data.len()==3、全要素がmedia_type==Anime、total==3
        // 🔵 信頼性レベル: タスク単体テスト要件TC-002・要件UC-2に直接対応

        let pool = test_pool().await;
        seed_items_by_media_type(&pool, MediaType::Anime, 3).await;
        seed_items_by_media_type(&pool, MediaType::Movie, 2).await;

        let mut query = empty_query();
        query.media_type = Some(MediaType::Anime);
        let items = list_items(&pool, &query).await.unwrap();

        assert_eq!(items.len(), 3); // 【確認内容】: anime種別のみ3件取得されることを確認 🔵
        assert!(items.iter().all(|i| i.media_type == MediaType::Anime)); // 【確認内容】: 全要素がanimeであることを確認 🔵
    }

    /// TC-0010-N03: 複数条件のAND絞り込み（実DB必要）
    /// 🟡 信頼性レベル: TASK-0010 単体テスト要件TC-003・要件 UC-3 に対応
    #[tokio::test]
    #[ignore]
    async fn list_items_applies_multiple_filters_with_and() {
        // 【テスト目的】: media_type=anime かつ is_favorite=true の両方を満たすitemのみが返ることを確認する
        // 【テスト内容】: anime/fav=true 2件, anime/fav=false 2件, movie/fav=true 1件を投入する
        // 【期待される動作】: data.len()==2、全要素がanime かつ fav=true、total==2
        // 🟡 信頼性レベル: タスク単体テスト要件TC-003・要件UC-3に対応（具体データ件数は妥当な推測）

        let pool = test_pool().await;
        seed_items_with_favorite(&pool, MediaType::Anime, true, 2).await;
        seed_items_with_favorite(&pool, MediaType::Anime, false, 2).await;
        seed_items_with_favorite(&pool, MediaType::Movie, true, 1).await;

        let mut query = empty_query();
        query.media_type = Some(MediaType::Anime);
        query.is_favorite = Some(true);
        let items = list_items(&pool, &query).await.unwrap();

        assert_eq!(items.len(), 2); // 【確認内容】: AND結合で両条件を満たす2件のみ取得されることを確認 🟡
        assert!(
            items
                .iter()
                .all(|i| i.media_type == MediaType::Anime && i.is_favorite)
        ); // 【確認内容】: 全要素が両条件を満たすことを確認 🟡
    }

    /// TC-0010-N04: status による絞り込み（実DB必要）
    /// 🔵 信頼性レベル: 要件 入力仕様表（status）・完了条件に直接対応
    #[tokio::test]
    #[ignore]
    async fn list_items_filters_by_status() {
        // 【テスト目的】: status=in_progressで絞り込んだ場合、該当ステータスのみ返ることを確認する
        // 【テスト内容】: in_progress2件, not_started1件, completed1件を投入する
        // 【期待される動作】: data.len()==2、全要素status==InProgress、total==2
        // 🔵 信頼性レベル: 要件入力仕様表（status）・完了条件に直接対応

        let pool = test_pool().await;
        seed_items_with_status(&pool, ItemStatus::InProgress, 2).await;
        seed_items_with_status(&pool, ItemStatus::NotStarted, 1).await;
        seed_items_with_status(&pool, ItemStatus::Completed, 1).await;

        let mut query = empty_query();
        query.status = Some(ItemStatus::InProgress);
        let items = list_items(&pool, &query).await.unwrap();

        assert_eq!(items.len(), 2); // 【確認内容】: in_progressのみ2件取得されることを確認 🔵
        assert!(items.iter().all(|i| i.status == ItemStatus::InProgress)); // 【確認内容】: 全要素がin_progressであることを確認 🔵
    }

    /// TC-0010-N05: is_favorite による絞り込み（実DB必要）
    /// 🔵 信頼性レベル: 要件 入力仕様表（is_favorite）に直接対応
    #[tokio::test]
    #[ignore]
    async fn list_items_filters_by_is_favorite() {
        // 【テスト目的】: is_favorite=trueで絞り込んだ場合、お気に入りitemのみ返ることを確認する
        // 【テスト内容】: fav=true3件, fav=false2件を投入する
        // 【期待される動作】: data.len()==3、全要素is_favorite==true、total==3
        // 🔵 信頼性レベル: 要件入力仕様表（is_favorite）に直接対応

        let pool = test_pool().await;
        seed_items_with_favorite(&pool, MediaType::Anime, true, 3).await;
        seed_items_with_favorite(&pool, MediaType::Anime, false, 2).await;

        let mut query = empty_query();
        query.is_favorite = Some(true);
        let items = list_items(&pool, &query).await.unwrap();

        assert_eq!(items.len(), 3); // 【確認内容】: fav=trueのみ3件取得されることを確認 🔵
        assert!(items.iter().all(|i| i.is_favorite)); // 【確認内容】: 全要素がfav=trueであることを確認 🔵
    }

    /// TC-0010-N06: tag_id による絞り込み（EXISTSサブクエリ、実DB必要）
    /// 🟡 信頼性レベル: 要件 UC-4・統合テスト要件に対応
    #[tokio::test]
    #[ignore]
    async fn list_items_filters_by_tag_id_without_duplicates() {
        // 【テスト目的】: tag_id指定時、item_tags経由で当該タグを持つitemのみが重複なく返ることを確認する
        // 【テスト内容】: TAG_A紐付け2件、TAG_B紐付け1件、タグなし1件を投入する
        // 【期待される動作】: data.len()==2（重複なし）、total==2
        // 🟡 信頼性レベル: 要件UC-4・統合テスト要件に対応（SQL形状は確定3のEXISTSパターンの推測）

        let pool = test_pool().await;
        let tag_a = Uuid::new_v4();
        let tag_b = Uuid::new_v4();
        seed_items_with_tag(&pool, tag_a, 2).await;
        seed_items_with_tag(&pool, tag_b, 1).await;
        seed_items(&pool, 1).await; // タグなし1件

        let mut query = empty_query();
        query.tag_id = Some(tag_a);
        let items = list_items(&pool, &query).await.unwrap();

        assert_eq!(items.len(), 2); // 【確認内容】: TAG_Aを持つitemのみ2件取得され、重複しないことを確認 🟡
    }

    /// TC-0010-N07: category_id による絞り込み（EXISTSサブクエリ、実DB必要）
    /// 🟡 信頼性レベル: 要件 UC-5・統合テスト要件に対応
    #[tokio::test]
    #[ignore]
    async fn list_items_filters_by_category_id_without_duplicates() {
        // 【テスト目的】: category_id指定時、item_categories経由で当該カテゴリのitemのみが返ることを確認する
        // 【テスト内容】: CAT_A紐付け2件、CAT_B1件、カテゴリなし1件を投入する
        // 【期待される動作】: data.len()==2、total==2
        // 🟡 信頼性レベル: 要件UC-5・統合テスト要件に対応（SQL形状は確定3のEXISTSパターンの推測）

        let pool = test_pool().await;
        let cat_a = Uuid::new_v4();
        let cat_b = Uuid::new_v4();
        seed_items_with_category(&pool, cat_a, 2).await;
        seed_items_with_category(&pool, cat_b, 1).await;
        seed_items(&pool, 1).await; // カテゴリなし1件

        let mut query = empty_query();
        query.category_id = Some(cat_a);
        let items = list_items(&pool, &query).await.unwrap();

        assert_eq!(items.len(), 2); // 【確認内容】: CAT_Aを持つitemのみ2件取得されることを確認 🟡
    }

    /// TC-0010-N08: tag_id と media_type の AND 複合（実DB必要）
    /// 🟡 信頼性レベル: 完了条件 + 確定3 からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn list_items_combines_tag_id_and_media_type_with_and() {
        // 【テスト目的】: media_type条件（通常カラム）とtag_id条件（EXISTSサブクエリ）がAND結合されることを確認する
        // 【テスト内容】: anime+TAG_A1件、anime+TAG_B1件、movie+TAG_A1件を投入する
        // 【期待される動作】: data.len()==1（anime+TAG_Aのみ）、total==1
        // 🟡 信頼性レベル: 完了条件「AND結合」+確定3を組み合わせた妥当な推測

        let pool = test_pool().await;
        let tag_a = Uuid::new_v4();
        let tag_b = Uuid::new_v4();
        seed_item_with_media_type_and_tag(&pool, MediaType::Anime, tag_a).await;
        seed_item_with_media_type_and_tag(&pool, MediaType::Anime, tag_b).await;
        seed_item_with_media_type_and_tag(&pool, MediaType::Movie, tag_a).await;

        let mut query = empty_query();
        query.media_type = Some(MediaType::Anime);
        query.tag_id = Some(tag_a);
        let items = list_items(&pool, &query).await.unwrap();

        assert_eq!(items.len(), 1); // 【確認内容】: anime+TAG_Aの組合せのみ1件取得されることを確認 🟡
    }

    /// TC-0010-B07: 末尾を過ぎたカーソル指定 → 空配列（実DB必要）
    /// 🟡 信頼性レベル: keysetページネーションへの変更に伴う書き直し
    #[tokio::test]
    #[ignore]
    async fn list_items_returns_empty_array_for_cursor_past_last_row() {
        // 【テスト目的】: 最後の行より後のカーソルを指定した場合に空配列を返すことを確認する
        // 【テスト内容】: items5件を投入し、最後の行のcreated_at/idをカーソルとして指定して取得する
        // 【期待される動作】: data==[]

        let pool = test_pool().await;
        seed_items(&pool, 5).await;

        let all_items = list_items(&pool, &empty_query()).await.unwrap();
        let last = all_items
            .last()
            .expect("5件投入済みのため最後の行が存在する");

        let mut query = empty_query();
        query.after_created_at = Some(last.created_at);
        query.after_id = Some(last.id);
        let items = list_items(&pool, &query).await.unwrap();

        assert_eq!(items.len(), 0); // 【確認内容】: 最後の行より後を指定すると空配列が返ることを確認
    }

    /// TC-0010-B08: 全件0件（空テーブル）→ data=[]（実DB必要）
    /// 🟡 信頼性レベル: 要件 2.2 から妥当な推測（空集合の自明ケース）
    #[tokio::test]
    #[ignore]
    async fn list_items_returns_empty_array_when_table_is_empty() {
        // 【テスト目的】: itemsテーブルが0件の場合、data=[]のフォーマットが維持されることを確認する

        let pool = test_pool().await;
        // 【テスト前準備】: 事前データなし（空テーブルの状態をそのまま利用）

        let query = empty_query();
        let items = list_items(&pool, &query).await.unwrap();

        assert_eq!(items.len(), 0); // 【確認内容】: 0件テーブルでdataが空配列であることを確認 🟡
    }

    /// TC-0010-E04: DBエラー時 → 500 INTERNAL_ERROR（実DB必要、接続不能プールで再現）
    /// 🟡 信頼性レベル: 要件 EC-2・既存 db_error 関数からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn list_items_converts_db_error_to_internal_error() {
        // 【テスト目的】: DB接続障害時、list_itemsの戻り値がApiErrorCode::InternalError（500）に変換されることを確認する
        // 【テスト内容】: 接続不能なPgPool（不正な接続文字列）に対しlist_itemsを呼ぶ
        // 【期待される動作】: Err(ApiError)が返り、error.codeが"INTERNAL_ERROR"、statusが500
        // 🟡 信頼性レベル: 要件EC-2・既存db_error関数の方針からの妥当な推測

        // 【テスト前準備】: 意図的に接続不能なプールを構築する（接続失敗を誘発）
        let pool = unreachable_pool().await;
        let query = empty_query();

        // 【実際の処理実行】: list_itemsを呼び、エラー変換を確認する
        let result = list_items(&pool, &query).await;

        // 【結果検証】: DB内部情報が漏洩せず汎用INTERNAL_ERRORに変換されることを確認
        let err = result.unwrap_err();
        assert_eq!(err.error.code, "INTERNAL_ERROR"); // 【確認内容】: DBエラーが汎用INTERNAL_ERRORコードに変換されることを確認 🟡
        assert_eq!(err.status, axum::http::StatusCode::INTERNAL_SERVER_ERROR); // 【確認内容】: HTTPステータスが500であることを確認 🟡
    }

    /// TC-0011-N01: 存在するitemの詳細取得（details JSONBあり、実DB必要）
    /// 🟡 信頼性レベル: items.details（正規化済みMediaDetails JSON）方式への移行に対応
    #[tokio::test]
    #[ignore]
    async fn get_item_by_id_and_detail_returns_anime_details() {
        let pool = test_pool().await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;
        sqlx::query("UPDATE items SET details = $1 WHERE id = $2")
            .bind(serde_json::json!({
                "media_type": "anime",
                "external_id": "12345",
                "title": "テストアニメ",
                "episodes": 24,
                "studios": ["Test Studio"],
                "trailer_url": "https://example.com/pv"
            }))
            .bind(item_id)
            .execute(&pool)
            .await
            .expect("items.detailsの更新に失敗しました");

        let item = get_item_by_id(&pool, item_id).await.unwrap().unwrap();
        let detail = get_item_detail(&pool, item_id).await.unwrap();

        assert_eq!(item.id, item_id); // 【確認内容】: 取得したitemのIDが一致することを確認 🟡
        let detail = detail.unwrap();
        assert_eq!(detail["episodes"], serde_json::json!(24)); // 【確認内容】: MediaDetailsのepisodesがそのまま返ることを確認 🟡
        assert_eq!(detail["studios"], serde_json::json!(["Test Studio"])); // 【確認内容】: 配列フィールドが保持されることを確認 🟡
    }

    /// TC-0011-N02: タグ・カテゴリが紐付いている場合の取得（実DB必要）
    /// 🟡 信頼性レベル: タスクファイル完了条件「紐づくタグ・カテゴリ一覧をレスポンスに含める」に対応
    #[tokio::test]
    #[ignore]
    async fn get_item_tags_and_categories_returns_linked_records() {
        let pool = test_pool().await;
        let tag_id = insert_test_tag(&pool, "タグA").await;
        let category_id = insert_test_category(&pool, "カテゴリA").await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            Some(tag_id),
            Some(category_id),
        )
        .await;

        let tags = get_item_tags(&pool, item_id).await.unwrap();
        let categories = get_item_categories(&pool, item_id).await.unwrap();

        assert_eq!(tags.len(), 1); // 【確認内容】: 紐付けたタグが1件取得されることを確認 🟡
        assert_eq!(tags[0].id, tag_id);
        assert_eq!(categories.len(), 1); // 【確認内容】: 紐付けたカテゴリが1件取得されることを確認 🟡
        assert_eq!(categories[0].id, category_id);
    }

    /// TC-0011-N03: detailsがNULLの場合（実DB必要）
    /// 🟡 信頼性レベル: 手動作成・移行前インポートitem（details未保存）を想定した境界ケース
    #[tokio::test]
    #[ignore]
    async fn get_item_detail_returns_none_when_detail_record_missing() {
        let pool = test_pool().await;
        // 【テストデータ準備】: detailsを設定せずitems単体のみ作成する
        let item_id: Uuid = sqlx::query_scalar(
            "INSERT INTO items (media_type, title, status, is_favorite, source, external_id) \
            VALUES ('anime', 'テストアイテム', 'not_started', false, 'manual', NULL) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("詳細レコード無しitemの投入に失敗しました");

        let detail = get_item_detail(&pool, item_id).await.unwrap();

        assert!(detail.is_none()); // 【確認内容】: detailsがNULLの場合エラーにならずNoneが返ることを確認 🟡
    }

    /// TC-0011-N04: タグ・カテゴリが紐付いていない場合（実DB必要）
    /// 🟡 信頼性レベル: タスクファイル完了条件からの妥当な推測（境界ケース）
    #[tokio::test]
    #[ignore]
    async fn get_item_tags_and_categories_returns_empty_when_not_linked() {
        let pool = test_pool().await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;

        let tags = get_item_tags(&pool, item_id).await.unwrap();
        let categories = get_item_categories(&pool, item_id).await.unwrap();

        assert!(tags.is_empty()); // 【確認内容】: 紐付けが無い場合空配列が返ることを確認 🟡
        assert!(categories.is_empty()); // 【確認内容】: 紐付けが無い場合空配列が返ることを確認 🟡
    }

    /// TC-0011-E01: 存在しないitemで取得結果がNone（実DB必要）
    /// 🔵 信頼性レベル: タスクファイル テストケース2（存在しないitemで404）に直接対応
    #[tokio::test]
    #[ignore]
    async fn get_item_by_id_returns_none_for_nonexistent_id() {
        let pool = test_pool().await;
        let item = get_item_by_id(&pool, Uuid::new_v4()).await.unwrap();
        assert!(item.is_none()); // 【確認内容】: 存在しないUUIDではNoneが返り、ハンドラ側で404に変換できることを確認 🔵
    }

    /// TC-0011-N05: item毎に保存したdetails JSONがそれぞれ取得できる（movie/game、実DB必要）
    /// 🔵 信頼性レベル: items.details方式（media_type別テーブル分岐の廃止）に対応
    #[tokio::test]
    #[ignore]
    async fn get_item_detail_dispatches_to_correct_table_per_media_type() {
        let pool = test_pool().await;

        let movie_id = insert_test_item(
            &pool,
            MediaType::Movie,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;
        sqlx::query("UPDATE items SET details = $1 WHERE id = $2")
            .bind(serde_json::json!({"media_type": "movie", "runtime_minutes": 120}))
            .bind(movie_id)
            .execute(&pool)
            .await
            .expect("movieのdetails更新に失敗しました");

        let game_id = insert_test_item(
            &pool,
            MediaType::Game,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;
        sqlx::query("UPDATE items SET details = $1 WHERE id = $2")
            .bind(serde_json::json!({"media_type": "game", "developers": ["Test Developer"]}))
            .bind(game_id)
            .execute(&pool)
            .await
            .expect("gameのdetails更新に失敗しました");

        let movie_detail = get_item_detail(&pool, movie_id).await.unwrap().unwrap();
        let game_detail = get_item_detail(&pool, game_id).await.unwrap().unwrap();

        assert_eq!(movie_detail["runtime_minutes"], serde_json::json!(120)); // 【確認内容】: movieのdetailsが取得されることを確認 🔵
        assert_eq!(
            game_detail["developers"],
            serde_json::json!(["Test Developer"])
        ); // 【確認内容】: gameのdetailsが取得されることを確認 🔵
    }

    /// TC-001-02-B: update_itemがrating・is_favoriteのみを更新し他フィールドを変化させない（実DB必要）
    /// 【テスト目的】: 実DB上でPATCH相当の更新を行い、対象2フィールドのみ変化し他フィールド
    /// （title等）が変化しないことを確認する
    /// 【テスト内容】: insert_test_itemで1件投入したitemに対しupdate_itemを呼び出す
    /// 【期待される動作】: 戻り値のrating==4.5・is_favorite==true、titleは投入時の値のまま、
    /// updated_atが更新前より新しい
    /// 🔵 信頼性レベル: 要件定義書シナリオ1・REQ-0012-04、タスクファイルテストケース1より
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn update_item_changes_only_rating_and_is_favorite() {
        // 【テスト前準備】: items一件を事前投入する
        let pool = test_pool().await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;
        let before = get_item_by_id(&pool, item_id).await.unwrap().unwrap();

        // 【実際の処理実行】: まだ実装されていないupdate_item関数を呼び出す
        let request = update_request_with(|r| {
            r.rating = Some(4.5);
            r.is_favorite = Some(true);
        });
        let updated = update_item(&pool, item_id, request).await.unwrap().unwrap();

        // 【結果検証】: 対象2フィールドのみ変化し、titleとupdated_atの整合性が保たれることを確認
        assert_eq!(updated.rating, Some(4.5)); // 【確認内容】: ratingが更新値に変化することを確認 🔵
        assert!(updated.is_favorite); // 【確認内容】: is_favoriteが更新値に変化することを確認 🔵
        assert_eq!(updated.title, before.title); // 【確認内容】: 対象外のtitleが変化しないことを確認 🔵
        assert!(updated.updated_at > before.updated_at); // 【確認内容】: トリガーによりupdated_atが新しくなることを確認 🔵
    }

    /// TC-NEW-01: update_itemがstatusのみを更新する（実DB必要）
    /// 【テスト目的】: 更新対象フィールドが1個のみの場合でもSET句生成・UPDATE実行が正しく機能するか
    /// （カンマ区切りロジックの境界）を確認する
    /// 【テスト内容】: statusのみSomeのUpdateItemRequestでupdate_itemを呼び出す
    /// 【期待される動作】: statusのみ変化し、他フィールドは変化しない
    /// 🟡 信頼性レベル: note.mdの「has_condition方式（カンマ区切り）」記載から妥当な推測
    #[tokio::test]
    #[ignore]
    async fn update_item_changes_only_status_when_single_field_specified() {
        let pool = test_pool().await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;

        let request = update_request_with(|r| {
            r.status = Some(ItemStatus::Completed);
        });
        let updated = update_item(&pool, item_id, request).await.unwrap().unwrap();

        assert_eq!(updated.status, ItemStatus::Completed); // 【確認内容】: statusが更新値に変化することを確認 🟡
        assert!(!updated.is_favorite); // 【確認内容】: 対象外のis_favoriteが変化しないことを確認 🟡
    }

    /// TC-001-EDGE01-B: update_itemが全フィールドNoneのとき現在の状態をそのまま返す（実DB必要）
    /// 【テスト目的】: 空オブジェクト相当の更新リクエストに対し、UPDATE文を実行せず、
    /// 現在のitem状態がそのまま返るかを確認する
    /// 【テスト内容】: 全フィールドNoneのUpdateItemRequestでupdate_itemを呼び出す
    /// 【期待される動作】: 戻り値が投入時の値と完全一致し、updated_atも変化しない
    /// 🔵 信頼性レベル: 要件定義書シナリオ2・REQ-0012-101・REQ-0012-202、タスクファイル
    /// 「全フィールドがNoneの場合は何もUPDATEせず現在の状態を返す」より
    #[tokio::test]
    #[ignore]
    async fn update_item_returns_current_state_when_all_fields_none() {
        let pool = test_pool().await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;
        let before = get_item_by_id(&pool, item_id).await.unwrap().unwrap();

        let request = update_request_with(|_| {});
        let updated = update_item(&pool, item_id, request).await.unwrap().unwrap();

        // 【結果検証】: updated_atが不変であることが「UPDATE文未実行」の最も強い検証となる
        assert_eq!(updated.updated_at, before.updated_at); // 【確認内容】: トリガー未発火＝UPDATE未実行であることを確認 🔵
        assert_eq!(updated.title, before.title); // 【確認内容】: titleが変化しないことを確認 🔵
    }

    /// TC-001-E02-A: update_itemが存在しないitem IDに対してOk(None)を返す（実DB必要）
    /// 【テスト目的】: 有効なUUID形式だがDB上に該当レコードが存在しない場合の処理を確認する
    /// 【テスト内容】: 未登録のUUIDに対しupdate_itemを呼び出す
    /// 【期待される動作】: Ok(None)が返る（ApiErrorへの変換はハンドラ層の責務）
    /// 🔵 信頼性レベル: 要件定義書REQ-0012-201・REQ-0012-202、タスクファイルテストケース2より
    #[tokio::test]
    #[ignore]
    async fn update_item_returns_ok_none_for_nonexistent_id() {
        let pool = test_pool().await;
        let request = update_request_with(|r| {
            r.rating = Some(3.0);
        });

        let result = update_item(&pool, Uuid::new_v4(), request).await.unwrap();

        assert!(result.is_none()); // 【確認内容】: 存在しないIDではOk(None)が返り、ハンドラ側で404に変換できることを確認 🔵
    }

    /// TC-NEW-05: update_itemがDB接続不能時にdb_error経由でInternalErrorへ変換される（実DB必要）
    /// 【テスト目的】: DB接続自体が失敗するケースでSQLやDB内部情報をクライアントに漏洩させないことを確認する
    /// 【テスト内容】: unreachable_pool()で構築した到達不能なPgPoolに対しupdate_itemを呼び出す
    /// 【期待される動作】: Err(ApiError)が返り、ApiErrorCode::InternalError（500）であること
    /// 🔵 信頼性レベル: 要件定義書REQ-0012-402、note.md「db_errorヘルパーを必ず通す」
    /// 「unreachable_poolパターン」より
    #[tokio::test]
    #[ignore]
    async fn update_item_converts_db_error_to_internal_error() {
        let pool = unreachable_pool().await;
        let request = update_request_with(|r| {
            r.rating = Some(3.0);
        });

        let result = update_item(&pool, Uuid::new_v4(), request).await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "INTERNAL_ERROR"); // 【確認内容】: DBエラーが汎用INTERNAL_ERRORコードに変換されることを確認 🔵
        assert_eq!(err.status, axum::http::StatusCode::INTERNAL_SERVER_ERROR); // 【確認内容】: HTTPステータスが500であることを確認 🔵
    }

    /// 【テスト用ヘルパー】: tagsテーブルへ1件投入しidを返す
    async fn insert_test_tag(pool: &PgPool, name: &str) -> Uuid {
        sqlx::query_scalar("INSERT INTO tags (name) VALUES ($1) RETURNING id")
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("テスト用tagの投入に失敗しました")
    }

    /// 【テスト用ヘルパー】: categoriesテーブルへ1件投入しidを返す
    async fn insert_test_category(pool: &PgPool, name: &str) -> Uuid {
        sqlx::query_scalar("INSERT INTO categories (name) VALUES ($1) RETURNING id")
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("テスト用categoryの投入に失敗しました")
    }

    // --- 以下はRedフェーズの統合テストが利用するヘルパー（未実装のためコンパイルエラーとなる想定） ---
    // Greenフェーズでテスト用DBセットアップユーティリティとして実装すること。

    // 【テスト用ヘルパー実装】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）を利用する。
    // これらのヘルパーは#[ignore]統合テストからのみ呼び出され、`cargo test -p mediavault-api`（無印）では
    // 実行されないため、実DB非依存のGreenフェーズ完了確認には影響しない。 🟡
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0010統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    async fn unreachable_pool() -> PgPool {
        // 【接続不能プール構築】: 到達不能なホストを指定し、接続失敗（DBエラー変換）を誘発する 🟡
        PgPool::connect("postgres://invalid:invalid@127.0.0.1:1/invalid")
            .await
            .expect("到達不能プールの構築検証用接続に失敗しました")
    }

    async fn seed_items(pool: &PgPool, count: u32) {
        for _ in 0..count {
            insert_test_item(
                pool,
                MediaType::Anime,
                ItemStatus::NotStarted,
                false,
                None,
                None,
            )
            .await;
        }
    }

    async fn seed_items_by_media_type(pool: &PgPool, media_type: MediaType, count: u32) {
        for _ in 0..count {
            insert_test_item(pool, media_type, ItemStatus::NotStarted, false, None, None).await;
        }
    }

    async fn seed_items_with_favorite(
        pool: &PgPool,
        media_type: MediaType,
        is_favorite: bool,
        count: u32,
    ) {
        for _ in 0..count {
            insert_test_item(
                pool,
                media_type,
                ItemStatus::NotStarted,
                is_favorite,
                None,
                None,
            )
            .await;
        }
    }

    async fn seed_items_with_status(pool: &PgPool, status: ItemStatus, count: u32) {
        for _ in 0..count {
            insert_test_item(pool, MediaType::Anime, status, false, None, None).await;
        }
    }

    async fn seed_items_with_tag(pool: &PgPool, tag_id: Uuid, count: u32) {
        for _ in 0..count {
            insert_test_item(
                pool,
                MediaType::Anime,
                ItemStatus::NotStarted,
                false,
                Some(tag_id),
                None,
            )
            .await;
        }
    }

    async fn seed_items_with_category(pool: &PgPool, category_id: Uuid, count: u32) {
        for _ in 0..count {
            insert_test_item(
                pool,
                MediaType::Anime,
                ItemStatus::NotStarted,
                false,
                None,
                Some(category_id),
            )
            .await;
        }
    }

    async fn seed_item_with_media_type_and_tag(pool: &PgPool, media_type: MediaType, tag_id: Uuid) {
        insert_test_item(
            pool,
            media_type,
            ItemStatus::NotStarted,
            false,
            Some(tag_id),
            None,
        )
        .await;
    }

    /// TC-001-03: 正常削除でtrueが返り、itemsテーブルからレコードが消える（実DB必要）
    /// 🔵 信頼性レベル: タスクファイル テストケース1（TC-001-03）に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn delete_item_removes_existing_item() {
        let pool = test_pool().await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;

        let deleted = delete_item(&pool, item_id).await.unwrap();

        assert!(deleted); // 【確認内容】: 既存itemの削除がtrueを返すことを確認 🔵
        let after = get_item_by_id(&pool, item_id).await.unwrap();
        assert!(after.is_none()); // 【確認内容】: itemsテーブルからレコードが消えていることを確認 🔵
    }

    /// 存在しないitemでfalseが返る（実DB必要）
    /// 🔵 信頼性レベル: タスクファイル テストケース2（存在しないitemで404）に直接対応
    #[tokio::test]
    #[ignore]
    async fn delete_item_returns_false_for_nonexistent_item() {
        let pool = test_pool().await;
        let id = Uuid::new_v4();

        let deleted = delete_item(&pool, id).await.unwrap();

        assert!(!deleted); // 【確認内容】: 存在しないitemの削除はfalseを返すことを確認 🔵
    }

    /// カスケード削除統合テスト: item_tags・item_categoriesが連動削除される（実DB必要）
    /// 🔵 信頼性レベル: database-schema.sqlのON DELETE CASCADE設定・タスクファイル統合テスト要件に対応
    #[tokio::test]
    #[ignore]
    async fn delete_item_cascades_to_item_tags_and_item_categories() {
        let pool = test_pool().await;
        let tag_id = insert_test_tag(&pool, "削除確認タグ").await;
        let category_id = insert_test_category(&pool, "削除確認カテゴリ").await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            Some(tag_id),
            Some(category_id),
        )
        .await;

        let deleted = delete_item(&pool, item_id).await.unwrap();
        assert!(deleted); // 【確認内容】: itemの削除が成功することを確認 🔵

        let remaining_tags: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM item_tags WHERE item_id = $1")
                .bind(item_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        let remaining_categories: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM item_categories WHERE item_id = $1")
                .bind(item_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(remaining_tags, 0); // 【確認内容】: item_tagsがカスケード削除されることを確認 🔵
        assert_eq!(remaining_categories, 0); // 【確認内容】: item_categoriesがカスケード削除されることを確認 🔵
    }

    /// テストケース1: update_item_statusがstatus・consumed_dateを正常に更新する（実DB必要）
    /// 🔵 信頼性レベル: TASK-0014 テストケース1（statusとconsumed_dateの正常更新）に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn update_item_status_updates_status_and_consumed_date() {
        let pool = test_pool().await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;

        let request = crate::models::item::UpdateStatusRequest {
            status: ItemStatus::Completed,
            consumed_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 20).unwrap()),
        };
        let updated = update_item_status(&pool, item_id, request)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.status, ItemStatus::Completed); // 【確認内容】: statusが更新値に変化することを確認 🔵
        assert_eq!(
            updated.consumed_date,
            Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 20).unwrap())
        ); // 【確認内容】: consumed_dateが更新値に変化することを確認 🔵
    }

    /// consumed_date未指定時は既存値を維持する（実DB必要）
    /// 🔵 信頼性レベル: タスクファイル注意事項「consumed_date未指定時は既存値を維持する」に直接対応
    #[tokio::test]
    #[ignore]
    async fn update_item_status_keeps_existing_consumed_date_when_omitted() {
        let pool = test_pool().await;
        let item_id = insert_test_item(
            &pool,
            MediaType::Anime,
            ItemStatus::NotStarted,
            false,
            None,
            None,
        )
        .await;
        let existing_date = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        sqlx::query("UPDATE items SET consumed_date = $1 WHERE id = $2")
            .bind(existing_date)
            .bind(item_id)
            .execute(&pool)
            .await
            .unwrap();

        let request = crate::models::item::UpdateStatusRequest {
            status: ItemStatus::InProgress,
            consumed_date: None,
        };
        let updated = update_item_status(&pool, item_id, request)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.status, ItemStatus::InProgress); // 【確認内容】: statusのみ変化することを確認 🔵
        assert_eq!(updated.consumed_date, Some(existing_date)); // 【確認内容】: consumed_dateが既存値のまま維持されることを確認 🔵
    }

    /// テストケース3: 存在しないitemでOk(None)が返る（実DB必要）
    /// 🟡 信頼性レベル: TASK-0014 テストケース3（存在しないitemで404）に対応
    #[tokio::test]
    #[ignore]
    async fn update_item_status_returns_none_for_nonexistent_item() {
        let pool = test_pool().await;
        let request = crate::models::item::UpdateStatusRequest {
            status: ItemStatus::Completed,
            consumed_date: None,
        };

        let result = update_item_status(&pool, Uuid::new_v4(), request)
            .await
            .unwrap();

        assert!(result.is_none()); // 【確認内容】: 存在しないIDではOk(None)が返り、ハンドラ側で404に変換できることを確認 🟡
    }

    /// 【テスト用ヘルパー】: items（+任意でitem_tags/item_categories）へ1件投入する共通処理
    async fn insert_test_item(
        pool: &PgPool,
        media_type: MediaType,
        status: ItemStatus,
        is_favorite: bool,
        tag_id: Option<Uuid>,
        category_id: Option<Uuid>,
    ) -> Uuid {
        let item_id: Uuid = sqlx::query_scalar(
            "INSERT INTO items (media_type, title, status, is_favorite, source, external_id) \
            VALUES ($1, 'テストアイテム', $2, $3, 'manual', NULL) RETURNING id",
        )
        .bind(media_type)
        .bind(status)
        .bind(is_favorite)
        .fetch_one(pool)
        .await
        .expect("テスト用itemの投入に失敗しました");

        if let Some(tag_id) = tag_id {
            sqlx::query("INSERT INTO item_tags (item_id, tag_id) VALUES ($1, $2)")
                .bind(item_id)
                .bind(tag_id)
                .execute(pool)
                .await
                .expect("テスト用item_tagsの投入に失敗しました");
        }

        if let Some(category_id) = category_id {
            sqlx::query("INSERT INTO item_categories (item_id, category_id) VALUES ($1, $2)")
                .bind(item_id)
                .bind(category_id)
                .execute(pool)
                .await
                .expect("テスト用item_categoriesの投入に失敗しました");
        }

        item_id
    }

    /// テスト用ヘルパー: 最小構成のCreateItemRequestを構築する
    fn create_item_request(media_type: MediaType, title: &str) -> CreateItemRequest {
        CreateItemRequest {
            media_type,
            title: title.to_string(),
            original_title: None,
            description: None,
            cover_image_url: None,
            release_date: None,
            homepage_url: None,
            rating: None,
            is_favorite: None,
            details: None,
            // 【TASK-0030拡張】: consumed_date追加に伴いテストヘルパーを更新（TC-REG-01の前提） 🔵
            consumed_date: None,
            additional_images: Vec::new(),
        }
    }

    /// テスト用ヘルパー: import_item呼び出し用の(media_type, external_id, CreateItemRequest)を構築する
    fn import_item_request(
        media_type: MediaType,
        external_id: &str,
        title: &str,
    ) -> (MediaType, String, CreateItemRequest) {
        (
            media_type,
            external_id.to_string(),
            create_item_request(media_type, title),
        )
    }

    /// TC-0025-N03: create_item_with_sourceがsource=api/external_idでitems本体とanime_detailsを
    /// 同一トランザクションで作成する（実DB必要）
    /// 【テスト目的】: 再利用可能な内部関数create_item_with_sourceが、source=api・external_id付きで
    /// items+詳細テーブルを作成することを確認する
    /// 【テスト内容】: media_type=Anime, source=Api, external_id=Some("12345")でcreate_item_with_source
    /// を呼び出す
    /// 【期待される動作】: 戻り値item.source==Api、item.external_id==Some("12345")
    /// 🔵 信頼性レベル: 要件4.1 TC-002-03、TASK-0025.mdテストケース1、既存create_item実装より
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn create_item_with_source_creates_item_with_api_source_and_external_id() {
        // 【テスト前準備】: 実DBプールを取得する
        let pool = test_pool().await;
        let request = create_item_request(MediaType::Anime, "鬼滅の刃");

        // 【実際の処理実行】: まだsource/external_idを反映しないcreate_item_with_sourceを呼び出す
        let item =
            create_item_with_source(&pool, request, ItemSource::Api, Some("12345".to_string()))
                .await
                .unwrap();

        // 【結果検証】: source/external_idが引数通りに反映されることを確認
        assert_eq!(item.source, ItemSource::Api); // 【確認内容】: sourceがApiとして作成されることを確認 🔵
        assert_eq!(item.external_id, Some("12345".to_string())); // 【確認内容】: external_idが引数通り保持されることを確認 🔵
    }

    /// TC-0025-N04: 既存create_itemがcreate_item_with_sourceのラッパー化後もsource=manual/
    /// external_id=NULLを維持する（実DB・回帰確認必要）
    /// 【テスト目的】: リファクタ後もPOST /items（手動作成）の挙動が不変であることを確認する
    /// 【テスト内容】: CreateItemRequest{media_type: Movie, title: "君の名は。"}でcreate_itemを呼ぶ
    /// 【期待される動作】: item.source==Manual、item.external_id==None
    /// 🔵 信頼性レベル: 要件3.1（Option B再利用方針・既存テスト非破壊）、note.md L237-238より
    #[tokio::test]
    #[ignore]
    async fn create_item_wrapper_still_creates_manual_source_with_null_external_id() {
        let pool = test_pool().await;
        let request = create_item_request(MediaType::Movie, "君の名は。");

        // 【実際の処理実行】: 薄いラッパー化後のcreate_itemを呼び出す
        let item = create_item(&pool, request).await.unwrap();

        // 【結果検証】: 既存の挙動（manual/NULL固定）が維持されることを確認
        assert_eq!(item.source, ItemSource::Manual); // 【確認内容】: ラッパー化後もsource=manualが維持されることを確認 🔵
        assert_eq!(item.external_id, None); // 【確認内容】: ラッパー化後もexternal_id=NULLが維持されることを確認 🔵
    }

    /// TC-0025-N05: 同等の詳細データでmanual作成とapiインポートを行うと、source/external_id以外の
    /// Item内容が一致する（実DB必要）
    /// 【テスト目的】: 手動作成とインポートが同一のトランザクション処理経路を通り、差分が
    /// source/external_idのみであることを確認する
    /// 【テスト内容】: 同一media_type/titleでcreate_item（manual）とcreate_item_with_source（api）を
    /// それぞれ呼び出し、結果を比較する
    /// 【期待される動作】: media_type/titleが一致し、source/external_idのみ異なる
    /// 🔵 信頼性レベル: 要件4.1「TASK-0009一貫性」、TASK-0025.mdテストケース4に直接対応
    #[tokio::test]
    #[ignore]
    async fn create_item_and_create_item_with_source_share_same_fields_except_source_and_external_id()
     {
        let pool = test_pool().await;
        let manual_request = create_item_request(MediaType::Anime, "作品X");
        let import_request = create_item_request(MediaType::Anime, "作品X");

        let manual_item = create_item(&pool, manual_request).await.unwrap();
        let import_item = create_item_with_source(
            &pool,
            import_request,
            ItemSource::Api,
            Some("999".to_string()),
        )
        .await
        .unwrap();

        // 【結果検証】: 共通カラムが一致し、source/external_idのみ差分であることを確認
        assert_eq!(manual_item.media_type, import_item.media_type); // 【確認内容】: media_typeが一致することを確認 🔵
        assert_eq!(manual_item.title, import_item.title); // 【確認内容】: titleが一致することを確認 🔵
        assert_eq!(manual_item.source, ItemSource::Manual); // 【確認内容】: manual側がsource=Manualであることを確認 🔵
        assert_eq!(import_item.source, ItemSource::Api); // 【確認内容】: import側がsource=Apiであることを確認 🔵
        assert_eq!(manual_item.external_id, None); // 【確認内容】: manual側のexternal_idがNoneであることを確認 🔵
        assert_eq!(import_item.external_id, Some("999".to_string())); // 【確認内容】: import側のexternal_idが引数通りであることを確認 🔵
    }

    /// TC-0025-N07: 全8 media_typeでcreate_item_with_sourceが対応詳細テーブルへ振り分ける（実DB必要）
    /// 【テスト目的】: インポート経路でも8種すべてのmedia_typeで対応詳細テーブルへ正しくINSERTされる
    /// ことを確認する
    /// 【テスト内容】: 8種のmedia_typeそれぞれでcreate_item_with_source(Api)を呼び出す
    /// 【期待される動作】: 8回すべてエラーなく成功し、source=Apiのitemが作成される
    /// 🟡 信頼性レベル: 既存detail_table_nameの8 variant網羅テストとの整合からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn create_item_with_source_handles_all_eight_media_types() {
        let pool = test_pool().await;
        let media_types = [
            MediaType::Anime,
            MediaType::Movie,
            MediaType::Drama,
            MediaType::Manga,
            MediaType::Novel,
            MediaType::Game,
            MediaType::AcademicBook,
            MediaType::Paper,
        ];

        for (idx, media_type) in media_types.iter().enumerate() {
            let request = create_item_request(*media_type, "テスト作品");
            let external_id = format!("ext-{idx}");

            // 【実際の処理実行】: 各media_typeでインポート経路の作成処理を呼び出す
            let item =
                create_item_with_source(&pool, request, ItemSource::Api, Some(external_id.clone()))
                    .await
                    .unwrap();

            // 【結果検証】: 各media_typeで例外なく成功し、source=Apiが反映されることを確認
            assert_eq!(item.media_type, *media_type); // 【確認内容】: media_typeが指定通りであることを確認 🟡
            assert_eq!(item.source, ItemSource::Api); // 【確認内容】: 8種すべてでsource=Apiが反映されることを確認 🟡
        }
    }

    /// TC-DB-01: create_item_with_sourceがconsumed_dateをbindし、RETURNINGで反映する（実DB必要）
    /// 【テスト目的】: TASK-0030設計判断#1で拡張したINSERT文が、request.consumed_dateを正しく
    /// bindし、返却されたItemのconsumed_dateにCSV由来の値が反映されることを確認する
    /// 【テスト内容】: consumed_date=Some(2024-03-10)・source=Manual・external_id=Some("isbn")の
    /// CreateItemRequestでcreate_item_with_sourceを呼び出す
    /// 【期待される動作】: 返却itemのconsumed_date==Some(2024-03-10)、source==Manual、
    /// external_id==Some("isbn")
    /// 🔵 信頼性レベル: テストケース定義書TC-DB-01・item_repository.rs既存INSERT構造（拡張後）に基づく
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn create_item_with_source_binds_and_returns_consumed_date() {
        // 【テスト前準備】: 実DBプールを取得する
        let pool = test_pool().await;

        // 【テストデータ準備】: 読了日2024-03-10を持つCreateItemRequestを構築する
        // 【初期条件設定】: ブクログCSVの「読了日」をパースした結果を模したリクエスト
        let mut request = create_item_request(MediaType::Novel, "斜陽");
        request.consumed_date = Some(chrono::NaiveDate::from_ymd_opt(2024, 3, 10).unwrap());

        // 【実際の処理実行】: source=Manual・external_id=Some("isbn")でcreate_item_with_sourceを呼び出す
        // 【処理内容】: items本体INSERT（consumed_date含む）+ novel_details INSERTを同一トランザクションで実行する
        let item =
            create_item_with_source(&pool, request, ItemSource::Manual, Some("isbn".to_string()))
                .await
                .unwrap();

        // 【結果検証】: consumed_date/source/external_idがすべて引数・入力通りに反映されることを確認
        assert_eq!(
            item.consumed_date,
            Some(chrono::NaiveDate::from_ymd_opt(2024, 3, 10).unwrap())
        ); // 【確認内容】: consumed_dateがCSV値どおりにDBへ保存・RETURNINGされることを確認 🔵
        assert_eq!(item.source, ItemSource::Manual); // 【確認内容】: sourceが引数通りManualであることを確認 🔵
        assert_eq!(item.external_id, Some("isbn".to_string())); // 【確認内容】: external_idが引数通り保持されることを確認 🔵
    }

    /// create_item_with_sourceがdetails JSONBを永続化し、get_item_detailで同一JSONが返る（実DB必要）
    /// 【テスト目的】: インポート経路で渡される正規化済みMediaDetails JSONがitems.detailsへ保存され、
    /// GET /items/:id相当の読み出しでそのまま取得できることを確認する
    #[tokio::test]
    #[ignore]
    async fn create_item_with_source_persists_details_json() {
        let pool = test_pool().await;
        let details = serde_json::json!({
            "media_type": "anime",
            "external_id": "det-1",
            "title": "詳細付き作品",
            "episodes": 12,
            "studios": ["Studio A", "Studio B"],
            "trailer_url": "https://example.com/pv"
        });
        let mut request = create_item_request(MediaType::Anime, "詳細付き作品");
        request.details = Some(details.clone());

        let item =
            create_item_with_source(&pool, request, ItemSource::Api, Some("det-1".to_string()))
                .await
                .unwrap();

        let stored = get_item_detail(&pool, item.id).await.unwrap();

        assert_eq!(stored, Some(details)); // 【確認内容】: 保存したMediaDetails JSONがそのまま返ることを確認
    }

    /// TC-REG-01: create_item（manualラッパー）はconsumed_date拡張後もconsumed_date=Noneで
    /// 従来通り動作する（実DB・回帰確認必要）
    /// 【テスト目的】: CreateItemRequestへのconsumed_date追加（Option + #[serde(default)]）が、
    /// 既存の手動作成パス（create_item薄いラッパー、TASK-0009）を破壊しないことを確認する
    /// 【テスト内容】: consumed_date省略（=None）のCreateItemRequestでcreate_itemを呼び出す
    /// 【期待される動作】: 登録成功、item.consumed_date==None、source==Manual・external_id==None
    /// （既存挙動を維持）
    /// 🟡 信頼性レベル: item_repository.rs既存create_itemラッパー構造＋設計判断#1からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn create_item_wrapper_keeps_consumed_date_none_after_extension() {
        // 【テスト前準備】: 実DBプールを取得する
        let pool = test_pool().await;
        // 【テストデータ準備】: consumed_date省略（既存ヘルパー）のリクエストを用意する
        // 【初期条件設定】: TASK-0009時点の既存手動作成シナリオを再現する
        let request = create_item_request(MediaType::Anime, "回帰確認用作品");

        // 【実際の処理実行】: 薄いラッパー化されたcreate_itemを呼び出す
        // 【処理内容】: consumed_date拡張後もmanual/external_id=None固定の挙動が保たれるかを確認する
        let item = create_item(&pool, request).await.unwrap();

        // 【結果検証】: 既存の挙動（manual/NULL/consumed_date None）が維持されることを確認
        assert_eq!(item.consumed_date, None); // 【確認内容】: consumed_date省略時にDB側もNone（NULL）であることを確認 🟡
        assert_eq!(item.source, ItemSource::Manual); // 【確認内容】: 拡張後もsource=Manualが維持されることを確認 🟡
        assert_eq!(item.external_id, None); // 【確認内容】: 拡張後もexternal_id=NULLが維持されることを確認 🟡
    }

    /// TC-0025-E06: 同一media_type+external_idのitemが既存の状態で再インポートすると
    /// 409 ITEM_ALREADY_IMPORTEDになり重複作成されない（実DB必要）
    /// 【テスト目的】: 重複検知ロジック（アプリ層SELECT）と409マッピングを確認する
    /// 【テスト内容】: 事前にmedia_type=anime/external_id="12345"を1件投入し、同一値で
    /// find_existing_importを呼んでtrueが返ることを確認する（importハンドラ層の重複判定の土台）
    /// 【期待される動作】: find_existing_importがtrueを返し、再投入してもitems行数が増えない
    /// 🟡 信頼性レベル: 要件第6章の決定（案A: 409 ITEM_ALREADY_IMPORTED）、TASK-0025.mdテストケース3より
    #[tokio::test]
    #[ignore]
    async fn find_existing_import_detects_duplicate_media_type_and_external_id() {
        let pool = test_pool().await;
        let request = create_item_request(MediaType::Anime, "鬼滅の刃");
        create_item_with_source(&pool, request, ItemSource::Api, Some("12345".to_string()))
            .await
            .unwrap();

        // 【実際の処理実行】: find_existing_importを呼び出す
        let exists = find_existing_import(&pool, MediaType::Anime, "12345")
            .await
            .unwrap();

        // 【結果検証】: 同一(media_type, external_id)が既存と判定されることを確認
        assert!(exists); // 【確認内容】: 重複検知が機能し、trueが返ることを確認 🟡
    }

    /// TC-0025-E06-B: import_itemが重複インポート時に409 ITEM_ALREADY_IMPORTEDを返し、
    /// items行数が増えない（実DB必要）
    /// 【テスト目的】: import_item関数全体（重複チェック+INSERT）が409マッピング・原子性を
    /// 保つことを確認する
    /// 【テスト内容】: 事前に1件インポート済みの状態で同一media_type+external_idを再度import_item
    /// に渡す
    /// 【期待される動作】: 2回目はErr(ApiError{code:"ITEM_ALREADY_IMPORTED", status:409})、
    /// items総数が1件のまま変化しない
    /// 🟡 信頼性レベル: 要件第6章6.3（トランザクション内SELECT検知）より
    #[tokio::test]
    #[ignore]
    async fn import_item_returns_409_and_does_not_create_duplicate_row() {
        let pool = test_pool().await;
        let (media_type, external_id, first_request) =
            import_item_request(MediaType::Anime, "12345", "鬼滅の刃");
        import_item(&pool, media_type, external_id, first_request)
            .await
            .unwrap();

        let count_before: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM items WHERE external_id = $1")
                .bind("12345")
                .fetch_one(&pool)
                .await
                .unwrap();

        // 【実際の処理実行】: 同一media_type+external_idで再度import_itemを呼び出す
        let (media_type, external_id, second_request) =
            import_item_request(MediaType::Anime, "12345", "鬼滅の刃");
        let err = import_item(&pool, media_type, external_id, second_request)
            .await
            .unwrap_err();

        // 【結果検証】: 409 ITEM_ALREADY_IMPORTED・items行数不変であることを確認
        assert_eq!(err.error.code, "ITEM_ALREADY_IMPORTED"); // 【確認内容】: 重複時にITEM_ALREADY_IMPORTEDが返ることを確認 🟡
        assert_eq!(err.status, axum::http::StatusCode::CONFLICT); // 【確認内容】: HTTPステータスが409であることを確認 🟡

        let count_after: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM items WHERE external_id = $1")
                .bind("12345")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count_before, count_after); // 【確認内容】: 重複検知時にitems行数が増えないことを確認（最重要） 🟡
    }

    /// TC-0025-E08: create_item_with_source実行中のDBエラーが500 INTERNAL_ERRORへ変換され、
    /// DB内部情報を漏洩しない（接続不能プールで再現）
    /// 【テスト目的】: DBエラーの汎用500変換と情報漏洩防止を確認する
    /// 【テスト内容】: 接続不能なPgPoolに対しcreate_item_with_sourceを呼ぶ
    /// 【期待される動作】: Err(ApiError)が返り、error.code=="INTERNAL_ERROR"、status==500
    /// 🟡 信頼性レベル: 既存db_error関数・既存list_items_converts_db_error_to_internal_errorとのパリティ
    #[tokio::test]
    #[ignore]
    async fn create_item_with_source_converts_db_error_to_internal_error() {
        // 【テスト前準備】: 接続不能なプールを構築する
        let pool = unreachable_pool().await;
        let request = create_item_request(MediaType::Anime, "鬼滅の刃");

        // 【実際の処理実行】: create_item_with_sourceを呼び、DB接続不能エラーの変換を確認する
        let result =
            create_item_with_source(&pool, request, ItemSource::Api, Some("1".to_string())).await;

        // 【結果検証】: DB内部情報が漏洩せず汎用INTERNAL_ERRORに変換されることを確認
        let err = result.unwrap_err();
        assert_eq!(err.error.code, "INTERNAL_ERROR"); // 【確認内容】: DB接続不能が汎用INTERNAL_ERRORコードに変換されることを確認 🟡
        assert_eq!(err.status, axum::http::StatusCode::INTERNAL_SERVER_ERROR); // 【確認内容】: HTTPステータスが500であることを確認 🟡
    }

    /// TC-0025-B01: 必須3項目のみ（任意項目すべて省略）の最小構成でインポートが成功する（実DB必要）
    /// 【テスト目的】: 任意項目を一切伴わない最小構成でもトランザクションが成功することを確認する
    /// 【テスト内容】: media_type=anime, external_id="1", title="A"のみでcreate_item_with_sourceを呼ぶ
    /// 【期待される動作】: 201相当の成功（Item作成）、任意項目はNone/NULL
    /// 🟡 信頼性レベル: 要件2.1入力仕様表（任意項目）からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn create_item_with_source_succeeds_with_minimal_fields_only() {
        let pool = test_pool().await;
        let request = create_item_request(MediaType::Anime, "A");

        // 【実際の処理実行】: 任意項目を一切指定しない最小構成で呼び出す
        let item = create_item_with_source(&pool, request, ItemSource::Api, Some("1".to_string()))
            .await
            .unwrap();

        // 【結果検証】: 任意項目がNoneのまま登録成功することを確認
        assert_eq!(item.original_title, None); // 【確認内容】: 任意項目省略時にoriginal_titleがNoneであることを確認 🟡
        assert_eq!(item.external_id, Some("1".to_string())); // 【確認内容】: external_idが最短境界値("1")でも保持されることを確認 🟡
    }

    /// TC-0025-B03: 異なるmedia_type・同一external_idは重複とみなされず両方作成される（実DB必要）
    /// 【テスト目的】: 重複判定キーが(media_type, external_id)の複合キーであることを確認する
    /// 【テスト内容】: media_type=anime/external_id="100"を投入後、find_existing_importに
    /// media_type=movie/external_id="100"を渡す
    /// 【期待される動作】: find_existing_importがfalseを返す（異なるmedia_typeのため重複ではない）
    /// 🟡 信頼性レベル: 要件第6章6.3（重複チェックWHERE media_type=$1 AND external_id=$2）からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn find_existing_import_does_not_treat_different_media_type_as_duplicate() {
        let pool = test_pool().await;
        let request = create_item_request(MediaType::Anime, "アニメ100");
        create_item_with_source(&pool, request, ItemSource::Api, Some("100".to_string()))
            .await
            .unwrap();

        // 【実際の処理実行】: 異なるmedia_type（movie）+同一external_id（100）で重複判定を呼ぶ
        let exists = find_existing_import(&pool, MediaType::Movie, "100")
            .await
            .unwrap();

        // 【結果検証】: media_typeが異なるため重複と判定されないことを確認
        assert!(!exists); // 【確認内容】: external_id単独一致のみでは重複と判定されないことを確認 🟡
    }
}
