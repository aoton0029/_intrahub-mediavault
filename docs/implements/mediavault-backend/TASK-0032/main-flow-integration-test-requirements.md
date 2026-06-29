# TASK-0032 要件定義書: 主要フロー統合テスト実装

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 Phase1〜5で実装済みの各REST APIエンドポイントに対し、実PostgreSQLテストDBを用いた`cargo test`統合テストを新設し、複数ハンドラ・複数テーブルをまたいだ整合性を担保する。
- 🔵 単体テストではカバーされない「手動追加→一覧→詳細→外部API検索→インポート→ファイル登録（パス指定／アップロード）→カスケード削除」という主要シナリオの結合動作を検証する。
- 🔵 想定ユーザー: 本プロジェクトの開発者・CI（TASK-0033で同一構成のテストDBが起動される前提）。
- 🔵 システム内での位置づけ: Phase6（統合テスト・仕上げ）の最初のタスク。TASK-0033（CI設定）の前提。
- **参照したEARS要件**: acceptance-criteria.md 全体（TC-001系, TC-002系, TC-007/019系, TC-016/017系, TC-018系, TC-EDGE-101-01）
- **参照した設計文書**: docs/tasks/mediavault-backend/TASK-0032.md, docs/design/mediavault-backend/dataflow.md（主要シーケンス）

## 2. 入力・出力の仕様

- 🔵 対象エンドポイント（実装済み・ルート確認済み: backend/mediavault-api/src/routes/mod.rs, routes/internal.rs）
  - `POST /items`, `GET /items`, `GET /items/:id`, `PATCH /items/:id`, `DELETE /items/:id`
  - `GET /items/search`（外部API検索、wiremockでスタブ化）, `POST /items/import`
  - `POST /items/:id/files`（パス指定）, `POST /items/:id/files/upload`（multipart）
  - `POST /items/:id/groups`, `POST /groups/:group_id/episodes`（EDGE-101対象）
  - `/internal/items`系（`api_key_auth`ミドルウェア配下）
- 🔵 入力: 各エンドポイントの既存`CreateItemRequest`/`UpdateItemRequest`/`CreateItemFileRequest`/`multipart`ボディ等（既存モデル定義をそのまま使用、新規DTOなし）。
- 🔵 出力: 既存`ApiOk<T>`/`ApiError`レスポンス形式（response.rs）。テストではHTTPステータスコードとJSONボディ、加えて直接SQLでのDB状態確認（カスケード削除確認等）を行う。
- 🔵 データフロー: クライアント→Axumルーター→ハンドラ→リポジトリ→Postgres、の既存フローをテストコードから`tower::ServiceExt::oneshot`または実サーバー起動経由で検証する。
- **参照した設計文書**: backend/mediavault-api/src/models/item_file.rs, backend/mediavault-api/src/handlers/item_episodes.rs:21-49

## 3. 制約条件

- 🔵 外部API（Jikan/TMDb/IGDB/NDL/OpenLibrary/Steam等）への実通信は禁止。`ExternalSearchService::with_test_base_urls()` + `wiremock`（既存dev-dependency）でスタブ化する（TASK-0032.md注意事項）。
- 🔵 テスト用DB接続情報は環境変数（`DATABASE_URL`、既存`test_app_state()`パターンを踏襲）から取得する。CI（TASK-0033）でも同一構成を前提とする。
- 🔵 カスケード削除の確認は、削除後に関連テーブル（`item_tags`/`item_links`/`item_files`等）へ直接SELECTし0件であることを検証する形で行う。
- 🔵 内部APIキー認証は`std::env::var("INTERNAL_API_KEY")`比較方式（middleware/api_key_auth.rs）。テストでは`std::env::set_var`で値を設定する（既存internal.rsパターン踏襲、Rust新版でunsafe化されている点に注意）。
- 🔵 既存の実DB依存テストは`#[ignore]`付与・`cargo test -- --ignored`実行が規約。本タスクの統合テストも同方針を継続する。
- **参照したEARS要件**: TASK-0032.md「注意事項」セクション
- **参照した設計文書**: backend/mediavault-api/src/routes/mod.rs:186-196, backend/mediavault-api/src/routes/internal.rs:63-83

## 4. 想定される使用例

- 🔵 統合テスト1: 手動追加→一覧→PATCH更新→DELETE削除→関連テーブルカスケード削除確認（TC-001-01/02/03相当）
- 🔵 統合テスト2: 外部API検索（モック）→インポートで`source=api`/`external_id`一致確認（TC-002-01〜03相当）
- 🔵 統合テスト3: ファイル登録（パス指定・アップロード両方式）（TC-007-01, TC-019-01相当）
- 🔵 統合テスト4: EDGE-101（`group_type=volume`への episode 登録が400で拒否）（TC-EDGE-101-01相当）
- 🔵 統合テスト5: 内部APIキー認証（正しいキー／キーなし／誤りキーの3パターン）（TC-018-01, TC-018-E01, TC-018-E02相当）
- **参照したEARS要件**: acceptance-criteria.md L31-47（items基本フロー）, L67-69（検索）, L207-208（files）, L242-244（EDGE-101）, L183（内部API認証）

## 5. EARS要件・設計文書との対応関係

- **参照した受け入れ基準**: TC-001-01/02/03, TC-002-01/02/03, TC-007-01, TC-019-01, TC-EDGE-101-01, TC-018-01（TC-018-E01/E02は受け入れ基準書に明示の異常系項番なしのため、本タスクファイルの完了条件記述に基づき追加で実装する 🟡）
- **参照した設計文書**:
  - ルート定義: backend/mediavault-api/src/routes/mod.rs, backend/mediavault-api/src/routes/internal.rs
  - 認証ミドルウェア: backend/mediavault-api/src/middleware/api_key_auth.rs
  - 外部検索テストDI: backend/mediavault-api/src/services/external_search.rs:136-190
  - エラーコード: backend/mediavault-api/src/models/response.rs

## 品質判定

✅ 高品質
- 要件の曖昧さ: なし（対象エンドポイント・検証内容はTASK-0032.mdとacceptance-criteria.mdに明記）
- 入出力定義: 完全（既存実装済みモデル・レスポンス形式を利用、新規DTOなし）
- 制約条件: 明確（外部API禁止・DB環境変数・カスケード削除確認方法が明記）
- 実装可能性: 確実（全依存タスクのハンドラ・ルートは実装済みと確認済み）
- 信頼性レベル: 🔵が大半（TC-018-E01/E02のみ🟡、要件定義書に直接記載がなくタスクファイルの完了条件からの妥当な推測）
