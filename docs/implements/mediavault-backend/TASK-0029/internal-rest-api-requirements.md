# TASK-0029: 内部REST APIルート群実装（/internal/items等） - TDD要件定義書

- **機能名**: internal-rest-api（内部REST APIルート群）
- **タスクID**: TASK-0029
- **要件名**: mediavault-backend
- **作成日**: 2026-06-26
- **フェーズ**: Phase 5 - 内部API・インポート
- **タスクタイプ**: TDD

**【信頼性レベル凡例】**:
- 🔵 **青信号**: EARS要件定義書・設計文書を参考にしてほぼ推測していない
- 🟡 **黄信号**: EARS要件定義書・設計文書から妥当な推測
- 🔴 **赤信号**: EARS要件定義書・設計文書にない推測

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: 巡回バッチ・ファイルサーバー監視プロセスなどの内部プロセスから利用される `/internal/*` ルート群を実装する。Phase2で実装済みのitems/groups/episodesハンドラロジック、Phase4で実装済みのファイル登録ロジックを再利用し、TASK-0006のAPIキー検証ミドルウェアを適用した専用ルーターにマウントする。
  - *参照: docs/tasks/mediavault-backend/TASK-0029.md「タスク概要」、note.md「4. 設計文書」*

- 🔵 **どのような問題を解決するか**: 利用者向けエンドポイント（`/api/v1/*`）はユーザー認証を持たない単一ユーザー前提だが、外部プロセス（巡回バッチ・監視プロセス）からの自動書き込みには認証境界が必要。`/internal/*` をAPIキー1本で保護することで、外部プロセスからの安全なメタデータ同期・ファイル登録を可能にする。
  - *参照: docs/design/mediavault-backend/api-endpoints.md「内部REST API」、architecture.md「セキュリティ」*

- 🔵 **想定されるユーザー（As a）**: 巡回バッチプロセス・ファイルサーバー監視プロセス（人間ユーザーではなく内部自動プロセス）。user-stories 6.1 に対応。
  - *参照: docs/tasks/mediavault-backend/TASK-0029.md 信頼性レベル「user-stories 6.1」*

- 🔵 **システム内での位置づけ**: レイヤードアーキテクチャ（routes → handlers → (services) → repositories → db/sqlx）の routes・handlers 層に位置づけられる。利用者向けルーター（`/api/v1`）とは `Router::merge` で統合され、認証方式（APIキー1本）のみが異なる。DB層・リポジトリ層は既存実装をそのまま利用し、本タスクでの新規実装はハンドラ・ルーティング層のみ。
  - *参照: architecture.md「アーキテクチャパターン」「システム構成図」、note.md「1. 技術スタック」*

- **参照したEARS要件**: REQ-018, REQ-403, NFR-101, user-stories 6.1
- **参照した設計文書**: architecture.md「コンポーネント構成 / APIサーバー」「セキュリティ」、api-endpoints.md「内部REST API」

---

## 2. 入力・出力の仕様（EARS機能要件・型定義ベース）

### 2.1 共通仕様

- 🔵 **認証ヘッダー（全エンドポイント必須）**:
  ```http
  Authorization: Bearer {INTERNAL_API_KEY}
  ```
  未設定・不一致の場合は `401 Unauthorized`。`AppState.internal_api_key` と照合する。
  - *参照: api-endpoints.md「認証」、note.md「2. 開発ルール / APIキー認証」*

- 🔵 **成功レスポンス形式**: `{ "success": true, "data": {...} }`（`ApiOk` 構造体）。一覧系は `pagination` フィールド付き。
- 🟡 **エラーレスポンス形式**: `{ "success": false, "error": { "code": "...", "message": "..." } }`（`ApiError`）。
  - *参照: api-endpoints.md「エラーレスポンス共通フォーマット」（🟡推測）、note.md「2. 開発ルール / レスポンス形式」*

- 🔵 **ステータスコード規約**: 作成時 201、更新時 200、削除時 204、入力エラー 400、認証失敗 401、リソース不在 404。
  - *参照: note.md「2. 開発ルール / レスポンス形式」*

### 2.2 エンドポイント別 入出力仕様

| # | メソッド・パス | 入力 | 出力（成功） | 主なエラー |
|---|---|---|---|---|
| 1 | POST /internal/items | `CreateItemRequest`（body）🔵 | 作成済みitem, 201 🔵 | `VALIDATION_ERROR`(400) 🔵 |
| 2 | PATCH /internal/items/:id | path: item_id(UUID), body: `UpdateItemRequest` 🔵 | 更新後item, 200 🔵 | `ITEM_NOT_FOUND`(404), `VALIDATION_ERROR`(400) 🔵 |
| 3 | GET /internal/items/search | query: `title`,`media_type`,`tag_ids`,`external_id`,`page`,`limit`（全optional）🔵 | item配列 + pagination, 200 🔵 | （なし、未指定時は全件）🟡 |
| 4 | POST /internal/items/:id/groups | path: item_id(UUID), body: `CreateItemGroupRequest` 🔵 | 作成/更新済みグループ, 201/200 🔵 | `ITEM_NOT_FOUND`(404) 🔵 |
| 5 | POST /internal/groups/:group_id/episodes | path: group_id(UUID), body: `CreateItemEpisodeRequest` 🔵 | 作成/更新済みエピソード, 201/200 🔵 | 404（group不在）🔵 |
| 6 | POST /internal/items/:id/files | path: item_id(UUID), body: `CreateItemFileRequest`（パス指定方式）🔵 | 登録済みファイル, 201 🔵 | `ITEM_NOT_FOUND`(404) 🔵 |

- *参照: api-endpoints.md「内部REST API」各エンドポイント、note.md「4. 設計文書 / API仕様」*

### 2.3 入出力の関係性・データフロー

- 🔵 **DTO再利用**: 入力DTO（`CreateItemRequest`, `UpdateItemRequest`, `CreateItemGroupRequest`, `CreateItemEpisodeRequest`, `CreateItemFileRequest`）はすべて既存定義を再利用する（TASK-0008/0018/0019/0026）。
- 🔵 **検索クエリ**: `GET /internal/items/search` はTASK-0010の `list_items_handler` の検索ロジック（QueryBuilderによる動的WHERE句）を再利用する。クエリパラメータ未指定時はページネーション付きで全件返す。
- 🟡 **連鎖フロー（統合テスト想定）**: `POST /internal/items` → `GET /internal/items/search` で登録結果が検索に反映される。`POST /internal/items/:id/groups` → `POST /internal/groups/:group_id/episodes` で巡回バッチの話数同期フローが成立する。
  - *参照: TASK-0029.md「統合テスト要件」*

- **参照したEARS要件**: REQ-018, REQ-403, REQ-019
- **参照した設計文書**: api-endpoints.md「内部REST API」、models/item.rs・item_file.rs（既存DTO定義）

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

### 3.1 セキュリティ要件

- 🔵 **APIキー必須**: `/internal/*` 配下の全ルートにTASK-0006の `api_key_auth` ミドルウェアを `.layer(axum::middleware::from_fn_with_state(state.clone(), api_key_auth))` で適用する（REQ-403, NFR-101）。
- 🔵 **401時のハンドラ非実行**: APIキー未設定・不一致の場合、ミドルウェアが `401 Unauthorized` を返し、ハンドラ本体は実行されないこと（TC-018-E01）。
- 🔵 **DB内部情報の非露出**: sqlx::Error はクライアントへ内部情報を含めず統一エラーコードへ変換し、詳細は `tracing::error!` でサーバーログのみに出力する。
- 🔵 **APIキー非コミット**: APIキーはリポジトリにコミットしない（`.env` 管理, REQ-404）。
  - *参照: architecture.md「セキュリティ制約」、note.md「6. 注意事項 / セキュリティ・パフォーマンス要件」*

### 3.2 アーキテクチャ制約

- 🔵 **バージョンプレフィックスなし**: `/internal` ルーターは `/api/v1` のようなバージョンプレフィックスを持たない。ルートに直接 `/internal` でマウントする（`/api/v1/internal` にしない）。
- 🔵 **ルーター分離・統合**: 利用者向けルーター（`/api/v1`）と内部ルーター（`/internal`）は `Router::merge` で分離・統合する。
- 🔵 **ハンドラ共通化方針**: `/api/v1` と `/internal` で同一ハンドラ関数を共用すること（選択肢A・推奨）。当面は重複実装を避け、必要に応じて分岐を検討する。
- 🔵 **DB層変更不要**: リポジトリ層は既に完成。新規実装はハンドラ・ルーティング層のみ。
  - *参照: TASK-0029.md「実装詳細」「注意事項」、note.md「6. 注意事項 / 技術的制約」*

### 3.3 パフォーマンス要件

- 🟡 **ページネーション**: リスト/検索APIは `page`/`limit`（デフォルト page=1, limit=20, 最大100）で大量件数応答を防止する。
- 🟡 **応答時間**: 数千件規模のitemsで1秒以内応答（NFR-002）。コネクションプールは最大5〜10接続程度。
  - *参照: api-endpoints.md「ページネーション」（🟡）、architecture.md「パフォーマンス」（🟡）*

### 3.4 データ整合性・トランザクション制約

- 🔵 **Upsert処理**: グループ・エピソードは既存なら更新、なければ新規作成するupsert振る舞いとする。
- 🔵 **トランザクション原子性**: 複数テーブルへの操作は `sqlx::Transaction<Postgres>` でアトミックに行い、失敗時は自動ロールバックで一貫性を保証する。
  - *参照: note.md「2. 開発ルール / トランザクション処理」「6. Upsert処理の一貫性保証」*

### 3.5 互換性・対象外

- 🔵 **パス指定方式のみ**: 内部APIのファイル登録は「既存パス指定方式」（REQ-019）のみ。バイナリ直接アップロードは利用者向けエンドポイント（`/items/:id/files/upload`）のみで提供。
- 🔴 **レート制限**: 本要件では不要（単一ユーザー・セルフホスト前提）。将来拡張点としてモジュール化が望ましい。
  - *参照: TASK-0029.md「実装詳細5」「注意事項」、api-endpoints.md「レート制限」（🔴）*

- **参照したEARS要件**: REQ-018, REQ-019, REQ-403, REQ-404, NFR-101, NFR-002
- **参照した設計文書**: architecture.md「セキュリティ制約」「技術的制約」、api-endpoints.md「認証」「バージョニング」「レート制限」

---

## 4. 想定される使用例（Edgeケース・データフローベース）

### 4.1 基本的な使用パターン（正常系）

- 🔵 **アイテム新規登録**: 巡回バッチが正しいAPIキーで `POST /internal/items` を呼び、`201 Created` で作成済みitemを受け取る（TC-018-01）。
- 🔵 **メタデータ部分更新**: 既存item_idに対し `PATCH /internal/items/:id` で `UpdateItemRequest` を送り、`200 OK` で更新後itemを受け取る。
- 🔵 **条件検索**: `GET /internal/items/search?title=...&media_type=...` でフィルタ済み一覧を取得。
- 🟡 **クエリ未指定検索**: `GET /internal/items/search`（パラメータなし）でページネーション付き全件一覧を取得（TC-018-04）。
- 🔵 **話数同期フロー**: `POST /internal/items/:id/groups`（グループupsert）→ `POST /internal/groups/:group_id/episodes`（エピソードupsert）の連鎖でバッチ同期を実行。
- 🔵 **ファイル登録**: 監視プロセスが新規ファイル検知時に `POST /internal/items/:id/files` でファイルサーバー上のパスを紐付け登録。

### 4.2 エッジ・エラーケース

- 🔵 **APIキー不一致/未設定 → 401**: 誤った値または未設定の `Authorization` ヘッダーで `/internal/*` を呼ぶと `401 Unauthorized` が返り、ハンドラ本体は実行されない（TC-018-E01）。
- 🔵 **存在しないitem_id → 404**: 正しいAPIキー + 存在しないitem_idで `PATCH /internal/items/:id` または `POST /internal/items/:id/groups` を呼ぶと `404 Not Found`（`ITEM_NOT_FOUND`）が返る（TC-018-E02）。
- 🔵 **存在しないgroup_id → 404**: `POST /internal/groups/:group_id/episodes` で存在しないgroup_idを指定すると `404 Not Found`。
- 🔵 **入力検証エラー → 400**: `POST /internal/items` で不正なmedia_type等を送ると `VALIDATION_ERROR`（400）。

- **参照したEdgeケース**: TC-018-E01, TC-018-E02
- **参照した設計文書**: api-endpoints.md「内部REST API」、note.md「4. 設計文書 / テスト対応」

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: user-stories 6.1（巡回バッチ・監視プロセスからの内部API利用）
- **参照した機能要件**:
  - REQ-018（内部REST API）
  - REQ-403（内部APIはAPIキー必須）
  - REQ-404（APIキー非コミット）
  - REQ-019（ファイルサーバー上の既存パス指定登録）
  - REQ-401（利用者向けはユーザー認証なし・単一ユーザー前提）
- **参照した非機能要件**:
  - NFR-101（APIキーヘッダー検証）
  - NFR-002（数千件で1秒以内応答）
- **参照したEdgeケース**: TC-018-E01（APIキー不一致401）, TC-018-E02（存在しないID 404）
- **参照した受け入れ基準**:
  - TC-018-01: APIキー検証ミドルウェア適用確認（正しいキーで201）
  - TC-018-E01: APIキー不一致での401（ハンドラ非実行）
  - TC-018-E02: 存在しないitem_idでの404
  - TC-018-04: 検索クエリ未指定時の全件取得（🟡）
- **参照した設計文書**:
  - **アーキテクチャ**: architecture.md「アーキテクチャパターン」「コンポーネント構成 / APIサーバー」「システム構成図」「セキュリティ制約」
  - **API仕様**: api-endpoints.md「認証」「内部REST API」「バージョニング」「レート制限」
  - **型定義（既存DTO）**: models/item.rs（`CreateItemRequest`/`UpdateItemRequest`）, models/item_file.rs（`CreateItemFileRequest`）, グループ・エピソードDTO（TASK-0018/0019）
  - **タスクノート**: docs/implements/mediavault-backend/TASK-0029/note.md

---

## 6. 実装対象ファイル（参考）

| 区分 | ファイルパス | 内容 |
|---|---|---|
| ルーター | `backend/mediavault-api/src/routes/internal.rs`（新規） | `/internal` 専用ルーター生成・APIキーミドルウェア適用 |
| ルーター統合 | `backend/mediavault-api/src/routes/mod.rs`（変更） | `build_router()` で `/internal` ルーターをmerge |
| ハンドラ | `backend/mediavault-api/src/handlers/internal_items.rs` 等（新規 or 既存共用） | 既存 `create_item_handler`/`update_item_handler`/`list_items_handler` 等を再利用 |
| アプリ構築 | `backend/mediavault-api/src/main.rs`（変更） | ルーターマージ・`AppState.internal_api_key` 受け渡し |

- 🔵 *参照: TASK-0029.md「実装詳細 / 実装ファイル」、note.md「2. コード規約 / モジュール構成」*
- 注: ハンドラは「選択肢A（共通ハンドラ共用）」を優先するため、`internal_*.rs` の新規作成は最小限とし、既存ハンドラをルーターから直接マウントする方針を基本とする。

---

## 7. 品質判定結果

```
✅ 高品質:
- 要件の曖昧さ: なし（エンドポイント・入出力・エラーが明確）
- 入出力定義: 完全（6エンドポイントすべてに入力DTO・出力・エラーコードを定義）
- 制約条件: 明確（認証・ルーター分離・トランザクション・対象外を明記）
- 実装可能性: 確実（既存Phase2/Phase4実装の再利用が中心、新規はルーティング・薄いハンドラのみ）
- 信頼性レベル: 🔵が大多数
```

### 信頼性レベル分布

| カテゴリ | 🔵 | 🟡 | 🔴 | 合計 |
|---|---|---|---|---|
| 機能の概要 | 5 | 0 | 0 | 5 |
| 入出力仕様 | 11 | 3 | 0 | 14 |
| 制約条件 | 9 | 3 | 1 | 13 |
| 使用例 | 9 | 1 | 0 | 10 |

- 🟡（黄信号）: エラーレスポンス共通形式・ページネーション仕様・検索クエリ未指定時挙動（TC-018-04）。いずれも既存実装パターン・一般的API設計から妥当な推測。
- 🔴（赤信号）: レート制限非導入の判断のみ（本要件では対象外と明記済み）。

**総合評価**: 高品質。要件・設計文書・既存コードとの対応が明確で、実装可能性が確実。

---

## 次のステップ

次のお勧めステップ: `/tsumiki:tdd-testcases mediavault-backend TASK-0029` でテストケースの洗い出しを行います。
