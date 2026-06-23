# TASK-0012 要件定義書: PATCH /items/:id 部分更新実装

**作成日**: 2026-06-23
**関連タスク**: [TASK-0012](../../tasks/mediavault-backend/TASK-0012.md)
**関連ノート**: [note.md](note.md)
**親要件**: [requirements.md](requirements.md) REQ-001 / [acceptance-criteria.md](acceptance-criteria.md) TC-001-02・TC-001-E02・TC-001-B01

**【信頼性レベル凡例】**:
- 🔵 **青信号**: タスク仕様・既存コード（note.md記載）から確実な要件
- 🟡 **黄信号**: タスク仕様から妥当な推測による要件
- 🔴 **赤信号**: 推測による要件（本ドキュメントには無し）

---

## 1. 概要

`PATCH /items/:id` は、既存 `UpdateItemRequest`（`backend/mediavault-api/src/models/item.rs` L104-119）で表現された任意のフィールド集合のみを対象アイテムに適用し、更新後のアイテムを返すエンドポイントである。`media_type`・`source`・`external_id` はDTOに存在せず更新対象にならない。`updated_at` はDBトリガー `trg_items_updated_at` が自動更新するため、アプリ側でSET句に含めてはならない。

## 2. 機能要件（EARS記法）

### 通常要件

- REQ-0012-01: システムは `PATCH /items/:id` リクエストを受理し、`UpdateItemRequest` のうち値が `Some` であるフィールドのみを対象アイテムのカラムに適用しなければならない 🔵 *タスク完了条件・TC-001-02より*
- REQ-0012-02: システムは更新後のアイテムをHTTP 200で返さなければならない 🔵 *タスク完了条件より*
- REQ-0012-03: システムは動的SET句の構築に `sqlx::QueryBuilder` を用い、`item_repository.rs` の既存パターン（`push_item_filters` のカンマ区切り方式）を踏襲しなければならない 🔵 *note.md記載の実装方針より*
- REQ-0012-04: システムは `updated_at` をUPDATE文のSET句に含めてはならず、DBトリガー `trg_items_updated_at` による自動更新に委ねなければならない 🔵 *タスク完了条件・note.mdより*
- REQ-0012-05: システムは `media_type`・`source`・`external_id` を更新対象として受理してはならない（`UpdateItemRequest` に該当フィールドが存在しないことで保証する） 🔵 *タスク注意事項より*

### 条件付き要件

- REQ-0012-101: `UpdateItemRequest` の全フィールドが `None` の場合、システはUPDATE文を実行せず、現在のアイテムの状態をそのまま200で返さなければならない 🔵 *タスク完了条件・item_repository.rsの方針（note.md L36）より*
- REQ-0012-102: `title` フィールドが `Some("")`（空文字）の場合、システムはUPDATEを実行せず `VALIDATION_ERROR`（400）を返さなければならない 🔵 *タスク完了条件・TC-001-B01より*
- REQ-0012-103: パスパラメータの `:id` が既存の `parse_item_id` でパース可能なUUID形式でない場合、システムは既存のID検証エラー処理に従わなければならない（GET /items/:id と同様の挙動） 🟡 *既存`parse_item_id`関数の再利用より妥当な推測*

### 状態要件

- REQ-0012-201: 対象UUIDに一致するアイテムが存在しない場合、システムは `ITEM_NOT_FOUND`（404）を返さなければならない 🔵 *タスク完了条件・TC-001-E02より*
- REQ-0012-202: UPDATE実行後に影響行数が0件（＝対象が存在しなかった、または同時に削除された）の場合も、システムは `ITEM_NOT_FOUND`（404）として扱わなければならない 🔵 *note.md記載の`RETURNING`+`fetch_optional`方式より*

### 制約要件

- REQ-0012-401: システムは `title` 以外のフィールド（`rating`・`is_favorite`・`status`等）の値の妥当性を本タスクの範囲としてバリデーションしてはならない（タスク範囲外） 🔵 *タスク注意事項「titleを空文字に更新しようとした場合のみVALIDATION_ERROR」より*
- REQ-0012-402: システムはDBエラー発生時、クライアントへSQLやDB内部情報を含むメッセージを返してはならず、既存の `db_error()` ヘルパーを経由して `InternalError`（500）に正規化しなければならない 🔵 *note.md記載のdb_error()方針より*

## 3. 非機能要件

- NFR-0012-01: 本エンドポイントの実装は既存のエラーレスポンス規約（`ApiError`/`ApiErrorCode`/`ApiOk<T>`）に準拠し、独自のレスポンス形式を新設してはならない 🔵 *note.mdのエラーハンドリング規約より*
- NFR-0012-02: 動的SQL生成部分はSQLインジェクションを防止するため、`push_bind` によるバインドパラメータ方式のみを用い、文字列結合によるSQL構築を行ってはならない 🔵 *既存`push_item_filters`実装方針の踏襲より*
- NFR-0012-03: 単一テーブル（`items`）のみを更新するため、トランザクション（`pool.begin()`）は必須としない（`create_item`のような複数テーブル更新ではないため） 🟡 *note.md記載のトランザクション必要性に関する考察より*

## 4. Edgeケース

### エラー処理

- EDGE-0012-01: リクエストボディが空オブジェクト `{}` の場合、全フィールドが `None` として扱われ、REQ-0012-101（無更新で現状返却）が適用されなければならない 🔵 *「全フィールドがNoneの場合」の具体例として明示*
- EDGE-0012-02: `title` 以外のフィールドのみを含む更新で、`title` を含まない（`None`）場合、既存の `title` 値は変更されず維持されなければならない 🔵 *部分更新の基本動作（TC-001-02）より*
- EDGE-0012-03: 存在しないUUID形式が `:id` に渡された場合と、UUID形式として有効だがレコードが存在しない場合の両方で、最終的にクライアントへ返るエラーは整合した形式（`ApiError`構造）でなければならない 🟡 *エラーレスポンス一貫性の観点から妥当な推測*

### 境界値

- EDGE-0012-101: `title` が `Some("")`（長さ0の文字列）の場合のみVALIDATION_ERRORとし、`title` が `None`（更新対象外）の場合はバリデーション自体を実行してはならない 🔵 *タスク仕様「titleを空文字に更新しようとした場合」の厳密な解釈より*
- EDGE-0012-102: `title` が空白のみの文字列（例: `" "`）の場合の扱いは、本タスクの完了条件に明記がないため、既存の `CreateItemRequest` 用 `validate_title` 相当ロジックがあれば流用し、なければ完全な空文字（長さ0）のみを対象とする 🟡 *タスク範囲の曖昧性に関する妥当な推測（実装時に既存validate_title実装を確認して決定）*

## 5. Given/When/Then シナリオ

### シナリオ1: 部分更新の正常動作（TC-001-02） 🔵

- **Given**: 既存アイテムが1件存在する
- **When**: `PATCH /items/:id` に `{ "rating": 4.5, "is_favorite": true }` を送信する
- **Then**: 200が返り、`rating=4.5`・`is_favorite=true` に更新され、他フィールドは変化せず、`updated_at` はDBトリガーにより更新される

### シナリオ2: 全フィールドNoneでの無更新（EDGE-0012-01） 🔵

- **Given**: 既存アイテムが1件存在する
- **When**: `PATCH /items/:id` に `{}` を送信する
- **Then**: 200が返り、UPDATE文は実行されず、アイテムは現在の状態のまま返却される

### シナリオ3: 存在しないアイテムへの更新（TC-001-E02） 🔵

- **Given**: 指定したUUIDに対応するアイテムが存在しない
- **When**: `PATCH /items/:id` を呼び出す
- **Then**: 404 `ITEM_NOT_FOUND` が返る

### シナリオ4: titleを空文字に更新（TC-001-B01） 🔵

- **Given**: 既存アイテムが1件存在する
- **When**: `PATCH /items/:id` に `{ "title": "" }` を送信する
- **Then**: 400 `VALIDATION_ERROR` が返り、UPDATEは実行されず、DBの状態は変化しない

### シナリオ5: 更新不可フィールドの拒否（REQ-0012-05） 🔵

- **Given**: 既存アイテムが1件存在する
- **When**: `PATCH /items/:id` のリクエストボディに `media_type`・`source`・`external_id` を含めて送信する
- **Then**: これらのフィールドは `UpdateItemRequest` にデシリアライズされず（serdeにより無視される）、対応するカラムは更新されない

## 6. 信頼性レベルサマリー

| カテゴリ | 🔵 | 🟡 | 🔴 | 合計 |
|---|---|---|---|---|
| 機能要件（通常） | 5 | 0 | 0 | 5 |
| 機能要件（条件付き） | 2 | 1 | 0 | 3 |
| 機能要件（状態） | 2 | 0 | 0 | 2 |
| 機能要件（制約） | 2 | 0 | 0 | 2 |
| 非機能要件 | 2 | 1 | 0 | 3 |
| Edgeケース | 4 | 2 | 0 | 6 |
| シナリオ | 5 | 0 | 0 | 5 |

**全体評価**: 高品質（赤信号なし。黄信号はUUIDパース時のエラー詳細、トランザクション要否、空白文字titleの扱いなど実装時に既存コードを確認して確定すべき細部）

---

## 次フェーズへの引き渡し事項

- `tdd-testcases` フェーズでは、本ドキュメントのシナリオ1〜5を基本テストケースとし、EDGE-0012-101/102（空文字判定の境界）を追加のテストケースとして洗い出すこと。
- 既存 `validate_title`（`CreateItemRequest`用）の実装箇所（`models/item.rs`）を確認し、`UpdateItemRequest` 用に流用可能か、新規 `parse_update_item_request` 相当の関数が必要かを `tdd-red` 着手前に確定すること。
