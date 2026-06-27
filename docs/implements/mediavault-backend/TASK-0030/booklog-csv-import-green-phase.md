# TASK-0030 Greenフェーズ記録: ブクログCSVインポート実装（POST /import/booklog）

**作成日**: 2026-06-27
**機能名**: booklog-csv-import
**タスクID**: TASK-0030
**要件名**: mediavault-backend

## 1. 実装方針

Redフェーズで指摘された3点を中心に最小実装を行った。

1. `import::booklog_csv::extract_external_id` を `normalize_optional_field(row.external_id.clone())` 相当の薄い実装へ差し替えた（`unimplemented!()`を削除）。
2. `parse_booklog_csv` の戻り値構造を `(Vec<CreateItemRequest>, Vec<ImportFailure>)` から `(Vec<ParsedBooklogRow>, Vec<ImportFailure>)` へ変更した。`ParsedBooklogRow { row_number, request, external_id }` により、行番号とISBN(external_id)をDB登録時まで保持できるようにした。
3. `handlers::import_booklog::import_booklog_handler` 内の `row_number=0` 固定・`extract_external_id_placeholder()`（`unimplemented!()`）を撤廃し、`parsed_rows` から取得した実際の `row_number`・`external_id` を `create_item_with_source` 呼び出しへ渡すよう変更した。
4. TC-E-08（DB登録失敗時の扱い）の方針を「行スキップとして記録し処理継続（500にしない）」に確定した（詳細は3章）。
5. `BOOKLOG_CSV_HEADER` 定数に `#[cfg(test)]` を付与し、dead_code警告を解消した（本番ロジックはcsv crateの `#[serde(rename)]` ベースデシリアライズに依存し、この定数自体は本番コードパスでは使われないため、テストヘルパー専用と位置づけた）。

## 2. 変更したファイル

- `backend/mediavault-api/src/import/booklog_csv.rs`
  - `extract_external_id` の実装化（`unimplemented!()` 削除）
  - `ParsedBooklogRow` 構造体を新規追加
  - `parse_booklog_csv` の戻り値型を `Vec<ParsedBooklogRow>` へ変更（`extract_external_id` をパース処理内で都度呼び出すよう変更）
  - `BOOKLOG_CSV_HEADER` に `#[cfg(test)]` を付与
- `backend/mediavault-api/src/handlers/import_booklog.rs`
  - `extract_external_id_placeholder` 関数を削除
  - `parsed_rows`（`ParsedBooklogRow`）をループし、`row_number`・`external_id` を実値で `create_item_with_source` へ渡すよう変更
  - `summary.failure_count` をパース層の失敗取り込み直後に明示同期するコードを追加

## 3. TC-E-08（DB登録失敗時の扱い）の最終方針

**確定内容**: パース・バリデーションを通過した正常行に対してDB登録（`create_item_with_source`）がエラーを返した場合、**当該行のみを `ImportFailure{row_number, reason: "db error"}` として記録し、後続行の処理を継続する**。HTTPレスポンスは常に200（`ImportSummary`）であり、500 INTERNAL_ERRORには昇格させない。

**判断理由**:
- 要件定義書2章のエラーレスポンス表では「パース処理自体の致命的失敗」のみ500 INTERNAL_ERRORの対象とし、「行単位の不正はここに含めない」と明記されている。DB登録失敗は個々の行に対する処理結果であり、パース処理自体の致命的失敗（multipart解析不能等）とは性質が異なる。
- EDGE-002「一部行が形式不正の場合、不正行のみスキップし正常行の取込を継続する」という設計方針と一貫させることで、ハンドラの責務（行単位の失敗は集約してスキップ）をパース由来の失敗・DB由来の失敗の両方で統一できる。
- TC-E-08のテスト自体も `assert_ne!(status, StatusCode::INTERNAL_SERVER_ERROR)` のみを要求しており、500を返さないことが必須要件。200で `failures` に記録する方が「パニックしない・情報漏洩しない・処理継続する」という安全性要件を満たしやすい。
- DBエラーの内部詳細（クエリ文字列・制約名等）はクライアントへ返さず、`tracing::error!` でログ出力のみに留め、`reason` は固定の汎用文言 `"db error"` とした（`item_repository::db_error()` の方針＝内部スキーマ情報を漏らさないに準拠）。

## 4. その他の判断

- `extract_external_id` の呼び出しタイミング: `parse_booklog_csv` 内で、デシリアライズ成功直後（バリデーション前）に `BooklogCsvRow` の参照から呼び出す。`parse_booklog_csv_row` が `row` の所有権を消費するため、所有権が渡る前に抽出する必要があった。
- `ParsedBooklogRow` に `PartialEq` を derive しなかった（`CreateItemRequest` 自体が `PartialEq` を実装していないため、テストでの構造比較が不要な本構造体には付与しなかった）。

## 5. テスト実行結果

```
cargo build -p mediavault-api    → 成功。BOOKLOG_CSV_HEADERのdead_code警告解消。残る警告は本タスク無関係の既存警告のみ（6件）
cargo test -p mediavault-api     → 183 passed; 0 failed; 186 ignored
```

`import::booklog_csv::tests::extract_external_id_maps_isbn_and_normalizes_blank_to_none`（Redフェーズで意図的に失敗していたテスト）は pass に変わった。

DB必須統合テスト10件（`#[ignore]`）は本環境にPostgresが無いため実行不可。実装ロジックを以下の観点で自己検証した:
- TC-N-01/N-05/N-06/N-07: `parse_booklog_csv_row` で `media_type=Novel` 固定・`consumed_date` パースが行われ、`create_item_with_source` の INSERT 文（`item_repository.rs` L78-101）が `consumed_date` ($12)・`source` ($10)・`external_id` ($11) を bind することを確認済み。`ParsedBooklogRow.external_id` がハンドラから `create_item_with_source` の `external_id` 引数として正しく渡される。
- TC-B-01/B-02/B-03/B-05: `parse_booklog_csv` がデータ0行・全行不正・1行のみ・300行のいずれでもパニックせず完走することをユニットテスト（`parse_booklog_csv_with_header_only_returns_empty_results`等）で確認済み。ハンドラはこれらの結果から常に200を返す（早期returnはファイル未添付・0バイトの場合のみ）。
- TC-E-06/E-07: ファイル未添付・0バイトの早期検証ロジックは変更していない（Redフェーズのまま）。
- TC-E-08: 上記3章の方針により、DBエラー発生時も500にならず200で `failures` に記録されることをコードレビューで確認済み（実DBでの確認は別途 `cargo test -- --ignored` 実行が必要）。

## 6. 品質判定

```
✅ 高品質:
- テスト結果: 非DB依存テストは全件pass（183 passed; 0 failed）。意図的Red失敗だった1件も解消
- 実装のシンプルさ: extract_external_idは1行の薄い実装、parse_booklog_csvの変更も既存ロジックへの最小限の構造追加（ParsedBooklogRow導入のみ）
- リファクタ候補: 後述（Refactorフェーズへ）
- 機能的問題: なし（自己レビューによりDB依存テストのロジックも妥当と判断）
- ファイルサイズ: booklog_csv.rs 約385行、import_booklog.rs 約470行。いずれも800行制限内
- モック使用: 実装コードにモック・スタブは含まれていない（unimplemented!()はすべて解消済み）
```

## 7. Refactorフェーズへの課題

- `extract_external_id` を `parse_booklog_csv` 内で呼び出すタイミングが「デシリアライズ成功直後」であり、`parse_booklog_csv_row` 内のバリデーション失敗時（title空等）でも `external_id` が抽出されてから捨てられる構造になっている。許容範囲だが、`ParsedBooklogRow` 構築ロジックを `parse_booklog_csv_row` 側に統合する余地がある。
- TC-E-08は実DB環境（Docker Compose Postgres）での `cargo test -- --ignored` 実行による最終確認が未済（本環境の制約）。
