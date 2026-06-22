-- ========================================
-- mediavault-backend データベーススキーマ
-- ========================================
--
-- 作成日: 2026-06-22
-- 関連設計: architecture.md
--
-- 信頼性レベル:
-- - 🔵 青信号: PRDデータモデル・EARS要件定義書を参考にした確実な定義
-- - 🟡 黄信号: PRD・要件定義から妥当な推測による定義
-- - 🔴 赤信号: PRD・要件定義にない推測による定義

-- ========================================
-- ENUM型定義
-- ========================================

-- 🔵 PRDデータモデルより
CREATE TYPE media_type AS ENUM (
    'anime', 'movie', 'drama', 'manga', 'novel', 'game', 'academic_book', 'paper'
);

-- 🟡 PRD「視聴中/読了/未着手」から妥当な推測（具体的な値の数は推測）
CREATE TYPE item_status AS ENUM ('not_started', 'in_progress', 'completed');

-- 🔵 REQ-201/201bより
CREATE TYPE item_source AS ENUM ('api', 'manual');

-- 🔵 PRDデータモデルより
CREATE TYPE group_type AS ENUM ('season', 'volume', 'chapter');

-- 🔵 PRDデータモデルより
CREATE TYPE relation_type AS ENUM ('reference', 'dlc');

-- 🟡 PRD「ファイルアップロード」から妥当な推測
CREATE TYPE file_type AS ENUM ('pdf', 'image', 'other');

-- 🔵 PRD・interview-record.md Q5より（Jikanはキー不要のため対象外）
CREATE TYPE api_provider AS ENUM ('tmdb', 'igdb', 'ndl', 'steam', 'openlibrary', 'anilist');

-- ========================================
-- 共通テーブル
-- ========================================

-- items: 共通項目テーブル 🔵 PRDデータモデル「共通項目」・REQ-405より
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

    -- 🟡 REQ-201/201bより：source=manualの場合external_idはNULL許容、source=apiの場合は保持
    CONSTRAINT chk_items_source_external_id CHECK (
        (source = 'manual') OR (source = 'api' AND external_id IS NOT NULL)
    ),
    CONSTRAINT chk_items_title_not_empty CHECK (title <> '') -- 🟡 TC-001-B01より
);

CREATE INDEX idx_items_media_type ON items(media_type); -- 🔵 一覧・絞り込みREQ-001より
CREATE INDEX idx_items_status ON items(status); -- 🔵 一覧・絞り込みREQ-001より
CREATE INDEX idx_items_is_favorite ON items(is_favorite); -- 🔵 一覧・絞り込みREQ-001より
CREATE INDEX idx_items_external_id ON items(external_id); -- 🟡 重複インポート防止の観点から妥当な推測

-- ========================================
-- メディア別詳細テーブル（item_idで1:1）
-- ========================================

-- anime_details 🔵 PRDメディア別機能より
CREATE TABLE anime_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    episode_count INTEGER,
    season_count INTEGER,
    studio VARCHAR(255),
    genre_list TEXT[] NOT NULL DEFAULT '{}',
    source_type VARCHAR(100),
    jikan_id VARCHAR(100)
);

-- movie_details 🔵
CREATE TABLE movie_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    runtime_minutes INTEGER,
    director VARCHAR(255),
    genre_list TEXT[] NOT NULL DEFAULT '{}',
    tmdb_id VARCHAR(100)
);

-- drama_details 🔵
CREATE TABLE drama_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    episode_count INTEGER,
    season_count INTEGER,
    network VARCHAR(255),
    genre_list TEXT[] NOT NULL DEFAULT '{}',
    tmdb_id VARCHAR(100)
);

-- manga_details 🔵
CREATE TABLE manga_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    volume_count INTEGER,
    chapter_count INTEGER,
    author VARCHAR(255),
    illustrator VARCHAR(255),
    magazine VARCHAR(255),
    jikan_id VARCHAR(100)
);

-- novel_details 🔵
CREATE TABLE novel_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    volume_count INTEGER,
    author VARCHAR(255),
    publisher VARCHAR(255),
    isbn VARCHAR(50),
    openlibrary_id VARCHAR(100),
    google_books_id VARCHAR(100)
);

-- game_details 🔵
CREATE TABLE game_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    platform_list TEXT[] NOT NULL DEFAULT '{}',
    developer VARCHAR(255),
    publisher VARCHAR(255),
    steam_appid VARCHAR(100),
    igdb_id VARCHAR(100)
);

-- academic_book_details 🔵
CREATE TABLE academic_book_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    author VARCHAR(255),
    publisher VARCHAR(255),
    isbn VARCHAR(50),
    ndl_id VARCHAR(100),
    google_books_id VARCHAR(100)
);

-- paper_details 🔵
CREATE TABLE paper_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    doi VARCHAR(255),
    journal_name VARCHAR(255),
    volume_issue VARCHAR(100),
    page_range VARCHAR(100),
    author_list TEXT[] NOT NULL DEFAULT '{}',
    ndl_id VARCHAR(100)
);

-- ========================================
-- タグ・カテゴリ・マイリスト
-- ========================================

-- tags 🔵 PRDデータモデルより
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE
);

-- item_tags（多対多） 🔵
CREATE TABLE item_tags (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);

-- categories 🔵
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE
);

-- item_categories（多対多） 🔵
CREATE TABLE item_categories (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, category_id)
);

-- mylists 🔵
CREATE TABLE mylists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- mylist_items（多対多） 🔵
CREATE TABLE mylist_items (
    mylist_id UUID NOT NULL REFERENCES mylists(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    PRIMARY KEY (mylist_id, item_id)
);

-- ========================================
-- 関連付け・リンク・ファイル・トレーラー
-- ========================================

-- item_relations 🔵 PRDデータモデル・REQ-006/013より
CREATE TABLE item_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    related_item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    relation_type relation_type NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_item_relations_not_self CHECK (item_id <> related_item_id), -- 🟡 自己参照防止の妥当な推測
    CONSTRAINT uq_item_relations UNIQUE (item_id, related_item_id, relation_type)
);

CREATE INDEX idx_item_relations_item_id ON item_relations(item_id); -- 🔵
CREATE INDEX idx_item_relations_related_item_id ON item_relations(related_item_id); -- 🔵

-- item_links 🔵 PRDデータモデルより
CREATE TABLE item_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    url VARCHAR(1000) NOT NULL,
    label VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_links_item_id ON item_links(item_id); -- 🔵

-- item_files 🔵 PRDデータモデル・REQ-007/019/020より
CREATE TABLE item_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    path VARCHAR(1000) NOT NULL,
    label VARCHAR(255),
    file_type file_type NOT NULL,
    calibre_book_id VARCHAR(100), -- 🔵 REQ-020より（file_type=pdfの場合のみ使用）
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_files_item_id ON item_files(item_id); -- 🔵

-- item_trailers 🔵 PRDデータモデルより
CREATE TABLE item_trailers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    url VARCHAR(1000) NOT NULL,
    label VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_trailers_item_id ON item_trailers(item_id); -- 🔵

-- ========================================
-- グループ・エピソード
-- ========================================

-- item_groups（シーズン/巻/章の汎用モデル、入れ子構造可） 🔵 PRDデータモデルより
CREATE TABLE item_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    parent_item_id UUID REFERENCES items(id) ON DELETE CASCADE, -- 🔵 PRD「parent_item_idを通じて入れ子構造」より
    group_type group_type NOT NULL,
    group_name VARCHAR(255) NOT NULL,
    number INTEGER,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_groups_item_id ON item_groups(item_id); -- 🔵
CREATE INDEX idx_item_groups_parent_item_id ON item_groups(parent_item_id); -- 🔵

-- item_episodes（season/chapter配下のみ使用） 🔵 PRDデータモデル・EDGE-101より
CREATE TABLE item_episodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES item_groups(id) ON DELETE CASCADE,
    episode_number INTEGER NOT NULL,
    title VARCHAR(500),
    original_title VARCHAR(500),
    air_date DATE,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_item_episodes UNIQUE (group_id, episode_number)
);

CREATE INDEX idx_item_episodes_group_id ON item_episodes(group_id); -- 🔵

-- EDGE-101: group_type=volumeへのepisode追加を防ぐトリガー
-- 🔵 EDGE-101「volume配下のグループへの登録はリクエストを拒否」より
-- 備考: アプリケーション層（ハンドラ）でも検証するが、DB層でも二重に保証する
CREATE OR REPLACE FUNCTION check_episode_group_type()
RETURNS TRIGGER AS $$
DECLARE
    v_group_type group_type;
BEGIN
    SELECT group_type INTO v_group_type FROM item_groups WHERE id = NEW.group_id;
    IF v_group_type = 'volume' THEN
        RAISE EXCEPTION 'item_episodes cannot be added to a volume-type group (group_id=%)', NEW.group_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_check_episode_group_type
    BEFORE INSERT OR UPDATE ON item_episodes
    FOR EACH ROW
    EXECUTE FUNCTION check_episode_group_type();

-- ========================================
-- スタッフ
-- ========================================

-- staff 🔵 PRDデータモデルより
CREATE TABLE staff (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(100),
    name VARCHAR(255) NOT NULL,
    image_url VARCHAR(1000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_staff_external_id ON staff(external_id); -- 🟡 重複登録防止の妥当な推測

-- item_staff（多対多、role付き） 🔵 PRDデータモデルより
CREATE TABLE item_staff (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    staff_id UUID NOT NULL REFERENCES staff(id) ON DELETE CASCADE,
    role VARCHAR(100) NOT NULL,
    character_name VARCHAR(255)
);

CREATE INDEX idx_item_staff_item_id ON item_staff(item_id); -- 🔵
CREATE INDEX idx_item_staff_staff_id ON item_staff(staff_id); -- 🔵

-- ========================================
-- 外部APIキー管理
-- ========================================

-- api_credentials 🔵 REQ-015/NFR-202・interview-record Q5より
CREATE TABLE api_credentials (
    provider api_provider PRIMARY KEY,
    api_key VARCHAR(500) NOT NULL, -- 🔵 平文保存（暗号化は本フェーズ対象外）
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ========================================
-- updated_at 自動更新トリガー（共通） 🔵 既存実装の共通パターンとして採用
-- ========================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_items_updated_at BEFORE UPDATE ON items
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER trg_item_groups_updated_at BEFORE UPDATE ON item_groups
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER trg_item_episodes_updated_at BEFORE UPDATE ON item_episodes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER trg_item_files_updated_at BEFORE UPDATE ON item_files
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER trg_api_credentials_updated_at BEFORE UPDATE ON api_credentials
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ========================================
-- 信頼性レベルサマリー
-- ========================================
-- - 🔵 青信号: 47件 (85%)
-- - 🟡 黄信号: 8件 (15%)
-- - 🔴 赤信号: 0件 (0%)
--
-- 品質評価: 高品質
