# mediavault-mcp コンテキストノート

**作成日**: 2026-08-07
**作業規模**: フル機能開発

## 技術スタック

[docs/backend/mediavault-mcp/tech-stack.md](../tech-stack.md) 参照。要点:

- Rust edition 2024 / `rmcp` 3.1系 / axum 0.8 / tokio
- トランスポート: Streamable HTTP のみ（stdio は第2段階）
- 認証: 静的Bearerトークン（`MCP_AUTH_TOKEN`）をMCPプロセス内で定数時間比較
- MediaVault-api クライアントは `mediavault-mcp/src/api/` に reqwest 0.12 で自前実装
- テスト: `cargo test` + `wiremock` 0.6（実API結合テストは非対象）
- デプロイ: `backend/` workspace に別クレート追加、独立Dockerイメージ

## 開発ルール

- [intrahub-mediavault/CLAUDE.md](../../../../CLAUDE.md): 曖昧・不慣れ・多段・アーキテクチャに影響する実装前に `unknowns-field-guide` スキルを読む
- `docs/rule/` は存在しない

## 関連実装・設計文書

| 文書 | 内容 |
|---|---|
| [PRD.md](../PRD.md) | MediaVault-mcp PRD（§15に決定事項） |
| [docs/backend/mediavault-api/](../../mediavault-api/) | 既存REST API仕様（items, tags, categories, mylists, item-relations, item-links, item-streaming-links, item-trailers, item-files, extraction ほか） |
| `backend/mediavault-api/src/routes/mod.rs` | 実装済みエンドポイント一覧 |
| `backend/mediavault-api/migrations/20260623000001_init_schema.up.sql` | `relation_type` / `item_status` の PostgreSQL ENUM 定義 |

## 既存実装の調査結果（2026-08-07 時点）

### 実装済みで MCP から利用できるもの

- `GET /api/v1/items`（`media_type` / `tag_id` / `category_id` / `is_favorite` / `status` / `title`部分一致 / keysetページネーション、`limit` 既定20・最大100）
- `GET /api/v1/items/counts-by-media-type`（media_type別件数 + total）
- `GET /api/v1/items/search`（外部プロバイダ横断検索、`media_type` + `q` 必須）
- `POST /api/v1/items/import`（`external_id` + `provider`、重複時 409 `ITEM_ALREADY_IMPORTED`）
- `POST /api/v1/items` / `PATCH /api/v1/items/{id}` / `PATCH /api/v1/items/{id}/status`
- `GET /api/v1/items/{id}`（`ItemDetail` = Item + detail + タグ + カテゴリ + 配信リンク + 画像 + Calibre連携）
- タグ / カテゴリ / マイリスト の一覧・作成・付与・解除
- `item-relations` / `item-links` / `item-streaming-links` / `item-trailers` / `item-files` / `item-groups` / `staff` / `cast` / `item-episodes` / `item-images`
- `GET /api/v1/health`、`GET /api/v1/settings/api-keys`
- `/api/v1/internal/*`（`INTERNAL_API_KEY` 認証、items CRUD・files・groups）

### 未実装・制約（PRD §8 の要求との差分）

| 項目 | 実態 |
|---|---|
| `relation_type` | PostgreSQL ENUM `('reference','dlc')` のみ。拡張にはマイグレーションが必要 |
| 別名・原題検索 | `GET /items` の `title` は本題の部分一致のみ |
| 検索結果の総件数 | `GET /items` は `total` を返さない（COUNT回避のため意図的） |
| `GET /items/{id}/context` | 未実装 |
| `GET /collection/overview` | 未実装（`counts-by-media-type` のみ） |
| `GET /items/{id}/text` | 実装済み。抽出済み本文をチャンク単位で返す |
| 抽出リソース | 公開API 3本（依頼・状態確認・キャンセル）と worker 内部APIを実装済み |
| `item_status` ENUM | `not_started` / `in_progress` / `completed` の3値 |
| `streaming_links.platform` | `netflix` / `amazon_prime` / `disney_plus` / `dmm_tv` / `apple_tv` の5種固定 |
| 公開API の認証 | `/api/v1/*` は認証なし（単一ユーザー前提） |

## 注意事項

- MediaVault-api の公開APIは無認証のため、MCP の Bearer 認証が AIエージェント経路における唯一の関門になる。
- MVP で `search_library` / `relate_items` / `collection_overview` を PRD 通りに提供するには、mediavault-api 側の先行改修（3件）が必要（[prep.md](prep.md)）。
- 第2段階（`get_item_text` / jobs 系）は本要件定義のスコープ外とし、要件IDのみ将来枠として確保する。
