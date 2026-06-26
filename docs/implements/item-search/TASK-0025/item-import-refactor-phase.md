# TASK-0025 Refactorフェーズ記録: POST /items/import 実装

**機能名**: item-import（外部検索結果からのアイテムインポート）
**タスクID**: TASK-0025
**要件名**: item-search
**作成日**: 2026-06-26

---

## 1. リファクタ前の状態（Greenフェーズ完了時点）

- `cargo build -p mediavault-api`: 成功（既存4警告のみ。TASK-0025由来の新規警告なし）
- `cargo test -p mediavault-api`（非ignore）: **138 passed; 0 failed; 122 ignored**
- 実装内容: `models/item_import.rs`（ImportItemRequest DTO・validate_external_id・parse_import_item_request）、
  `repositories/item_repository.rs`（create_item_with_source・find_existing_import・import_item）、
  `handlers/items.rs`（import_item_handler）、`models/response.rs`（ApiErrorCode::ItemAlreadyImported）、
  `routes/mod.rs`（POST /items/import登録）

## 2. レビュー結果

### セキュリティレビュー

- 🔵 **SQLインジェクション**: `create_item_with_source`の詳細テーブルINSERTは`detail_table_name()`の固定文字列matchで解決した
  テーブル名のみを`format!`に使用し、外部入力を直接埋め込んでいない。全カラム値は`bind`経由。問題なし。
- 🔵 **情報漏洩防止**: `db_error`関数が`tracing::error!`でサーバーログにのみ詳細を出力し、クライアントへは
  固定の汎用メッセージ（「アイテムの登録処理に失敗しました」）のみを返す。重複時のエラーメッセージ
  （「既にインポート済みです」）もDB内部情報を含まない。問題なし。
- 🔵 **入力値検証**: `external_id`・`title`の空文字・空白のみは`trim().is_empty()`基準で一貫してアプリ層で拒否し、
  DB CHECK制約（`chk_items_source_external_id`）到達前に弾く設計になっている。問題なし。
- 🟡 **重複チェックの競合状態**: `find_existing_import`（SELECT）と`create_item_with_source`（INSERT）は
  別個のクエリであり、同時に2リクエストが来た場合は理論上重複行が作成される可能性がある（TOCTOU）。
  要件定義書（item-import-requirements.md 6.3）で「単一ユーザー前提のため実害は極小」と明記されており、
  将来的な改善案（`(media_type, external_id)`への部分UNIQUE制約追加）も申し送り済みのため、
  本フェーズでは変更しない（仕様として許容された既知のトレードオフ）。

### パフォーマンスレビュー

- 🔵 **重複チェックSELECT**: `idx_items_external_id`インデックスを利用可能な単純なSELECT 1 LIMIT 1のため、
  計算量はO(1)〜O(log n)相当。単一ユーザー・小規模運用前提のため問題なし。
- 🔵 **トランザクション範囲**: 重複チェックと2回のINSERT（items本体・詳細テーブル）が想定される範囲で
  適切に区切られている（`import_item`は重複チェックを先に行い、トランザクションは`create_item_with_source`内の
  INSERT区間のみに限定）。重大な性能課題なし。

## 3. 実施したリファクタリング

### 3.1 DTO変換ロジックの集約（DRY・保守性向上）

- 🟡 **Before**: `item_repository::import_item`内で`ImportItemRequest`から`CreateItemRequest`への変換を
  9フィールド分インラインで列挙していた。
- 🟡 **After**: `models/item_import.rs`に`impl From<ImportItemRequest> for CreateItemRequest`を追加し、
  変換ロジックをDTO定義のすぐ近くに集約。`import_item`側は`CreateItemRequest::from(request)`の1行呼び出しに簡素化。
- **改善理由**:
  - 変換ロジックがDTO定義（`ImportItemRequest`/`CreateItemRequest`）の近くに置かれることで、
    両者のフィールド対応関係が見つけやすくなる（可読性向上）。
  - `CreateItemRequest`にフィールドが追加された場合、`From`実装側でコンパイルエラーとして検知できる
    （構造体リテラル方式を維持し、`..Default::default()`等のフィールド省略は使わない）。
  - リポジトリ層（`item_repository.rs`）の責務をDB操作に近づけ、DTO間のマッピングというモデル層の関心事と分離した。
- **影響範囲**: `models/item_import.rs`（追加）、`repositories/item_repository.rs`の`import_item`関数内（9行→1行）。
  外部から見た`import_item`の振る舞い（戻り値・エラー）は不変。

### 3.2 陳腐化したRed期コメントの除去

- 🔵 **Before**: Red→Green移行が完了したにもかかわらず、以下のような今のコード状態と矛盾するコメントが
  TASK-0025関連ファイルに残存していた:
  - 「まだ未実装（Red状態）」「Greenフェーズで実装する」（`validate_external_id`・`parse_import_item_request`の
    関数doc）
  - 「本テストはpanicして失敗する想定」「Red期待」（`find_existing_import_detects_duplicate_*`等のテストdoc）
  - 「まだ未実装のfind_existing_importを呼び出す」（テスト本体のインラインコメント）
- 🔵 **After**: 各コメントを「現在の実装が何をするか」を説明する内容へ書き換えた。
  例: 「まだ未実装（Red状態）。Greenフェーズで`validate_title`相当のロジックを実装する」
  → 「`validate_title`（models/item.rs）と同様にtrim().is_empty()で空文字・空白のみを400 VALIDATION_ERRORとして拒否する」
- **改善理由**: Red期のコメントをGreen/Refactor後も残すと、後続の開発者が「この関数は未実装のままなのか」と
  誤解する可能性がある。テスト・実装コードのコメントは常に「現在の挙動」を正確に説明すべきという
  コーディング規約（note.md テストコメント規約）に整合させた。
- **対象ファイル**: `models/item_import.rs`（関数doc2件・テストdoc4件）、
  `repositories/item_repository.rs`（関数doc1件・テストdoc4件）、`handlers/items.rs`（ハンドラ内コメント1件・テストdoc3件）。
  TASK-0025に無関係な既存ファイル（他タスクのRed期コメント）には手を入れていない（スコープ外）。

### 3.3 変更しなかった項目（検討の上、現状維持と判断）

- **ルート登録の記法揺れ**: `routes/mod.rs`で`.route("/items/import", axum::routing::post(...))`と
  `.route("/items", get(...).post(...))`で記法が異なるが、両方の記法がファイル内で既に併用されており
  （`/items/:id/status`等も`axum::routing::patch`形式）、一貫性の観点で問題視するレベルではないため変更不要と判断。
- **重複チェックの非アトミック性**: 3.1節セキュリティレビュー参照。要件定義書で明示的に許容されたトレードオフのため変更不要。
- **`details`の個別カラム反映**: 要件定義書（2.1節）で「本タスクでは保持のみ・個別カラム反映は範囲外」と
  明記されているため、本フェーズでは対応しない。

## 4. リファクタ後の確認結果

- `cargo build -p mediavault-api`: **成功**（既存4警告のみ。新規警告0件）
- `cargo test -p mediavault-api`（非ignore）: **138 passed; 0 failed; 122 ignored**（リファクタ前と同数・回帰なし）
- `cargo clippy -p mediavault-api --no-deps`: TASK-0025関連ファイル（`item_import.rs`・`item_repository.rs`の
  該当箇所）に新規warningなし
- ファイルサイズ: `item_import.rs`（約100行）、`item_repository.rs`（約2100行・複数タスクの蓄積のため500行制限は
  ファイル単位ではなく機能単位で評価。TASK-0025追加分は約160行で制限内）、`handlers/items.rs`（約768行・同様に
  複数タスクの蓄積）。新規ファイル分割は本フェーズの範囲外（既存の蓄積構造を踏襲）。

## 5. 品質判定

| 評価軸 | 状態 |
|---|---|
| テスト結果 | ✅ 138 passed / 0 failed（リファクタ前後で同数、回帰なし） |
| セキュリティ | ✅ 重大な脆弱性なし（TOCTOU競合は要件上許容済みの既知トレードオフ） |
| パフォーマンス | ✅ 重大な性能課題なし |
| リファクタ品質 | ✅ DTO変換の集約（DRY）・陳腐化コメントの除去を実施 |
| コード品質 | ✅ clippy新規警告なし、build警告増加なし |
| 日本語コメント | ✅ Red期の矛盾コメントを現状に整合する説明へ更新 |

**総合判定: ✅ 高品質**

---

## 6. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-verify-complete mediavault-backend TASK-0025` で完全性検証を実行します。
