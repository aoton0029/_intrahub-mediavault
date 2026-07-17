-- ========================================
-- mediavault-backend initial schema migration (down)
-- ========================================

DROP TRIGGER IF EXISTS trg_api_credentials_updated_at ON api_credentials;
DROP TRIGGER IF EXISTS trg_item_files_updated_at ON item_files;
DROP TRIGGER IF EXISTS trg_item_episodes_updated_at ON item_episodes;
DROP TRIGGER IF EXISTS trg_item_groups_updated_at ON item_groups;
DROP TRIGGER IF EXISTS trg_items_updated_at ON items;

DROP TRIGGER IF EXISTS trg_check_episode_group_type ON item_episodes;

DROP FUNCTION IF EXISTS update_updated_at_column();
DROP FUNCTION IF EXISTS check_episode_group_type();

DROP TABLE IF EXISTS api_credentials;
DROP TABLE IF EXISTS item_cast;
DROP TABLE IF EXISTS cast_members;
DROP TABLE IF EXISTS item_staff;
DROP TABLE IF EXISTS staff;
DROP TABLE IF EXISTS item_episodes;
DROP TABLE IF EXISTS item_groups;
DROP TABLE IF EXISTS item_trailers;
DROP TABLE IF EXISTS item_files;
DROP TABLE IF EXISTS item_links;
DROP TABLE IF EXISTS item_relations;
DROP TABLE IF EXISTS mylist_items;
DROP TABLE IF EXISTS mylists;
DROP TABLE IF EXISTS item_categories;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS item_tags;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS items;

DROP TYPE IF EXISTS api_provider;
DROP TYPE IF EXISTS file_type;
DROP TYPE IF EXISTS relation_type;
DROP TYPE IF EXISTS group_type;
DROP TYPE IF EXISTS item_source;
DROP TYPE IF EXISTS item_status;
DROP TYPE IF EXISTS media_type;
-- ========================================
-- item_streaming_links migration (down)
-- ========================================

DROP TABLE IF EXISTS item_streaming_links;
DROP TYPE IF EXISTS streaming_platform;

-- ========================================
-- item_images migration (down)
-- ========================================

DROP TABLE IF EXISTS item_images;
DROP TYPE IF EXISTS image_source;
DROP TYPE IF EXISTS image_kind;
