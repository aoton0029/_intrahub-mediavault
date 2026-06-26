# TASK-0025 TDDテストケース定義書: POST /items/import 実装

**機能名**: item-import（外部検索結果からのアイテムインポート）
**タスクID**: TASK-0025
**要件名**: mediavault-backend
**作成日**: 2026-06-26
**出力ファイル**: `docs/implements/item-search/TASK-0025/item-import-testcases.md`

---

## 0. 信頼性レベルの凡例

各テストケースについて、元資料（要件定義書 `item-import-requirements.md`、note.md、実コード、設計文書）との照合状況を以下の信号で示す。

- 🔵 **青信号**: 元資料を参照し、ほぼ推測していない
- 🟡 **黄信号**: 元資料から妥当な推測をした
- 🔴 **赤信号**: 元資料に根拠がない推測

---

## 1. 技術スタック・テスト方針（実コード・note.md準拠）

- **プログラミング言語**: Rust（edition 2024） 🔵
  - **言語選択の理由**: 既存 `mediavault-api` クレートがRust製。型システムにより「`source`/`external_id` をコンパイル時に保証」「DTOのデシリアライズ失敗を `Result` で安全に処理」できる。*（note.md L13、Cargo.toml より）*
  - **テストに適した機能**: `#[cfg(test)] mod tests` のインラインユニットテスト、`#[tokio::test]` による非同期テスト、`?`/`Result` によるエラー伝播検証。
- **テストフレームワーク**: Rust標準 `#[test]` / `#[tokio::test]`（tokio 1.52.3）+ `tower::ServiceExt::oneshot`（ルーター統合）+ `sqlx`（実DB統合） 🔵
  - **フレームワーク選択の理由**: 既存TASK-0009〜0024と完全に同一のテストパターンを踏襲し、レビュー容易性とパリティを担保する。*（note.md L51-55, L144-149 より）*
  - **テスト実行環境**:
    - ユニット（DB非依存）: `cargo test -p mediavault-api`
    - 実DB統合・ルーター統合: `#[ignore]` を付与し `cargo test -- --ignored`（`docker compose up -d db` + `DATABASE_URL` 前提）
- 🔵 信頼性レベル: note.md「テスト規約」「テスト関連情報」、既存 `handlers/items.rs` / `routes/mod.rs` のテスト実装に直接対応。

### テスト配置方針（既存パターン踏襲）

| テスト分類 | 配置ファイル | DB依存 | 実行方法 |
|---|---|---|---|
| DTOデシリアライズ・バリデーション（純関数） | `src/models/item_import.rs`（新規）内 `#[cfg(test)] mod tests` | なし | `cargo test` |
| `ApiErrorCode::ItemAlreadyImported` のステータスマッピング | `src/models/response.rs` 内 `mod tests` | なし | `cargo test` |
| `created_response`/201形式・ハンドラ単体 | `src/handlers/items.rs` 内 `mod tests` | 一部あり（`#[ignore]`） | `cargo test -- --ignored` |
| リポジトリ `create_item_with_source` の挙動・重複検知 | `src/repositories/item_repository.rs` 内 `mod tests` | あり（`#[ignore]`） | `cargo test -- --ignored` |
| ルーター経由のE2E（POST /items/import） | `src/routes/mod.rs` 内 `mod tests` | あり（`#[ignore]`） | `cargo test -- --ignored` |

---

## 2. 正常系テストケース（基本的な動作）

### TC-0025-N01: ImportItemRequest の正常デシリアライズ（必須項目のみ）

- **テスト名**: ImportItemRequestが必須項目（media_type/external_id/title）のみで正常にデシリアライズされる
  - **何をテストするか**: 新規DTO `ImportItemRequest` が、最小構成のJSONから正しくパースされること。
  - **期待される動作**: `parse_import_item_request`（新規。`parse_create_item_request` 相当）が `Ok(ImportItemRequest)` を返し、`media_type=Anime`/`external_id="12345"`/`title` が保持される。
- **入力値**: `{ "media_type": "anime", "external_id": "12345", "title": "鬼滅の刃" }`
  - **入力データの意味**: 要件 2.1 で必須とされる3フィールドのみを与え、任意項目省略時もデシリアライズが成立することを代表する。
- **期待される結果**: `request.media_type == MediaType::Anime`、`request.external_id == "12345"`、`request.title == "鬼滅の刃"`。
  - **期待結果の理由**: snake_case enumデシリアライズは既存 `MediaType` 実装済みで、必須フィールドが揃えばパース成功するため。
- **テストの目的**: 新規DTOの基本デシリアライズ契約を確認する。
  - **確認ポイント**: `external_id` が `String`（必須・非Option）として保持されること。
- 🔵 信頼性レベル: 要件 2.1 入力仕様表、既存 `create_item_request_deserializes_successfully`（item.rs L306-312）のパリティより。

### TC-0025-N02: created_response が import経路でも 201 + 統一フォーマットを返す

- **テスト名**: import_item_handler成功時のレスポンスがHTTP 201・`{"success":true,"data":<Item>}`になる
  - **何をテストするか**: インポート成功時のレスポンス構築が、手動作成 `POST /items` と同一の `created_response` を再利用して201を返すこと。
  - **期待される動作**: `created_response(item)` が `StatusCode::CREATED` を返す（`source=api`/`external_id=Some(...)` の `Item` でも同一）。
- **入力値**: `source=ItemSource::Api`, `external_id=Some("12345")` を持つ `Item`（テスト用 `sample_item()` 派生）。
  - **入力データの意味**: 手動作成テスト（`created_response_returns_201_with_success_envelope`）の `source=manual` 版に対し、api版でも同じ201契約が成り立つことを代表する。
- **期待される結果**: `response.status() == StatusCode::CREATED`。
  - **期待結果の理由**: 要件 2.2「成功時 201 Created・`ApiOk<Item>`」、`created_response` 再利用方針より。
- **テストの目的**: レスポンス形式のパリティを確認する。
  - **確認ポイント**: import専用のレスポンス関数を新設せず、既存 `created_response` を流用していること。
- 🔵 信頼性レベル: 要件 2.2、既存 `created_response`（handlers/items.rs L49-52, L256-274）に直接対応。

### TC-0025-N03: create_item_with_source が source=api・external_id付きでitems+詳細テーブルを作成する（実DB）

- **テスト名**: create_item_with_sourceがsource=api/external_idでitems本体とanime_detailsを同一トランザクションで作成する
  - **何をテストするか**: 再利用可能な内部関数 `create_item_with_source(pool, request, source, external_id)` が、`items`（`source=api`, `external_id="12345"`）と `anime_details`（`item_id`）を作成すること。
  - **期待される動作**: 戻り値 `Item.source == Api`、`Item.external_id == Some("12345")`、`anime_details` に当該 `item_id` の行が1件存在する。
- **入力値**: `media_type=Anime`, `title="鬼滅の刃"`, `source=ItemSource::Api`, `external_id=Some("12345")`。
  - **入力データの意味**: 要件 4.1 TC-002-03 の代表的な正常インポート。anime を選ぶのは詳細テーブル振り分け（`anime_details`）の正常系を兼ねるため。
- **期待される結果**: items行が1件作成され `source='api'`/`external_id='12345'`、`anime_details` 行も1件作成される。`id`/`created_at`/`updated_at` がDB採番される。
  - **期待結果の理由**: 要件 4.1、note.md トランザクション処理（item_repository.rs L51-94）の再利用方針より。
- **テストの目的**: インポートのコア処理（トランザクション内2テーブルINSERT）の正常動作を確認する。
  - **確認ポイント**: `source`/`external_id` がハードコードでなく引数で反映されること、詳細テーブルが作成されること。
- 🔵 信頼性レベル: 要件 4.1 TC-002-03、TASK-0025.md テストケース1、既存 `create_item` 実装より。

### TC-0025-N04: create_item（manual薄ラッパー）が従来通りsource=manual/external_id=NULLで作成する（実DB・回帰）

- **テスト名**: 既存create_itemがcreate_item_with_sourceのラッパー化後もsource=manual/external_id=NULLを維持する
  - **何をテストするか**: リファクタ（`create_item` を `create_item_with_source(.., Manual, None)` の薄いラッパーへ変更）後も、`POST /items`（手動作成）の挙動が不変であること。
  - **期待される動作**: `create_item(pool, req)` の戻り `Item.source == Manual`、`Item.external_id == None`。詳細テーブルも従来通り作成される。
- **入力値**: `CreateItemRequest { media_type: Movie, title: "君の名は。", .. }`。
  - **入力データの意味**: 既存TASK-0009の回帰確認。anime以外（movie）を選び詳細テーブル振り分けの別経路も併せて確認する。
- **期待される結果**: `source='manual'`、`external_id IS NULL`、`movie_details` 行が1件作成される。
  - **期待結果の理由**: 要件 3.1「既存 `create_item` のテストを壊さない（Option B）」より、ラッパー化は外部挙動を変えてはならない。
- **テストの目的**: リファクタによる既存機能の非破壊（回帰防止）を確認する。
  - **確認ポイント**: ラッパー化後も `source=manual`/`external_id=NULL` が保たれること。
- 🔵 信頼性レベル: 要件 3.1（Option B 再利用方針）、note.md L237-238、既存 `create_item` より。

### TC-0025-N05: POST /items と POST /items/import のパリティ（同一詳細データ→source/external_idのみ差分、実DB）

- **テスト名**: 同等の詳細データでmanual作成とapiインポートを行うと、source/external_id以外のItem内容が一致する
  - **何をテストするか**: 手動作成（`source=manual`, `external_id=NULL`）とインポート（`source=api`, `external_id=Some`）が**同一のトランザクション処理経路**を通り、差分が `source`/`external_id` のみであること。
  - **期待される動作**: 両者の `Item` で `media_type`/`title`/`original_title` 等が一致し、`source`/`external_id` のみ異なる。
- **入力値**: manual側 `CreateItemRequest{media_type:Anime, title:"作品X"}`、import側 同一の `media_type`/`title` + `external_id="999"`。
  - **入力データの意味**: 要件 4.1「TASK-0009一貫性」、TASK-0025.md テストケース4 のパリティ要件を代表する。
- **期待される結果**: `manual.media_type == import.media_type`、`manual.title == import.title`、`manual.source==Manual && import.source==Api`、`manual.external_id==None && import.external_id==Some("999")`。
  - **期待結果の理由**: 要件 3.1「コード重複禁止・同一トランザクション処理を経由」より、分岐点は `source`/`external_id` のみ。
- **テストの目的**: 手動作成との一貫性（再利用設計の正しさ）を確認する。
  - **確認ポイント**: 詳細テーブルが両経路とも作成され、共通カラムの値生成ロジックが同一であること。
- 🔵 信頼性レベル: 要件 4.1、TASK-0025.md テストケース4 に直接対応。

### TC-0025-N06: POST /items/import がルーター経由で201を返す（実DB・E2E）

- **テスト名**: POST /items/importがルーター経由で有効リクエストに対し201を返す
  - **何をテストするか**: `build_router` に登録された `POST /items/import` が、`tower::oneshot` 駆動で201を返すE2Eパス。
  - **期待される動作**: `response.status() == StatusCode::CREATED`、ボディが `{"success":true,"data":{...}}`。
- **入力値**: `POST /items/import` ボディ `{"media_type":"anime","external_id":"12345","title":"鬼滅の刃"}`。
  - **入力データの意味**: ルーティング登録・ハンドラ結線・DTO抽出・DB登録の全経路を貫く統合確認。
- **期待される結果**: HTTP 201、ボディに `source:"api"`・`external_id:"12345"` を含む `Item`。
  - **期待結果の理由**: 要件 2.2・2.3 データフロー、note.md ルーティング方針より。
- **テストの目的**: エンドツーエンドのルーティング結線を確認する。
  - **確認ポイント**: `/items/import` がリテラルパスとして `/items/:id` より前に登録され誤マッチしないこと。
- 🔵 信頼性レベル: 要件 2.3、note.md L46-49・L190-192、既存ルーターテストパターンより。

### TC-0025-N07: 全media_type（8種）が対応する詳細テーブルへ振り分けられる（パラメータ化・実DB or detail_table_name単体）

- **テスト名**: 8つのmedia_typeすべてでインポートが対応詳細テーブルにレコードを作成する
  - **何をテストするか**: `detail_table_name` 経由で anime/movie/drama/manga/novel/game/academic_book/paper の8種すべてが正しい詳細テーブルへINSERTされること。
  - **期待される動作**: 各 `media_type` に対し対応詳細テーブル（`anime_details`〜`paper_details`）に `item_id` 行が作成される。
- **入力値**: 8種の `media_type` それぞれで `external_id` を変えた8リクエスト（ループ）。
  - **入力データの意味**: 詳細テーブル作成が anime 限定でないこと（media_type別振り分けの網羅）を保証する。
- **期待される結果**: 各 `media_type` で対応詳細テーブルに1行作成される（`detail_table_name(mt)` の戻り値テーブルに存在）。
  - **期待結果の理由**: 既存 `detail_table_name` の8 variant網羅テスト（item_repository.rs L558-642）と整合。
- **テストの目的**: media_type別の詳細テーブル作成網羅を確認する。
  - **確認ポイント**: 既存 `detail_table_name` を再利用し、import経路でも8種すべてが機能すること。
- 🟡 信頼性レベル: 要件 2.1（8種enum）・既存 `detail_table_name` テスト網羅からの妥当な推測（importでの8種網羅は明記なしだが既存実装の自然な拡張）。

---

## 3. 異常系テストケース（エラーハンドリング）

### TC-0025-E01: external_id欠落 → 400 VALIDATION_ERROR

- **テスト名**: external_idフィールドが欠落したリクエストで400 VALIDATION_ERRORになる
  - **エラーケースの概要**: 必須 `external_id` をボディに含めないインポートリクエスト。
  - **エラー処理の重要性**: `external_id` は `source=api` の必須項目（DB CHECK制約 `chk_items_source_external_id` 対象）。アプリ層で先に弾くことでDB到達前に防ぐ。
- **入力値**: `{ "media_type": "anime", "title": "鬼滅の刃" }`（external_idなし）。
  - **不正な理由**: 要件 2.1 で `external_id` は必須。欠落は契約違反。
  - **実際の発生シナリオ**: フロントエンドのバグ・外部検索結果の `external_id` 未取得時。
- **期待される結果**: `Err(ApiError)` で `error.code == "VALIDATION_ERROR"`、`status == 400`、`items` への書き込みなし。
  - **エラーメッセージの内容**: 「external_idは必須です」等の汎用メッセージ（DB内部情報を含まない）。
  - **システムの安全性**: トランザクション開始前にreturnし、副作用なし。
- **テストの目的**: 必須 `external_id` のバリデーションを確認する。
  - **品質保証の観点**: 不正データのDB混入（`source=api` で `external_id=NULL`）を防止する。
- 🔵 信頼性レベル: 要件 2.2 エラー表・4.2、TASK-0025.md テストケース2 に直接対応。

### TC-0025-E02: external_id空文字 → 400 VALIDATION_ERROR

- **テスト名**: external_idが空文字（""）のリクエストで400 VALIDATION_ERRORになる
  - **エラーケースの概要**: `external_id` キーは存在するが値が空文字。
  - **エラー処理の重要性**: 空文字は実質的に外部ID欠如と同義であり、欠落と同じく弾く必要がある（`title` の `trim().is_empty()` 規約と同方針）。
- **入力値**: `{ "media_type": "anime", "external_id": "", "title": "鬼滅の刃" }`。
  - **不正な理由**: 要件 2.1「空文字・欠落は400」。
  - **実際の発生シナリオ**: 外部APIが空のIDを返した／フロントの初期値が空のまま送信された場合。
- **期待される結果**: `error.code == "VALIDATION_ERROR"`、`status == 400`、`items` への書き込みなし。
  - **エラーメッセージの内容**: 「external_idは空にできません」等の汎用メッセージ。
  - **システムの安全性**: 書き込み前にreturnし副作用なし。
- **テストの目的**: 空文字 `external_id` を欠落と同等に拒否することを確認する。
  - **品質保証の観点**: 空ID行の混入防止。
- 🔵 信頼性レベル: 要件 2.1 制約欄「空文字・欠落は400」、既存 `validate_title` の trim 規約より。

### TC-0025-E03: external_id空白のみ（"   "） → 400 VALIDATION_ERROR

- **テスト名**: external_idが空白のみのリクエストで400 VALIDATION_ERRORになる
  - **エラーケースの概要**: `external_id` が半角/全角スペースのみ。
  - **エラー処理の重要性**: 既存 `validate_title` が `trim().is_empty()` 基準で空白のみを拒否しているため、`external_id` バリデーションも同基準で一貫させる。
- **入力値**: `{ "media_type": "anime", "external_id": "   ", "title": "鬼滅の刃" }`。
  - **不正な理由**: 空白のみは有効な外部IDとはみなせない（既存 `validate_title` 規約との整合）。
  - **実際の発生シナリオ**: 入力フォームの空白混入。
- **期待される結果**: `error.code == "VALIDATION_ERROR"`、`status == 400`、書き込みなし。
  - **エラーメッセージの内容**: 空文字と同じ汎用メッセージ。
  - **システムの安全性**: 副作用なし。
- **テストの目的**: 空白のみの `external_id` を拒否し、`title` バリデーションと基準を揃えることを確認する。
  - **品質保証の観点**: バリデーション基準の一貫性。
- 🟡 信頼性レベル: 既存 `validate_title` の `trim().is_empty()` 方式（item.rs L161-169, L331-337 blank_title テスト）からの妥当な推測（要件は「空文字」のみ明記、空白のみは整合のため拡張）。

### TC-0025-E04: media_type不正値 → 400 VALIDATION_ERROR

- **テスト名**: 未知のmedia_type文字列で400 VALIDATION_ERRORになる
  - **エラーケースの概要**: enum 8種に存在しない `media_type` 値。
  - **エラー処理の重要性**: 不正種別は詳細テーブル振り分け不能。デシリアライズ段階で弾く。
- **入力値**: `{ "media_type": "invalid_type", "external_id": "12345", "title": "X" }`。
  - **不正な理由**: 要件 2.1「8種のみ。不正値は400」。
  - **実際の発生シナリオ**: フロントの種別パラメータ不整合。
- **期待される結果**: `error.code == "VALIDATION_ERROR"`、`status == 400`。
  - **エラーメッセージの内容**: 「リクエストの形式が不正です」等（既存 `deserialize_request` のフォーマット）。
  - **システムの安全性**: デシリアライズ失敗で早期return。
- **テストの目的**: 不正 `media_type` のデシリアライズ失敗→400変換を確認する。
  - **品質保証の観点**: 既存 `invalid_media_type_returns_validation_error`（item.rs L314-321）とのパリティ。
- 🟡 信頼性レベル: 要件 2.2 エラー表・4.2、既存CreateItem系の規約からの妥当な推測。

### TC-0025-E05: title空文字 → 400 VALIDATION_ERROR

- **テスト名**: titleが空文字のインポートリクエストで400 VALIDATION_ERRORになる
  - **エラーケースの概要**: 必須 `title` が空文字。
  - **エラー処理の重要性**: `CreateItemRequest` の `validate_title` 規約をインポートでも踏襲し、空タイトル行の混入を防ぐ。
- **入力値**: `{ "media_type": "anime", "external_id": "12345", "title": "" }`。
  - **不正な理由**: 要件 2.1「title空文字は400」。
  - **実際の発生シナリオ**: 外部検索結果のタイトル欠落。
- **期待される結果**: `error.code == "VALIDATION_ERROR"`、`status == 400`、書き込みなし。
  - **エラーメッセージの内容**: 「titleは空にできません」（既存 `validate_title` 流用）。
  - **システムの安全性**: 書き込み前にreturn。
- **テストの目的**: `title` バリデーションのパリティを確認する。
  - **品質保証の観点**: 既存 `empty_title_returns_validation_error`（item.rs L323-329）との一貫性。
- 🟡 信頼性レベル: 要件 2.1（`CreateItemRequest` の `validate_title` 踏襲）からの妥当な推測。

### TC-0025-E06: 重複インポート（同一media_type+external_id既存）→ 409 ITEM_ALREADY_IMPORTED（実DB）

- **テスト名**: 同一media_type+external_idのitemが既存の状態で再インポートすると409 ITEM_ALREADY_IMPORTEDになり重複作成されない
  - **エラーケースの概要**: 既に取り込み済みの作品（同一 `media_type`+`external_id`）を再度インポートする。
  - **エラー処理の重要性**: DBに `(media_type, external_id)` のUNIQUE制約が**無い**（`idx_items_external_id` は非UNIQUE）ため、アプリ層でSELECT検知しないと重複行が作られライブラリが汚れる。
- **入力値**: 事前に `media_type=anime`/`external_id="12345"` を1件投入 → 同一 `media_type`+`external_id` で再度インポートリクエスト。
  - **不正な理由**: 要件第6章の決定（案A採用）により、重複は409エラーとする。
  - **実際の発生シナリオ**: 利用者が同じ検索結果を二度クリックした／既に取り込み済みと気づかず再取り込みした場合。
- **期待される結果**: `error.code == "ITEM_ALREADY_IMPORTED"`、`status == 409 CONFLICT`、`items` の行数が増えない（再送前後で件数不変）。
  - **エラーメッセージの内容**: 「既にインポート済みです」等の汎用メッセージ。
  - **システムの安全性**: トランザクション内のSELECT検知で早期returnしロールバック、原子性を保つ。
- **テストの目的**: 重複検知ロジック（アプリ層SELECT）と409マッピングを確認する。
  - **確認ポイント**: 重複検知時にitems行が増えないこと（最重要・件数で直接検証）。
- 🟡 信頼性レベル: 要件第6章の決定（案A: 409 `ITEM_ALREADY_IMPORTED`）、TASK-0025.md テストケース3、note.md L185-188 より。要件上は黄（PO確認推奨）だが本書で確定済み。

### TC-0025-E07: ApiErrorCode::ItemAlreadyImported が 409 CONFLICT へマッピングされる（DB非依存ユニット）

- **テスト名**: 新規ApiErrorCode::ItemAlreadyImportedがHTTP 409・ワイヤーコードITEM_ALREADY_IMPORTEDを返す
  - **エラーケースの概要**: 新規追加するエラーコード variant のステータス・文字列マッピング検証。
  - **エラー処理の重要性**: 重複検知（TC-0025-E06）が正しいHTTPステータス・コード文字列で返るための土台。既存 `DuplicateTagName→409` パターン踏襲を保証する。
- **入力値**: `ApiError::new(ApiErrorCode::ItemAlreadyImported, "...")` を `into_response()`。
  - **不正な理由**: （正常系のマッピング検証だが異常系コードの定義として本節に配置）。
  - **実際の発生シナリオ**: 重複インポート時のレスポンス生成。
- **期待される結果**: `response.status() == StatusCode::CONFLICT`（409）、`error.code == "ITEM_ALREADY_IMPORTED"`。
  - **エラーメッセージの内容**: variant定義に依存。
  - **システムの安全性**: 既存409系（`DuplicateTagName` 等）と同一の挙動。
- **テストの目的**: 新規 variant のステータス・文字列マッピングを確認する。
  - **品質保証の観点**: 500（デフォルト誤マッピング）等に落ちないこと。既存 `invalid_provider_returns_400`（response.rs L345-363）と同型のテスト。
- 🔵 信頼性レベル: 要件 3.3、既存 `ApiErrorCode` の409系パターン（response.rs L112, L119, L124-126）に直接対応。

### TC-0025-E08: DB障害時 → 500 INTERNAL_ERROR（情報漏洩なし、実DB or 接続不能プール）

- **テスト名**: create_item_with_source実行中のDBエラーが500 INTERNAL_ERRORへ変換されDB内部情報を漏洩しない
  - **エラーケースの概要**: トランザクション中のDB接続障害・SQL実行失敗。
  - **エラー処理の重要性**: 既存 `db_error`（item_repository.rs L38-43）の方針（詳細はログ、クライアントへは汎用文言）を維持し、スキーマ推測攻撃の材料を与えない。
- **入力値**: 接続不能な `PgPool`（不正接続文字列）に対し `create_item_with_source` を呼ぶ。
  - **不正な理由**: DB到達不能。
  - **実際の発生シナリオ**: DBダウン・コネクション枯渇。
- **期待される結果**: `error.code == "INTERNAL_ERROR"`、`status == 500`、メッセージにSQLエラー詳細・テーブル構造を含まない。
  - **エラーメッセージの内容**: 「アイテムの登録処理に失敗しました」等の固定文言。
  - **システムの安全性**: トランザクションはコミットされず、部分書き込みが残らない。
- **テストの目的**: DBエラーの汎用500変換と情報漏洩防止を確認する。
  - **品質保証の観点**: 既存 `list_items_converts_db_error_to_internal_error`（item_repository.rs L1136-1155）とのパリティ。
- 🟡 信頼性レベル: 要件 3.3（情報漏洩防止NFR）、既存 `db_error` 関数・既存DBエラーテストからの妥当な推測。

---

## 4. 境界値テストケース（最小値・最大値・null等）

### TC-0025-B01: external_id が任意項目を一切伴わない最小構成で成功する（実DB）

- **テスト名**: 必須3項目のみ（任意項目すべて省略）の最小リクエストでインポートが成功する
  - **境界値の意味**: 入力フィールドの「最小集合」境界。任意項目（original_title/description/cover_image_url/release_date/homepage_url/details）をすべて省略しても成立すること。
  - **境界値での動作保証**: Optionフィールドが `None` でもトランザクションが成功する。
- **入力値**: `{ "media_type": "anime", "external_id": "1", "title": "A" }`。
  - **境界値選択の根拠**: 必須項目のみの下限構成。`external_id="1"`（1文字）も最短ID境界を兼ねる。
  - **実際の使用場面**: 外部APIが最小限の情報（ID・タイトル・種別）しか返さない場合。
- **期待される結果**: 201、`Item` 作成成功、任意項目は `None`/NULL。
  - **境界での正確性**: 省略項目がNULLとして正しく登録される。
  - **一貫した動作**: 任意項目ありの TC-0025-N03 と同一の成功経路。
- **テストの目的**: 最小入力での堅牢性を確認する。
  - **堅牢性の確認**: Optionフィールド未指定でpanicやエラーが起きないこと。
- 🟡 信頼性レベル: 要件 2.1 入力仕様表（任意項目）からの妥当な推測（最小構成成功は明記なしだが任意項目定義から導出）。

### TC-0025-B02: details（serde_json::Value）が省略されてもデシリアライズ・登録が成功する

- **テスト名**: detailsフィールド省略時に#[serde(default)]でNone扱いとなりデシリアライズが成功する
  - **境界値の意味**: 任意JSON項目 `details` の有無の境界（None境界）。要件 2.1 で `Option` 化を推奨。
  - **境界値での動作保証**: `details` 未指定でもパース成功し、現状実装（詳細テーブルへは `item_id` のみINSERT）と整合する。
- **入力値**: `{ "media_type": "anime", "external_id": "1", "title": "A" }`（`details` キーなし）。
  - **境界値選択の根拠**: 要件 2.1「現状の `create_item` は詳細テーブルへ `item_id` のみINSERTするため、本タスクでは保持のみ／個別カラム反映は範囲外」を境界で確認する。
  - **実際の使用場面**: 詳細データを送らないシンプルなインポート。
- **期待される結果**: `request.details == None`、デシリアライズ・登録成功。
  - **境界での正確性**: `#[serde(default)]` により欠落がエラーにならない。
  - **一貫した動作**: 既存 `CreateItemRequest.details`（item.rs L82-83）と同一の `Option<serde_json::Value>` + `#[serde(default)]` 方針。
- **テストの目的**: `details` の Option化・default挙動を確認する。
  - **堅牢性の確認**: 任意JSONの欠落で落ちないこと。
- 🟡 信頼性レベル: 要件 2.1 L49-51（`details` のOption化推奨・範囲外明記）、既存 `CreateItemRequest.details` より。

### TC-0025-B03: 異なるmedia_type・同一external_id は重複とみなされず両方作成される（実DB・重複判定境界）

- **テスト名**: external_idが同一でもmedia_typeが異なれば重複扱いされず別レコードとして作成される
  - **境界値の意味**: 重複判定キー `(media_type, external_id)` の境界。「`external_id` のみ一致」では重複としないこと（複合キー判定の正しさ）。
  - **境界値での動作保証**: 重複検知SELECTが `media_type AND external_id` の両条件で行われ、片方だけ一致では409にしない。
- **入力値**: `media_type=anime`/`external_id="100"` を投入後、`media_type=movie`/`external_id="100"` をインポート。
  - **境界値選択の根拠**: 要件第6章「同一 `media_type`+`external_id`」の複合キー定義を境界で検証。異なるプロバイダ間で `external_id` 文字列が偶然衝突しても種別が違えば別作品。
  - **実際の使用場面**: アニメ「100」と映画「100」を両方取り込む。
- **期待される結果**: 2件目も201で作成され、`items` に2行存在（409にならない）。
  - **境界での正確性**: 重複判定が複合キーで正しく行われる。
  - **一貫した動作**: 完全一致（TC-0025-E06）のみ409、片方違いは成功。
- **テストの目的**: 重複判定キーが `(media_type, external_id)` 複合であることを確認する。
  - **堅牢性の確認**: `external_id` 単独一致を誤って重複扱いしないこと。
- 🟡 信頼性レベル: 要件第6章 6.3（重複チェック `WHERE media_type=$1 AND external_id=$2`）からの妥当な推測。

### TC-0025-B04: ルート誤マッチ防止 — /items/import が /items/:id に吸われない（実DB・E2E）

- **テスト名**: POST /items/importがリテラルパスとして優先され/items/:id（UUID必須）に誤マッチしない
  - **境界値の意味**: ルーティングのリテラル/動的パス境界。`"import"` が `/items/:id` の `:id` としてUUIDパースされ400になってはならない。
  - **境界値での動作保証**: `/items/import` がリテラル登録され、POSTは import ハンドラへ到達する。
- **入力値**: `POST /items/import`（有効ボディ）。
  - **境界値選択の根拠**: note.md L46-49・L190-192「リテラルパスを動的パスより前に登録」の境界確認。既存 `/items/search` の誤マッチ防止テスト（routes/mod.rs L300-327）と同型。
  - **実際の使用場面**: 通常のインポートリクエストが正しいハンドラに届くこと。
- **期待される結果**: UUIDパースエラー由来の400（"idはUUID形式である必要があります"）にならない。500でもない。importハンドラの結果（201等）に到達する。
  - **境界での正確性**: リテラルパス優先が機能する。
  - **一貫した動作**: 既存 `/items/search` のルーティング方針と同一。
- **テストの目的**: ルーティング誤マッチ防止を確認する。
  - **堅牢性の確認**: パス設計の安全性。
- 🔵 信頼性レベル: note.md L46-49・L190-192、要件 3.1、既存 `get_items_search_does_not_fall_through_to_item_id_route` より。

---

## 5. テストケース実装時の日本語コメント指針（Rust版・既存パターン踏襲）

各テストは既存実装（handlers/items.rs, item.rs, response.rs）と同一のコメント規約に従う。

### テスト関数冒頭

```rust
/// TC-0025-XXX: <テスト名>
/// 【テスト目的】: <このテストで確認すること>
/// 【テスト内容】: <具体的な処理>
/// 【期待される動作】: <正常時の結果>
/// 🔵🟡🔴 信頼性レベル: <根拠>
```

### Given / When / Then

```rust
// 【テストデータ準備】: <なぜこのデータか>
// 【初期条件設定】: <テスト前の状態>
let value = serde_json::json!({ "media_type": "anime", "external_id": "12345", "title": "鬼滅の刃" });

// 【実際の処理実行】: <呼び出す関数>
// 【処理内容】: <実行内容>
let result = parse_import_item_request(value);

// 【結果検証】: <何を検証するか>
// 【期待値確認】: <期待結果とその理由>
let request = result.unwrap();
assert_eq!(request.external_id, "12345"); // 【確認内容】: external_idが必須項目として保持されることを確認 🔵
```

### セットアップ / クリーンアップ（実DB統合）

```rust
// 【テスト前準備】: DATABASE_URLからテスト用PgPool/AppStateを構築する
// 【環境初期化】: 重複テストでは事前に対象external_idの行をクリーンアップする
// 【テスト後処理】: 投入したテストitemを削除し次テストへ影響させない（または一意external_idでisolation）
```

---

## 6. 要件定義との対応関係

- **参照した機能概要**: 要件 1章（POST /items/import 新設・`source=api`・詳細テーブル作成）→ TC-0025-N03, N06, N07
- **参照した入力・出力仕様**: 要件 2.1（必須 media_type/external_id/title）→ TC-0025-N01, E01〜E05、要件 2.2（201/400/409/500）→ TC-0025-N02, E01〜E08
- **参照した制約条件**: 要件 3.1（create_item再利用・Option B・ルーティング）→ TC-0025-N04, N05, B04、要件 3.2（UNIQUE制約なし→アプリ層SELECT）→ TC-0025-E06, B03、要件 3.3（ITEM_ALREADY_IMPORTED追加・情報漏洩防止）→ TC-0025-E07, E08
- **参照した使用例**: 要件 4.1（正常インポート・TASK-0009一貫性）→ TC-0025-N03, N05、4.2（external_id欠落・media_type不正）→ TC-0025-E01, E04、4.3（重複インポート）→ TC-0025-E06
- **参照した決定事項**: 要件第6章（重複→409 ITEM_ALREADY_IMPORTED、複合キー `(media_type, external_id)`、トランザクション内SELECT）→ TC-0025-E06, E07, B03

### TASK-0025.md タスクファイルテストケースとの対応

| タスクファイル記載 | 本書テストケース |
|---|---|
| TC-002-03（検索結果からitem作成・正常） | TC-0025-N03, N06 |
| external_id欠落時に400 | TC-0025-E01（+ E02, E03 で空文字・空白拡張） |
| 重複external_idのインポート | TC-0025-E06（+ E07 マッピング、B03 複合キー境界） |
| TASK-0009ロジックとの一貫性 | TC-0025-N04, N05 |

---

## 7. テストケース一覧サマリー

| ID | 分類 | 概要 | DB依存 | 信頼性 |
|---|---|---|---|---|
| TC-0025-N01 | 正常 | ImportItemRequest最小デシリアライズ | なし | 🔵 |
| TC-0025-N02 | 正常 | created_responseが201統一形式 | なし | 🔵 |
| TC-0025-N03 | 正常 | create_item_with_source（api）でitems+詳細作成 | あり | 🔵 |
| TC-0025-N04 | 正常 | create_item（manualラッパー）回帰 | あり | 🔵 |
| TC-0025-N05 | 正常 | manual/importパリティ（source/external_idのみ差分） | あり | 🔵 |
| TC-0025-N06 | 正常 | ルーター経由POST /items/importで201 | あり | 🔵 |
| TC-0025-N07 | 正常 | 8種media_type→対応詳細テーブル網羅 | あり/一部なし | 🟡 |
| TC-0025-E01 | 異常 | external_id欠落→400 | なし | 🔵 |
| TC-0025-E02 | 異常 | external_id空文字→400 | なし | 🔵 |
| TC-0025-E03 | 異常 | external_id空白のみ→400 | なし | 🟡 |
| TC-0025-E04 | 異常 | media_type不正→400 | なし | 🟡 |
| TC-0025-E05 | 異常 | title空文字→400 | なし | 🟡 |
| TC-0025-E06 | 異常 | 重複→409 ITEM_ALREADY_IMPORTED・件数不変 | あり | 🟡 |
| TC-0025-E07 | 異常 | ItemAlreadyImported→409マッピング | なし | 🔵 |
| TC-0025-E08 | 異常 | DB障害→500・情報漏洩なし | あり | 🟡 |
| TC-0025-B01 | 境界 | 必須3項目のみの最小構成成功 | あり | 🟡 |
| TC-0025-B02 | 境界 | details省略でdefault None成功 | なし | 🟡 |
| TC-0025-B03 | 境界 | 同一external_id・異media_typeは別作成 | あり | 🟡 |
| TC-0025-B04 | 境界 | /items/importが/items/:idに誤マッチしない | あり | 🔵 |

**合計**: 19ケース（正常7 / 異常8 / 境界4）

---

## 8. 品質判定

| 評価軸 | 状態 |
|---|---|
| テストケース分類 | 正常系・異常系・境界値を網羅（必須欠落/空文字/空白/不正enum/重複/DB障害/最小構成/複合キー境界/ルート誤マッチ） |
| 期待値定義 | 各ケースで HTTP status・error.code・DB副作用（件数）を明確化 |
| 技術選択 | Rust + `#[test]`/`#[tokio::test]` + `oneshot` + `sqlx`（既存パターン確定） |
| 実装可能性 | 既存 `create_item`/`detail_table_name`/`created_response`/`db_error`/ApiErrorCode の再利用で実現可能 |
| 信頼性レベル分布 | 🔵 約47%（9件）/ 🟡 約53%（10件）/ 🔴 0件 |

**総合判定: ✅ 高品質（🔴なし）**
- 🟡 は主に「重複挙動（要件第6章で確定済み・PO確認推奨）」「空白のみ拒否（既存validate_title整合）」「8種網羅・最小構成（既存実装の自然な拡張）」に集中し、いずれも実装方針・根拠が明記済み。

---

## 9. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-red mediavault-backend TASK-0025` でRedフェーズ（失敗テスト作成）を開始します。

### Redフェーズ着手前の確認事項（要件第6章・note.md より確定済み）
1. `create_item_with_source(pool, request, source, external_id)` を新設し `create_item` を薄いラッパー化（Option B）
2. `ImportItemRequest` を `src/models/item_import.rs` に新規定義（`external_id: String` 必須、`details: Option<serde_json::Value>` + `#[serde(default)]`）
3. `ApiErrorCode::ItemAlreadyImported => ("ITEM_ALREADY_IMPORTED", StatusCode::CONFLICT)` を追加
4. 重複チェックはトランザクション内 `SELECT 1 FROM items WHERE media_type=$1 AND external_id=$2 LIMIT 1`
5. `POST /items/import` をリテラルパスとして `/items/:id` より前に登録
