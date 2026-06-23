-- ========================================
-- mediavault-backend 初期マイグレーション (down)
-- ========================================

-- 詳細テーブル8種を先に削除（items への FK を持つため）
DROP TABLE IF EXISTS paper_details CASCADE;
DROP TABLE IF EXISTS academic_book_details CASCADE;
DROP TABLE IF EXISTS game_details CASCADE;
DROP TABLE IF EXISTS novel_details CASCADE;
DROP TABLE IF EXISTS manga_details CASCADE;
DROP TABLE IF EXISTS drama_details CASCADE;
DROP TABLE IF EXISTS movie_details CASCADE;
DROP TABLE IF EXISTS anime_details CASCADE;

-- items テーブル削除
DROP TABLE IF EXISTS items CASCADE;

-- ENUM型を依存順に削除
DROP TYPE IF EXISTS api_provider;
DROP TYPE IF EXISTS file_type;
DROP TYPE IF EXISTS relation_type;
DROP TYPE IF EXISTS group_type;
DROP TYPE IF EXISTS item_source;
DROP TYPE IF EXISTS item_status;
DROP TYPE IF EXISTS media_type;
