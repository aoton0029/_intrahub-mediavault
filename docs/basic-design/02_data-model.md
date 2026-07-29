# MediaVault 基本設計 — データモデル概要

← [00_overview.md](00_overview.md)

本ページは `items` を中心とした概念モデル・テーブル間の関係性の全体像を示す。フィールド定義など詳細は [../backend/mediavault-api/data-model.md](../backend/mediavault-api/data-model.md) および各リソースAPIドキュメント（[items.md](../backend/mediavault-api/items.md) 等）を参照。

## ER概要

```mermaid
erDiagram
    ITEMS ||--o{ ITEM_FILES : has
    ITEMS ||--o{ ITEM_LINKS : has
    ITEMS ||--o{ KNOWLEDGE : derives
    ITEMS ||--o{ JOBS : targets
    ITEMS }o--o{ TAGS : tagged_with
    ITEMS }o--o{ CATEGORIES : classified_as
    ITEMS }o--o{ STAFF : credits
    ITEMS }o--o{ MYLISTS : favorited_in
    ITEMS ||--o{ ITEM_RELATIONS : relates_to
```

## テーブル分類と役割

| テーブル | 分類 | 役割 |
|---|---|---|
| `items` | コア | メタデータ本体。`media_type`（video/photo/book/PDF/manga等）で分類 |
| `tags` / `categories` / `staff` / `mylists` | コア | 分類・関連エンティティ・お気に入り |
| `item_files` | コア | `/data` 内の相対パスとファイル実体を紐付ける |
| `item_links` | 拡張 | 外部リンクの統一表現。`kind` = `jellyfin` / `calibre` / `url` |
| `knowledge` | 拡張 | itemから派生するwikiページ/抽出トピック・エンティティ/embedding参照（KnowledgeHub側エージェントが書き込む） |
| `jobs` | 拡張 | workerのジョブキュー（種別/状態/対象item/リトライ） |

正準キーは `items.id`。1つの `item` がファイル・外部リンク・関連ナレッジを束ねる（[00_overview.md](00_overview.md) の設計原則2）。

## 設計上の注意（旧設計からの移行）

- 旧 `item_files.calibre_book_id` のようなスキーマ残骸や、旧 `データ連携設計.md`（UUIDシャーディング、`_by-title/` シンボリックリンクビュー）は `item_links` に統合済み。新規実装は `item_links` ベースで行う。
- インフラ設計側 `Jellyfin/MediaVault連携プラグイン設計.md` は旧 `MediaVault-backend`/`MediaVault-frontend` 2分割構成を前提にした古い設計であり、`item_links` ベースへの書き直しが必要な旨が明記されている。新規のJellyfin連携設計は本データモデル（`item_links`）を前提とすること。

## 関連ドキュメント

- [../backend/mediavault-api/data-model.md](../backend/mediavault-api/data-model.md)（フィールド定義）
- [../backend/mediavault-api/items.md](../backend/mediavault-api/items.md)、[item-links.md](../backend/mediavault-api/item-links.md)、[item-files.md](../backend/mediavault-api/item-files.md) 等（各リソースAPI）
- [03_api-design.md](03_api-design.md)
