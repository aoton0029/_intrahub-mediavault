# TASK-0020 Redフェーズ記録: スタッフ管理CRUD実装

**タスクID**: TASK-0020
**機能名**: staff（スタッフ管理CRUD）
**要件名**: mediavault-backend
**作成日**: 2026-06-24

---

## 1. 作成したファイル

### 実装スタブ（todo!()で未実装を明示）
- `backend/mediavault-api/src/models/staff.rs`: `Staff`, `ItemStaff`, `CreateStaffRequest`, `CreateItemStaffRequest`, `parse_create_staff_request()`（todo!()）, `parse_create_item_staff_request()`（todo!()）
- `backend/mediavault-api/src/repositories/staff_repository.rs`: `create_staff()`, `link_staff()`, `unlink_staff()`（いずれもtodo!()）
- `backend/mediavault-api/src/handlers/staff.rs`: `create_staff_handler()`, `create_item_staff_handler()`, `delete_item_staff_handler()`（いずれもtodo!()）

### モジュール登録・ルーティング追加
- `backend/mediavault-api/src/models/mod.rs`: `pub mod staff;` 追加
- `backend/mediavault-api/src/handlers/mod.rs`: `pub mod staff;` 追加
- `backend/mediavault-api/src/repositories/mod.rs`: `pub mod staff_repository;` 追加
- `backend/mediavault-api/src/routes/mod.rs`: `POST /staff`, `POST /items/:id/staff`, `DELETE /items/:id/staff/:item_staff_id` ルート追加

---

## 2. 作成したテストケース一覧

### models/staff.rs（単体テスト、DB不要） — 9件
| テストID | テスト名 | 信頼性 | 結果 |
|---|---|---|---|
| TC-N-06 | `parse_create_staff_request_accepts_valid_name` | 🔵 | FAILED（todo!()） |
| TC-E-01 | `parse_create_staff_request_rejects_empty_name` | 🔵 | FAILED（todo!()） |
| TC-E-06 | `parse_create_item_staff_request_rejects_empty_role` | 🔵 | FAILED（todo!()） |
| TC-B-02(a) | `parse_create_item_staff_request_accepts_role_at_max_length` | 🔵 | FAILED（todo!()） |
| TC-B-02(b) | `parse_create_item_staff_request_rejects_role_exceeding_max_length` | 🔵 | FAILED（todo!()） |
| TC-B-03(a) | `parse_create_item_staff_request_accepts_character_name_at_max_length` | 🔵 | FAILED（todo!()） |
| TC-B-03(b) | `parse_create_item_staff_request_rejects_character_name_exceeding_max_length` | 🔵 | FAILED（todo!()） |
| TC-N-02 | `create_staff_request_deserializes_all_fields` | 🔵 | PASSED（serdeのみ） |
| TC-E-07 | `create_item_staff_request_with_invalid_uuid_fails_deserialization` | 🟡 | PASSED（serdeのみ） |

### repositories/staff_repository.rs（統合テスト、DATABASE_URL必要・`#[ignore]`） — 9件
| テストID | テスト名 | 信頼性 |
|---|---|---|
| TC-N-01 | `create_staff_with_required_fields_only_returns_staff_with_null_optionals` | 🔵 |
| TC-N-02 | `create_staff_with_all_fields_persists_optional_fields` | 🔵 |
| TC-N-03 | `link_staff_without_character_name_creates_item_staff_record` | 🔵 |
| TC-N-04 | `link_staff_with_character_name_persists_character_name` | 🟡 |
| TC-E-02 | `link_staff_with_nonexistent_staff_id_returns_staff_not_found` | 🔵 |
| TC-E-03 | `link_staff_with_nonexistent_item_id_returns_item_not_found` | 🟡 |
| TC-N-05 | `unlink_staff_with_existing_record_returns_true` | 🟡 |
| TC-E-04 | `unlink_staff_with_nonexistent_item_staff_id_returns_false` | 🟡 |
| TC-E-05 | `unlink_staff_with_mismatched_item_id_returns_false` | 🟡 |
| TC-B-05 | `deleting_staff_cascades_to_item_staff_records` | 🟡 |

### handlers/staff.rs（ルーター統合テスト、DATABASE_URL必要・`#[ignore]`） — 4件
| テストID | テスト名 | 信頼性 |
|---|---|---|
| TC-N-01 | `post_staff_with_required_fields_only_returns_201` | 🔵 |
| TC-E-01 | `post_staff_with_empty_name_returns_400` | 🔵 |
| TC-E-02 | `post_item_staff_with_nonexistent_staff_id_returns_404_staff_not_found` | 🔵 |
| TC-E-04 | `delete_item_staff_with_nonexistent_id_returns_404` | 🟡 |

**合計: 22テストケース**（models単体9 + repository統合10 + handler統合4。一部はテストケース定義書のサブケース[境界値の正常/異常両方]を1件として個別カウント）

---

## 3. テスト実行結果

### 実行コマンド
```bash
cd backend/mediavault-api
cargo test models::staff   # DB不要の単体テスト
cargo test --no-run        # DB依存テストも含め全体のコンパイル確認
```

### 結果
- `models::staff` 単体テスト: **9件中7件FAILED（todo!()によるpanic）、2件PASSED（serdeデシリアライズのみで完結する境界テスト）**
- 全体ビルド（`cargo test --no-run`）: **成功**（DB依存テストも含め全テストバイナリがコンパイル可能）
- DB依存テスト（repository/handler、計14件）は `#[ignore]` 付与のため通常実行ではスキップされ、`cargo test -- --ignored`（DATABASE_URL要）実行時に同様に`todo!()`でFAILEDになる設計

### 期待された失敗内容
```
thread '...' panicked at mediavault-api\src\models\staff.rs:69:5:
not yet implemented: TASK-0020 Greenフェーズで実装: nameの空文字チェック

thread '...' panicked at mediavault-api\src\models\staff.rs:80:5:
not yet implemented: TASK-0020 Greenフェーズで実装: roleの空文字・長さチェック、character_nameの長さチェック
```
repository/handler層のテストは、実行時（`--ignored`）に同様の`todo!()`メッセージでpanicする想定（`create_staff`, `link_staff`, `unlink_staff`, 各ハンドラ）。

---

## 4. Greenフェーズで実装すべき内容

1. **`models/staff.rs`**
   - `parse_create_staff_request()`: nameが空文字（trim後）の場合に`ApiError::new(ApiErrorCode::ValidationError, ...)`を返す
   - `parse_create_item_staff_request()`: roleが空文字、role長>100、character_name長>255の場合にVALIDATION_ERRORを返す

2. **`models/response.rs`**
   - `ApiErrorCode::StaffNotFound`バリアントを新規追加し、`STAFF_NOT_FOUND` / 404 にマッピングする（staff-requirements.md 3 制約条件で明記）

3. **`repositories/staff_repository.rs`**
   - `create_staff()`: `INSERT INTO staff (...) VALUES (...) RETURNING ...`
   - `link_staff()`: staff_id/item_idの事前存在確認（またはFK制約違反のSQLSTATE判定）→ `STAFF_NOT_FOUND`/`ITEM_NOT_FOUND`へマッピング → `INSERT INTO item_staff`
   - `unlink_staff()`: `DELETE FROM item_staff WHERE id = $1 AND item_id = $2`、`rows_affected() > 0`をbool判定

4. **`handlers/staff.rs`**
   - `create_staff_handler()`: `deserialize_request` → `parse_create_staff_request` → `staff_repository::create_staff` → 201応答
   - `create_item_staff_handler()`: `parse_item_id`（パス）→ `deserialize_request` → `parse_create_item_staff_request` → `staff_repository::link_staff` → 201応答
   - `delete_item_staff_handler()`: `parse_item_id` × 2 → `staff_repository::unlink_staff` → false時404、true時204

5. **DBマイグレーション確認**
   - `staff`, `item_staff`テーブルが既存マイグレーションに存在するか確認（TASK-0004で作成済みのはず）。未作成ならGreenフェーズ前にマイグレーション追加が必要。
