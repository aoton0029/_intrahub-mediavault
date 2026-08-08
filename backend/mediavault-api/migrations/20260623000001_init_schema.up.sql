-- ========================================
-- mediavault-backend initial schema migration (up)
-- ========================================

CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ========================================
-- Enum types
-- ========================================

CREATE TYPE media_type AS ENUM (
    'anime', 'movie', 'drama', 'manga', 'novel', 'game', 'academic_book', 'paper'
);

CREATE TYPE item_status AS ENUM ('not_started', 'in_progress', 'completed');

CREATE TYPE item_source AS ENUM ('api', 'manual');

CREATE TYPE group_type AS ENUM ('season', 'volume', 'chapter');

CREATE TYPE relation_type AS ENUM ('adaptation', 'sequel', 'prequel', 'spinoff', 'dlc', 'reference');

CREATE TYPE file_type AS ENUM ('pdf', 'image', 'video', 'audio', 'archive', 'other');

CREATE TYPE api_provider AS ENUM ('tmdb', 'igdb', 'ndl', 'steam', 'openlibrary', 'anilist', 'annict', 'rakuten');

-- ========================================
-- Core tables
-- ========================================

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
    details JSONB,
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

CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE item_tags (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);

CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE item_categories (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, category_id)
);

CREATE TABLE mylists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE mylist_items (
    mylist_id UUID NOT NULL REFERENCES mylists(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    PRIMARY KEY (mylist_id, item_id)
);

CREATE TABLE item_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    related_item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    relation_type relation_type NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_item_relations_not_self CHECK (item_id <> related_item_id),
    CONSTRAINT uq_item_relations UNIQUE (item_id, related_item_id, relation_type)
);

CREATE INDEX idx_item_relations_item_id ON item_relations(item_id);
CREATE INDEX idx_item_relations_related_item_id ON item_relations(related_item_id);

CREATE TABLE item_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    url VARCHAR(1000) NOT NULL,
    label VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_links_item_id ON item_links(item_id);

CREATE TABLE item_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    path VARCHAR(1000) NOT NULL,
    label VARCHAR(255),
    file_type file_type NOT NULL,
    calibre_book_id VARCHAR(100),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_files_item_id ON item_files(item_id);

CREATE TABLE item_trailers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    url VARCHAR(1000) NOT NULL,
    label VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_trailers_item_id ON item_trailers(item_id);

CREATE TABLE item_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    parent_item_id UUID REFERENCES items(id) ON DELETE CASCADE,
    group_type group_type NOT NULL,
    group_name VARCHAR(255) NOT NULL,
    number INTEGER,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_groups_item_id ON item_groups(item_id);
CREATE INDEX idx_item_groups_parent_item_id ON item_groups(parent_item_id);

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

CREATE INDEX idx_item_episodes_group_id ON item_episodes(group_id);

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

CREATE TABLE staff (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(100),
    name VARCHAR(255) NOT NULL,
    image_url VARCHAR(1000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_staff_external_id ON staff(external_id);

CREATE TABLE item_staff (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    staff_id UUID NOT NULL REFERENCES staff(id) ON DELETE CASCADE,
    role VARCHAR(100) NOT NULL,
    character_name VARCHAR(255)
);

CREATE INDEX idx_item_staff_item_id ON item_staff(item_id);
CREATE INDEX idx_item_staff_staff_id ON item_staff(staff_id);

CREATE TABLE cast_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(100),
    name VARCHAR(255) NOT NULL,
    image_url VARCHAR(1000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_cast_members_external_id ON cast_members(external_id);

CREATE TABLE item_cast (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    cast_id UUID NOT NULL REFERENCES cast_members(id) ON DELETE CASCADE,
    character_name VARCHAR(255)
);

CREATE INDEX idx_item_cast_item_id ON item_cast(item_id);
CREATE INDEX idx_item_cast_cast_id ON item_cast(cast_id);

CREATE TABLE api_credentials (
    provider api_provider PRIMARY KEY,
    api_key VARCHAR(500) NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ========================================
-- item_streaming_links migration (up)
-- ========================================

CREATE TYPE streaming_platform AS ENUM (
    'netflix', 'amazon_prime', 'disney_plus', 'dmm_tv', 'apple_tv'
);

CREATE TABLE item_streaming_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    platform streaming_platform NOT NULL,
    url VARCHAR(1000) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_item_streaming_links UNIQUE (item_id, platform)
);

CREATE INDEX idx_item_streaming_links_item_id ON item_streaming_links(item_id);

-- ========================================
-- item_images migration (up)
-- ========================================

CREATE TYPE image_kind AS ENUM ('cover', 'backdrop', 'screenshot', 'thumbnail', 'other');

CREATE TYPE image_source AS ENUM ('manual', 'annict', 'jikan', 'tmdb', 'rakuten', 'steam', 'ndl');

CREATE TABLE item_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    url VARCHAR(1000) NOT NULL,
    kind image_kind NOT NULL DEFAULT 'other',
    source image_source NOT NULL DEFAULT 'manual',
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_item_images UNIQUE (item_id, url)
);

CREATE INDEX idx_item_images_item_id ON item_images(item_id);

-- ========================================
-- citations migration (up)
-- ========================================

CREATE TYPE locator_type AS ENUM ('page', 'timestamp', 'location', 'chapter', 'none');

CREATE TABLE citations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    quote_text TEXT NOT NULL,
    note TEXT,
    locator_type locator_type NOT NULL DEFAULT 'none',
    page_number INTEGER,
    timestamp_seconds INTEGER,
    location_number INTEGER,
    chapter VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_citations_quote_text_not_empty CHECK (quote_text <> '')
);

CREATE INDEX idx_citations_item_id ON citations(item_id);

CREATE TRIGGER trg_citations_updated_at BEFORE UPDATE ON citations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

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
