# TASK-0024 テストケース一覧: GET /items/search 実装（ExternalSearchServiceのHTTP層公開）

**作成日**: 2026-06-25
**関連要件**: [TASK-0024-requirements.md](TASK-0024-requirements.md)
**関連タスク**: [TASK-0024.md](../../tasks/mediavault-backend/TASK-0024.md)
**関連ノート**: [note.md](note.md) TASK-0024セクション
**前提タスク**: [TASK-0023-testcases.md](TASK-0023-testcases.md)（`ExternalSearchService` 契約・テスト方針）
**対象ハンドラ**: `handlers::items::search_items`（`GET /items/search?media_type=&q=`）

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・タスク仕様・既存実装（note.md記載）から確実な根拠があるテストケース
- 🟡 **黄信号**: 要件定義書・既存実装から妥当な推測によるテストケース
- 🔴 **赤信号**: 元の資料にない推測によるテストケース（本ドキュメントには無し）

---

## 0. テスト分類・配置方針

本タスクの主対象は **`GET /items/search` ハンドラのHTTP層振る舞い**（クエリデシリアライズ→ステータスコード／ワイヤーコードのマッピング→ルート登録順序）である。テストは既存 `routes/mod.rs` の `#[cfg(test)] mod tests` に合流させ、`build_router(state)` を `tower::ServiceExt::oneshot` で駆動するルーターレベルテストを基本とする（既存 `GET /items` 不正値テスト `routes/mod.rs` L156-179 と同一パターン）。🔵 *NFR-0024-03・note.md L39・既存 routes/mod.rs テストより*

### テスト手段の確定事項（tdd-red着手前に固定すべき項目）

`ExternalSearchService` は `PgPool` を直接構築しモック化困難（要件定義書 第3章 L99・第9章 引き渡し3）。本タスクのテストは以下の2系統に分かれる。

| 種別 | 実行属性 | 実行コマンド | 対象 | 配置先 |
|---|---|---|---|---|
| ユニット（DB非依存・サービス非依存） | `#[test]` / `#[tokio::test]` | `cargo test -p mediavault-api` | (a) `From<ExternalSearchError> for ApiError` のステータス／ワイヤーコード変換、(b) `ItemSearchQuery` のデシリアライズ（必須欠落・不正値→失敗）、(c) ルート登録順序（`/items/search` が `/items/:id` に誤マッチしない）の検証 | `models/response.rs` / `models/item_search.rs` / `routes/mod.rs` の `#[cfg(test)] mod tests` |
| 統合（実DB＋外部APIモック必要） | `#[tokio::test]` + `#[ignore]` | `cargo test -- --ignored`（事前 `docker compose up -d db`、外部APIはTASK-0023同様の `wiremock` ベースURL注入） | `build_router` 経由の200成功（anime/Jikan・movie/TMDb）、422、502のEnd-to-End（実DBキー＋HTTPモック注入） | `routes/mod.rs` の `#[cfg(test)] mod tests`（`#[ignore]`付与・`test_app_state()` 利用） |

**前提（要件定義書 第9章 引き渡し）**: 以下4点を tdd-red 前に確定する。
1. **`Query` Rejection整形**: 不正クエリで統一 `ApiError`（`VALIDATION_ERROR`ボディ）を返すか、素のAxum 400で許容するか（要件定義書 第2章注記・note.md L27）。**本テストケースはステータス==400 を主アサーションとし、ボディ形式（`VALIDATION_ERROR`コード）アサーションは「整形採用時のみ有効」として明示分離する**（既存 `GET /items` 不正値テストもボディ未検証）。
2. **`From<ExternalSearchError> for ApiError` の配置**: `handlers/items.rs` 内か `models/response.rs` か（`errors.rs` 不在・note.md L31）。配置先に依存しない形で `From` の出力（`ApiError`→ステータス／コード）をアサートする。
3. **エラーマッピングのテスト容易化**: `From` 実装をDB非依存で単体テスト（要件定義書 第3章 L99）。→ TC-0024-E01/E02 のユニット版（U系）を採用。
4. **新規 `ApiErrorCode` variant名**: `ApiKeyNotConfigured`(422/`"API_KEY_NOT_CONFIGURED"`) ・ `ExternalApiTimeout`(502/`"EXTERNAL_API_TIMEOUT"`)（要件定義書 第4章）。

**🚨 サービスモック化困難への対処**: 200成功（TC-0024-01/02）と422/502のEnd-to-End（TC-0024-E01-I/E02-I）は `ExternalSearchService` が実 `PgPool` を要するため `#[ignore]` 統合テストとし、外部APIはTASK-0023同様 `wiremock` ベースURL注入で「ライブ外部API／本番DBに依存しない」レベルに留める。エラーマッピングの核（422/502/400 のステータス・コード）は **DB非依存ユニット（TC-0024-E01-U / E02-U / 400系）** で先行・確実に担保する。これにより live external API・DB なしで主要検証が成立する。🔵 *要件定義書 第3章 L99・第9章 引き渡し1-3・TASK-0023-testcases.md 第0章 方針より*

---

## 1. 正常系テストケース（基本的な動作）

### TC-0024-01: anime/Jikan検索で200・検索結果一覧が返る（統合・`#[ignore]`）

- **テスト名**: `GET /items/search?media_type=anime&q=鬼滅` が `200 OK` を返し、ボディにJikan検索結果一覧（`Vec<ExternalSearchResult>`）が含まれる
  - **何をテストするか**: `media_type=anime` でハンドラが `ExternalSearchService::new(state.db.clone())` を構築し `search(Anime, "鬼滅")` を呼び、`ApiOk<Vec<ExternalSearchResult>>` を200で返すか（REQ-0024-01/02/03/05）
  - **期待される動作**: ステータス200。レスポンスボディが `ApiOk` ラップの検索結果配列（各要素が `media_type/provider/external_id/title/raw_data` を保持）。anime はキー不要（Jikan）のためDBキー未登録でも成功し得る
- **入力値**: `GET /items/search?media_type=anime&q=鬼滅`、JikanモックサーバーURLを注入した `AppState`、Jikanモックが既知の検索結果JSONを返す設定
  - **入力データの意味**: 要件定義書 シナリオ1（TC-002-01）の代表入力。anime→Jikanの🔵確定経路をHTTP層で検証する
- **期待される結果**: `response.status() == 200`、ボディをデシリアライズすると `ExternalSearchResult` 配列を含む。`result[0].media_type == Anime`、`title`/`external_id` がJikanモック応答に対応
  - **期待結果の理由**: REQ-0024-03（anime→Jikan 200）・REQ-0024-05（`ApiOk<Vec<ExternalSearchResult>>`）に直接対応
- **テストの目的**: anime/Jikan の200成功HTTP経路（ハンドラ構築→サービス呼び出し→`ApiOk`整形）を保証する
  - **確認ポイント**: 200であること、ボディが `ApiOk` 形式の配列であること、Jikanはキー不要で動くこと
- 🔵 信頼性レベル: 要件定義書 REQ-0024-03・シナリオ1・TC-002-01、タスクファイル テストケース1 L62-66より

### TC-0024-02: movie/TMDb検索で200・検索結果一覧が返る（統合・`#[ignore]`）

- **テスト名**: `GET /items/search?media_type=movie&q=タイトル` が `200 OK` を返し、ボディにTMDb検索結果一覧が含まれる
  - **何をテストするか**: `media_type=movie` でDBから取得したTMDbキーで初期化されたクライアント経由の検索結果が200で返るか（REQ-0024-01/02/04/05）
  - **期待される動作**: ステータス200。レスポンスボディが `ApiOk` ラップのTMDb検索結果配列。`api_credentials` に `tmdb` キー登録済み・TMDbモックが既知JSONを返す
- **入力値**: `GET /items/search?media_type=movie&q=タイトル`、`api_credentials` に `provider=tmdb` キー登録済み、TMDbモックURL注入済みの `AppState`
  - **入力データの意味**: 要件定義書 シナリオ2（TC-002-02）の代表入力。キー必須プロバイダ（TMDb）の200正常経路をHTTP層で検証する。`drama` も同一TMDb経路（REQ-0024-04）
- **期待される結果**: `response.status() == 200`、ボディの `ExternalSearchResult` 配列で `result[0].media_type == Movie`、`provider == Some(Tmdb)`、`title`/`external_id` がTMDbモック応答に対応
  - **期待結果の理由**: REQ-0024-04（movie/drama→TMDb 200）・REQ-0024-05に対応
- **テストの目的**: movie/TMDb（キー必須）の200成功HTTP経路をEnd-to-Endで保証する
  - **確認ポイント**: 200であること、DBキー取得→クライアント初期化→ディスパッチが通ること、ボディが配列であること
- 🔵 信頼性レベル: 要件定義書 REQ-0024-04・シナリオ2・TC-002-02、タスクファイル テストケース2 L68-72より

---

## 2. 異常系テストケース（エラーハンドリング）

### TC-0024-E01-U: ApiKeyNotConfigured → 422・`API_KEY_NOT_CONFIGURED`（ユニット・DB非依存）

- **テスト名**: `ApiError::from(ExternalSearchError::ApiKeyNotConfigured(Tmdb))` が `422 Unprocessable Entity`・ワイヤーコード `"API_KEY_NOT_CONFIGURED"` を生成する
  - **エラーケースの概要**: サービスがAPIキー未設定エラーを返した場合の `From` 変換でステータス・コードが正しく決まるか
  - **エラー処理の重要性**: HTTP層へ正しいステータス／コードを供給することが要件（REQ-0024-102・第4章マッピング表）の中核。DB非依存ユニットで先行・確実に担保する
- **入力値**: `ExternalSearchError::ApiKeyNotConfigured(ApiProvider::Tmdb)`
  - **不正な理由**: 外部API認証情報未登録（クライアント制御外の設定不備）
  - **実際の発生シナリオ**: 初期セットアップ直後、TMDbキー未登録のままmovie検索した場合（EDGE-001）
- **期待される結果**: 変換後 `ApiError` の `code_and_status()`（相当）が `(StatusCode::UNPROCESSABLE_ENTITY, "API_KEY_NOT_CONFIGURED")`。`IntoResponse` 後のステータスが422、ボディの `code` フィールドが `"API_KEY_NOT_CONFIGURED"`
  - **エラーメッセージの内容**: ワイヤーコードが要件指定文字列と完全一致（既存 `UNPROCESSABLE_ENTITY` 流用不可・要件 REQ-0024-401）
  - **システムの安全性**: DB内部情報・外部API生エラー詳細をボディへ漏洩させない（NFR-0024-02）
- **テストの目的**: REQ-0024-102・第4章マッピング表（ApiKeyNotConfigured→422/`API_KEY_NOT_CONFIGURED`）をDB非依存で保証する
  - **品質保証の観点**: 新規 `ApiErrorCode::ApiKeyNotConfigured` variantの追加（REQ-0024-401）が要件のコード文字列・ステータスに合致することの回帰防止
- 🔵 信頼性レベル: 要件定義書 REQ-0024-102・第4章マッピング表 L109・シナリオ3、タスクファイル テストケース3 L74-78より

### TC-0024-E01-I: APIキー未設定で `GET /items/search` が422を返す（統合・`#[ignore]`）

- **テスト名**: TMDbキー未登録状態で `GET /items/search?media_type=movie&q=タイトル` が `422` を返し、ボディの `code` が `"API_KEY_NOT_CONFIGURED"`
  - **エラーケースの概要**: ハンドラ→サービス→`From` 変換→`IntoResponse` のEnd-to-Endで422が返るか
  - **エラー処理の重要性**: ユニット（E01-U）で担保した変換が、実ルーター経由でも同一ステータス・コードで返ることを統合確認する
- **入力値**: `GET /items/search?media_type=movie&q=タイトル`、`api_credentials` に `tmdb` 行が存在しないクリーンな状態、`build_router(state)` を `oneshot`
  - **不正な理由**: キー必須プロバイダ（TMDb）にキー未登録（`find_by_provider(Tmdb)==None`）
  - **実際の発生シナリオ**: EDGE-001（キー未設定での外部検索）
- **期待される結果**: `response.status() == 422`、ボディの `code == "API_KEY_NOT_CONFIGURED"`、外部APIモックへのリクエスト到達==0（キー確認段階で停止）
  - **エラーメッセージの内容**: 422・`API_KEY_NOT_CONFIGURED`。DB/外部API詳細を漏洩しない
  - **システムの安全性**: 無駄な外部API呼び出しが発生しないこと
- **テストの目的**: REQ-0024-102のHTTP End-to-End（422・コード）を保証する
  - **品質保証の観点**: ユニット変換とルーター結線の整合確認
- 🟡 信頼性レベル: 要件定義書 シナリオ3・TC-002-E01（信頼性🟡指定）・タスクファイル L74-78（統合経路は妥当な推測）より

### TC-0024-E02-U: ExternalApiError → 502・`EXTERNAL_API_TIMEOUT`・非panic（ユニット・DB非依存）

- **テスト名**: `ApiError::from(ExternalSearchError::ExternalApiError(ApiError::Timeout))` が `502 Bad Gateway`・ワイヤーコード `"EXTERNAL_API_TIMEOUT"` を生成し、変換中にpanicしない
  - **エラーケースの概要**: 外部APIタイムアウト等のエラーを `From` 変換した際のステータス・コード決定とpanic非発生
  - **エラー処理の重要性**: 外部API障害でプロセスがクラッシュせず502へ安全にマッピングされる必要（REQ-0024-103・完了条件5）
- **入力値**: `ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Timeout)`（および後述の全6 variant）
  - **不正な理由**: 外部API側の障害（クライアント制御外）
  - **実際の発生シナリオ**: TMDb/外部API高負荷・ネットワーク遅延・5xx応答時（TC-002-E02）
- **期待される結果**: 変換後 `ApiError` が `(StatusCode::BAD_GATEWAY, "EXTERNAL_API_TIMEOUT")`。`IntoResponse` 後ステータス502・ボディ `code == "EXTERNAL_API_TIMEOUT"`。変換処理がpanic/unwrap失敗しない
  - **エラーメッセージの内容**: ワイヤーコードが要件指定文字列と完全一致（既存 `EXTERNAL_API_ERROR` 流用不可・REQ-0024-401）
  - **システムの安全性**: `?` 伝播・`From` 変換でpanicしないこと（プロセス非クラッシュ）
- **テストの目的**: REQ-0024-103・第4章マッピング表（ExternalApiError→502/`EXTERNAL_API_TIMEOUT`・非panic）をDB非依存で保証する
  - **品質保証の観点**: 新規 `ApiErrorCode::ExternalApiTimeout` variantの追加（REQ-0024-401）が要件コード文字列・ステータスに合致することの回帰防止
- 🔵 信頼性レベル: 要件定義書 REQ-0024-103・第4章マッピング表 L110・シナリオ4、タスクファイル テストケース4 L80-84より

### TC-0024-E02-U2: 全 `ApiError` 6 variant が 502・`EXTERNAL_API_TIMEOUT` へ集約・非panic（ユニット・パラメタライズド）

- **テスト名**: `Http{status,body}`/`Auth`/`RateLimit{retry_after}`/`Parse`/`Timeout`/`Network` を内包する `ExternalApiError` がいずれも `502`・`"EXTERNAL_API_TIMEOUT"` へ変換され、panicしない
  - **エラーケースの概要**: `api-client-lib::ApiError` の全6 variantが漏れなく502へ集約されるか（EDGE-0024-05）
  - **エラー処理の重要性**: 一部variantが取りこぼされると502マッピングが破綻しpanic/別ステータスを生むため
- **入力値**: `ExternalApiError(Http{..})` / `(Auth)` / `(RateLimit{..})` / `(Parse)` / `(Timeout)` / `(Network)` の6組
  - **不正な理由**: 各種外部API障害（クライアント制御外）
  - **実際の発生シナリオ**: 5xx・認証失敗・レート制限・不正レスポンス・ネットワーク断など多様な外部障害
- **期待される結果**: 6組すべてで変換後 `ApiError` が `(502, "EXTERNAL_API_TIMEOUT")`、いずれもpanicしない
  - **エラーメッセージの内容**: 6 variant全てが同一の502・`EXTERNAL_API_TIMEOUT` へ集約
  - **システムの安全性**: 想定外variantで未処理panicが発生しないこと
- **テストの目的**: EDGE-0024-05（全ApiError集約・非panic）を保証する
  - **品質保証の観点**: `ApiError` variant追加時のmatch漏れ回帰防止
- 🔵 信頼性レベル: 要件定義書 EDGE-0024-05・第4章 L113・REQ-0024-103（note.md L70 ApiError 6 variant）より

### TC-0024-E02-I: 外部APIタイムアウトで `GET /items/search` が502を返しpanicしない（統合・`#[ignore]`）

- **テスト名**: TMDbモックが応答遅延（タイムアウト誘発）するとき `GET /items/search?media_type=movie&q=タイトル` が `502` を返し、ボディ `code == "EXTERNAL_API_TIMEOUT"`、サーバープロセスはクラッシュしない
  - **エラーケースの概要**: ハンドラ→サービス→外部API障害→`From` 変換→`IntoResponse` のEnd-to-Endで502が返り非panicか
  - **エラー処理の重要性**: 実ルーター経由で502・非panicが成立することの統合確認（完了条件5）
- **入力値**: `GET /items/search?media_type=movie&q=タイトル`、TMDbキー登録済み、TMDbモックが応答遅延/接続不能で `ApiError::Timeout` を誘発、`build_router(state)` を `oneshot`
  - **不正な理由**: 外部API側の遅延（クライアント制御外）
  - **実際の発生シナリオ**: TMDb API高負荷時（TC-002-E02）
- **期待される結果**: `response.status() == 502`、ボディ `code == "EXTERNAL_API_TIMEOUT"`、`oneshot` の `Future` が `Ok(Response)` を返す（`unwrap` が成功＝ハンドラ内でpanicしていない）
  - **エラーメッセージの内容**: 502・`EXTERNAL_API_TIMEOUT`。外部API生エラー詳細を漏洩しない（NFR-0024-02）
  - **システムの安全性**: ハンドラがpanicせずResponseを返すこと（`?` 伝播の確認）
- **テストの目的**: REQ-0024-103のHTTP End-to-End（502・コード・非panic）を保証する
  - **品質保証の観点**: 外部依存障害がプロセスを巻き込まないことの確認
- 🟡 信頼性レベル: 要件定義書 シナリオ4・TC-002-E02（信頼性🟡指定）・タスクファイル L80-84（統合経路は妥当な推測）より

### TC-0024-E03: `q` パラメータ欠落 → 400（ルーター経由）

- **テスト名**: `GET /items/search?media_type=anime`（`q` 欠落）が `400 Bad Request` を返す
  - **エラーケースの概要**: 必須クエリ `q` 未指定で `Query<ItemSearchQuery>` デシリアライズが失敗
  - **エラー処理の重要性**: 必須パラメータ欠落をextractor段階で拒否し、サービス層へ不完全な入力を渡さないため
- **入力値**: `GET /items/search?media_type=anime`、`build_router(state)` を `oneshot`
  - **不正な理由**: 必須フィールド `q: String` が欠落しデシリアライズ不能
  - **実際の発生シナリオ**: クライアントが検索語を付けずにリクエストした場合（EDGE-0024-01）
- **期待される結果**: `response.status() == 400`。【整形採用時のみ】ボディ `code == "VALIDATION_ERROR"`
  - **エラーメッセージの内容**: 400。`Query` Rejection整形採用時は `VALIDATION_ERROR`（要件 第2章注記により整形要否はtdd-red前確定）
  - **システムの安全性**: サービス・DB・外部APIへ到達しないこと
- **テストの目的**: REQ-0024-101・EDGE-0024-01（`q`欠落→400）を保証する
  - **品質保証の観点**: 必須パラメータ検証がHTTP層で機能すること
- 🟡 信頼性レベル: 要件定義書 REQ-0024-101・EDGE-0024-01・タスクファイル テストケース5 L86-90（信頼性🟡）より

### TC-0024-E04: `media_type` パラメータ欠落 → 400（ルーター経由）

- **テスト名**: `GET /items/search?q=鬼滅`（`media_type` 欠落）が `400 Bad Request` を返す
  - **エラーケースの概要**: 必須クエリ `media_type` 未指定で `Query<ItemSearchQuery>` デシリアライズが失敗
  - **エラー処理の重要性**: もう一方の必須パラメータ欠落も一貫して400で拒否される必要があるため（`q` 欠落と対の検証）
- **入力値**: `GET /items/search?q=鬼滅`、`build_router(state)` を `oneshot`
  - **不正な理由**: 必須フィールド `media_type: MediaType` が欠落
  - **実際の発生シナリオ**: クライアントがメディア種別を指定せずにリクエストした場合（EDGE-0024-02）
- **期待される結果**: `response.status() == 400`。【整形採用時のみ】ボディ `code == "VALIDATION_ERROR"`
  - **エラーメッセージの内容**: 400（整形採用時 `VALIDATION_ERROR`）
  - **システムの安全性**: サービス・DB・外部APIへ到達しないこと
- **テストの目的**: REQ-0024-101・EDGE-0024-02（`media_type`欠落→400）を保証する
  - **品質保証の観点**: 両必須パラメータの欠落が一貫して400になることの確認
- 🔵 信頼性レベル: 要件定義書 REQ-0024-101・EDGE-0024-02・完了条件 L26より

### TC-0024-E05: `media_type` 不正値 → 400（ルーター経由）

- **テスト名**: `GET /items/search?media_type=foo&q=鬼滅`（enum外の値）が `400 Bad Request` を返す
  - **エラーケースの概要**: `media_type` がMediaType 8 variant外の文字列で `Query` デシリアライズが失敗
  - **エラー処理の重要性**: enum外の値を400で拒否し、未定義のメディア種別がサービス層に渡らないため（既存 `GET /items` 不正値テストと同方針）
- **入力値**: `GET /items/search?media_type=foo&q=鬼滅`、`build_router(state)` を `oneshot`
  - **不正な理由**: `media_type=foo` がMediaTypeのsnake_case variant（anime/movie/drama/manga/novel/game/academic_book/paper）のいずれにも一致せずデシリアライズ不能
  - **実際の発生シナリオ**: 未対応のメディア種別・タイポ・改ざんされたクエリ（EDGE-0024-03）
- **期待される結果**: `response.status() == 400`。【整形採用時のみ】ボディ `code == "VALIDATION_ERROR"`
  - **エラーメッセージの内容**: 400（整形採用時 `VALIDATION_ERROR`）
  - **システムの安全性**: 不正enum値がサービス層へ到達しないこと
- **テストの目的**: REQ-0024-101・EDGE-0024-03（media_type不正値→400）を保証する
  - **品質保証の観点**: 既存 `GET /items?media_type=invalid` テスト（routes/mod.rs L156-179）と同パターンの一貫性確認
- 🔵 信頼性レベル: 要件定義書 REQ-0024-101・EDGE-0024-03・タスクファイル L42、既存 routes/mod.rs 不正値テストより

---

## 3. 境界値テストケース（最小値、最大値、ルート優先順位等）

### TC-0024-B01: 空 `q` 文字列が透過的にサービス層へ渡される（境界・統合 `#[ignore]` または擬似）

- **テスト名**: `GET /items/search?media_type=anime&q=`（空クエリ）でハンドラがバリデーションせず `search(Anime, "")` を呼ぶ
  - **境界値の意味**: `q` 文字列長0の境界。要件定義書 第2章 L53 では空文字バリデーションは本タスク要件未指定でサービス層へ透過（TASK-0023 TC-002-B01 と整合）
  - **境界値での動作保証**: 空 `q` でハンドラが400化せず（`q` は存在し空文字＝デシリアライズ成功）、空文字をサービス層へそのまま渡すことを固定する
- **入力値**: `GET /items/search?media_type=anime&q=`（`q` キーは存在し値が空文字）
  - **境界値選択の根拠**: 要件定義書 第2章 L53「空文字バリデーションは本タスクでは要件未指定（サービス層へ透過）」。`q` 欠落（400）との区別が境界
  - **実際の使用場面**: 検索語を空のまま送信したケース
- **期待される結果**: `Query` デシリアライズは**成功**（`q=""` は有効なString）。ハンドラは400を返さず `search(Anime, "")` を呼ぶ。結果はサービス／Jikanモックの応答に従う（200 または外部API由来エラー）。panicしない
  - **境界での正確性**: 空文字を `q` 欠落（400）と混同しないこと
  - **一貫した動作**: 非空 `q` と同じコードパスを通ること。**サービス層で空文字を拒否すべきか否かはTASK-0023同様 透過方針で確定**
- **テストの目的**: 空 `q` 境界でハンドラがバリデーション責務を持たず透過する方針（第2章 L53）を固定する
  - **堅牢性の確認**: 空 `q`（値あり）と `q` 欠落（値なし=400）の境界が正しく分かれること
- 🟡 信頼性レベル: 要件定義書 第2章 L53（空文字バリデーション未指定・透過）・TASK-0023-testcases.md TC-002-B01 からの妥当な推測より

### TC-0024-B02: `/items/search` が `/items/:id` に誤マッチしない（ルート登録順序境界・ルーター経由）

- **テスト名**: `GET /items/search?media_type=anime&q=鬼滅` が `search_items` ハンドラへルーティングされ、`/items/:id`（`get_item_handler` で `id="search"`）へは誤マッチしない
  - **境界値の意味**: リテラルパス（`/items/search`）と動的パス（`/items/:id`）のルーティング優先順位境界。`search` という文字列が `:id` として捕捉される誤マッチ境界（REQ-0024-402・EDGE-0024-04）
  - **境界値での動作保証**: `/items/search` が `/items/:id` より前に登録され（または Axum 0.8 のリテラル優先で）、`search` が `id` パラメータとして誤解釈されないことを固定する
- **入力値**: `GET /items/search?media_type=anime&q=鬼滅`（`search` を含むパス）、`build_router(state)` を `oneshot`
  - **境界値選択の根拠**: タスク注意事項 L106・要件定義書 第5章「`/items/search` を `/items/:id` より前に登録」。最も混入しやすいルーティング設定ミスの直接検証
  - **実際の使用場面**: `/items/search` と `/items/{uuid}` が同一プレフィックスを共有する構成
- **期待される結果**: `search_items` ハンドラが起動した証跡（200成功 or サービス由来の422/502/外部APIモック到達）が得られ、`get_item_handler`（`/items/:id`）の応答（`id="search"` をUUIDパースして400/404 等）にはならない。**区別観点**: `/items/:id` は `id="search"` をUUIDとしてパースし失敗→別ステータスになるはずなので、`search_items` 経由の応答（クエリ依存）と峻別できる
  - **境界での正確性**: `search` が `:id` として捕捉されないこと
  - **一貫した動作**: 登録順序に依らず `/items/search` が常に `search_items` へ向かうこと
- **テストの目的**: REQ-0024-402・EDGE-0024-04（ルート誤マッチ防止）を保証する
  - **堅牢性の確認**: 将来のAxumバージョン変更でも前方登録により安全であること（タスク注意事項 L106）
- 🔵 信頼性レベル: 要件定義書 REQ-0024-402・EDGE-0024-04・第5章 L122-123、タスク注意事項 L106より

### TC-0024-B03: 8 MediaType variant いずれもハンドラがデシリアライズ受理する（境界・網羅性・ユニット）

- **テスト名**: `ItemSearchQuery` が8 variant（anime/movie/drama/manga/novel/game/academic_book/paper）のsnake_case文字列すべてを `media_type` として正しくデシリアライズする
  - **境界値の意味**: MediaType enum全列挙の網羅境界。受理すべき有効値の上限／下限（8 variant漏れなく受理し、それ以外は拒否）（REQ-0024-01）
  - **境界値での動作保証**: 有効 `media_type` 全variantが400にならず受理されること、および将来variant追加時の受理漏れ検知
- **入力値**: `media_type=anime/movie/drama/manga/novel/game/academic_book/paper`（各 `q=test` 付き）の8組クエリ文字列を `Query<ItemSearchQuery>` 相当でデシリアライズ
  - **境界値選択の根拠**: 要件定義書 第2章 入力表（MediaType 8 variant・snake_case・既存 `Deserialize` 再利用）。有効値網羅と不正値（TC-0024-E05）の境界
  - **実際の使用場面**: 全メディア種別での検索リクエスト
- **期待される結果**: 8組すべて `ItemSearchQuery` へデシリアライズ成功（`media_type` が対応variant・`q=="test"`）。不正値 `foo`（TC-0024-E05）のみ失敗
  - **境界での正確性**: snake_case表記（`academic_book` 等）が正しくマッチすること
  - **一貫した動作**: 有効8 variantと不正値の境界が明確に分かれること
- **テストの目的**: REQ-0024-01（`media_type`・`q` のデシリアライズ）の有効値網羅をDB非依存で保証する
  - **堅牢性の確認**: `academic_book`/`paper` 等の複合語snake_caseが取りこぼされないこと
- 🟡 信頼性レベル: 要件定義書 第2章 入力表 L52（MediaType 8 variant・snake_case・Deserialize再利用）からの妥当な推測より

---

## 4. テストケース総覧（TC-ID対応表）

| TC-ID | 概要 | 種別 | 信頼性 | 要件対応 |
|---|---|---|---|---|
| TC-0024-01 | anime/Jikan → 200・結果一覧 | 統合 `#[ignore]`(ルーター+HTTPモック) | 🔵 | REQ-0024-03/05, TC-002-01 |
| TC-0024-02 | movie/TMDb → 200・結果一覧 | 統合 `#[ignore]`(ルーター+実DBキー+HTTPモック) | 🔵 | REQ-0024-04/05, TC-002-02 |
| TC-0024-E01-U | ApiKeyNotConfigured → 422・`API_KEY_NOT_CONFIGURED` | ユニット(From変換・DB非依存) | 🔵 | REQ-0024-102/401, 第4章 |
| TC-0024-E01-I | キー未設定で `/items/search` → 422 | 統合 `#[ignore]`(ルーター) | 🟡 | REQ-0024-102, TC-002-E01 |
| TC-0024-E02-U | ExternalApiError(Timeout) → 502・`EXTERNAL_API_TIMEOUT`・非panic | ユニット(From変換・DB非依存) | 🔵 | REQ-0024-103/401, 第4章 |
| TC-0024-E02-U2 | 全ApiError 6 variant → 502 集約・非panic | ユニット(パラメタライズド) | 🔵 | REQ-0024-103, EDGE-0024-05 |
| TC-0024-E02-I | タイムアウトで `/items/search` → 502・非panic | 統合 `#[ignore]`(ルーター+HTTPモック) | 🟡 | REQ-0024-103, TC-002-E02 |
| TC-0024-E03 | `q` 欠落 → 400 | ルーター経由 | 🟡 | REQ-0024-101, EDGE-0024-01 |
| TC-0024-E04 | `media_type` 欠落 → 400 | ルーター経由 | 🔵 | REQ-0024-101, EDGE-0024-02 |
| TC-0024-E05 | `media_type` 不正値 → 400 | ルーター経由 | 🔵 | REQ-0024-101, EDGE-0024-03 |
| TC-0024-B01 | 空 `q` の透過処理（400化しない） | 境界(統合 `#[ignore]`/擬似) | 🟡 | 入力仕様 第2章 L53 |
| TC-0024-B02 | `/items/search` が `/items/:id` に誤マッチしない | 境界(ルーター経由) | 🔵 | REQ-0024-402, EDGE-0024-04 |
| TC-0024-B03 | 8 MediaType variant 受理網羅 | 境界(デシリアライズ・ユニット) | 🟡 | REQ-0024-01, 第2章入力表 |

**集計**: 全13ケース（🔵8 / 🟡5 / 🔴0）。
- ユニット系（DB非依存・live外部API不要）: 5件（E01-U, E02-U, E02-U2, B03、および B02 のルーター誤マッチはDB非依存ルーターで実行可）
- ルーター経由（DB非依存・extractor検証）: E03/E04/E05/B02（`build_router` の `oneshot`。デシリアライズ失敗はサービス到達前のため実DB不要だが、`test_app_state()` 利用時は `#[ignore]` 付与で既存規約に合流）
- 統合 `#[ignore]`（実DB＋外部APIモック）: TC-0024-01, 02, E01-I, E02-I, B01

**カテゴリ別内訳**:
- 正常系: 2ケース（TC-0024-01, 02）
- 異常系: 8ケース（E01-U, E01-I, E02-U, E02-U2, E02-I, E03, E04, E05）
- 境界値: 3ケース（B01, B02, B03）

> ライブ外部API／本番DBに依存しない検証の確保: エラーマッピングの核（422/502/400 のステータス・ワイヤーコード・非panic）は **TC-0024-E01-U / E02-U / E02-U2 / E03〜E05 / B02 / B03**（DB非依存ユニット／ルーター）で先行・確実に担保する。200成功・End-to-End（TC-0024-01/02/E01-I/E02-I/B01）は `ExternalSearchService` が実 `PgPool` を要するため `#[ignore]` 統合とし、外部APIはTASK-0023同様 `wiremock` ベースURL注入でライブAPI非依存に留める。

---

## 5. 開発言語・テストフレームワーク

- **プログラミング言語**: Rust（edition 2024）
  - **言語選択の理由**: 既存プロジェクト全体がRust + axum 0.8 + sqlx 0.8 + api-client-lib（ワークスペース内）で構築されており、本タスクも同一クレート（`mediavault-api`）内のハンドラ／ルート／エラーマッピング追加実装のため
  - **テストに適した機能**: `match` の網羅性検査で `ExternalSearchError`／`ApiError` variant処理を静的担保。`From` トレイト実装をDB非依存で直接 `assert` でき、エラーマッピングの核を高速・確実に検証できる。`Result`/`?` 伝播で非panicを構造的に保証
- **テストフレームワーク**: Rust標準テストハーネス（`cargo test`） + `tokio::test`（非同期 `search_items`／`oneshot`） + `tower::ServiceExt::oneshot`（ルーターレベル駆動・既存 routes/mod.rs パターン） + HTTPモック（`wiremock` 等・TASK-0023同様 `new_with_base_url` 注入）
  - **フレームワーク選択の理由**: 既存 `routes/mod.rs` の `#[cfg(test)] mod tests` が `build_router(state)` + `oneshot` でルーター駆動する確立済みパターンを持つ（不正値→400 の前例 L156-179）。`From<ExternalSearchError>` 変換はDB非依存ユニットで先行担保し、`ExternalSearchService` の実 `PgPool` 依存部（200/E2E）は `#[ignore]` 統合へ分離することで、live外部API／本番DBなしで主要検証を成立させる
  - **テスト実行環境**: ユニット（From変換・デシリアライズ・ルーター誤マッチ）はDB不要で `cargo test -p mediavault-api` にて即時実行。実DBキー取得＋外部APIモックを要するEnd-to-End（TC-0024-01/02/E01-I/E02-I/B01）は `docker compose up -d db` + `DATABASE_URL` を前提に `cargo test -- --ignored` で別実行（NFR-0024-03・`test_app_state()` 利用）
  - **依存追加**: 統合系の外部APIモックに `wiremock` を `backend/mediavault-api/Cargo.toml` `[dev-dependencies]` へ追加（TASK-0023で既に追加済みなら流用）。`tower` はルーター `oneshot` 用に既存利用（routes/mod.rs テストで使用済み）
- 🔵 信頼性レベル: 既存 routes/mod.rs テストパターン（`build_router`+`oneshot`+`#[ignore]`+`test_app_state`）・NFR-0024-03・要件定義書 第3章 L99・TASK-0023-testcases.md 第5章に直接対応

### テストケース実装時の日本語コメント指針（ルーターレベル例）

```rust
// 【テスト目的】: media_type欠落時にQuery抽出段階で400が返ることを確認する（REQ-0024-101）
// 【テスト内容】: GET /items/search?q=鬼滅 を build_router 経由で oneshot 実行する
// 【期待される動作】: レスポンスステータスが400であること
// 🔵 信頼性レベル: 要件 REQ-0024-101・EDGE-0024-02 に対応
#[tokio::test]
#[ignore] // test_app_state利用時は実DB前提。cargo test -- --ignored で実行
async fn search_items_without_media_type_returns_400() {
    // 【テストデータ準備】: media_type欠落クエリを用意（必須フィールド欠落を再現）
    // 【初期条件設定】: build_router で全ルート登録済みのルーターを構築
    let state = test_app_state().await;
    let app = build_router(state);

    // 【実際の処理実行】: GET /items/search?q=鬼滅 をルーター経由で実行
    // 【処理内容】: Query<ItemSearchQuery> のデシリアライズが必須media_type欠落で失敗する
    let response = app
        .oneshot(
            Request::builder()
                .uri("/items/search?q=%E9%AC%BC%E6%BB%85")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap(); // 【品質保証】: oneshotがOkを返す＝ハンドラ内panic無し

    // 【結果検証】: 必須パラメータ欠落が400で拒否されることを確認
    // 【検証項目】: ステータスコード==400
    assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 🔵
}
```

---

## 6. 要件定義との対応関係

- **参照した機能概要**: TASK-0024-requirements.md 第1章「機能の概要」（`ExternalSearchService::search` のHTTP層公開・media_type/q受理・エラーマッピング）
- **参照した入力・出力仕様**: 第2章（クエリ契約 L30-77＝`ItemSearchQuery`・MediaType 8 variant・`q` 空文字透過 L53、出力 `ApiOk<Vec<ExternalSearchResult>>` L58-70、データフロー L72-74）
- **参照した制約条件**: 第3章 ハンドラ設計（サービス都度構築・テスト容易化 L99）、第4章 エラーマッピング表（422/`API_KEY_NOT_CONFIGURED`・502/`EXTERNAL_API_TIMEOUT`・全ApiError集約 L113・非panic L114）、第5章 ルート登録（`/items/search` 前方登録）、第6章 機能要件（REQ-0024-01〜05・101〜103・401〜403）、第7章 NFR（NFR-0024-01〜03）
- **参照した使用例**: 第8章 シナリオ1〜4（TC-002-01/02/E01/E02）・エッジケース EDGE-0024-01〜05
- **参照した既存パターン**: `backend/mediavault-api/src/routes/mod.rs` `#[cfg(test)] mod tests`（`build_router`+`tower::ServiceExt::oneshot`+`#[ignore]`+`test_app_state()`、不正値→400 前例 L156-179）

## 7. 次フェーズ（tdd-red）への引き渡し事項

`tdd-red` 着手前に以下を確定すること（要件定義書 第9章 引き渡しL239-243より）:

1. **`Query` Rejection整形の要否**: 不正クエリ（E03/E04/E05）で統一 `ApiError`（`VALIDATION_ERROR`ボディ）を返すか、素のAxum 400で許容するか。→ E03/E04/E05/B01 の**ステータス==400 を必須アサーション**、`code=="VALIDATION_ERROR"` ボディアサーションは**整形採用時のみ有効**として実装時に有効化する（要件 第2章注記・既存 `GET /items` 不正値テストはボディ未検証）。
2. **`From<ExternalSearchError> for ApiError` の配置**: `handlers/items.rs` 内か `models/response.rs` か（`errors.rs` 不在・note.md L31）。→ E01-U/E02-U/E02-U2 は配置先に依存しない `ApiError::from(..)` の出力（ステータス・コード）をアサート。
3. **新規 `ApiErrorCode` variant名の確定**: `ApiKeyNotConfigured`(422/`API_KEY_NOT_CONFIGURED`)・`ExternalApiTimeout`(502/`EXTERNAL_API_TIMEOUT`)（要件 第4章・REQ-0024-401）。→ E01-U/E02-U の期待ワイヤーコード文字列を確定。
4. **`ExternalSearchService` のテスト時インスタンス化方針**: 実 `PgPool` 必須のため200/E2E（TC-0024-01/02/E01-I/E02-I/B01）は `#[ignore]` 統合とし、外部APIは `wiremock` ベースURL注入（TASK-0023流用）。→ live外部API／本番DB非依存を維持。
5. **空 `q` 方針**: 要件 第2章 L53「サービス層へ透過（本タスク要件未指定）」で確定。ハンドラは空 `q`（値あり）を400にせず `search(_, "")` を呼ぶ（`q` 欠落=400 とは区別）。→ B01 の期待値を透過で固定。
6. **`wiremock` `[dev-dependencies]` 追加**: TASK-0023で追加済みなら流用、未追加なら `backend/mediavault-api/Cargo.toml` へ追加。

**次のお勧めステップ**: `/tsumiki:tdd-red mediavault-backend TASK-0024` でRedフェーズ（失敗テスト作成）を開始します。
