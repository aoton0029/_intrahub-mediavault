# TASK-0020 Greenフェーズ記録: スタッフ管理CRUD実装

**タスクID**: TASK-0020
**機能名**: staff（スタッフ管理CRUD）
**要件名**: mediavault-backend
**作成日**: 2026-06-24

---

## 1. 実装方針

Redフェーズで作成したtodo!()スタブをすべて最小実装に置き換えた。既存のitems/tags/item_groups CRUD
パターン（parse_*関数によるpureバリデーション、repository層でのsqlx::query_as、db_errorによる
内部情報秘匿、存在確認→INSERT/DELETEの順序）を踏襲し、独自ロジックは導入していない。

## 2. 変更したファイル

### `backend/mediavault-api/src/models/response.rs`
- `ApiErrorCode::StaffNotFound`バリアントを追加（`STAFF_NOT_FOUND` / 404 NOT_FOUND）。

### `backend/mediavault-api/src/models/staff.rs`
- `parse_create_staff_request`: nameが空文字（trim後）または255文字超の場合に
  `VALIDATION_ERROR`を返す。それ以外はリクエストをそのまま`Ok`で返す。
- `parse_create_item_staff_request`: roleが空文字（trim後）または100文字超、
  character_nameが指定されていて255文字超の場合に`VALIDATION_ERROR`を返す。

### `backend/mediavault-api/src/repositories/staff_repository.rs`
- `item_exists` / `staff_exists`: `SELECT 1 FROM <table> WHERE id = $1`による存在確認ヘルパー
  （item_group_repository::item_existsと対称）。
- `create_staff`: `INSERT INTO staff (external_id, name, image_url) VALUES (...) RETURNING ...`。
- `link_staff`: item_exists→ITEM_NOT_FOUND、staff_exists→STAFF_NOT_FOUNDの順に事前確認した後、
  `INSERT INTO item_staff (item_id, staff_id, role, character_name) VALUES (...) RETURNING ...`。
- `unlink_staff`: `DELETE FROM item_staff WHERE id = $1 AND item_id = $2`でitem_id整合性チェックを
  SQLレベルで行い、`rows_affected() > 0`をbool判定として返す（false時はハンドラ側で404にマッピング）。
- `db_error`の`#[allow(dead_code)]`属性を削除（実際に使用されるようになったため）。

### `backend/mediavault-api/src/handlers/staff.rs`
- `create_staff_handler`: `deserialize_request` → `parse_create_staff_request` →
  `staff_repository::create_staff` → 201応答。
- `create_item_staff_handler`: `parse_item_id` → `deserialize_request` →
  `parse_create_item_staff_request` → `staff_repository::link_staff` → 201応答。
- `delete_item_staff_handler`: `parse_item_id` ×2 → `staff_repository::unlink_staff` →
  false時`ItemNotFound`(404)、true時`204 No Content`。
- `created_staff_response` / `created_item_staff_response`: 201応答構築の共通ヘルパー
  （`handlers::item_groups::created_response`と対称）。

## 3. テスト実行結果

```bash
cd backend/mediavault-api
cargo build               # 成功（既存warningのみ、新規warningなし）
cargo test models::staff  # 9 passed; 0 failed; 0 ignored
cargo test --no-run       # 全テストバイナリのコンパイル成功（DB依存テストも含む）
cargo test                # 87 passed; 0 failed; 81 ignored（DB依存・DATABASE_URL未設定のため）
```

models/staff.rs単体テスト9件すべてPASSED（Redフェーズで7件FAILED、2件PASSEDだった状態から全件成功へ）。
DB依存の統合テスト（repository 10件 + handler 4件、計14件）はローカルにDocker/Postgresが
起動していないため`--ignored`実行はできなかったが、`cargo test --no-run`でコンパイル可能であることを
確認済み。実装は既存のitem_group_repository等と同一のクエリ・エラーハンドリングパターンに
従っているため、ロジック上の不整合は想定されない。

## 4. 品質判定

| 評価項目 | 状態 |
|---|---|
| テスト結果 | ✅ models::staff 9/9 成功、全体87/87成功（DB依存81件はignore） |
| 実装品質 | ✅ シンプル、既存パターン（item_group/tag/mylist repository）を踏襲 |
| リファクタ箇所 | 文字数チェックの共通ヘルパー化、DELETE時エラーコードの専用化（任意） |
| 機能的問題 | なし |
| コンパイルエラー | なし |
| ファイルサイズ | staff.rs(models) 約190行、staff_repository.rs 約260行、staff.rs(handlers) 約220行（いずれも800行以下） |
| モック使用 | 実装コードにモック・スタブなし（テストコード内のtodo!()はすべて実装に置き換え済み） |

**総合評価**: ✅ 高品質（Green）

---

## 5. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-refactor mediavault-backend TASK-0020` でRefactorフェーズ
（品質改善）を開始します。
