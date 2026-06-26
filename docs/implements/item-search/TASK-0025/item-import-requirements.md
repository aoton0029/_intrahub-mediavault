# TASK-0025 TDD要件定義書: POST /items/import 実装

**機能名**: item-import（外部検索結果からのアイテムインポート）
**タスクID**: TASK-0025
**要件名**: mediavault-backend
**作成日**: 2026-06-26
**出力ファイル**: `docs/implements/item-search/TASK-0025/item-import-requirements.md`

---

## 0. 信頼性レベルの凡例

- 🔵 **青信号**: EARS要件定義書・設計文書・実コードを参照し、ほぼ推測していない
- 🟡 **黄信号**: 要件定義書・設計文書・実コードから妥当な推測をした
- 🔴 **赤信号**: 元資料に根拠がない推測

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: `POST /items/import` エンドポイントを新設し、`GET /items/search`（TASK-0024）で得た外部API検索結果のうち利用者が選択した1件を受け取り、`items` テーブル（`source=api`、`external_id` 必須）＋メディア種別ごとの詳細テーブル（`anime_details` 等）へレコードを作成する。*（TASK-0025.md タスク概要 L16-17、api-endpoints.md POST /items/import より）*
- 🔵 **解決する問題**: 外部API検索で見つけた作品情報を、利用者が手入力し直すことなくワンクリックで自分のメディアライブラリへ取り込めるようにする（手動作成 `POST /items` の外部データ版）。*（requirements.md REQ-002「外部API検索結果からアイテム新規作成」より）*
- 🔵 **想定ユーザー**: MediaVaultの単一利用者（自分のメディアコレクションを管理する個人。セルフホスト前提）。*（architecture.md「単一ユーザー・小規模運用前提」、note.md L22 より）*
- 🔵 **システム内での位置づけ**: レイヤードアーキテクチャ（routes → handlers → services/repositories → db）のHTTP層エンドポイント。Phase3「外部API連携」の最終タスク。`source=manual` の `POST /items`（TASK-0009）のトランザクション処理を再利用し、`source` と `external_id` の設定のみ拡張する。*（architecture.md L20-46、overview.md Phase3、note.md L20-28 より）*

- **参照したEARS要件**: REQ-002（外部API検索結果からアイテム新規作成）、REQ-201b（`source=api` の場合 `external_id` を保持）
- **参照した設計文書**: architecture.md（レイヤード構造 L20-46）、api-endpoints.md（外部API検索・インポート節 POST /items/import）、dataflow.md（機能2: 外部API検索結果からのインポート）

---

## 2. 入力・出力の仕様（EARS機能要件・Rust型定義ベース）

### 2.1 入力（リクエスト）

🔵 **エンドポイント**: `POST /items/import`（認証なし＝内部API用APIキー検証ミドルウェアの対象外、ユーザー向けエンドポイント）。*（note.md L196「APIキー検証不要」より）*

🔵 **リクエストボディ**: `ImportItemRequest`（`application/json`）*（TASK-0025.md 実装詳細1 L32-47、api-endpoints.md より）*

| フィールド | 型 | 必須 | 制約・説明 |
|---|---|---|---|
| `media_type` | `MediaType`（enum, snake_case） | ✅必須 | anime/movie/drama/manga/novel/game/academic_book/paper の8種。不正値は400 🔵 |
| `external_id` | `String` | ✅必須 | 外部API上の一意ID。**空文字・欠落は400 VALIDATION_ERROR** 🔵 |
| `title` | `String` | ✅必須 | 空文字は400（`CreateItemRequest` の `validate_title` 規約踏襲）🟡 |
| `original_title` | `Option<String>` | 任意 | 🔵 |
| `description` | `Option<String>` | 任意 | 🔵 |
| `cover_image_url` | `Option<String>` | 任意 | 🔵 |
| `release_date` | `Option<NaiveDate>` | 任意 | 🔵 |
| `homepage_url` | `Option<String>` | 任意 | 🔵 |
| `details` | `serde_json::Value`（`Option` 化が妥当） | 任意 | メディア別詳細テーブル用データ。現状の `create_item` は詳細テーブルへ `item_id` のみINSERTするため、本タスクでは保持のみ／個別カラム反映は範囲外 🟡 |

- 🟡 **実装上の判断**: TASK-0025.md L45 は `details: serde_json::Value`（非Option）だが、既存 `CreateItemRequest.details` は `#[serde(default)] Option<serde_json::Value>` であり、整合のため `Option` 化を推奨。*（models/item.rs L81-83 より妥当推測）*

### 2.2 出力（レスポンス）

🔵 **成功時**: HTTP `201 Created`、統一フォーマット `ApiOk<Item>`（`{ "success": true, "data": <Item> }`）。`Item` には DB採番された `id`、`source="api"`、`external_id=<入力値>`、`created_at`/`updated_at` 等が含まれる。*（TASK-0025.md 実装詳細3 L57、既存 `created_response` handlers/items.rs L49-52、api-endpoints.md より）*

🔵 **エラー時**: 統一エラーフォーマット `ApiError`（`{ "success": false, "error": { "code", "message" } }`）。*（models/response.rs L36-48 より）*

| 状況 | HTTP | code |
|---|---|---|
| `external_id` 欠落・空文字 | 400 | `VALIDATION_ERROR` 🔵 |
| `media_type` 不正・`title` 空文字 | 400 | `VALIDATION_ERROR` 🟡 |
| 同一 `media_type`+`external_id` が既存（重複） | 409 | `ITEM_ALREADY_IMPORTED`（**決定事項。下記第6章参照**）🟡 |
| DB障害等 | 500 | `INTERNAL_ERROR` 🔵 |

### 2.3 データフロー

🔵 クライアント → `POST /items/import` → `import_item_handler`（バリデーション＋`ImportItemRequest`→中間入力型へ変換）→ 重複チェック → リポジトリのトランザクション（`items` INSERT → 詳細テーブル INSERT → commit）→ `Item` を201で返却。*（dataflow.md 機能2、note.md L123-125 より）*

- **参照したEARS要件**: REQ-002、REQ-201b
- **参照した設計文書**: api-endpoints.md（POST /items/import）、types.rs（`Item`/`ItemSource`/`MediaType`）、models/item.rs（実装済み `Item` L46-64, `CreateItemRequest` L70-84）

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

### 3.1 アーキテクチャ・実装制約

- 🔵 **TASK-0009ロジックの再利用（コード重複禁止）**: `POST /items`（manual）と `POST /items/import`（api）は同一のトランザクション処理（`items`＋詳細テーブル同時INSERT）を経由しなければならない。`source`/`external_id` のみが分岐点。*（TASK-0025.md 完了条件 L27、実装詳細2 L50-53 より）*
  - 🟡 **現状の制約**: 実コードの `item_repository::create_item`（item_repository.rs L51-94）は `source='manual', external_id=NULL` を **SQLリテラルでハードコード**している。そのまま再利用できないため拡張が必要。**推奨方針（note.md L237-238 Option B）**: 既存 `create_item` を変更せず、`source`/`external_id` を引数で受け取る内部関数（例: `create_item_with_source(pool, request, source, external_id)`）を追加し、既存 `create_item` をその薄いラッパーにする。これにより `POST /items` の既存テストを壊さない。
- 🔵 **ルーティング登録**: `POST /items/import` は `routes/mod.rs` の `build_router` 内へフラットに追記する。タスクファイル記載の `routes/items.rs` は **実在しない**（note.md L13-14、TASK-0024開発ノート L13-14 で確認済み）。リテラルパス `/items/import` は動的パス `/items/:id` より **前** に登録する（Axum 0.8はリテラル優先で実害は低いが安全側に倣う）。*（note.md L46-49, L190-192 より）*
- 🔵 **DTO配置**: `ImportItemRequest` は新規ファイル `backend/mediavault-api/src/models/item_import.rs` に定義する。*（TASK-0025.md 実装詳細1 L48 より）*

### 3.2 データベース制約

- 🔵 **CHECK制約**: `items` に `chk_items_source_external_id`（`(source='manual') OR (source='api' AND external_id IS NOT NULL)`）が存在する（database-schema.sql L64-66）。よって `source=api` で `external_id=NULL` を渡すとDB側でも違反するが、**アプリ層で先に400バリデーションを行う**ことでDB到達前に弾く（情報漏洩・無駄なトランザクション回避）。
- 🟡 **一意制約は存在しない（重複判定の前提）**: `external_id` には `idx_items_external_id`（**非UNIQUE** の通常INDEX、database-schema.sql L73）のみ。`(media_type, external_id)` のUNIQUE制約は **無い**。したがって重複検知はDBの一意制約違反に頼れず、**アプリ層で明示的にSELECTして判定する必要がある**。*（database-schema.sql L73、grep確認より＝この発見が第6章の決定の根拠）*

### 3.3 エラーコード・レスポンス制約

- 🔵 **新規 `ApiErrorCode` variant が必要**: 重複時の `ITEM_ALREADY_IMPORTED`（409 CONFLICT）は現状 `models/response.rs` の `ApiErrorCode` に存在しない。既存 `DuplicateTagName`→409 等のパターン（L112, L119）を踏襲し新規variant `ItemAlreadyImported => ("ITEM_ALREADY_IMPORTED", StatusCode::CONFLICT)` を追加する。*（response.rs L100-135、note.md L114-116, L247-248 より）*
- 🔵 **情報漏洩防止（NFR）**: エラーレスポンスにDB内部情報・SQLエラー詳細を含めない。既存 `db_error`（item_repository.rs L38-43）が `tracing::error!` でログ出力し、クライアントへは汎用メッセージのみ返す方針を踏襲する。*（architecture.md L37-39、note.md L200 より）*

### 3.4 パフォーマンス・セキュリティ

- 🟡 **パフォーマンス**: 単一ユーザー・セルフホスト前提のため厳密なスループット要件はない。重複チェックSELECTは `idx_items_external_id` を活用できる。*（note.md L201-203 より）*
- 🔵 **SQLインジェクション防止**: 詳細テーブル名は `detail_table_name()`（item_repository.rs L18-30）の固定文字列matchで解決済み。外部入力は全て `bind` 経由。*（note.md L201-202 より）*

- **参照したEARS要件**: REQ-002, REQ-201b, NFR（情報漏洩防止）
- **参照した設計文書**: database-schema.sql（items L58-73, CHECK制約・index）、architecture.md（L20-46）、response.rs（ApiErrorCode）、item_repository.rs（create_item L51-94）

---

## 4. 想定される使用例（EARSエッジケース・データフローベース）

### 4.1 基本パターン（正常系）

- 🔵 **TC-002-03**: `media_type=anime`、`external_id="12345"`、`title` ありの有効リクエスト → 201、`items`（`source=api, external_id=12345`）＋`anime_details`（`item_id`）が作成される。*（acceptance-criteria.md TC-002-03、TASK-0025.md テストケース1 L77-81 より）*
- 🔵 **TASK-0009一貫性**: `POST /items`（manual）と `POST /items/import`（api）に同等の詳細データを渡すと、両者とも同一トランザクション処理を経由し、`source`/`external_id` のみ異なる `Item` が作成される。*（TASK-0025.md テストケース4 L95-99 より）*

### 4.2 エラーケース

- 🔵 **external_id欠落 → 400**: `external_id` を含まない／空文字のリクエスト → 400 `VALIDATION_ERROR`、`items` への書き込みは発生しない。*（TASK-0025.md テストケース2 L83-87、api-endpoints.md「external_id必須」より）*
- 🟡 **media_type不正 → 400**: 未知の `media_type` 文字列はデシリアライズ失敗→400。*（CreateItem系の既存規約から妥当推測）*

### 4.3 エッジケース（重複インポート）

- 🟡 **重複external_idのインポート（TC-002相当）**: 同一 `media_type`+`external_id` のitemが既存の状態で同じデータを再送 → 重複作成は行われず、決定方針（**409 `ITEM_ALREADY_IMPORTED`**）に従うレスポンスが返る。`items` の行数は増えない。*（TASK-0025.md テストケース3 L89-93、第6章の決定より）*

- **参照したEARS要件**: REQ-002
- **参照した設計文書**: dataflow.md（機能2フロー）、acceptance-criteria.md（TC-002-03）、api-endpoints.md（エラーコード）

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: 外部API検索結果からのアイテム取り込み（user-stories.md 機能2相当）
- **参照した機能要件**: REQ-002（外部API検索結果からアイテム新規作成）
- **参照した状態要件**: REQ-201b（`source=api` の場合 `external_id` を保持）
- **参照したEdgeケース**: 重複インポート（要件定義に明記なし＝🟡、本書第6章で決定）
- **参照した受け入れ基準**: TC-002-03（検索結果からのitem作成）
- **参照した設計文書**:
  - **アーキテクチャ**: architecture.md L20-46（レイヤード構造・情報漏洩防止 L37-39）
  - **データフロー**: dataflow.md 機能2（外部API検索結果からのインポート）
  - **型定義**: types.rs / models/item.rs（`Item` L46-64, `ItemSource` L37-43, `MediaType` L15-24, `CreateItemRequest` L70-84）
  - **データベース**: database-schema.sql（items L58-73, `chk_items_source_external_id` L64-66, `idx_items_external_id` L73）
  - **API仕様**: api-endpoints.md（POST /items/import）
- **参照した実コード**:
  - `backend/mediavault-api/src/repositories/item_repository.rs`（`create_item` L51-94, `db_error` L38-43, `detail_table_name` L18-30）
  - `backend/mediavault-api/src/handlers/items.rs`（`create_item_handler` L27-42, `created_response` L49-52）
  - `backend/mediavault-api/src/models/response.rs`（`ApiErrorCode` L52-135, `ApiOk`/`ApiError`）
  - `backend/mediavault-api/src/routes/mod.rs`（`build_router`）

---

## 6. 🟡 未解決事項の決定: 重複 external_id の扱い

TASK-0025.md L26/L65-68/L116 および note.md L185-188, L240-242 で「要件未確定」とされていた **同一 `media_type`+`external_id` の重複インポート時の挙動** について、本要件定義で以下のとおり決定する。

### 6.1 決定内容

🟡 **方針: 重複検知時は `409 ITEM_ALREADY_IMPORTED` を返し、重複レコードを作成しない（安全側）**

| 検討案 | 採否 | 理由 |
|---|---|---|
| (A) 409エラーを返す（既存レコード作成しない） | ✅**採用** | 同一作品の二重登録を防止。利用者に「既に取り込み済み」を明示でき、データ整合性が最も安全。TASK-0025.md L67 の提案方針と一致 |
| (B) 既存レコードを返す（冪等的に200/201） | 不採用 | 冪等性は魅力的だが、ユーザーが意図せず別バージョンを上書き取得したと誤認するリスク。要件に冪等性要求の記載なし |
| (C) 重複作成を許容 | 不採用 | 同一作品が複数行できライブラリが汚れる。`idx_items_external_id` が重複防止目的（schema L73コメント）である意図に反する |

### 6.2 決定の根拠

- 🟡 DBに `(media_type, external_id)` のUNIQUE制約が **存在しない**（database-schema.sql L73 は非UNIQUE index）ため、重複防止はアプリ層の責務である。
- 🔵 schema L73 のコメント「重複インポート防止の観点から妥当な推測」が、重複を防ぐ設計意図を示している。
- 🟡 TASK-0025.md L67 が「安全側として既存レコードがあれば 409 `ITEM_ALREADY_IMPORTED` 相当のエラーを返す方針を提案」しており、これを正式採用する。

### 6.3 実装方針（重複チェック）

- 🟡 トランザクション内で `items` INSERT 前に `SELECT 1 FROM items WHERE media_type=$1 AND external_id=$2 LIMIT 1` を実行し、存在すれば `ItemAlreadyImported` エラーで早期return（ロールバック）する。
  - 競合（同時2リクエスト）は単一ユーザー前提のため実害は極小。より厳密にするなら将来マイグレーションで `(media_type, external_id) WHERE source='api'` の部分UNIQUE制約を追加し、一意制約違反を409へマッピングする方式へ移行できる（本タスク範囲外の改善メモ）。
- 🔵 重複チェック・INSERTは同一トランザクション内で行い、原子性を保つ。

### 6.4 残課題・申し送り

- 🟡 **プロダクトオーナー確認推奨**: TASK-0025.md L116 に従い、(A)409 とする本決定は実装前に確認することが望ましい。確認結果により (B) 冪等返却へ変更する余地を残す。
- 🟡 `ApiErrorCode::ItemAlreadyImported`（409）の新規追加が必要（第3.3章）。

---

## 7. 品質判定

| 評価軸 | 状態 |
|---|---|
| 要件の曖昧さ | ほぼなし（重複挙動を第6章で確定） |
| 入出力定義 | 完全（リクエスト/レスポンス/エラー網羅） |
| 制約条件 | 明確（DB制約・実コード制約・エラーコード追加を特定） |
| 実装可能性 | 確実（再利用元 `create_item` を実コードで確認済み） |
| 信頼性レベル分布 | 🔵 約60% / 🟡 約40% / 🔴 0% |

**総合判定: ✅ 高品質（要改善点なし、🔴なし）**
- 主な🟡は「重複挙動（第6章で決定済み、PO確認推奨）」「`details` のカラム反映範囲」「`details` の `Option` 化」に集中し、いずれも実装方針が明記されている。

---

## 8. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-testcases mediavault-backend TASK-0025` でテストケースの洗い出しを行います。
