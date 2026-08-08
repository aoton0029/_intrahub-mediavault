# TASK-0001 コンテキストノート

**作成日**: 2026-08-07
**タスク**: [TASK-0001: relation_type ENUM の6種別拡張](../../tasks/TASK-0001.md)
**対象**: MediaVault-api（Phase 0 先行改修）

## 技術スタック

| 項目 | 内容 |
|---|---|
| 言語 | Rust（edition 2024）|
| Webフレームワーク | axum 0.8.9 |
| DBアクセス | sqlx 0.8（`postgres`, `runtime-tokio`, `macros`, `chrono`, `uuid`, `migrate`）|
| DB | PostgreSQL（Docker）|
| マイグレーション | sqlx-cli（`sqlx migrate`）|
| テスト | `cargo test` + wiremock 0.6 + tempfile |
| Lint | `cargo clippy --all-targets` / `cargo fmt` |

出典: [docs/backend/tech-stack.md](../../../tech-stack.md)、`backend/mediavault-api/Cargo.toml`

## 開発ルール

- [intrahub-mediavault/CLAUDE.md](../../../../../CLAUDE.md): 曖昧・不慣れ・多段・アーキテクチャに影響する実装前に `unknowns-field-guide` スキルを読む
- 既存コードは信頼性レベル（🔵🟡🔴）をdocコメントに記載する慣例がある（`models/item_relation.rs` 参照）
- テストは `#[cfg(test)] mod tests` を同一ファイル内に置く慣例

## 関連実装（調査済み）

### 現行の ENUM 定義

`backend/mediavault-api/migrations/20260623000001_init_schema.up.sql`

```sql
-- L21
CREATE TYPE relation_type AS ENUM ('reference', 'dlc');

-- L99, L102
    relation_type relation_type NOT NULL,
    CONSTRAINT uq_item_relations UNIQUE (item_id, related_item_id, relation_type)
```

`down.sql` L37 に `DROP TYPE IF EXISTS relation_type;`

### 現行の Rust モデル

`backend/mediavault-api/src/models/item_relation.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "relation_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RelationType {
    Reference,
    Dlc,
}
```

- `ItemRelation`（id / item_id / related_item_id / relation_type / created_at）
- `CreateItemRelationRequest`（3フィールドすべて必須）
- `validate_not_self_reference()` — **自己参照を400で拒否する実装が既にある**
- 既存テスト: 正常デシリアライズ / 不正値の失敗 / 自己参照の検出

### 関連する既存ファイル

| ファイル | 関係 |
|---|---|
| `src/handlers/item_relations.rs` | create / list / delete ハンドラ |
| `src/models/backup.rs:137` | `relation_type: RelationType` を含む。バックアップ往復の確認が必要 |
| `src/models/response.rs` | `ApiErrorCode::DuplicateRelation` = 409 `DUPLICATE_RELATION` |
| `docs/backend/mediavault-api/item-relations.md` | API仕様書。向きの定義を追記する対象 |

## 設計文書

- **要件**: [requirements.md](../../spec/requirements.md) REQ-071, REQ-072
- **型定義**: [interfaces.rs](../../design/interfaces.rs) の `RelationType`（6値）
- **ツール仕様**: [mcp-tools.md](../../design/mcp-tools.md) 8. relate_items の関係種別表
- **PRD決定**: [PRD.md](../../../PRD.md) §15.1「関係種別」— 固定一覧・利用者定義は将来候補

## 注意事項

### PostgreSQL の ENUM 制約

- **値の削除ができない**。`ALTER TYPE ... ADD VALUE` で追加のみ
- PostgreSQL 12 未満では `ALTER TYPE ... ADD VALUE` をトランザクション内で実行できない。sqlx のマイグレーション実行方式（既定でトランザクション内）との相性を確認する必要がある
- down マイグレーションは型の再作成（新型作成 → カラム変換 → 旧型削除）が必要。追加値を使ったデータがあると失敗する

### 既存挙動を壊さないこと

- `reference` / `dlc` を使った既存データとテストが引き続き動作すること
- `uq_item_relations UNIQUE (item_id, related_item_id, relation_type)` の3つ組一意制約は維持
- `validate_not_self_reference()` は変更しない

### 調査で判明した事項（タスクファイル作成時点で未確認だったもの）

| 事項 | 内容 | 影響 |
|---|---|---|
| 自己参照の扱い | api 側に `validate_not_self_reference()` とDB制約 `chk_item_relations_not_self` の二重防御が**既に存在する** | TASK-0021 の EDGE-005「api側に制約があるか未確認」が解消。MCP側の事前チェックは冗長だが、api呼び出しを1回節約できるため残してよい |
| `GET /items/{id}/relations` の返却範囲 | 仕様書に「指定アイテムを**起点**とする関連付けを一覧取得」とある | TASK-0014 の `RelationDirection::Incoming` が実際には現れない可能性。**TASK-0014 着手前に要確認** |

### 下流への影響

本タスクの完了は [TASK-0021: relate_items ツールの実装](../../tasks/TASK-0021.md) のブロッカーである。MVP の完了条件（PRD §7.1）に直結する。
