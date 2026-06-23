-- ========================================
-- mediavault-backend 関連テーブル群マイグレーション (up)
-- ========================================
--
-- 関連設計: docs/design/mediavault-backend/database-schema.sql
-- 内容: タグ/カテゴリ/マイリスト、関連付け/リンク/ファイル/トレーラー、
--       グループ/エピソード、スタッフ、APIキー管理、updated_atトリガー

-- ========================================
-- タグ・カテゴリ・マイリスト
-- ========================================

-- tags
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE
);

-- item_tags（多対多）
CREATE TABLE item_tags (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);

-- categories
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE
);

-- item_categories（多対多）
CREATE TABLE item_categories (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, category_id)
);

-- mylists
CREATE TABLE mylists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- mylist_items（多対多）
CREATE TABLE mylist_items (
    mylist_id UUID NOT NULL REFERENCES mylists(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    PRIMARY KEY (mylist_id, item_id)
);

-- ========================================
-- 関連付け・リンク・ファイル・トレーラー
-- ========================================

-- item_relations
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

-- item_links
CREATE TABLE item_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    url VARCHAR(1000) NOT NULL,
    label VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_links_item_id ON item_links(item_id);

-- item_files
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

-- item_trailers
CREATE TABLE item_trailers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    url VARCHAR(1000) NOT NULL,
    label VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_item_trailers_item_id ON item_trailers(item_id);

-- ========================================
-- グループ・エピソード
-- ========================================

-- item_groups（シーズン/巻/章の汎用モデル、入れ子構造可）
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

-- item_episodes（season/chapter配下のみ使用）
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

-- EDGE-101: group_type=volumeへのepisode追加を防ぐトリガー
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

-- staff
CREATE TABLE staff (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(100),
    name VARCHAR(255) NOT NULL,
    image_url VARCHAR(1000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_staff_external_id ON staff(external_id);

-- item_staff（多対多、role付き）
CREATE TABLE item_staff (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    staff_id UUID NOT NULL REFERENCES staff(id) ON DELETE CASCADE,
    role VARCHAR(100) NOT NULL,
    character_name VARCHAR(255)
);

CREATE INDEX idx_item_staff_item_id ON item_staff(item_id);
CREATE INDEX idx_item_staff_staff_id ON item_staff(staff_id);

-- ========================================
-- 外部APIキー管理
-- ========================================

-- api_credentials
CREATE TABLE api_credentials (
    provider api_provider PRIMARY KEY,
    api_key VARCHAR(500) NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ========================================
-- updated_at 自動更新トリガー（共通）
-- ========================================
-- 備考: items.updated_atトリガーはTASK-0003（init_schema）側で未作成のため、
--       本マイグレーションでまとめて作成する。

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
