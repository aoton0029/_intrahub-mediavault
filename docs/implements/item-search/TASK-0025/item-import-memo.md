# TDD開発メモ: item-import

## 概要

- 機能名: item-import（POST /items/import、外部検索結果からのアイテムインポート）
- 開発開始: 2026-06-26
- 現在のフェーズ: 完了（Red→Green→Refactor）

## 関連ファイル

- 元タスクファイル: `docs/tasks/mediavault-backend/TASK-0025.md`
- 要件定義: `docs/implements/item-search/TASK-0025/item-import-requirements.md`
- テストケース定義: `docs/implements/item-search/TASK-0025/item-import-testcases.md`
- Redフェーズ記録: `docs/implements/item-search/TASK-0025/item-import-red-phase.md`
- Refactorフェーズ記録: `docs/implements/item-search/TASK-0025/item-import-refactor-phase.md`
- 実装ファイル:
  - `backend/mediavault-api/src/models/item_import.rs`（新規）
  - `backend/mediavault-api/src/models/response.rs`（ApiErrorCode::ItemAlreadyImported追加）
  - `backend/mediavault-api/src/repositories/item_repository.rs`（create_item_with_source・find_existing_import・import_item追加）
  - `backend/mediavault-api/src/handlers/items.rs`（import_item_handler追加）
  - `backend/mediavault-api/src/routes/mod.rs`（POST /items/import登録）
- テストファイル: 上記各実装ファイル内の`#[cfg(test)] mod tests`（インライン配置）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2026-06-26

### テストケース

19ケース（正常7・異常8・境界4）+ ハンドラ単体補助1。DB非依存9ケースは
`cargo test -p mediavault-api`で実行確認、DB依存11ケースは`#[ignore]`付与で
`cargo test -- --ignored`実行を想定（未実行・実DB必要）。

### テストコード

`item-import-red-phase.md`の1章・2章を参照。主要追加点:
- `models/item_import.rs`: `ImportItemRequest` DTO + `parse_import_item_request`（バリデーション未呼び出し）
- `models/response.rs`: `ApiErrorCode::ItemAlreadyImported`（409マッピング実装済み）
- `repositories/item_repository.rs`: `create_item_with_source`（SQL未更新）・`find_existing_import`/`import_item`（`todo!()`）
- `handlers/items.rs`: `import_item_handler`（骨格完成、リポジトリ未実装に依存）
- `routes/mod.rs`: `POST /items/import`登録済み

### 期待される失敗

`cargo build -p mediavault-api`は成功（コンパイルエラーなし）。
`cargo test -p mediavault-api`は135 passed / 3 failed：
- `import_item_request_empty_external_id_returns_validation_error`
- `import_item_request_blank_external_id_returns_validation_error`
- `import_item_request_empty_title_returns_validation_error`

いずれも`parse_import_item_request`が`validate_external_id`/`validate_title`を
呼び出していないことに起因する想定通りの失敗（コンパイルエラーではなくロジック未実装による失敗）。

### 次のフェーズへの要求事項

1. `validate_external_id`の実装（trim().is_empty()判定）
2. `parse_import_item_request`から`validate_external_id`・`validate_title`を呼び出す
3. `create_item_with_source`のINSERT文にsource/external_idをバインド（SQLリテラル撤廃）
4. `find_existing_import`の実装（複合キーSELECT）
5. `import_item`の実装（重複チェック→create_item_with_source相当のINSERTを同一トランザクションで）

## Greenフェーズ（最小実装）

### 実施日時

2026-06-26（Refactor着手時点で完了済み）

### 実装内容

- `models/item_import.rs`: `validate_external_id`を`trim().is_empty()`判定で実装し、
  `parse_import_item_request`から`validate_external_id`・`validate_title`を呼び出すよう変更
- `repositories/item_repository.rs`: `create_item_with_source`のINSERT文をsource/external_id
  バインド（$10/$11）へ変更しSQLリテラルを撤廃。`find_existing_import`（複合キーSELECT）・
  `import_item`（重複チェック→create_item_with_source呼び出し）を実装
- `handlers/items.rs`: `import_item_handler`は骨格のまま動作（追加実装不要だった）
- `routes/mod.rs`: 変更不要（Redフェーズで登録済み）

### テスト結果

`cargo test -p mediavault-api`: 138 passed; 0 failed; 122 ignored（DB依存テストは未実行）

## Refactorフェーズ（品質改善）

### 実施日時

2026-06-26

### 改善内容

1. **DTO変換ロジックの集約（DRY）**: `item_repository::import_item`内にインラインで列挙していた
   `ImportItemRequest`→`CreateItemRequest`の9フィールド変換を、`models/item_import.rs`の
   `impl From<ImportItemRequest> for CreateItemRequest`へ集約。呼び出し側は`CreateItemRequest::from(request)`
   の1行に簡素化し、リポジトリ層の責務をDB操作に集中させた。
2. **陳腐化したRed期コメントの除去**: Green完了後も残っていた「まだ未実装（Red状態）」
   「本テストはpanicして失敗する想定」等、現在のコード状態と矛盾するコメントを、
   実際の挙動を説明する内容へ書き換えた（`item_import.rs`・`item_repository.rs`・`handlers/items.rs`の
   TASK-0025関連箇所のみ。他タスクの同種コメントはスコープ外）。

### セキュリティレビュー結果

- SQLインジェクション・情報漏洩防止・入力値検証: 問題なし（既存パターンを踏襲済み）
- 重複チェックのTOCTOU競合: 要件定義書で「単一ユーザー前提のため実害は極小」と明記された
  既知のトレードオフのため変更せず、申し送りとして記録（item-import-refactor-phase.md参照）

### パフォーマンスレビュー結果

- 重複チェックSELECT・トランザクション範囲ともに重大な性能課題なし

### テスト結果（リファクタ後）

`cargo build -p mediavault-api`: 成功（既存4警告のみ、新規警告なし）
`cargo test -p mediavault-api`: **138 passed; 0 failed; 122 ignored**（リファクタ前と同数・回帰なし）

### 品質評価

✅ 高品質（詳細は item-import-refactor-phase.md 第5章を参照）
