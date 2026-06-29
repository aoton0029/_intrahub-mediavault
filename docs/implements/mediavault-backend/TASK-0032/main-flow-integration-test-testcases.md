# TASK-0032 テストケース定義: 主要フロー統合テスト実装

実装ファイル構成（要件定義書・タスクファイルのファイル構成に準拠）:
- backend/mediavault-api/tests/common/mod.rs（テストDBセットアップ・共通ヘルパー）
- backend/mediavault-api/tests/items_flow_test.rs
- backend/mediavault-api/tests/search_import_flow_test.rs
- backend/mediavault-api/tests/files_flow_test.rs
- backend/mediavault-api/tests/groups_episodes_flow_test.rs
- backend/mediavault-api/tests/auth_test.rs

## 1. 正常系テストケース

### IT-001: 手動追加→一覧取得→詳細取得（source=manual確認）
- **何をテストするか**: `POST /items`でアイテムを作成し、`GET /items`一覧および`GET /items/:id`詳細にそれが含まれること
- **期待される動作**: 作成したアイテムが一覧・詳細の両方に現れ、`source="manual"`、`external_id=null`であること
- **入力値**: `{"media_type":"anime","title":"テストアニメ1"}`（必須項目のみ）
  - **入力データの意味**: TC-001-01相当の最小構成での手動作成を代表する
- **期待される結果**: `POST /items`→201、`GET /items`一覧に対象IDが含まれる、`GET /items/:id`→200で`source="manual"`/`external_id=null`
- **テストの目的**: 手動追加フローの基本整合性確認
- 🔵 信頼性レベル: 高（acceptance-criteria.md TC-001-01ベース）

### IT-002: PATCH部分更新→DELETE削除→関連テーブルカスケード削除確認
- **何をテストするか**: IT-001で作成したアイテムを`PATCH /items/:id`で更新後、`DELETE /items/:id`で削除し、`item_tags`等の関連レコードが連動して削除されること
- **期待される動作**: PATCHで`rating`等が更新される、DELETE後は`GET /items/:id`が404、関連テーブルへの直接SELECTが0件
- **入力値**: PATCH: `{"rating":4.5,"is_favorite":true}`。事前にタグを1件アタッチしておく
- **期待される結果**: PATCH→200で更新後値を返す、DELETE→204(or200)、削除後`GET /items/:id`→404、`item_tags`へのSELECTが0件
- **テストの目的**: TC-001-02/03相当、カスケード削除の実DB確認
- 🔵 信頼性レベル: 高（acceptance-criteria.md TC-001-02/03ベース）

### IT-003: 外部API検索（モック）→インポート（source=api確認）
- **何をテストするか**: `wiremock`でJikan APIをスタブ化し、`GET /items/search?media_type=anime&q=...`の結果を`POST /items/import`に渡してitemが作成されること
- **期待される動作**: 検索結果の`external_id`がインポート後の`items.external_id`と一致し、`source="api"`
- **入力値**: モックJikanレスポンス（固定JSON、`mal_id=12345`等）
- **期待される結果**: `GET /items/search`→200で検索結果配列、`POST /items/import`→201、作成itemの`source="api"`かつ`external_id="12345"`
- **テストの目的**: TC-002-01〜03相当、外部API連携フローの結合確認（実通信なし）
- 🔵 信頼性レベル: 高（acceptance-criteria.md TC-002-01/02/03、note.mdのwiremock/with_test_base_urlsパターンベース）

### IT-004: ファイル登録（パス指定方式）
- **何をテストするか**: `POST /items/:id/files`に既存パスを指定してitem_filesが作成されること
- **期待される動作**: 指定した`path`がそのまま`item_files.path`に保存される
- **入力値**: `{"path":"/data/test/sample.pdf","file_type":"pdf"}`
- **期待される結果**: 201、レスポンスの`path`が入力と一致、DB直接SELECTでも同値
- **テストの目的**: TC-007-01相当
- 🔵 信頼性レベル: 高（acceptance-criteria.md TC-007-01、models/item_file.rsベース）

### IT-005: ファイル登録（バイナリアップロード方式）
- **何をテストするか**: `POST /items/:id/files/upload`にmultipartでバイナリを送信し、配置後の相対パスがDBに保存されること
- **期待される動作**: テスト用一時ディレクトリ（`tempfile`クレート、既存dev-dependency）をマウント先として、アップロードされたファイルが配置され、相対パスが`item_files.path`に保存される
- **入力値**: multipartフォーム、ダミーバイナリ（数バイトのテストデータ）+ ファイル名
- **期待される結果**: 201、レスポンスの`path`が配置後の相対パス形式、実ファイルがテンポラリディレクトリ上に存在する
- **テストの目的**: TC-019-01相当
- 🔵 信頼性レベル: 高（acceptance-criteria.md TC-019-01ベース、ルート確認済み: `/items/:id/files/upload`, multipart機能有効）

### IT-006: 内部APIキー認証（正しいキー）
- **何をテストするか**: 正しい`INTERNAL_API_KEY`を`Authorization: Bearer <key>`で送信し、`/internal/items`へのPOSTが成功すること
- **期待される動作**: 200/201で処理が継続する
- **入力値**: 正しいAPIキー、`POST /internal/items`の有効なボディ
- **期待される結果**: 201
- **テストの目的**: TC-018-01相当
- 🔵 信頼性レベル: 高（acceptance-criteria.md TC-018-01、middleware/api_key_auth.rsベース）

## 2. 異常系テストケース

### IT-007: EDGE-101 — volume配下へのepisode登録拒否
- **エラーケースの概要**: `group_type=volume`の`item_groups`に対し`POST /groups/:group_id/episodes`を呼ぶと拒否される
- **エラー処理の重要性**: シーズン/話数の構造的整合性を守るための重要なバリデーション
- **入力値**: 事前に`group_type="volume"`のグループを作成し、そのgroup_idに対しepisode作成リクエストを送る
- **不正な理由**: volumeグループは話数を持たない設計のため
- **実際の発生シナリオ**: クライアントが誤って巻（volume）IDをエピソード登録APIに渡した場合
- **期待される結果**: 400、`error.code="INVALID_GROUP_TYPE_FOR_EPISODES"`。`item_episodes`にレコードが作成されないことをDB直接SELECTで確認
- **テストの目的**: TC-EDGE-101-01相当の再確認
- 🔵 信頼性レベル: 高（acceptance-criteria.md TC-EDGE-101-01、handlers/item_episodes.rs:21-49で確認済みのエラーコードベース）

### IT-008: 内部APIキー認証（キーなし）
- **エラーケースの概要**: `Authorization`ヘッダーなしで内部エンドポイントへアクセス
- **入力値**: ヘッダーなしで`POST /internal/items`
- **実際の発生シナリオ**: クライアントの実装ミスや未認証アクセス試行
- **期待される結果**: 401 Unauthorized
- **テストの目的**: TC-018-E01相当
- 🟡 信頼性レベル: 中（acceptance-criteria.mdに直接の項番記載はないが、TASK-0032.md完了条件およびmiddleware/api_key_auth.rsの実装から妥当に推測）

### IT-009: 内部APIキー認証（誤ったキー）
- **エラーケースの概要**: 誤った値を`Authorization: Bearer wrong-key`として送信
- **入力値**: 誤ったAPIキー文字列
- **実際の発生シナリオ**: キーのコピペミスや古いキーの利用
- **期待される結果**: 401 Unauthorized
- **テストの目的**: TC-018-E02相当
- 🟡 信頼性レベル: 中（同上、middleware実装の比較ロジックから妥当に推測）

## 3. 境界値テストケース

### IT-010: 全フィールドNoneでのPATCH（変更なし確認）
- **境界値の意味**: PATCH更新で更新対象フィールドを一切指定しない最小入力ケース
- **入力値**: `{}`（空オブジェクト）
- **境界値選択の根拠**: note.md TASK-0012ノートに「全フィールドNoneの場合は何もUPDATEせず現在の状態を返す」と明記された既存仕様の統合テストレベルでの再確認
- **期待される結果**: 200、更新前と同じ内容のitemが返る
- **テストの目的**: 既存単体テスト済み仕様の統合経路での動作保証
- 🟡 信頼性レベル: 中（note.md記載の既存実装仕様からの妥当な推測、TASK-0032.md自体には明記なし）

### IT-011: 削除後の再GET（境界: 存在しないID）
- **境界値の意味**: DELETE直後のIDという「ちょうど消えた直後」の境界状態
- **入力値**: IT-002で削除したitem_idに対する`GET /items/:id`
- **期待される結果**: 404 `ITEM_NOT_FOUND`
- **テストの目的**: 削除整合性とエラーコードの一貫性確認
- 🔵 信頼性レベル: 高（acceptance-criteria.md TC-001-E02ベース）

## 4. 開発言語・フレームワーク

- **プログラミング言語**: Rust（edition 2024）
  - **言語選択の理由**: 既存プロジェクト言語に統一
  - **テストに適した機能**: `#[tokio::test]`による非同期テスト、`sqlx`のコンパイル時SQL検証
- **テストフレームワーク**: 標準`#[test]`/`#[tokio::test]` + `wiremock`（HTTPスタブ） + `tempfile`（一時ディレクトリ） + `tower::ServiceExt`（Axum Router直接呼び出し）
  - **フレームワーク選択の理由**: 既存dev-dependency（wiremock 0.6, tempfile 3）をそのまま利用でき追加依存不要
  - **テスト実行環境**: `DATABASE_URL`環境変数で指す実Postgres（`docker-compose up -d db`前提）、`#[ignore]`属性付与で`cargo test -- --ignored`実行
- 🔵 信頼性レベル: 高（note.md記載の既存技術スタック・dev-dependency確認済み）

## 5. 要件定義との対応関係

- **参照した機能概要**: main-flow-integration-test-requirements.md 「1. 機能の概要」
- **参照した入力・出力仕様**: 同requirements.md 「2. 入力・出力の仕様」
- **参照した制約条件**: 同requirements.md 「3. 制約条件」（外部API通信禁止、DB環境変数、カスケード削除確認方法）
- **参照した使用例**: 同requirements.md 「4. 想定される使用例」（統合テスト1〜5）

## 品質判定

✅ 高品質
- テストケース分類: 正常系6件・異常系3件・境界値2件で正常系/異常系/境界値を網羅
- 期待値定義: 各テストケースにHTTPステータス・レスポンス内容・DB状態の期待値を明記
- 技術選択: Rust + tokio::test + wiremock + tempfile（既存依存のみで実現可能）確定
- 実装可能性: 全対象エンドポイントが実装済みであることをコード確認済み
- 信頼性レベル: 🔵 9件、🟡 2件（内部APIキー異常系2件のみ、acceptance-criteria.mdに直接項番がないための🟡判定）
