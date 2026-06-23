-- ========================================
-- mediavault-backend 初期マイグレーション (up)
-- ========================================
--
-- 関連設計: docs/design/mediavault-backend/database-schema.sql
-- 内容: ENUM型7種 + items テーブル + メディア別詳細テーブル8種

-- ========================================
-- ENUM型定義
-- ========================================

CREATE TYPE media_type AS ENUM (
    'anime', 'movie', 'drama', 'manga', 'novel', 'game', 'academic_book', 'paper'
);

CREATE TYPE item_status AS ENUM ('not_started', 'in_progress', 'completed');

CREATE TYPE item_source AS ENUM ('api', 'manual');

CREATE TYPE group_type AS ENUM ('season', 'volume', 'chapter');

CREATE TYPE relation_type AS ENUM ('reference', 'dlc');

CREATE TYPE file_type AS ENUM ('pdf', 'image', 'other');

CREATE TYPE api_provider AS ENUM ('tmdb', 'igdb', 'ndl', 'steam', 'openlibrary', 'anilist');

-- ========================================
-- 共通テーブル
-- ========================================

-- items: 共通項目テーブル
CREATE TABLE items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_type media_type NOT NULL,
    title VARCHAR(500) NOT NULL,
    original_title VARCHAR(500),
    description TEXT,
    cover_image_url VARCHAR(1000),
    release_date DATE,
    homepage_url VARCHAR(1000),
    status item_status NOT NULL DEFAULT 'not_started',
    consumed_date DATE,
    rating REAL,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    source item_source NOT NULL,
    external_id VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT chk_items_source_external_id CHECK (
        (source = 'manual') OR (source = 'api' AND external_id IS NOT NULL)
    ),
    CONSTRAINT chk_items_title_not_empty CHECK (title <> '')
);

CREATE INDEX idx_items_media_type ON items(media_type);
CREATE INDEX idx_items_status ON items(status);
CREATE INDEX idx_items_is_favorite ON items(is_favorite);
CREATE INDEX idx_items_external_id ON items(external_id);

-- ========================================
-- メディア別詳細テーブル（item_idで1:1）
-- ========================================

-- anime_details
CREATE TABLE anime_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    episode_count INTEGER,
    season_count INTEGER,
    studio VARCHAR(255),
    genre_list TEXT[] NOT NULL DEFAULT '{}',
    source_type VARCHAR(100),
    jikan_id VARCHAR(100)
);

-- movie_details
CREATE TABLE movie_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    runtime_minutes INTEGER,
    director VARCHAR(255),
    genre_list TEXT[] NOT NULL DEFAULT '{}',
    tmdb_id VARCHAR(100)
);

-- drama_details
CREATE TABLE drama_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    episode_count INTEGER,
    season_count INTEGER,
    network VARCHAR(255),
    genre_list TEXT[] NOT NULL DEFAULT '{}',
    tmdb_id VARCHAR(100)
);

-- manga_details
CREATE TABLE manga_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    volume_count INTEGER,
    chapter_count INTEGER,
    author VARCHAR(255),
    illustrator VARCHAR(255),
    magazine VARCHAR(255),
    jikan_id VARCHAR(100)
);

-- novel_details
CREATE TABLE novel_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    volume_count INTEGER,
    author VARCHAR(255),
    publisher VARCHAR(255),
    isbn VARCHAR(50),
    openlibrary_id VARCHAR(100),
    google_books_id VARCHAR(100)
);

-- game_details
CREATE TABLE game_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    platform_list TEXT[] NOT NULL DEFAULT '{}',
    developer VARCHAR(255),
    publisher VARCHAR(255),
    steam_appid VARCHAR(100),
    igdb_id VARCHAR(100)
);

-- academic_book_details
CREATE TABLE academic_book_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    author VARCHAR(255),
    publisher VARCHAR(255),
    isbn VARCHAR(50),
    ndl_id VARCHAR(100),
    google_books_id VARCHAR(100)
);

-- paper_details
CREATE TABLE paper_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    doi VARCHAR(255),
    journal_name VARCHAR(255),
    volume_issue VARCHAR(100),
    page_range VARCHAR(100),
    author_list TEXT[] NOT NULL DEFAULT '{}',
    ndl_id VARCHAR(100)
);
