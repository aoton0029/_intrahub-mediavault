//! バックアップ（エクスポート/インポート）のDB操作
//!
//! エクスポート側は全行SELECT、インポート側は `ON CONFLICT DO NOTHING` による
//! 行単位INSERT（戻り値: 挿入されたらtrue、既存衝突でスキップならfalse）。
//! いずれもトランザクション内から `&mut PgConnection` で呼び出す想定。

use sqlx::PgConnection;

use crate::models::backup::{
    BackupCastMember, BackupCategory, BackupItem, BackupItemCast, BackupItemCategory,
    BackupItemEpisode, BackupItemFile, BackupItemGroup, BackupItemImage, BackupItemLink,
    BackupItemRelation, BackupItemStaff, BackupItemStreamingLink, BackupItemTag, BackupItemTrailer,
    BackupMylist, BackupMylistItem, BackupStaff, BackupTag,
};

/// SELECT・INSERT文のペアを共通化するマクロ。
/// fetch_all_*: 全行を主キー順に取得する。
/// insert_*: 1行INSERTし、ON CONFLICT DO NOTHINGでスキップされたらfalseを返す。
macro_rules! backup_table {
    (
        $fetch_fn:ident, $insert_fn:ident, $row:ty,
        select: $select_sql:expr,
        insert: $insert_sql:expr,
        bind: |$q:ident, $r:ident| $binds:expr
    ) => {
        pub async fn $fetch_fn(conn: &mut PgConnection) -> Result<Vec<$row>, sqlx::Error> {
            sqlx::query_as::<_, $row>($select_sql).fetch_all(conn).await
        }

        pub async fn $insert_fn(conn: &mut PgConnection, $r: &$row) -> Result<bool, sqlx::Error> {
            let $q = sqlx::query($insert_sql);
            let result = $binds.execute(conn).await?;
            Ok(result.rows_affected() > 0)
        }
    };
}

backup_table!(
    fetch_all_items, insert_item, BackupItem,
    select: "SELECT id, media_type, title, original_title, description, cover_image_url, \
             release_date, homepage_url, status, consumed_date, rating, is_favorite, source, \
             external_id, details, created_at, updated_at FROM items ORDER BY created_at, id",
    insert: "INSERT INTO items (id, media_type, title, original_title, description, \
             cover_image_url, release_date, homepage_url, status, consumed_date, rating, \
             is_favorite, source, external_id, details, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17) \
             ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.media_type)
        .bind(&row.title)
        .bind(&row.original_title)
        .bind(&row.description)
        .bind(&row.cover_image_url)
        .bind(row.release_date)
        .bind(&row.homepage_url)
        .bind(row.status)
        .bind(row.consumed_date)
        .bind(row.rating)
        .bind(row.is_favorite)
        .bind(row.source)
        .bind(&row.external_id)
        .bind(&row.details)
        .bind(row.created_at)
        .bind(row.updated_at)
);

backup_table!(
    fetch_all_tags, insert_tag, BackupTag,
    select: "SELECT id, name FROM tags ORDER BY id",
    insert: "INSERT INTO tags (id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    bind: |q, row| q.bind(row.id).bind(&row.name)
);

backup_table!(
    fetch_all_item_tags, insert_item_tag, BackupItemTag,
    select: "SELECT item_id, tag_id FROM item_tags ORDER BY item_id, tag_id",
    insert: "INSERT INTO item_tags (item_id, tag_id) VALUES ($1, $2) \
             ON CONFLICT (item_id, tag_id) DO NOTHING",
    bind: |q, row| q.bind(row.item_id).bind(row.tag_id)
);

backup_table!(
    fetch_all_categories, insert_category, BackupCategory,
    select: "SELECT id, name FROM categories ORDER BY id",
    insert: "INSERT INTO categories (id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    bind: |q, row| q.bind(row.id).bind(&row.name)
);

backup_table!(
    fetch_all_item_categories, insert_item_category, BackupItemCategory,
    select: "SELECT item_id, category_id FROM item_categories ORDER BY item_id, category_id",
    insert: "INSERT INTO item_categories (item_id, category_id) VALUES ($1, $2) \
             ON CONFLICT (item_id, category_id) DO NOTHING",
    bind: |q, row| q.bind(row.item_id).bind(row.category_id)
);

backup_table!(
    fetch_all_mylists, insert_mylist, BackupMylist,
    select: "SELECT id, name, created_at FROM mylists ORDER BY created_at, id",
    insert: "INSERT INTO mylists (id, name, created_at) VALUES ($1, $2, $3) \
             ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q.bind(row.id).bind(&row.name).bind(row.created_at)
);

backup_table!(
    fetch_all_mylist_items, insert_mylist_item, BackupMylistItem,
    select: "SELECT mylist_id, item_id FROM mylist_items ORDER BY mylist_id, item_id",
    insert: "INSERT INTO mylist_items (mylist_id, item_id) VALUES ($1, $2) \
             ON CONFLICT (mylist_id, item_id) DO NOTHING",
    bind: |q, row| q.bind(row.mylist_id).bind(row.item_id)
);

backup_table!(
    fetch_all_item_relations, insert_item_relation, BackupItemRelation,
    select: "SELECT id, item_id, related_item_id, relation_type, created_at \
             FROM item_relations ORDER BY created_at, id",
    insert: "INSERT INTO item_relations (id, item_id, related_item_id, relation_type, created_at) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.item_id)
        .bind(row.related_item_id)
        .bind(row.relation_type)
        .bind(row.created_at)
);

backup_table!(
    fetch_all_item_links, insert_item_link, BackupItemLink,
    select: "SELECT id, item_id, url, label, created_at FROM item_links ORDER BY created_at, id",
    insert: "INSERT INTO item_links (id, item_id, url, label, created_at) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.item_id)
        .bind(&row.url)
        .bind(&row.label)
        .bind(row.created_at)
);

backup_table!(
    fetch_all_item_trailers, insert_item_trailer, BackupItemTrailer,
    select: "SELECT id, item_id, url, label, created_at FROM item_trailers ORDER BY created_at, id",
    insert: "INSERT INTO item_trailers (id, item_id, url, label, created_at) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.item_id)
        .bind(&row.url)
        .bind(&row.label)
        .bind(row.created_at)
);

backup_table!(
    fetch_all_item_streaming_links, insert_item_streaming_link, BackupItemStreamingLink,
    select: "SELECT id, item_id, platform, url, created_at \
             FROM item_streaming_links ORDER BY created_at, id",
    insert: "INSERT INTO item_streaming_links (id, item_id, platform, url, created_at) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.item_id)
        .bind(row.platform)
        .bind(&row.url)
        .bind(row.created_at)
);

backup_table!(
    fetch_all_item_images, insert_item_image, BackupItemImage,
    select: "SELECT id, item_id, url, kind, source, sort_order, created_at \
             FROM item_images ORDER BY created_at, id",
    insert: "INSERT INTO item_images (id, item_id, url, kind, source, sort_order, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.item_id)
        .bind(&row.url)
        .bind(row.kind)
        .bind(row.source)
        .bind(row.sort_order)
        .bind(row.created_at)
);

backup_table!(
    fetch_all_item_files, insert_item_file, BackupItemFile,
    select: "SELECT id, item_id, path, label, file_type, calibre_book_id, created_at, updated_at \
             FROM item_files ORDER BY created_at, id",
    insert: "INSERT INTO item_files (id, item_id, path, label, file_type, calibre_book_id, \
             created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.item_id)
        .bind(&row.path)
        .bind(&row.label)
        .bind(row.file_type)
        .bind(&row.calibre_book_id)
        .bind(row.created_at)
        .bind(row.updated_at)
);

backup_table!(
    fetch_all_item_groups, insert_item_group, BackupItemGroup,
    select: "SELECT id, item_id, parent_item_id, group_type, group_name, number, display_order, \
             created_at, updated_at FROM item_groups ORDER BY created_at, id",
    insert: "INSERT INTO item_groups (id, item_id, parent_item_id, group_type, group_name, \
             number, display_order, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.item_id)
        .bind(row.parent_item_id)
        .bind(row.group_type)
        .bind(&row.group_name)
        .bind(row.number)
        .bind(row.display_order)
        .bind(row.created_at)
        .bind(row.updated_at)
);

backup_table!(
    fetch_all_item_episodes, insert_item_episode, BackupItemEpisode,
    select: "SELECT id, group_id, episode_number, title, original_title, air_date, description, \
             created_at, updated_at FROM item_episodes ORDER BY created_at, id",
    insert: "INSERT INTO item_episodes (id, group_id, episode_number, title, original_title, \
             air_date, description, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.group_id)
        .bind(row.episode_number)
        .bind(&row.title)
        .bind(&row.original_title)
        .bind(row.air_date)
        .bind(&row.description)
        .bind(row.created_at)
        .bind(row.updated_at)
);

backup_table!(
    fetch_all_staff, insert_staff, BackupStaff,
    select: "SELECT id, external_id, name, image_url, created_at FROM staff ORDER BY created_at, id",
    insert: "INSERT INTO staff (id, external_id, name, image_url, created_at) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(&row.external_id)
        .bind(&row.name)
        .bind(&row.image_url)
        .bind(row.created_at)
);

backup_table!(
    fetch_all_item_staff, insert_item_staff, BackupItemStaff,
    select: "SELECT id, item_id, staff_id, role, character_name FROM item_staff ORDER BY id",
    insert: "INSERT INTO item_staff (id, item_id, staff_id, role, character_name) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.item_id)
        .bind(row.staff_id)
        .bind(&row.role)
        .bind(&row.character_name)
);

backup_table!(
    fetch_all_cast_members, insert_cast_member, BackupCastMember,
    select: "SELECT id, external_id, name, image_url, created_at \
             FROM cast_members ORDER BY created_at, id",
    insert: "INSERT INTO cast_members (id, external_id, name, image_url, created_at) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(&row.external_id)
        .bind(&row.name)
        .bind(&row.image_url)
        .bind(row.created_at)
);

backup_table!(
    fetch_all_item_cast, insert_item_cast, BackupItemCast,
    select: "SELECT id, item_id, cast_id, character_name FROM item_cast ORDER BY id",
    insert: "INSERT INTO item_cast (id, item_id, cast_id, character_name) \
             VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    bind: |q, row| q
        .bind(row.id)
        .bind(row.item_id)
        .bind(row.cast_id)
        .bind(&row.character_name)
);
