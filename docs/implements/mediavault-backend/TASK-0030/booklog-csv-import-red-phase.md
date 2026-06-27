# TASK-0030 Redフェーズ記録: ブクログCSVインポート実装（POST /import/booklog）

**作成日**: 2026-06-27
**機能名**: booklog-csv-import
**タスクID**: TASK-0030
**要件名**: mediavault-backend

## 1. 対象テストケース

テストケース定義書（[booklog-csv-import-testcases.md](booklog-csv-import-testcases.md)）の24件すべてを対象とした。

| ID | 配置場所 | 種別 | 結果 |
|---|---|---|---|
| TC-N-01 | `handlers/import_booklog.rs` | 統合(DB)・`#[ignore]` | 未実行（DB必須） |
| TC-N-02 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-N-03 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-N-04 | `models/item.rs` | ユニット | ✅ pass |
| TC-N-05 | `handlers/import_booklog.rs` | 統合(DB)・`#[ignore]` | 未実行（DB必須） |
| TC-N-06 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-N-07 | `import/booklog_csv.rs` | ユニット | 🔴 FAILED（`unimplemented!()`、意図的Red） |
| TC-N-08 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-E-01 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-E-02 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-E-03 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-E-04 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-E-05 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-E-06 | `handlers/import_booklog.rs` | 統合(DB)・`#[ignore]` | 未実行（DB必須） |
| TC-E-07 | `handlers/import_booklog.rs` | 統合(DB)・`#[ignore]` | 未実行（DB必須） |
| TC-E-08 | `handlers/import_booklog.rs` | 統合(DB)・`#[ignore]` | 未実行（DB必須） |
| TC-B-01 | `import/booklog_csv.rs` + `handlers/import_booklog.rs` | ユニット＋統合(DB) | ユニット側 ✅ pass |
| TC-B-02 | `import/booklog_csv.rs` + `handlers/import_booklog.rs` | ユニット＋統合(DB) | ユニット側 ✅ pass |
| TC-B-03 | `handlers/import_booklog.rs` | 統合(DB)・`#[ignore]` | 未実行（DB必須） |
| TC-B-04 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-B-05 | `handlers/import_booklog.rs` | 統合(DB)・`#[ignore]` | 未実行（DB必須） |
| TC-B-06 | `import/booklog_csv.rs` | ユニット | ✅ pass |
| TC-REG-01 | `repositories/item_repository.rs` | 統合(DB)・`#[ignore]` | 未実行（DB必須） |
| TC-DB-01 | `repositories/item_repository.rs` | 統合(DB)・`#[ignore]` | 未実行（DB必須） |

## 2. 作成・変更したファイル

### 新規作成
- `backend/mediavault-api/src/models/import.rs` — `ImportSummary`・`ImportFailure`型（実装済み、TC-N-01・TC-E-05系のユニットテスト含む）
- `backend/mediavault-api/src/import/mod.rs` — importモジュール入口
- `backend/mediavault-api/src/import/booklog_csv.rs` — `BooklogCsvRow`・`parse_booklog_csv_row`・`parse_booklog_csv`・`extract_external_id`（後者のみ`unimplemented!()`スタブ）
- `backend/mediavault-api/src/handlers/import_booklog.rs` — `import_booklog_handler`（仮実装、DB登録時のrow_number対応等はGreenで詳細化）

### 変更
- `backend/mediavault-api/Cargo.toml` — `csv = "1.3"`, `encoding_rs = "0.8"` を追加
- `backend/mediavault-api/src/models/mod.rs` — `pub mod import;` 追加
- `backend/mediavault-api/src/models/item.rs` — `CreateItemRequest`に`consumed_date: Option<chrono::NaiveDate>`（`#[serde(default)]`）追加。TC-N-04対応テスト2件追加
- `backend/mediavault-api/src/models/item_import.rs` — `From<ImportItemRequest> for CreateItemRequest`に`consumed_date: None`を追加（コンパイル対応）
- `backend/mediavault-api/src/repositories/item_repository.rs` — `create_item_with_source`のINSERT文に`consumed_date`カラム・bindを追加（$12）。テストヘルパー`create_item_request`に`consumed_date: None`追加。TC-DB-01・TC-REG-01テスト追加
- `backend/mediavault-api/src/handlers/mod.rs` — `pub mod import_booklog;` 追加
- `backend/mediavault-api/src/main.rs` — `mod import;` 追加
- `backend/mediavault-api/src/routes/mod.rs` — `POST /import/booklog` ルート追加

## 3. 意図的なRed失敗

`import::booklog_csv::tests::extract_external_id_maps_isbn_and_normalizes_blank_to_none`（TC-N-07対応）が
`unimplemented!("Greenフェーズで実装: ISBN列をexternal_idへ抽出する")`によりpanicし、失敗する。

これは実装漏れ（コンパイルエラー）ではなく、「ISBN列→external_id抽出」というGreenフェーズで実装すべき
機能が未実装であることを示す意図的な失敗である。

```
thread 'import::booklog_csv::tests::extract_external_id_maps_isbn_and_normalizes_blank_to_none' panicked at
mediavault-api\src\import\booklog_csv.rs:119:5:
not implemented: Greenフェーズで実装: ISBN列をexternal_idへ抽出する
```

## 4. テスト実行結果サマリ

```
cargo build -p mediavault-api    → 成功（warningのみ、dead_code警告は既存パターンと同種）
cargo build --workspace          → 成功
cargo test -p mediavault-api     → 182 passed; 1 failed（意図的Red）; 186 ignored
```

`186 ignored`はDB必須統合テスト（既存分含む）。本タスクで追加したDB必須テストは以下の通り、
すべて`#[ignore]`として登録されている（`cargo test -- --ignored`実行時、Docker Compose Postgres必要）:

- `handlers::import_booklog::tests::*`（8件: TC-N-01, TC-E-06, TC-E-07, TC-B-01, TC-B-02, TC-B-03, TC-N-05, TC-B-05, TC-E-08）
- `repositories::item_repository::tests::create_item_with_source_binds_and_returns_consumed_date`（TC-DB-01）
- `repositories::item_repository::tests::create_item_wrapper_keeps_consumed_date_none_after_extension`（TC-REG-01）

## 5. Greenフェーズで実装すべき内容

1. **`extract_external_id`の実装**: `normalize_optional_field(row.external_id.clone())`相当の薄い実装へ差し替える（現在`unimplemented!()`）。
2. **`import_booklog_handler`の本実装確定**:
   - `parse_booklog_csv`の戻り値構造を見直し、`successes: Vec<(u32, CreateItemRequest, Option<String>)>`等に変更し、
     row_number・external_idをDB登録時まで保持できるようにする（現状は`row_number=0`固定・`extract_external_id_placeholder()`が`unimplemented!()`のままで仮実装）。
   - TC-E-08（DB登録失敗時の扱い）の最終方針確定（failure記録 or 500）。
   - `BOOKLOG_CSV_HEADER`定数の利用箇所確定（dead_code警告解消、または削除）。
3. **DB必須統合テスト（`#[ignore]`）の実行確認**: `docker compose up -d db` 後、`cargo test -p mediavault-api -- --ignored` で全件pass確認。

## 6. 品質判定

```
✅ 高品質:
- テスト実行: 新規追加コード全体がビルド成功。非DB依存テストは1件の意図的Red以外すべてpass
- 期待値: 各テストに具体的な期待値（success_count/failure_count/row_number/reason文言）を明記
- アサーション: 既存パターン（assert_eq!＋日本語コメント）を継続
- 実装方針: 明確（カラムマッピング分離・row_number算出・型変換失敗時のreason文言が確定済み）
- 信頼性レベル: 🔵が多数（要件定義書・確定済み設計判断に直結）、🟡は実サンプル未確認部分に限定
```
