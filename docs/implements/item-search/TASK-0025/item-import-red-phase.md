# TASK-0025 Redフェーズ記録: POST /items/import 実装

**機能名**: item-import（外部検索結果からのアイテムインポート）
**タスクID**: TASK-0025
**要件名**: item-search
**作成日**: 2026-06-26

---

## 1. 作成したテストケース一覧

### DB非依存（`cargo test -p mediavault-api`で実行・確認済み）

| ID | 配置ファイル | テスト関数名 | 結果 |
|---|---|---|---|
| TC-0025-N01 | `models/item_import.rs` | `import_item_request_deserializes_minimal_fields` | PASS |
| TC-0025-N02 | `handlers/items.rs` | `created_response_returns_201_for_api_sourced_item` | PASS |
| TC-0025-E01 | `models/item_import.rs` | `import_item_request_missing_external_id_returns_validation_error` | PASS（serde必須Stringのデシリアライズ失敗で偶然成立） |
| TC-0025-E02 | `models/item_import.rs` | `import_item_request_empty_external_id_returns_validation_error` | **FAIL（Red確認）** |
| TC-0025-E03 | `models/item_import.rs` | `import_item_request_blank_external_id_returns_validation_error` | **FAIL（Red確認）** |
| TC-0025-E04 | `models/item_import.rs` | `import_item_request_invalid_media_type_returns_validation_error` | PASS（既存MediaType Deserializeにより成立） |
| TC-0025-E05 | `models/item_import.rs` | `import_item_request_empty_title_returns_validation_error` | **FAIL（Red確認）** |
| TC-0025-E07 | `models/response.rs` | `item_already_imported_returns_409`, `item_already_imported_has_correct_wire_code` | PASS |
| TC-0025-B02 | `models/item_import.rs` | `import_item_request_omitted_details_defaults_to_none` | PASS |

### DB依存（`#[ignore]`付与・`cargo test -- --ignored`で実DB必要、未実行）

| ID | 配置ファイル | テスト関数名 |
|---|---|---|
| TC-0025-N03 | `repositories/item_repository.rs` | `create_item_with_source_creates_item_with_api_source_and_external_id` |
| TC-0025-N04 | `repositories/item_repository.rs` | `create_item_wrapper_still_creates_manual_source_with_null_external_id` |
| TC-0025-N05 | `repositories/item_repository.rs` | `create_item_and_create_item_with_source_share_same_fields_except_source_and_external_id` |
| TC-0025-N06 | `handlers/items.rs` | `import_item_handler_returns_201_for_valid_request` |
| TC-0025-N07 | `repositories/item_repository.rs` | `create_item_with_source_handles_all_eight_media_types` |
| TC-0025-E06 | `repositories/item_repository.rs` | `find_existing_import_detects_duplicate_media_type_and_external_id`, `import_item_returns_409_and_does_not_create_duplicate_row` |
| TC-0025-E08 | `repositories/item_repository.rs` | `create_item_with_source_converts_db_error_to_internal_error` |
| TC-0025-B01 | `repositories/item_repository.rs` | `create_item_with_source_succeeds_with_minimal_fields_only` |
| TC-0025-B03 | `repositories/item_repository.rs` | `find_existing_import_does_not_treat_different_media_type_as_duplicate` |
| TC-0025-B04 | `handlers/items.rs` | `post_items_import_does_not_fall_through_to_item_id_route` |
| (E01相当・ハンドラ単体) | `handlers/items.rs` | `import_item_handler_returns_400_for_missing_external_id` |

合計: 19ケース（要件定義書の正常7・異常8・境界4を網羅）+ ハンドラ単体補助1。

---

## 2. 実装した骨格コード

### 新規ファイル: `backend/mediavault-api/src/models/item_import.rs`
- `ImportItemRequest` DTO（`media_type`, `external_id: String`必須, `title`, 任意項目, `details: Option<serde_json::Value>` + `#[serde(default)]`）
- `validate_external_id()`: **未実装（`todo!()`）**
- `parse_import_item_request()`: デシリアライズのみ実施、`validate_external_id`・`validate_title`の呼び出しは未実装（コメントで明記）

### 変更: `backend/mediavault-api/src/models/response.rs`
- `ApiErrorCode::ItemAlreadyImported` variant追加 → `("ITEM_ALREADY_IMPORTED", StatusCode::CONFLICT)`マッピング実装済み（このマッピング自体はGreenスコープ外で完成扱い、TC-0025-E07はPASS）

### 変更: `backend/mediavault-api/src/repositories/item_repository.rs`
- `create_item`を`create_item_with_source(pool, request, ItemSource::Manual, None)`の薄いラッパーへ変更
- `create_item_with_source(pool, request, source, external_id)`新設: **SQLは旧来のリテラル('manual', NULL)のまま**のため、source/external_idを指定しても反映されない（Red状態）
- `find_existing_import()`: **未実装（`todo!()`）**
- `import_item()`: **未実装（`todo!()`）**

### 変更: `backend/mediavault-api/src/handlers/items.rs`
- `import_item_handler`新設: `parse_import_item_request` → `item_repository::import_item`（未実装のため呼び出すとpanic）→ `created_response`

### 変更: `backend/mediavault-api/src/routes/mod.rs`
- `POST /items/import`を`/items/search`の直後・`/items/:id`より前にリテラル登録済み

---

## 3. 期待される失敗内容

- `cargo build -p mediavault-api`: **成功**（dead_code警告のみ、エラーなし）
- `cargo test -p mediavault-api`（非ignore）: **135 passed; 3 failed**
  - 失敗3件: `import_item_request_empty_external_id_returns_validation_error`,
    `import_item_request_blank_external_id_returns_validation_error`,
    `import_item_request_empty_title_returns_validation_error`
  - 失敗理由: `parse_import_item_request`が`validate_external_id`・`validate_title`を呼び出していないため、
    本来400になるべき入力が`Ok`を返してしまう（`unwrap_err()`がpanic）
- `cargo test -p mediavault-api -- --ignored`: 未実行（実DB必要）。実行した場合、
  `find_existing_import`・`import_item`の`todo!()`呼び出しによりpanicし、
  `create_item_with_source`系はsource/external_idが反映されないためassert失敗が期待される。

---

## 4. Greenフェーズで実装すべき内容

1. `models/item_import.rs`:
   - `validate_external_id`: `external_id.trim().is_empty()`判定で`VALIDATION_ERROR`を返す
   - `parse_import_item_request`: `validate_external_id(&request.external_id)?` と
     `validate_title(&request.title)?` を呼び出す
2. `repositories/item_repository.rs`:
   - `create_item_with_source`のINSERT文を`$10`(source)・`$11`(external_id)バインドへ変更し、
     SQLリテラル`'manual', NULL`を撤廃する
   - `find_existing_import`: `SELECT 1 FROM items WHERE media_type=$1 AND external_id=$2 LIMIT 1`を実装
   - `import_item`: トランザクション内で`find_existing_import`→存在すれば`ItemAlreadyImported`エラーで
     ロールバック、存在しなければ`create_item_with_source`相当のINSERTを実行
3. `handlers/items.rs`の`import_item_handler`は既に骨格完成のため、上記2点が実装されれば
   そのまま動作する想定（変更不要の可能性が高い）
4. `routes/mod.rs`のルート登録は完了済み（変更不要）

---

## 5. 品質判定

- テスト実行: 実行可能・失敗確認済み（✅）
- 期待値: 明確（HTTPステータス・エラーコード文字列・DB副作用で検証）
- アサーション: 適切（unwrap_err / assert_eq の組み合わせ）
- 実装方針: 明確（Greenフェーズで実装すべき内容を上記4節に列挙済み）
- 信頼性レベル分布: 🔵 多数 / 🟡 一部（重複挙動・8種網羅・最小構成等）/ 🔴 0件

**総合判定: ✅ 高品質**
