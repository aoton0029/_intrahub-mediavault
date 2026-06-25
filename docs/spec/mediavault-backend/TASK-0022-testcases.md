# TASK-0022 テストケース一覧: api_credentials（外部APIキー管理）CRUD実装

**作成日**: 2026-06-25
**関連要件**: [TASK-0022-requirements.md](TASK-0022-requirements.md)
**関連タスク**: [TASK-0022.md](../../tasks/mediavault-backend/TASK-0022.md)
**関連ノート**: [note.md](note.md) TASK-0022セクション
**対象エンドポイント**: `PUT /settings/api-keys/:provider`

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・タスク仕様・既存実装（note.md記載）から確実な根拠があるテストケース
- 🟡 **黄信号**: 要件定義書・既存実装から妥当な推測によるテストケース
- 🔴 **赤信号**: 元の資料にない推測によるテストケース（本ドキュメントには無し）

## 0. テスト分類・配置方針

note.md「テスト規約（既存方針を継続）」L24-28に基づき、本タスクのテストは以下の2系統に分かれる。

| 種別 | 実行属性 | 実行コマンド | 対象 | 配置先 |
|---|---|---|---|---|
| ユニット（DB非依存） | `#[test]` | `cargo test -p mediavault-api` | provider文字列→`ApiProvider` enum変換ロジック、`UpdateApiKeyRequest` DTOのデシリアライズ、`ApiErrorCode::InvalidProvider`のHTTPステータス対応 | `backend/mediavault-api/src/models/api_credential.rs` / `backend/mediavault-api/src/models/response.rs` の `#[cfg(test)] mod tests` |
| 統合（実DB必要） | `#[tokio::test]` + `#[ignore]` | `cargo test -- --ignored`（事前に `docker compose up -d db`） | `upsert_api_credential`・`find_by_provider`リポジトリ関数のEnd-to-End動作、ハンドラ経由のHTTPステータス/レスポンス検証、`trg_api_credentials_updated_at`発火確認、不正provider時のDB無変更確認 | `backend/mediavault-api/src/repositories/api_credential_repository.rs` / `backend/mediavault-api/src/handlers/settings.rs` の `#[cfg(test)] mod tests`、`routes/mod.rs`の`tests`モジュール |

DBエラー変換（`db_error`→500）は既存パターン（`unreachable_pool()`相当）に倣い統合テスト側に配置する。

---

## 1. 正常系テストケース（基本的な動作）

### TC-015-01-A: 全6 provider文字列が`ApiProvider` enumに正しく変換される（ユニット）

- **テスト名**: provider変換関数が`tmdb`/`igdb`/`ndl`/`steam`/`open_library`/`ani_list`をそれぞれ対応するenumバリアントに変換する
  - **何をテストするか**: パスパラメータ文字列→`ApiProvider` enum変換ロジックが、許可された全6種のsnake_case文字列を正しくマッピングするか
  - **期待される動作**: 各文字列が`ApiProvider::Tmdb`/`Igdb`/`Ndl`/`Steam`/`OpenLibrary`/`AniList`に変換され`Ok`/`Some`を返す
- **入力値**: `["tmdb", "igdb", "ndl", "steam", "open_library", "ani_list"]`
  - **入力データの意味**: types.rs L86-94の`ApiProvider`全バリアント。`#[serde(rename_all = "snake_case")]`で定義される全許可値を網羅する代表入力
- **期待される結果**: 6文字列すべてが変換成功し、それぞれ期待するenumバリアントと一致する
  - **期待結果の理由**: REQ-0022-01（provider文字列をenumに変換できる場合upsert）の前提となる変換の全網羅。`open_library`/`ani_list`はcamelCaseバリアント名とsnake_case文字列の対応が崩れやすいため特に重要
- **テストの目的**: 6 provider全てに対する変換の正確性を実DB不要で高速検証する
  - **確認ポイント**: `OpenLibrary`→`open_library`、`AniList`→`ani_list`のような複合語のsnake_case変換が正しいこと
- 🔵 信頼性レベル: 要件定義書「許可値」L34・REQ-0022-01、note.md L15（ApiProvider定義）より

### TC-015-01-B: TMDbキー新規登録（upsert、実DB統合）

- **テスト名**: `upsert_api_credential`が`tmdb`の既存レコードが無い状態で新規レコードを作成する
  - **何をテストするか**: 実DB上で対象providerのレコードが存在しない状態からupsertを行い、新規行が作成されるか
  - **期待される動作**: 200相当の戻り値が返り、`api_credentials`に`provider=tmdb, api_key=valid-tmdb-key`の行が1件作成される
- **入力値**: 事前に`tmdb`行が存在しないクリーンな状態 + `ApiProvider::Tmdb` + `api_key="valid-tmdb-key"`
  - **入力データの意味**: 要件定義書シナリオ1（TC-015-01）の代表入力。INSERTパス（ON CONFLICT非発生）を表す
- **期待される結果**: 戻り値`ApiCredential`の`provider==Tmdb`・`api_key=="valid-tmdb-key"`、DB再取得でも同一の行が1件存在する
  - **期待結果の理由**: REQ-0022-02（レコードが存在しない場合は新規作成）に直接対応
- **テストの目的**: upsertのINSERT分岐のEnd-to-End動作を保証する
  - **確認ポイント**: 行が重複生成されず1件のみであること、`updated_at`がDEFAULT（CURRENT_TIMESTAMP）で設定されていること
- 🔵 信頼性レベル: 要件定義書シナリオ1・REQ-0022-02、タスクファイルテストケース1（TC-015-01）より

### TC-015-01-C: TMDbキー新規登録のHTTP 200レスポンス（ハンドラ統合）

- **テスト名**: `PUT /settings/api-keys/tmdb`が新規登録時に200と更新後`ApiCredential`を返す
  - **何をテストするか**: ルーティング→ハンドラ→リポジトリのフルパスで、新規登録時のHTTPレスポンスが規約通りか
  - **期待される動作**: HTTPステータス200、ボディが`ApiOk<ApiCredential>`形式（`{"success": true, "data": {...}}`相当）
- **入力値**: パス`/settings/api-keys/tmdb` + JSONボディ`{ "api_key": "valid-tmdb-key" }`
  - **入力データの意味**: 完了条件L27「正常系で更新後ApiCredentialを200で返す」のHTTPレベル検証
- **期待される結果**: HTTP 200、`data.provider=="tmdb"`、`data.api_key`の扱いは実装判断（平文返却 or マスキング、tdd-red前に確定）
  - **期待結果の理由**: REQ-0022-05（更新後ApiCredentialをHTTP 200で返す）・NFR-0022-01（既存ApiOk規約準拠）に対応
- **テストの目的**: エンドポイントが`build_router`に正しく登録され、HTTPレベルで正常系が成立することを保証する
  - **確認ポイント**: PUTメソッドでルーティングされること、`api_key`をレスポンスに含めるか否かの実装方針が一貫していること
- 🔵 信頼性レベル: 要件定義書REQ-0022-05・NFR-0022-01、完了基準L126・129より

### TC-015-03-A: キー更新が`find_by_provider`に反映される（upsert+取得、実DB統合）

- **テスト名**: 既存`tmdb`レコードへのupsert後、`find_by_provider(Tmdb)`が新しいキーを返す
  - **何をテストするか**: 既存レコードがある状態でupsert（UPDATE分岐）を行い、`find_by_provider`で取得した値が更新後のキーになっているか
  - **期待される動作**: `api_key`が`old-key`→`new-key`に更新され、`find_by_provider(Tmdb)`が`Some(ApiCredential{ api_key: "new-key", .. })`を返す
- **入力値**: 事前に`provider=tmdb, api_key=old-key`を投入 → `ApiProvider::Tmdb` + `api_key="new-key"`でupsert
  - **入力データの意味**: 要件定義書シナリオ3（TC-015-03）の代表入力。ON CONFLICT発生によるUPDATEパス＋後続タスク参照経路の確認
- **期待される結果**: upsert後の`find_by_provider(Tmdb)`が`api_key=="new-key"`を返す。行数は依然1件（重複なし）
  - **期待結果の理由**: REQ-0022-03（既存時はapi_key更新）・REQ-0022-06（find_by_provider提供）に対応
- **テストの目的**: キーローテーションが永続化され後続タスク（TASK-0023）から参照可能であることを保証する
  - **確認ポイント**: `find_by_provider`がupsert直後の最新値を返すこと、provider PRIMARY KEYにより行が増えないこと
- 🔵 信頼性レベル: 要件定義書シナリオ3・REQ-0022-03・REQ-0022-06、タスクファイルテストケース3（TC-015-03）より

### TC-015-03-B: キー更新時に`updated_at`がトリガーで更新される（実DB統合）

- **テスト名**: 既存レコードへのupsertで`trg_api_credentials_updated_at`が発火し`updated_at`が更新される
  - **何をテストするか**: UPDATE分岐実行時に、DBトリガーが`updated_at`を自動更新するか
  - **期待される動作**: upsert前の`updated_at`より、upsert後の`updated_at`が新しい
- **入力値**: 事前投入済み`tmdb`レコード（`updated_at`記録済み）+ `api_key="new-key"`でupsert
  - **入力データの意味**: REQ-0022-201（updated_atトリガー自動設定）・シナリオ3末尾「updated_atも更新される」の観測可能な検証
- **期待される結果**: `updated_at_after > updated_at_before`が真
  - **期待結果の理由**: REQ-0022-201（BEFORE UPDATEトリガーが優先）・note.md L14（updated_atを明示SETしなくても自動更新）に対応
- **テストの目的**: アプリ側で明示的に`updated_at`を更新せずともトリガーで更新が反映されることを保証する
  - **確認ポイント**: SQLのSET句に`updated_at`を含めても/含めなくても最終的にトリガー値になること（note.md L14方針）
- 🔵 信頼性レベル: 要件定義書REQ-0022-201・シナリオ3、note.md L14・database-schema.sql L375-376より

### TC-NEW-01: `UpdateApiKeyRequest` DTOが`{ "api_key": "..." }`をデシリアライズする（ユニット）

- **テスト名**: `UpdateApiKeyRequest`が正しいJSONボディをデシリアライズして`api_key`を保持する
  - **何をテストするか**: リクエストボディDTOのserdeデシリアライズが期待通り機能するか
  - **期待される動作**: `{ "api_key": "xxxxx" }`が`UpdateApiKeyRequest { api_key: "xxxxx" }`にデシリアライズされる
- **入力値**: `{ "api_key": "xxxxx" }`
  - **入力データの意味**: api-endpoints.md L384-386のリクエスト例。DTO契約の基本動作を表す
- **期待される結果**: `serde_json::from_value::<UpdateApiKeyRequest>(...)`が`Ok`を返し、`api_key == "xxxxx"`
  - **期待結果の理由**: 要件定義書「入力」L36-43（DTO定義）に直接対応
- **テストの目的**: DTO定義が設計通りであることを実DB不要で検証する
  - **確認ポイント**: フィールド名が`api_key`（snake_case）であること
- 🔵 信頼性レベル: 要件定義書L36-43（DTO定義）、タスクファイル実装詳細2 L37-45より

---

## 2. 異常系テストケース（エラーハンドリング）

### TC-015-02-A: 不明なprovider文字列が変換失敗する（ユニット）

- **テスト名**: provider変換関数が`unknown_provider`に対して変換失敗（`None`/`Err`）を返す
  - **エラーケースの概要**: `ApiProvider` enumに存在しない文字列が指定された場合の変換ロジック単体の挙動
  - **エラー処理の重要性**: DBアクセス前にアプリ層で不正providerを検知し、INVALID_PROVIDER（400）に変換する前段ロジックの保証
- **入力値**: `"unknown_provider"`
  - **不正な理由**: `ApiProvider`の許可値（tmdb/igdb/ndl/steam/open_library/ani_list）のいずれにも一致しない
  - **実際の発生シナリオ**: クライアントがタイプミスや未対応providerを指定した場合
- **期待される結果**: 変換関数が`None`（または`Err`）を返す。ハンドラ側でこれを`ApiError::new(ApiErrorCode::InvalidProvider, ...)`に変換する設計の前段
  - **エラーメッセージの内容**: 変換関数自体はNone/Errのみ返し、ApiErrorへの変換はハンドラ責務（責務分離）
  - **システムの安全性**: 変換失敗時点でDBアクセスに進まないことを保証する設計の起点
- **テストの目的**: REQ-0022-101（INVALID_PROVIDER・DB書き込み禁止）の前段である変換失敗ロジックを保証する
  - **品質保証の観点**: TC-015-02の中核ロジックを実DB不要で高速に回帰確認できる
- 🔵 信頼性レベル: 要件定義書REQ-0022-101・シナリオ2（TC-015-02）、タスクファイルテストケース2より

### TC-015-02-B: 不正provider指定でHTTP 400 INVALID_PROVIDERを返し、DBへ書き込まない（ハンドラ統合）

- **テスト名**: `PUT /settings/api-keys/unknown_provider`が400 `INVALID_PROVIDER`を返しDBを変更しない
  - **エラーケースの概要**: HTTPレベルで不正providerリクエストを受け、エラー応答とDB無変更の両方を確認する
  - **エラー処理の重要性**: 不正入力に対し統一エラー形式で応答しつつ、DBへ意図しない行が作られないことを保証する必要があるため
- **入力値**: パス`/settings/api-keys/unknown_provider` + JSONボディ`{ "api_key": "x" }`
  - **不正な理由**: providerがenum許可値外
  - **実際の発生シナリオ**: クライアントのルーティングミス、未対応provider指定
- **期待される結果**: HTTPステータス400、`ApiErrorCode::InvalidProvider`、かつ`api_credentials`テーブルに`unknown_provider`相当の行が作成されていない（DB直接クエリで0件確認）
  - **エラーメッセージの内容**: NFR-0022-01準拠の統一エラー形式（`{ code, message }`）
  - **システムの安全性**: 不正provider時にDBへの副作用が一切ないことを直接DBクエリで確認（最重要検証）
- **テストの目的**: REQ-0022-101の「INVALID_PROVIDER返却＋DB書き込み禁止」をEnd-to-Endで保証する
  - **品質保証の観点**: NFR-0022-01（既存ApiError規約準拠）・REQ-0022-102（新規バリアント）の確認
- 🔵 信頼性レベル: 要件定義書シナリオ2・REQ-0022-101、タスクファイルテストケース2・api-endpoints.md L388より

### TC-NEW-02: jikanは`ApiProvider`に含まれずINVALID_PROVIDERになる（ユニット/ハンドラ統合）

- **テスト名**: `jikan`がprovider変換で失敗し、`PUT /settings/api-keys/jikan`が400 INVALID_PROVIDERを返す
  - **エラーケースの概要**: キー不要のためenum対象外とされた`jikan`を明示的に検証する
  - **エラー処理の重要性**: 「キー不要provider」を誤って受理しないことが本タスクの明示仕様（REQ-015/対象外）であるため
- **入力値**: ユニット: `"jikan"` / 統合: パス`/settings/api-keys/jikan` + `{ "api_key": "x" }`
  - **不正な理由**: Jikanはキー不要のため`ApiProvider` enumに含まれない（要件定義書「対象外」L24）
  - **実際の発生シナリオ**: クライアントがJikanにもキー設定を試みた場合
- **期待される結果**: ユニットでは変換失敗（None/Err）、統合では400 `INVALID_PROVIDER`かつDB無変更
  - **エラーメッセージの内容**: 他のINVALID_PROVIDERケースと同一の統一形式
  - **システムの安全性**: jikan行がDBに作られないこと
- **テストの目的**: EDGE-0022-01（jikanのINVALID_PROVIDER扱い）を明示テストケース化する
  - **品質保証の観点**: 仕様上の「対象外provider」の境界を回帰テストで固定する
- 🔵 信頼性レベル: 要件定義書EDGE-0022-01・「対象外」L24、note.md L6・タスクファイル注意事項L98より

### TC-NEW-03: 大文字`TMDB`などsnake_case不一致はINVALID_PROVIDER（ユニット）

- **テスト名**: provider変換関数が`TMDB`（大文字）に対して変換失敗を返す
  - **エラーケースの概要**: 正しいproviderの大文字/別ケース表記がenumにマッチしないことの確認
  - **エラー処理の重要性**: `#[serde(rename_all = "snake_case")]`は大文字小文字を区別するため、`TMDB`を誤受理しない保証が必要
- **入力値**: `"TMDB"`（および補助的に`"Tmdb"`, `"tmdb "`末尾空白）
  - **不正な理由**: snake_caseの`tmdb`と一致しない（serdeのrename規約は厳密一致）
  - **実際の発生シナリオ**: クライアントが大文字でproviderを送信した場合
- **期待される結果**: 変換失敗（None/Err）→ ハンドラ経由で400 INVALID_PROVIDER
  - **エラーメッセージの内容**: 統一エラー形式
  - **システムの安全性**: DBへ書き込まないこと
- **テストの目的**: EDGE-0022-01（大文字TMDB等のsnake_case不一致）を保証する
  - **品質保証の観点**: ケースセンシティブなprovider照合の堅牢性確認
- 🔵 信頼性レベル: 要件定義書EDGE-0022-01（大文字TMDB例）より

### TC-NEW-04: `api_key`欠落リクエストでボディパースエラー（ハンドラ統合）

- **テスト名**: `PUT /settings/api-keys/tmdb`に`api_key`を欠いたボディを送るとデシリアライズ失敗で400系になる
  - **エラーケースの概要**: 必須フィールド`api_key`が欠落したリクエストボディの処理
  - **エラー処理の重要性**: 必須フィールド欠落時に既存のボディパースエラー処理に正しく従い、ハンドラ本体に到達しないことの確認
- **入力値**: パス`/settings/api-keys/tmdb` + JSONボディ`{}`（または`api_key`キーなし）
  - **不正な理由**: `UpdateApiKeyRequest.api_key`は必須`String`であり、欠落するとserdeデシリアライズが失敗する
  - **実際の発生シナリオ**: クライアントがボディを付け忘れた、または誤ったJSON構造を送った場合
- **期待される結果**: axumの`Json`エクストラクタによるデシリアライズ失敗で400系レスポンス（既存のボディパースエラー処理に従う）、DB無変更
  - **エラーメッセージの内容**: axum/既存の共通ボディパースエラー応答に準拠（実装時に既存ハンドラの挙動を確認して期待値確定）
  - **システムの安全性**: パース段階で早期リターンしDBアクセスが発生しないこと
- **テストの目的**: EDGE-0022-02（api_key欠落→ボディパースエラー）を保証する
  - **品質保証の観点**: 必須フィールド契約がHTTPレベルで強制されることの確認
- 🟡 信頼性レベル: 要件定義書EDGE-0022-02（DTO必須フィールドからの妥当な推測）より。期待値（400系の具体的なcode/形式）はtdd-red前に既存ボディパースエラー処理を確認して確定する

### TC-NEW-05: DB接続不能時にInternalError（500）へ正規化される（実DB統合、unreachable_pool）

- **テスト名**: `upsert_api_credential`がDB接続不能時に`db_error`経由でInternalErrorに変換される
  - **エラーケースの概要**: DB接続自体が失敗するケース（DB停止、ネットワーク障害）をシミュレートする
  - **エラー処理の重要性**: SQLやDB内部情報をクライアントへ漏らさないことがセキュリティ要件（REQ-0022-402）であるため
- **入力値**: 到達不能な`PgPool`（`unreachable_pool()`相当）+ `ApiProvider::Tmdb` + `api_key="x"`
  - **不正な理由**: 接続先が存在せず全クエリがエラーになる
  - **実際の発生シナリオ**: 本番DBの一時ダウンやネットワーク分断時
- **期待される結果**: `upsert_api_credential`が`Err(ApiError)`を返し、`ApiErrorCode::InternalError`（500）であること。エラーメッセージにSQL文・接続文字列が含まれないこと
  - **エラーメッセージの内容**: 既存`db_error`（`repositories/db_error_utils.rs`）が返す汎用メッセージと同一
  - **システムの安全性**: 機密情報がレスポンスに漏れないことを文字列検証で確認
- **テストの目的**: REQ-0022-402（db_error経由のInternalError正規化）を新規upsert処理パスでも保証する
  - **品質保証の観点**: EDGE-0022-04の保証。既存`db_error`変換テストと同型の回帰テストで規約逸脱を防ぐ
- 🔵 信頼性レベル: 要件定義書REQ-0022-402・EDGE-0022-04、note.md L22（db_error_utils.rs・unreachable_poolパターン）より

### TC-NEW-06: `find_by_provider`が未登録providerで`None`を返す（実DB統合）

- **テスト名**: `find_by_provider`が登録のないproviderに対し`Ok(None)`を返す
  - **エラーケースの概要**: 後続タスク向け取得関数が、レコード未登録時に正しく「なし」を表現するか
  - **エラー処理の重要性**: TASK-0023の`ExternalSearchService`が「キー未設定」を判定できる必要があり、誤ってエラーや空文字を返さないことが重要
- **入力値**: `api_credentials`に該当providerが存在しないクリーンな状態 + `ApiProvider::Igdb`
  - **不正な理由**: 該当providerのレコードがDBに存在しない（厳密にはエラーではなく「未登録」状態）
  - **実際の発生シナリオ**: あるproviderのキーがまだ登録されていない状態でTASK-0023が取得を試みる場合
- **期待される結果**: `find_by_provider(Igdb)`が`Ok(None)`を返す（パニック・Errではなく`None`）
  - **エラーメッセージの内容**: 該当なし。`Option`の`None`で表現
  - **システムの安全性**: 未登録を例外ではなく正常な「なし」として扱う
- **テストの目的**: REQ-0022-06（find_by_provider提供）の未登録ケースを保証する
  - **品質保証の観点**: 後続タスクが安全に「キー未設定」を分岐できることの確認
- 🟡 信頼性レベル: 要件定義書REQ-0022-06（`Result<Option<ApiCredential>, ...>`シグネチャ）からの妥当な推測。未登録時None挙動はシグネチャから自然だが明示テスト記載はない

---

## 3. 境界値テストケース（最小値、最大値、null等）

### TC-NEW-07: `api_key`空文字列でのupsert（境界、実DB統合）

- **テスト名**: `api_key=""`（空文字列）でupsertした場合の挙動
  - **境界値の意味**: `api_key`は`VARCHAR(500) NOT NULL`であり、空文字列（長さ0だがNOT NULL違反ではない）は「最小長」の境界。本タスクでは`api_key`の非空バリデーションが要件に明記されていないため、受理される想定
  - **境界値での動作保証**: 長さ0文字列がNOT NULL制約を満たし受理されるか、それともアプリ層で別途拒否すべきか、実装方針を固定する
- **入力値**: `ApiProvider::Tmdb` + `api_key=""`
  - **境界値選択の根拠**: DBカラムは`NOT NULL`だが空文字列は許容される（NULLではない）。要件定義書に`api_key`非空バリデーションの記載がないため、現仕様では200で保存される想定
  - **実際の使用場面**: クライアントが空のキー欄を送信した場合
- **期待される結果**: 要件に空文字拒否の記載がないため、現仕様では200で`api_key=""`が保存される想定。**ただしアプリ層で空文字を拒否（VALIDATION_ERROR）すべきかはtdd-red着手前に要件・既存バリデーション方針を確認して期待値を確定すること**
  - **境界での正確性**: 空文字がNOT NULL制約に違反しない（DB INSERTが成功する）ことを確認
  - **一貫した動作**: 仕様未確定部分を明示的にテスト化し、実装時の意思決定を強制する
- **テストの目的**: `api_key`最小長境界の挙動を固定し、空文字許容/拒否の方針を確定させる
  - **堅牢性の確認**: 将来`api_key`バリデーションが追加された場合に検知できる回帰テスト
- 🟡 信頼性レベル: 要件定義書「api_keyは必須のString」L43からの妥当な推測。非空バリデーション有無は要件に明記なく、実装時確認が前提

### TC-NEW-08: `api_key`が500文字（VARCHAR上限）でのupsert（境界、実DB統合）

- **テスト名**: `api_key`が500文字ちょうどの場合にupsertが成功する
  - **境界値の意味**: DBカラム`VARCHAR(500)`の最大長境界。500文字はちょうど上限、501文字は上限超過
  - **境界値での動作保証**: 上限ちょうどの値が切り詰めやエラーなく保存されることを保証する
- **入力値**: `ApiProvider::Tmdb` + `api_key=`500文字の文字列（補助的に501文字も別ケースとして検討可）
  - **境界値選択の根拠**: database-schema.sql L351の`VARCHAR(500)`制約の境界。500文字は成功、501文字はDBエラーになる想定
  - **実際の使用場面**: 非常に長いAPIキー/トークンの保存
- **期待される結果**: 500文字は200で保存成功し`find_by_provider`で同一文字列が取得できる。501文字はDB制約違反→`db_error`経由でInternalError（500）になる想定
  - **境界での正確性**: 500文字で切り詰めが発生しないこと
  - **一貫した動作**: 上限内/超過で挙動が一貫していること
- **テストの目的**: `api_key`最大長境界での保存の正確性を保証する
  - **堅牢性の確認**: VARCHAR(500)制約付近での極端な入力に対する安定動作
- 🟡 信頼性レベル: 要件定義書「DBカラムはVARCHAR(500) NOT NULL」L43・database-schema.sql L351からの妥当な推測。本タスクの完了条件に長さ検証の明記はなく補助的位置づけ

### TC-NEW-09: 同一providerへの連続upsertで重複行が生成されない（境界、実DB統合）

- **テスト名**: `tmdb`へ複数回upsertしても`api_credentials`の`tmdb`行は常に1件
  - **境界値の意味**: upsert回数0→1→2→3の境界。provider PRIMARY KEYにより、何回upsertしても行数は1のまま
  - **境界値での動作保証**: ON CONFLICT (provider) DO UPDATEが繰り返し呼ばれても重複INSERTが起きないことを保証する
- **入力値**: `ApiProvider::Tmdb`に対し`api_key="k1"`→`"k2"`→`"k3"`と3回連続upsert
  - **境界値選択の根拠**: EDGE-0022-03（連続upsertで2回目以降はUPDATE）の直接境界。1回目INSERT、2回目以降UPDATEの分岐切り替わり点
  - **実際の使用場面**: キーローテーションを短期間に複数回行う運用
- **期待される結果**: 各upsert後も`tmdb`行は1件のみ。最終的に`find_by_provider(Tmdb)`が`api_key=="k3"`（最後の値）を返す
  - **境界での正確性**: PRIMARY KEY制約により重複行が0であること
  - **一貫した動作**: 1回目（INSERT）と2回目以降（UPDATE）で最終的な行数・最新値が一貫していること
- **テストの目的**: EDGE-0022-03（連続upsert・重複なし）を保証する
  - **堅牢性の確認**: upsertの冪等性に近い性質（最終状態が最後の入力で決まる）を確認する
- 🔵 信頼性レベル: 要件定義書EDGE-0022-03・database-schema.sql L350（provider PRIMARY KEY）・TC-015-03より

### TC-NEW-10: `ApiErrorCode::InvalidProvider`が400ステータスにマッピングされる（ユニット）

- **テスト名**: 新規追加`ApiErrorCode::InvalidProvider`が`IntoResponse`でHTTP 400を返す
  - **境界値の意味**: 新規エラーバリアントのステータスコード対応の境界確認。既存の400系（ValidationError等）と同じ400であること
  - **境界値での動作保証**: 新規バリアント追加時にステータスコードのmatch漏れがないことを保証する
- **入力値**: `ApiError::new(ApiErrorCode::InvalidProvider, "...")`
  - **境界値選択の根拠**: REQ-0022-102（InvalidProviderを新規バリアントとして追加）・note.md L19（既存DuplicateTagName等の400/404追記パターン踏襲）の検証
  - **実際の使用場面**: 不正provider時のエラー応答生成
- **期待される結果**: `IntoResponse`実装が`StatusCode::BAD_REQUEST`（400）を返す。レスポンスボディが`{ code: "INVALID_PROVIDER", message: ... }`形式
  - **境界での正確性**: 新規バリアントが500（デフォルト）等に誤マッピングされていないこと
  - **一貫した動作**: 既存400系バリアントと同一のレスポンス構造であること
- **テストの目的**: REQ-0022-102（InvalidProvider新規追加・400対応）を実DB不要で保証する
  - **堅牢性の確認**: response.rsのステータスコードmatchに新規バリアントが正しく組み込まれていること
- 🔵 信頼性レベル: 要件定義書REQ-0022-102・完了基準L125、note.md L19（response.rs L50-65・既存バリアントパターン）より

---

## 4. テストケース総覧（TC-ID対応表）

| TC-ID | 概要 | 種別 | 信頼性 | 要件対応 |
|---|---|---|---|---|
| TC-015-01-A | 全6 provider文字列→enum変換 | ユニット | 🔵 | REQ-0022-01 |
| TC-015-01-B | TMDbキー新規登録（INSERT分岐） | 統合 #[ignore] | 🔵 | REQ-0022-02 |
| TC-015-01-C | 新規登録のHTTP 200レスポンス | 統合 #[ignore] | 🔵 | REQ-0022-05, NFR-0022-01 |
| TC-015-03-A | キー更新→find_by_provider反映 | 統合 #[ignore] | 🔵 | REQ-0022-03, REQ-0022-06 |
| TC-015-03-B | キー更新時のupdated_atトリガー発火 | 統合 #[ignore] | 🔵 | REQ-0022-201 |
| TC-NEW-01 | UpdateApiKeyRequest DTOデシリアライズ | ユニット | 🔵 | 入力仕様L36-43 |
| TC-015-02-A | 不明provider文字列の変換失敗 | ユニット | 🔵 | REQ-0022-101 |
| TC-015-02-B | 不正provider→400 INVALID_PROVIDER・DB無変更 | 統合 #[ignore] | 🔵 | REQ-0022-101/102 |
| TC-NEW-02 | jikan→INVALID_PROVIDER | ユニット+統合 | 🔵 | EDGE-0022-01, 対象外L24 |
| TC-NEW-03 | 大文字TMDB等snake_case不一致 | ユニット | 🔵 | EDGE-0022-01 |
| TC-NEW-04 | api_key欠落→ボディパースエラー(400系) | 統合 #[ignore] | 🟡 | EDGE-0022-02 |
| TC-NEW-05 | DB接続不能→InternalError正規化 | 統合 #[ignore] | 🔵 | REQ-0022-402, EDGE-0022-04 |
| TC-NEW-06 | find_by_provider未登録→Ok(None) | 統合 #[ignore] | 🟡 | REQ-0022-06 |
| TC-NEW-07 | api_key空文字列の挙動（要実装時確定） | 統合 #[ignore] | 🟡 | 入力仕様L43 |
| TC-NEW-08 | api_key 500文字（VARCHAR上限）境界 | 統合 #[ignore] | 🟡 | 入力仕様L43 |
| TC-NEW-09 | 連続upsertで重複行なし | 統合 #[ignore] | 🔵 | EDGE-0022-03 |
| TC-NEW-10 | InvalidProvider→400マッピング | ユニット | 🔵 | REQ-0022-102 |

**集計**: 全17ケース（🔵12 / 🟡5 / 🔴0）。ユニット系6件・統合系11件（TC-NEW-02はユニット+統合の両面）。

**カテゴリ別内訳**:
- 正常系（基本動作）: 6ケース（TC-015-01-A/B/C, TC-015-03-A/B, TC-NEW-01）
- 異常系（エラーハンドリング）: 6ケース（TC-015-02-A/B, TC-NEW-02/03/04/05/06のうちNEW-06は未登録正常表現だが取得失敗系として配置）→ 異常系7ケース
- 境界値: 4ケース（TC-NEW-07/08/09/10）

> 正確な分類: 正常系6 / 異常系7 / 境界値4 = 計17ケース。

---

## 5. 開発言語・テストフレームワーク

- **プログラミング言語**: Rust（edition 2024）
  - **言語選択の理由**: 既存プロジェクト全体がRust + axum 0.8 + sqlx 0.8で構築されており、本タスクも同一クレート（`mediavault-api`）内への追加実装のため
  - **テストに適した機能**: 標準テストフレームワーク内蔵（`#[test]`/`#[tokio::test]`）、`Result`型と`?`演算子によるエラー伝播がテストアサーションと自然に統合できる。enum変換の網羅性をコンパイル時+テストで二重に担保できる
- **テストフレームワーク**: Rust標準テストハーネス（`cargo test`） + `tokio::test`（非同期テスト用）+ `sqlx`（実DB接続）
  - **フレームワーク選択の理由**: 既存`item_repository.rs`・`handlers/items.rs`・`routes/mod.rs`が同フレームワークで実装済みであり、一貫性を保つため新規ライブラリ導入は不要
  - **テスト実行環境**: ユニットテスト（provider変換・DTOデシリアライズ・ErrorCodeステータス対応）はDB不要で`cargo test -p mediavault-api`にて即時実行。統合テスト（upsert/find_by_provider/ハンドラ/トリガー確認）は`docker compose up -d db`によるPostgresコンテナと`DATABASE_URL`環境変数を前提とし、`cargo test -- --ignored`で別実行する
- 🔵 信頼性レベル: note.md L24-28「テスト規約」・TASK-0012セクションL40-44「技術スタック」に直接対応

---

## 6. 要件定義との対応関係

- **参照した機能概要**: TASK-0022-requirements.md 第1章「機能の概要」（upsert・6 provider・jikan対象外・平文保存）
- **参照した入力・出力仕様**: 第2章（provider許可値L34、UpdateApiKeyRequest DTO L36-43、ApiOk<ApiCredential>出力L47、データフローL53）
- **参照した制約条件**: 第3章 機能要件（REQ-0022-01〜06, REQ-0022-101〜102, REQ-0022-201, REQ-0022-401〜403）、第4章 非機能要件（NFR-0022-01〜04）
- **参照した使用例**: 第5章 Edgeケース（EDGE-0022-01〜04）、第6章 Given/When/Thenシナリオ1〜3（TC-015-01/02/03）

## 7. 次フェーズへの引き渡し事項

- `tdd-red`着手前に以下を確定すること（要件定義書「次フェーズへの引き渡し事項」L161-163より）:
  1. `ApiErrorCode::InvalidProvider`（400）の追加箇所（`models/response.rs` L50-65付近、既存`DuplicateTagName`/`TagNotFound`等のパターン踏襲）→ TC-NEW-10の対象
  2. provider文字列→enum変換の実装方式（`serde_json`経由 or `match`文）→ TC-015-01-A・TC-015-02-Aの対象関数名を実装に合わせる
  3. `api_key`をレスポンス/ログに含めるか（平文返却 or マスキング）→ TC-015-01-Cの期待値を確定
- DB層ファイルはタスクファイル記載の`db/api_credentials.rs`ではなく既存規約`repositories/api_credential_repository.rs`に倣うこと（note.md L9）。`upsert_api_credential`・`find_by_provider`の関数名・配置を実装に合わせてテスト対象名を更新すること。
- TC-NEW-04（api_key欠落）の期待値（400系の具体的code/形式）は、既存のaxum `Json`エクストラクタ/共通ボディパースエラー処理の挙動を確認して固定すること。
- TC-NEW-07（api_key空文字列）は、`api_key`非空バリデーションを本タスクで行うか否か（要件に明記なし）を確認し、200保存 or VALIDATION_ERROR のいずれかに期待値を確定すること。
- 統合テストのDBクリーンアップ方針（`api_credentials`は`provider` PRIMARY KEYで全6行が固定的に衝突するため、テスト間で対象providerを分ける、または各テスト冒頭でDELETEするなど）を実装時に決めること。
