# TASK-0009: POST /items（手動作成） テストケース

## 1. 正常系テストケース

### TC-001-01: 必須項目のみで作成
- **テスト名**: 必須項目（media_type, title）のみのリクエストで201が返る
  - **何をテストするか**: `CreateItemRequest`の必須フィールドのみを指定した場合に、ハンドラがitemsテーブルへINSERTし作成済みitemを返すこと
  - **期待される動作**: items INSERT → 詳細テーブルINSERT（全カラムデフォルト値）→ 201レスポンス
- **入力値**: `{ "media_type": "anime", "title": "作品A" }`
  - **入力データの意味**: タスクファイルのTC-001-01相当。最小構成での正常系を代表する
- **期待される結果**: HTTPステータス201、レスポンスボディ`{ "success": true, "data": { "id": "<uuid>", "media_type": "anime", "title": "作品A", "source": "manual", "external_id": null, "status": "not_started", "is_favorite": false, ... } }`
  - **期待結果の理由**: タスク完了条件「成功時、作成済みitem（UUID付き）を201で返す」「media_type=anime, source=manual, external_id=null」に合致
- **テストの目的**: ハンドラ→リポジトリ→DBの基本フローが正しく動作することを確認
  - **確認ポイント**: `source`が`Manual`固定、`external_id`が`null`固定、`status`/`is_favorite`がDBデフォルト値になっていること
- 🔵 信頼性レベル: タスクファイルTC-001-01・REQ-003に直接記載

### TC-001-02: 全項目を指定して作成
- **テスト名**: 全フィールド（details含む）を指定したリクエストで201が返る
  - **何をテストするか**: `details`にanime_detailsの全カラム相当のJSONを指定した場合に、詳細テーブルへ正しくINSERTされること
  - **期待される動作**: items INSERT + anime_details INSERT（episode_count, season_count, studio, genre_list, source_type, jikan_idすべて反映）
- **入力値**: `{ "media_type": "anime", "title": "作品B", "original_title": "Work B", "description": "概要", "rating": 4.5, "is_favorite": true, "details": { "episode_count": 12, "season_count": 1, "studio": "Studio X", "genre_list": ["action"], "source_type": "original", "jikan_id": "123" } }`
  - **入力データの意味**: 詳細テーブルへの振り分けロジックが正しく機能することを確認する代表的な入力
- **期待される結果**: 201、`data.rating == 4.5`, `data.is_favorite == true`
  - **期待結果の理由**: 入力した値がそのまま保存・返却されることを確認するため
- **テストの目的**: media_type→詳細テーブルのmatch式振り分けロジックの動作確認
  - **確認ポイント**: anime_detailsテーブルに対応するレコードが作成されること（統合テストで確認）
- 🟡 信頼性レベル: タスクファイルの「実装詳細2」(match式振り分け)から妥当な推測。具体的なdetailsの全カラム例はタスクファイルに明記なし

### TC-001-03: media_type=movieで作成（別の詳細テーブルへの振り分け確認）
- **テスト名**: media_type=movieのリクエストでmovie_detailsへ振り分けられる
  - **何をテストするか**: match式によりmedia_type値ごとに異なる詳細テーブルが選択されること
  - **期待される動作**: items INSERT + movie_details INSERT
- **入力値**: `{ "media_type": "movie", "title": "映画C" }`
  - **入力データの意味**: anime以外のmedia_typeでも正しく動作することの代表例
- **期待される結果**: 201、`data.media_type == "movie"`
  - **期待結果の理由**: 8種類のmedia_typeすべてに対応する詳細テーブルがあるため、最低限anime以外で1件動作確認する
- **テストの目的**: 振り分けロジックがanime専用になっていないことの確認
  - **確認ポイント**: movie_detailsテーブルにレコードが作成されること
- 🟡 信頼性レベル: タスクファイルの注意事項「振り分けロジックはmatch式」から妥当な推測

## 2. 異常系テストケース

### TC-001-E01: media_type不正で400
- **テスト名**: media_typeに不正な文字列を指定した場合に400 VALIDATION_ERRORが返る
  - **エラーケースの概要**: enumに定義されていない値（デシリアライズ失敗）を想定
  - **エラー処理の重要性**: 不正なmedia_typeでitemsテーブルにINSERTされるとDB制約（ENUM型）違反やデータ不整合を起こすため、ハンドラ層で事前に検出する必要がある
- **入力値**: `{ "media_type": "invalid", "title": "作品A" }`
  - **不正な理由**: `MediaType`enumは`Anime/Movie/Drama/Manga/Novel/Game/AcademicBook/Paper`のいずれかのsnake_case文字列のみ許容するため
  - **実際の発生シナリオ**: フロントエンドのバグや手動APIテストでの誤入力
- **期待される結果**: HTTPステータス400、レスポンスボディ`{ "success": false, "error": { "code": "VALIDATION_ERROR", "message": "..." } }`
  - **エラーメッセージの内容**: 既存`parse_create_item_request`が返すメッセージ（"リクエストの形式が不正です: ..."）をそのまま利用
  - **システムの安全性**: DBへのINSERTは一切実行されないこと
- **テストの目的**: 既存バリデーション関数（TASK-0008実装済み）がハンドラから正しく呼び出されていることの確認
  - **品質保証の観点**: バリデーションをバイパスしてDBに不正データが入らないことを保証
- 🔵 信頼性レベル: タスクファイルTC-001-E01に直接記載、既存`parse_create_item_request`のテストで動作確認済み

### TC-001-B01: title空文字で400
- **テスト名**: titleが空文字の場合に400 VALIDATION_ERRORが返る
  - **エラーケースの概要**: 必須文字列フィールドが空であるケース
  - **エラー処理の重要性**: items.titleはNOT NULL制約があり、空文字でも保存自体は可能だがビジネスルール上意味のないデータになるため
- **入力値**: `{ "media_type": "anime", "title": "" }`
  - **不正な理由**: `validate_title`は`title.trim().is_empty()`をチェックするため、空文字は不正
  - **実際の発生シナリオ**: フォーム入力で必須項目チェックを回避した場合
- **期待される結果**: HTTPステータス400、`error.code == "VALIDATION_ERROR"`、`error.message == "titleは空にできません"`
  - **エラーメッセージの内容**: 既存`validate_title`のメッセージをそのまま利用
  - **システムの安全性**: DBへのINSERTは実行されないこと
- **テストの目的**: 既存バリデーション関数の再利用確認
  - **品質保証の観点**: TASK-0008の単体テスト（`empty_title_returns_validation_error`）と整合すること
- 🔵 信頼性レベル: タスクファイルTC-001-B01に直接記載、既存テストあり

### TC-001-B02: title空白のみで400
- **テスト名**: titleが空白文字のみの場合に400 VALIDATION_ERRORが返る
  - **エラーケースの概要**: 空文字ではないが実質的に空のケース
  - **エラー処理の重要性**: trim()による空白除去判定が正しく機能することを確認するため
- **入力値**: `{ "media_type": "anime", "title": "   " }`
  - **不正な理由**: `title.trim().is_empty()`がtrueになるため
  - **実際の発生シナリオ**: 誤ってスペースのみ入力された場合
- **期待される結果**: HTTPステータス400、`error.code == "VALIDATION_ERROR"`
  - **エラーメッセージの内容**: 既存`validate_title`のメッセージ
  - **システムの安全性**: DBへのINSERTは実行されないこと
- **テストの目的**: TASK-0008の既存テスト（`blank_title_returns_validation_error`）とハンドラ層の整合性確認
  - **品質保証の観点**: 境界的な不正入力でも確実に検出できること
- 🔵 信頼性レベル: TASK-0008の既存テストに直接対応

## 3. 境界値テストケース

### TC-001-B03: details未指定（None）で作成
- **テスト名**: detailsフィールドを省略した場合でも詳細テーブルへデフォルト値でINSERTされる
  - **境界値の意味**: detailsがNoneという「最小入力」の境界。詳細テーブルへのINSERT自体は必須（1:1関連）であるため、JSONが空でも振り分けロジックは実行される必要がある
  - **境界値での動作保証**: detailsの有無に関わらず詳細テーブルレコードが必ず作成される一貫した動作を保証する
- **入力値**: `{ "media_type": "anime", "title": "作品D" }`（detailsキーなし）
  - **境界値選択の根拠**: タスクファイル注意事項「details未指定時は詳細テーブルへのINSERTは全カラムNULL/デフォルト値（genre_list等は'{}'）で行う」に直接対応
  - **実際の使用場面**: フォームで詳細情報を入力せずタイトルのみ登録する最も基本的なユースケース
- **期待される結果**: 201、anime_detailsテーブルに`item_id`のみ設定されたレコードが作成される（episode_count等はNULL、genre_listは`{}`）
  - **境界での正確性**: NOT NULL DEFAULT制約のカラム（genre_list等）が正しくデフォルト値になること
  - **一貫した動作**: TC-001-01と同じ結果になる（TC-001-01のdetails省略パターンと同義）
- **テストの目的**: 詳細未指定時のデフォルト動作を統合テストで明示的に確認
  - **堅牢性の確認**: 詳細情報なしでも安定して動作すること
- 🔵 信頼性レベル: タスクファイル注意事項に直接記載

### TC-001-B04: details={}（空オブジェクト）で作成
- **テスト名**: detailsに空オブジェクトを指定した場合でも正常に作成される
  - **境界値の意味**: api-endpoints.mdのリクエスト例`{ "media_type": "anime", "title": "作品A", "details": {} }`に対応する境界値
  - **境界値での動作保証**: `details: null`と`details: {}`の両方が同じ結果（全カラムデフォルト）になることを保証
- **入力値**: `{ "media_type": "anime", "title": "作品A", "details": {} }`
  - **境界値選択の根拠**: api-endpoints.mdのサンプルリクエストに直接記載されている値
  - **実際の使用場面**: フロントエンドが常に`details`キーを送信するがフィールド未入力の場合
- **期待される結果**: 201、TC-001-B03と同様の結果（詳細テーブルは全カラムデフォルト）
  - **境界での正確性**: JSON空オブジェクトのデシリアライズが各詳細構造体のOptionフィールドすべてNoneになること
  - **一貫した動作**: NoneとSome({})の入力で出力が一致すること
- **テストの目的**: api-endpoints.mdのサンプルリクエストとの整合性確認
  - **堅牢性の確認**: JSON構造のバリエーションに対する耐性
- 🔵 信頼性レベル: api-endpoints.md記載のリクエスト例と完全一致

## 4. 開発言語・フレームワーク

- **プログラミング言語**: Rust（edition 2021相当）
  - **言語選択の理由**: 既存プロジェクト（mediavault-api crate）がRust/Axumで実装されているため
  - **テストに適した機能**: `#[tokio::test]`による非同期テスト、`sqlx::Transaction`によるテスト用ロールバック、強い型システムによるコンパイル時検証
- **テストフレームワーク**: Rust標準テスト（`#[test]` / `#[tokio::test]`、`#[cfg(test)] mod tests`）
  - **フレームワーク選択の理由**: 既存のTASK-0005/0008実装が同パターンを採用しており一貫性を保つため
  - **テスト実行環境**: 単体テスト（バリデーション・JSONシリアライズ確認）はDB不要でローカル実行可能。統合テスト（実INSERT確認）は`docker compose up -d db`で起動したPostgreSQLに対し`DATABASE_URL`を設定して`cargo test --workspace`で実行
- 🔵 信頼性レベル: note.md・既存テストファイル（item.rs, response.rs）から直接確認

## 5. 要件定義との対応関係

- **参照した機能概要**: `create-item-requirements.md` 1. 機能の概要
- **参照した入力・出力仕様**: `create-item-requirements.md` 2. 入力・出力の仕様（CreateItemRequest, Itemの構造）
- **参照した制約条件**: `create-item-requirements.md` 3. 制約条件（トランザクション、ApiOk 201構築の注意）
- **参照した使用例**: `create-item-requirements.md` 4. 想定される使用例（TC-001-01/E01/B01）

---

## 品質判定

✅ **高品質**
- テストケース分類: 正常系3件・異常系2件・境界値2件で網羅（必須項目のみ／全項目／別media_type／不正値／空文字／空白／details省略／details空オブジェクト）
- 期待値定義: 各ケースでHTTPステータス・レスポンスボディ・DBレコード状態を明記
- 技術選択: Rust標準テスト機構で確定、既存実装パターンと一致
- 実装可能性: TASK-0008の既存バリデーション関数を再利用するため実装リスク低い
- 信頼性レベル: 🔵5件、🟡2件（TC-001-02, TC-001-03はdetailsの具体例がタスクファイルに直接記載がないための妥当な推測）
