-- ========================================
-- mediavault-backend 関連テーブル群マイグレーション (down)
-- ========================================
-- 依存関係の逆順でトリガー・関数・テーブルをDROPする

-- updated_at トリガー
DROP TRIGGER IF EXISTS trg_api_credentials_updated_at ON api_credentials;
DROP TRIGGER IF EXISTS trg_item_files_updated_at ON item_files;
DROP TRIGGER IF EXISTS trg_item_episodes_updated_at ON item_episodes;
DROP TRIGGER IF EXISTS trg_item_groups_updated_at ON item_groups;
DROP TRIGGER IF EXISTS trg_items_updated_at ON items;

DROP FUNCTION IF EXISTS update_updated_at_column();

-- api_credentials
DROP TABLE IF EXISTS api_credentials;

-- スタッフ
DROP TABLE IF EXISTS item_staff;
DROP TABLE IF EXISTS staff;

-- EDGE-101 トリガー・関数
DROP TRIGGER IF EXISTS trg_check_episode_group_type ON item_episodes;
DROP FUNCTION IF EXISTS check_episode_group_type();

-- グループ・エピソード
DROP TABLE IF EXISTS item_episodes;
DROP TABLE IF EXISTS item_groups;

-- 関連付け・リンク・ファイル・トレーラー
DROP TABLE IF EXISTS item_trailers;
DROP TABLE IF EXISTS item_files;
DROP TABLE IF EXISTS item_links;
DROP TABLE IF EXISTS item_relations;

-- タグ・カテゴリ・マイリスト
DROP TABLE IF EXISTS mylist_items;
DROP TABLE IF EXISTS mylists;
DROP TABLE IF EXISTS item_categories;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS item_tags;
DROP TABLE IF EXISTS tags;
