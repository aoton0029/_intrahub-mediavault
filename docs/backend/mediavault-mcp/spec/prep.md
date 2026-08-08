# mediavault-mcp 準備タスク（ユーザー作業）

> **仕様**: [requirements.md](requirements.md)
> **生成日**: 2026-08-07

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・既存実装調査・ユーザヒアリングで明確に必要と判明したタスク
- 🟡 **黄信号**: 要件定義書・設計文書から妥当に推測されるタスク
- 🔴 **赤信号**: 推測による予防的タスク

## 必須（実装開始前に完了が必要）

以下が完了していないと、対応する MCP ツールを MVP の要件どおりに実装できません。

### mediavault-api の先行改修

- [ ] **PREP-01: `relation_type` ENUM の拡張** 🔵 *ヒアリング Q1・`migrations/20260623000001_init_schema.up.sql:21` より*
  - 現行: `CREATE TYPE relation_type AS ENUM ('reference', 'dlc');`
  - 追加する値: `adaptation` / `sequel` / `prequel` / `spinoff`
  - `ALTER TYPE relation_type ADD VALUE ...` のマイグレーションを追加する（PostgreSQLでは既存値の削除ができないため追加のみ）
  - `backend/mediavault-api/src/models/item_relation.rs` の `RelationType` にも同じ値を追加する
  - あわせて `item_id` → `related_item_id` の向きが種別ごとに何を意味するかを [item-relations.md](../../mediavault-api/item-relations.md) に定義する
  - 関連要件: REQ-071, REQ-072

- [ ] **PREP-02: `GET /items` の別名・原題検索対応** 🔵 *ヒアリング Q2・`docs/backend/mediavault-api/items.md` より*
  - 現行: `title` クエリは `items.title` の部分一致のみ
  - 拡張: `original_title` と `details->'alternative_titles'` も検索対象に含める
  - JSONB 配列内の部分一致になるためインデックス方針もあわせて検討する
  - 関連要件: REQ-010, REQ-011

- [ ] **PREP-03: 検索結果の該当件数の返却** 🔵 *ヒアリング Q3・`items.md`「件数の総数（total）は返さない」より*
  - 現行の keyset ページネーションは COUNT を意図的に避けている
  - `search_library` が該当件数を返すため、件数取得の手段を用意する（`GET /items` にオプトインの `include_total` を足す、または PREP-04 の集計APIに条件付き件数を持たせる）
  - 既存フロントエンドの性能を損なわないよう、既定では COUNT を実行しない設計にする
  - 関連要件: REQ-013

- [ ] **PREP-04: `GET /api/v1/collection/overview` の新設** 🔵 *ヒアリング Q3より*
  - 返す内容: media_type別件数、status別件数、お気に入り件数、最近追加された Item、最近更新された Item
  - `GET /items/counts-by-media-type` は既存のため、それを包含または併存させるか設計時に決める
  - 関連要件: REQ-090

### 環境・シークレット

- [ ] **PREP-05: `MCP_AUTH_TOKEN` の生成と配布** 🔵 *ヒアリング（認証方式）・[tech-stack.md](../tech-stack.md) より*
  - 十分な長さのランダム文字列を生成する（例: `openssl rand -base64 48`）
  - ミニPC側の `.env` に設定し、Claude Code などの MCP クライアント設定にも同じ値を登録する
  - リポジトリにコミットしない
  - 関連要件: REQ-115, REQ-122, NFR-101

## 推奨（実装中に用意できればOK）

- [ ] **PREP-06: 外部プロバイダAPIキーの設定確認** 🔵 *`GET /items/search` の 422 `API_KEY_NOT_CONFIGURED` より*
  - `search_external_catalog` / `import_external_item` を実際に動かすには、対象メディア種別のプロバイダキー（Jikan / Annict / TMDb / 楽天 / Steam / NDL 等）が MediaVault に設定済みである必要がある
  - `GET /api/v1/settings/api-keys` で設定状況を確認できる
  - 必要になるフェーズ: Phase 3（書き込み系ツール）
  - 関連要件: REQ-030, REQ-033, REQ-117

- [ ] **PREP-07: docker-compose への `mediavault-mcp` サービス追加** 🔵 *[tech-stack.md](../tech-stack.md) より*
  - 別コンテナ構成のため、`docker-compose.yml` にサービス定義と環境変数、api への依存を追加する
  - 公開ポートとリバースプロキシ経路は [06_deployment-routing.md](../../../basic-design/06_deployment-routing.md) と整合させる
  - 必要になるフェーズ: Phase 1（サーバー基盤）
  - 関連要件: REQ-001

- [ ] **PREP-08: MCPクライアント（Claude Code）側の接続設定** 🟡 *Streamable HTTP + Bearer 認証構成から妥当に推測*
  - Streamable HTTP エンドポイントの URL と `Authorization: Bearer` ヘッダーを MCP クライアントへ設定する
  - 動作確認は Phase 1 完了時点の `health` ツールで行える
  - 必要になるフェーズ: Phase 1
  - 関連要件: REQ-001, NFR-101

## 確認事項（判断が必要）

- [x] **PREP-09: `limit` 上限超過時の挙動（丸め or 拒否）** 🟡 *EDGE-101。ヒアリングQ7の派生*
  - **設計フェーズで「拒否」に決定**（[design-interview.md](../design/design-interview.md) 残課題）。丸めるとAIに上限が伝わらず同じ誤りを繰り返すため、`MCP_INVALID_ARGUMENT` を返す
  - 異論があればご指摘ください
  - 関連要件: REQ-143, EDGE-101

- [ ] **PREP-10: `rating` の許容範囲の明文化** 🟡 *EDGE-104。API仕様書に記載がない*
  - `update_consumption` の入力検証を MCP 側でも行うか、API のバリデーションに委ねるかの判断に必要
  - 実データ上は 8.5 のような小数が使われている
  - 関連要件: REQ-050, EDGE-104

- [ ] **PREP-11: リバースプロキシ経路での MCP 公開範囲** 🟡 *PRD §10「内部経路と外部経路の双方に適用」より*
  - MCP を LAN 内のみに閉じるか、リバースプロキシ経由で外部にも出すかで、Bearer 認証以外の追加防御（IP制限等）の要否が変わる
  - 関連要件: NFR-101

---

## サマリー

| 優先度 | 件数 | 🔵 | 🟡 | 🔴 |
|--------|------|-----|-----|-----|
| 必須 | 5 | 5 | 0 | 0 |
| 推奨 | 3 | 2 | 1 | 0 |
| 確認事項 | 3 | 0 | 3 | 0 |
| **合計** | **11** | **7** | **4** | **0** |

**最大のブロッカー**: PREP-01 ～ PREP-04 はいずれも mediavault-api 側の改修であり、MCP の実装とは独立して先行着手できます。特に PREP-01（ENUM拡張）と PREP-02（別名検索）は影響範囲が小さく、早期に片付けることを推奨します。

## 関連文書

- **要件定義書**: [requirements.md](requirements.md)
- **ヒアリング記録**: [interview-record.md](interview-record.md)
- **技術スタック**: [tech-stack.md](../tech-stack.md)
