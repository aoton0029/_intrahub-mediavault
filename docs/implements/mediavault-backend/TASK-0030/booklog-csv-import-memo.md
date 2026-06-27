# TDD開発メモ: booklog-csv-import

## 概要

- 機能名: ブクログCSVインポート実装（POST /import/booklog）
- 開発開始: 2026-06-27
- 現在のフェーズ: Green

## 関連ファイル

- 元タスクファイル: `docs/tasks/mediavault-backend/TASK-0030.md`
- 要件定義: `docs/implements/mediavault-backend/TASK-0030/booklog-csv-import-requirements.md`
- テストケース定義: `docs/implements/mediavault-backend/TASK-0030/booklog-csv-import-testcases.md`
- Redフェーズ記録: `docs/implements/mediavault-backend/TASK-0030/booklog-csv-import-red-phase.md`
- 実装ファイル:
  - `backend/mediavault-api/src/models/import.rs`（新規）
  - `backend/mediavault-api/src/import/mod.rs`（新規）
  - `backend/mediavault-api/src/import/booklog_csv.rs`（新規）
  - `backend/mediavault-api/src/handlers/import_booklog.rs`（新規）
  - `backend/mediavault-api/src/models/item.rs`（拡張: consumed_date追加）
  - `backend/mediavault-api/src/repositories/item_repository.rs`（拡張: consumed_date bind）
  - `backend/mediavault-api/src/models/item_import.rs`（コンパイル対応のみ）
  - `backend/mediavault-api/src/routes/mod.rs`（ルート追加）
  - `backend/mediavault-api/Cargo.toml`（csv/encoding_rs追加）
- テストファイル: 上記の各実装ファイル内の `#[cfg(test)] mod tests`（プロジェクト方針に従いインライン配置）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2026-06-27

### テストケース

テストケース定義書の24件すべてを対象に実装した。
- DB非依存ユニットテスト: 16件（`import/booklog_csv.rs` 13件、`models/import.rs` 3件、`models/item.rs` 2件の一部）
- DB必須統合テスト（`#[ignore]`）: 10件（`handlers/import_booklog.rs` 8件、`repositories/item_repository.rs` 2件）

### テストコード

各実装ファイル末尾の `#[cfg(test)] mod tests` を参照（プロジェクト方針により本メモには全文転記しない）。
主要なテストファイルパス:
- `backend/mediavault-api/src/import/booklog_csv.rs`
- `backend/mediavault-api/src/models/import.rs`
- `backend/mediavault-api/src/handlers/import_booklog.rs`
- `backend/mediavault-api/src/repositories/item_repository.rs`（追記部分）
- `backend/mediavault-api/src/models/item.rs`（追記部分）

### 期待される失敗

`import::booklog_csv::tests::extract_external_id_maps_isbn_and_normalizes_blank_to_none`（TC-N-07対応）が
`extract_external_id`関数の`unimplemented!()`によりpanicし、失敗する。これは意図的なRed失敗であり、
ISBN列→external_id抽出ロジックがGreenフェーズで実装すべき機能であることを示す。

他のDB非依存テスト15件はすべてpassする（パーサ・モデル拡張部分は要件が明確なため最小限の実装を
本フェーズで先行実装している）。DB必須統合テスト10件は`#[ignore]`のため`cargo test -p mediavault-api`では
実行されず、`cargo test -- --ignored`（Docker Compose Postgres必要）で別途確認する。

`cargo test -p mediavault-api`実行結果: `182 passed; 1 failed; 186 ignored`

### 次のフェーズへの要求事項

1. `import::booklog_csv::extract_external_id`を実装する（`normalize_optional_field`相当）。
2. `handlers::import_booklog::import_booklog_handler`内の仮実装（`row_number=0`固定、
   `extract_external_id_placeholder()`の`unimplemented!()`）を実装する。
   - `parse_booklog_csv`の戻り値構造を見直し、行番号・ISBN(external_id)をDB登録時まで保持できるようにする。
3. TC-E-08（DB登録失敗時の扱い）の最終方針を確定する。
4. `BOOKLOG_CSV_HEADER`定数の利用方針を確定する（テストヘルパーとしての利用継続、またはdead_code解消）。
5. DB必須統合テスト10件を`docker compose up -d db`環境で`cargo test -p mediavault-api -- --ignored`実行し、
   全件pass確認する。

## Greenフェーズ（最小実装）

### 実装日時

2026-06-27

### 実装方針

1. `extract_external_id`（`import/booklog_csv.rs`）を `normalize_optional_field(row.external_id.clone())` の薄い実装へ差し替え、`unimplemented!()` を解消した。
2. `parse_booklog_csv` の戻り値を `(Vec<ParsedBooklogRow>, Vec<ImportFailure>)` に変更。`ParsedBooklogRow{row_number, request, external_id}` で行番号・ISBNをDB登録時まで保持できるようにした。
3. `import_booklog_handler`（`handlers/import_booklog.rs`）の `row_number=0` 固定・`extract_external_id_placeholder()` を撤廃し、`parsed_rows` から実値を取って `create_item_with_source` に渡すよう変更。
4. TC-E-08の方針を確定: DB登録失敗は当該行のみ `ImportFailure{row_number, reason:"db error"}` として記録し、500には昇格させず処理継続する（詳細はgreen-phase.md 3章）。
5. `BOOKLOG_CSV_HEADER` に `#[cfg(test)]` を付与してdead_code警告を解消（本番コードパスでは使用されないテスト専用定数のため）。

### 実装コード

詳細は各実装ファイル本体および `booklog-csv-import-green-phase.md` を参照（本メモには全文転記しない、プロジェクト方針）。

### テスト結果

```
cargo build -p mediavault-api    → 成功。BOOKLOG_CSV_HEADERのdead_code警告は解消、残り6件は既存・無関係警告
cargo test -p mediavault-api     → 183 passed; 0 failed; 186 ignored
```

意図的Red失敗だった `extract_external_id_maps_isbn_and_normalizes_blank_to_none` はpassに変わった。
DB必須統合テスト10件（`#[ignore]`）は本環境にPostgres未接続のため未実行。実装ロジックのコードレビューにより
TC-N-01/N-05/N-06/N-07・TC-B-01/B-02/B-03/B-05・TC-E-08の挙動が要件・テストケース定義と整合することを確認済み。
実DB環境（`docker compose up -d db`）での `cargo test -- --ignored` による最終確認は未済。

### 課題・改善点（Refactorフェーズで対応）

- `extract_external_id` の呼び出しタイミング（デシリアライズ成功直後、バリデーション失敗行でも一旦抽出される）の整理。
- TC-E-08の実DB環境での最終動作確認。

## Refactorフェーズ（品質改善）

（未着手）
