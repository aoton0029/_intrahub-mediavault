# TASK-0024 TDDテストケース定義書: GET /items/search 外部API検索エンドポイント

**機能名**: item-search（外部API検索エンドポイント）
**タスクID**: TASK-0024
**要件名**: item-search
**フェーズ**: Phase 3 - 外部API連携
**作成日**: 2026-06-26
**出力ファイル**: `docs/implements/item-search/TASK-0024/item-search-testcases.md`

---

## 信頼性レベルの凡例

- 🔵 **青信号**: 要件定義書・設計文書・既存実装を参照し、ほぼ推測していない
- 🟡 **黄信号**: 元資料からの妥当な推測
- 🔴 **赤信号**: 元資料にない推測

---

## 0. テスト戦略・分類サマリ

本タスクは「外部API依存」かつ「実 PgPool 必須の `ExternalSearchService`」という特性があるため、テストを3層に分類する。

| 層 | 配置 | DB依存 | 実行方法 | 目的 |
|---|---|---|---|---|
| ユニット（DB非依存） | `models/response.rs` の `#[cfg(test)] mod tests` | なし | `cargo test -p mediavault-api` | 新規 `ApiErrorCode` variant のステータスマッピング・`From<ExternalSearchError>` 変換の検証 |
| ルーター統合（実DB） | `routes/mod.rs` の `#[cfg(test)] mod tests`（`#[ignore]`） | あり | `cargo test -- --ignored` | extractor 拒否（400）・ルート誤マッチ防止の検証 |
| E2E統合（実DB＋外部APIモック） | `routes/mod.rs`（`#[ignore]`、wiremock利用） | あり | `cargo test -- --ignored` | 200成功経路・422/502エラー経路のフルパス検証 |

- **テストフレームワーク**: Rust 標準 `#[test]` / `#[tokio::test]`（async）、ルーター駆動 `tower::ServiceExt::oneshot`、外部APIモック `wiremock` 0.6
- **信頼性レベル分布見込み**: 🔵 多数（要件・既存実装に直接対応）/ 🟡 一部（extractor 拒否ボディ形式・wiremock経路は妥当推測）/ 🔴 なし
- **参照元**: note.md 第5章、item-search-requirements.md 第2章・第7章

---

## 1. 正常系テストケース（基本的な動作）

### TC-0024-N01: anime 検索が 200 と Jikan 検索結果配列を返す（E2E統合）

- **テスト名**: anime 検索成功（Jikan 経由・provider=null）
  - **何をテストするか**: `media_type=anime` でリクエストすると、`ExternalSearchService` が Jikan クライアントへ振り分け、200 OK と `ApiOk<Vec<ExternalSearchResult>>` を返すこと
  - **期待される動作**: ハンドラ → `ExternalSearchService::new(db).search(MediaType::Anime, "鬼滅")` → wiremock がモックした Jikan 応答 → 200 で結果配列を返す
- **入力値**: `GET /items/search?media_type=anime&q=鬼滅`（Jikan ベースURL を wiremock に向ける）
  - **入力データの意味**: anime はキー不要の Jikan プロバイダへ振り分けられる代表ケース（TC-002-01）
- **期待される結果**: HTTPステータス `200 OK`、ボディ `{"success": true, "data": [...]}`、配列要素の `provider` は `null`（Jikan はキー不要のため `None`）
  - **期待結果の理由**: 要件 2.2・external_search.rs の `ExternalSearchResult.provider = Option<ApiProvider>`（Jikan時 None）に直接対応
- **テストの目的**: anime → Jikan の正常検索フローと成功エンベロープ形式を確認する
  - **確認ポイント**: ステータス200、`success=true`、`data` が配列、Jikan結果の `provider` が null
- 🔵 信頼性レベル: 要件 4.1 TC-002-01・external_search.rs L18-28・dataflow.md（機能1正常系）より

### TC-0024-N02: movie 検索が 200 と TMDb 検索結果配列を返す（E2E統合）

- **テスト名**: movie 検索成功（TMDb 経由・provider=tmdb）
  - **何をテストするか**: `media_type=movie` でリクエストすると、TMDb クライアントへ振り分けられ、200 OK と結果配列を返すこと（事前に TMDb キーが DB 登録済み）
  - **期待される動作**: ハンドラ → `search(MediaType::Movie, "...")` → wiremock がモックした TMDb 応答 → 200
- **入力値**: `GET /items/search?media_type=movie&q=Matrix`（TMDb キーを事前登録、TMDb ベースURL を wiremock に向ける）
  - **入力データの意味**: movie はキー必須の TMDb プロバイダへ振り分けられる代表ケース（TC-002-02）
- **期待される結果**: HTTPステータス `200 OK`、`{"success": true, "data": [...]}`、配列要素の `provider` が `"tmdb"`
  - **期待結果の理由**: 要件 4.1 TC-002-02・`ExternalSearchResult.provider = Some(ApiProvider::Tmdb)` に対応
- **テストの目的**: movie/drama → TMDb の正常検索フローと provider 値を確認する
  - **確認ポイント**: ステータス200、`data` が配列、要素の `provider` が `"tmdb"`
- 🔵 信頼性レベル: 要件 4.1 TC-002-02・external_search.rs テスト L73-86 より

### TC-0024-N03: drama 検索が 200 を返す（E2E統合・movie と同系）

- **テスト名**: drama 検索成功（TMDb 経由）
  - **何をテストするか**: `media_type=drama` も TMDb へ振り分けられ 200 を返すこと（movie と同じプロバイダ経路の網羅）
  - **期待される動作**: `search(MediaType::Drama, "...")` → TMDb モック → 200
- **入力値**: `GET /items/search?media_type=drama&q=半沢`（TMDb キー登録済み・wiremock）
  - **入力データの意味**: 要件 4.1 が movie/drama を同一プロバイダ（TMDb）扱いと明記しているため、drama も正常系として明示網羅する
- **期待される結果**: HTTPステータス `200 OK`、`{"success": true, "data": [...]}`
  - **期待結果の理由**: 要件 4.1「movie/drama検索（TC-002-02）→ TMDb」に対応
- **テストの目的**: drama が movie と同様 TMDb 経路で 200 を返すことを確認する
  - **確認ポイント**: drama の振り分けが movie と同経路で成功すること
- 🟡 信頼性レベル: 要件 4.1（movie/drama を併記）からの妥当な推測（drama 単独の受け入れ基準は明示されないが TMDb 同経路）

### TC-0024-N04: ItemSearchQuery が media_type/q を正しくデシリアライズする（ユニット）

- **テスト名**: ItemSearchQuery クエリ文字列デシリアライズ成功
  - **何をテストするか**: `media_type=anime&q=foo` のクエリ文字列が `ItemSearchQuery { media_type: Anime, q: "foo" }` にデシリアライズされること
  - **期待される動作**: `serde_urlencoded`（Axum Query の内部）相当のデシリアライズが成功する
- **入力値**: クエリ文字列 `media_type=anime&q=foo`
  - **入力データの意味**: DTO が必須2フィールドを正しく受理する最小正常ケース
- **期待される結果**: `media_type == MediaType::Anime`、`q == "foo"`
  - **期待結果の理由**: 要件 2.1 の DTO 定義（`pub media_type: MediaType, pub q: String`）に対応
- **テストの目的**: DTO のフィールド名・型・デシリアライズ契約を DB 非依存で確認する
  - **確認ポイント**: snake_case の `media_type` 文字列が enum へ正しく変換されること
- 🔵 信頼性レベル: 要件 2.1 DTO 定義・models/item.rs MediaType（snake_case Deserialize 実装済み）より

---

## 2. 異常系テストケース（エラーハンドリング）

### TC-0024-E01: APIキー未設定で 422 API_KEY_NOT_CONFIGURED（ユニット: From変換）

- **テスト名**: ApiKeyNotConfigured → ApiError(422) 変換
  - **エラーケースの概要**: TMDb 等キー必須プロバイダで DB にキー未登録のとき、サービスが `ExternalSearchError::ApiKeyNotConfigured(provider)` を返す状況
  - **エラー処理の重要性**: ユーザーがキー未設定を識別し設定画面へ誘導できるよう、汎用502と区別した専用コードが必要
- **入力値**: `ApiError::from(ExternalSearchError::ApiKeyNotConfigured(ApiProvider::Tmdb))`
  - **不正な理由**: キー必須プロバイダにキーが無い状態は検索不能であり処理継続できない
  - **実際の発生シナリオ**: 初回起動直後に TMDb キー未登録のまま movie を検索した場合
- **期待される結果**: 生成された `ApiError` の `status == 422 (UNPROCESSABLE_ENTITY)`、`error.code == "API_KEY_NOT_CONFIGURED"`
  - **エラーメッセージの内容**: 汎用メッセージ（プロバイダ名のみ、内部詳細は含めない）
  - **システムの安全性**: panic せず Result で伝播、DB 内部情報を漏らさない
- **テストの目的**: 新規 `ApiErrorCode` variant（422 / `API_KEY_NOT_CONFIGURED`）と `From` 変換を DB 非依存で確認する
  - **品質保証の観点**: 既存 `UnprocessableEntity`（`UNPROCESSABLE_ENTITY`）と文字列が異なる専用コードであることを保証
- 🔵 信頼性レベル: 要件 3（422 新規 variant 必要）・external_search.rs `ApiKeyNotConfigured` variant・note.md L91 より

### TC-0024-E02: 外部APIタイムアウトで 502 EXTERNAL_API_TIMEOUT（ユニット: From変換）

- **テスト名**: ExternalApiError(Timeout) → ApiError(502) 変換
  - **エラーケースの概要**: 外部API が応答せずタイムアウトした場合、サービスが `ExternalSearchError::ExternalApiError(ApiError::Timeout)` を返す状況
  - **エラー処理の重要性**: 外部障害時にサーバーが panic せず 502 を返し、クライアントに障害種別を伝える
- **入力値**: `ApiError::from(ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Timeout))`
  - **不正な理由**: 外部依存の障害でありローカル処理では回復不能
  - **実際の発生シナリオ**: 外部API が高負荷・ネットワーク遅延でタイムアウト
- **期待される結果**: `status == 502 (BAD_GATEWAY)`、`error.code == "EXTERNAL_API_TIMEOUT"`
  - **エラーメッセージの内容**: 汎用メッセージ（外部API生エラー詳細は含めない）
  - **システムの安全性**: `?` 伝播で panic 非発生（NFR エラー耐性）
- **テストの目的**: 新規 502 variant（`EXTERNAL_API_TIMEOUT`）と `From` 変換を確認する
  - **品質保証の観点**: 既存 `ExternalApiError`（`EXTERNAL_API_ERROR`）とコード文字列が異なる新コードであることを保証
- 🔵 信頼性レベル: 要件 3（502 新規 variant 必要・文字列不一致明記）・note.md L91 より

### TC-0024-E03: api_client_lib::ApiError 全variantが 502 へ集約される（ユニット: From変換）

- **テスト名**: ApiError 6 variant 全集約 → 502
  - **エラーケースの概要**: `Http / Auth / RateLimit / Parse / Timeout / Network` のいずれの外部エラーも一律 502 へ集約される
  - **エラー処理の重要性**: 外部エラーの分岐を増やさず一貫した 502 応答とし、情報漏洩・panic を防ぐ
- **入力値**: 各 `api_client_lib::ApiError` variant をラップした `ExternalSearchError::ExternalApiError(...)` を順に `From` 変換（パラメータ化テスト）
  - **不正な理由**: いずれも外部依存起因で処理継続不能
  - **実際の発生シナリオ**: レート制限超過・認証失敗・パース失敗など外部API側の多様な障害
- **期待される結果**: すべて `status == 502`、`error.code == "EXTERNAL_API_TIMEOUT"`
  - **エラーメッセージの内容**: いずれも汎用メッセージ
  - **システムの安全性**: match の網羅で未処理 variant が無いことを保証
- **テストの目的**: EDGE-0023-04（全 ApiError 集約）を変換層で確認する
  - **品質保証の観点**: 新 variant 追加時の取りこぼし（500 へ落ちる等）を防ぐ
- 🟡 信頼性レベル: 要件 3「全 variant を一律 502 へ集約」・EDGE-0023-04 より妥当推測（各 variant の構築引数は api_client_lib 実装に依存）

### TC-0024-E04: q パラメータ欠落で 400 VALIDATION_ERROR（ルーター統合）

- **テスト名**: 必須 q 欠落 → 400
  - **エラーケースの概要**: `q` が無い `?media_type=anime` のみのリクエストで、Axum `Query<ItemSearchQuery>` extractor がデシリアライズに失敗する状況
  - **エラー処理の重要性**: 必須パラメータ欠落をサービス層到達前に弾き、不要な外部API呼び出しを防ぐ
- **入力値**: `GET /items/search?media_type=anime`（q 欠落）
  - **不正な理由**: DTO の `q: String` は必須であり、欠落はデシリアライズ不能
  - **実際の発生シナリオ**: クライアントの実装ミスで検索語を付け忘れた場合
- **期待される結果**: HTTPステータス `400 BAD_REQUEST`
  - **エラーメッセージの内容**: 統一 `ApiError`（`VALIDATION_ERROR`）整形を採用する場合はボディも検証（第7章決定事項1に依存）
  - **システムの安全性**: 外部API未呼び出し、DB未アクセス
- **テストの目的**: 必須欠落時の extractor 拒否（400）を確認する
  - **品質保証の観点**: 既存 routes/mod.rs L156-179 の不正値400パターン踏襲
- 🟡 信頼性レベル: 要件 4.2「必須パラメータ欠落 → 400 VALIDATION_ERROR」より妥当推測（ボディ形式は決定事項1に依存）

### TC-0024-E05: media_type パラメータ欠落で 400 VALIDATION_ERROR（ルーター統合）

- **テスト名**: 必須 media_type 欠落 → 400
  - **エラーケースの概要**: `media_type` が無い `?q=foo` のみのリクエストで extractor が失敗する状況
  - **エラー処理の重要性**: どのプロバイダへ振り分けるか決定不能なため早期拒否が必要
- **入力値**: `GET /items/search?q=foo`（media_type 欠落）
  - **不正な理由**: `media_type: MediaType` は必須であり欠落はデシリアライズ不能
  - **実際の発生シナリオ**: クライアントが media_type を付与し忘れた場合
- **期待される結果**: HTTPステータス `400 BAD_REQUEST`
  - **エラーメッセージの内容**: 決定事項1に従い VALIDATION_ERROR 想定
  - **システムの安全性**: 外部API・DB 未アクセス
- **テストの目的**: media_type 欠落時の extractor 拒否（400）を確認する
  - **品質保証の観点**: 2つの必須パラメータそれぞれの欠落を独立に網羅
- 🟡 信頼性レベル: 要件 2.1（media_type 必須）・4.2 より妥当推測

### TC-0024-E06: 無効な media_type 値で 400 VALIDATION_ERROR（ルーター統合）

- **テスト名**: media_type=invalid → 400
  - **エラーケースの概要**: enum に存在しない `media_type=invalid` で extractor が enum デシリアライズに失敗する状況
  - **エラー処理の重要性**: 不正な列挙値を弾き、想定外プロバイダ振り分けを防止
- **入力値**: `GET /items/search?media_type=invalid&q=foo`
  - **不正な理由**: MediaType enum の8 variant（anime/movie/drama/manga/novel/game/academic_book/paper）に存在しない値
  - **実際の発生シナリオ**: クライアントのタイプミスや未対応メディア種別の指定
- **期待される結果**: HTTPステータス `400 BAD_REQUEST`
  - **エラーメッセージの内容**: VALIDATION_ERROR 想定
  - **システムの安全性**: 外部API・DB 未アクセス
- **テストの目的**: enum 外文字列の extractor 拒否（400）を確認する
  - **品質保証の観点**: 既存 GET /items の `media_type=invalid → 400`（routes/mod.rs L160-179）と同一方針の踏襲を保証
- 🔵 信頼性レベル: 要件 4.2「media_type 不正値 → 400 VALIDATION_ERROR」・routes/mod.rs L156-179 既存パターンより

---

## 3. 境界値テストケース（最小値、最大値、null等）

### TC-0024-B01: q 空文字（q=）は 400 化せず透過しサービス層へ委譲（E2E統合）

- **テスト名**: q 空文字の透過動作
  - **境界値の意味**: 「空文字」は「欠落」と区別される境界。欠落は 400 だが、空文字は透過して `search(_, "")` を呼ぶ（TASK-0023踏襲）
  - **境界値での動作保証**: 空文字でハンドラが 400 を返さず、サービス層の戻り値（200 空配列等）がそのまま反映される
- **入力値**: `GET /items/search?media_type=anime&q=`（wiremock で空クエリ応答をモック）
  - **境界値選択の根拠**: 要件 4.2・note.md L193「q 空文字は 400 化せず透過」に直接対応
  - **実際の使用場面**: 検索ボックスを空のまま送信した場合
- **期待される結果**: HTTPステータスは 400 ではない（モック応答に応じ 200 / `{"success": true, "data": [...]}` 等）。ハンドラが空文字を理由に拒否しない
  - **境界での正確性**: 空文字が DTO の `q: String` に空文字として束縛され、サービスへ `""` が渡る
  - **一貫した動作**: 「欠落=400（TC-0024-E04）」と「空文字=透過（本ケース）」の境界が一貫
- **テストの目的**: 空文字と欠落の扱いの差異（透過 vs 拒否）を確認する
  - **堅牢性の確認**: 空入力でもサーバーが安全に外部API/サービスへ委譲できること
- 🟡 信頼性レベル: 要件 4.2・note.md L193・第7章決定事項5（q 空文字透過確定 🔵）より妥当推測（応答ステータスはモック内容依存）

### TC-0024-B02: ルート誤マッチ防止 — /items/search が /items/:id に吸われない（ルーター統合）

- **テスト名**: /items/search と /items/:id の登録順序による誤マッチ防止
  - **境界値の意味**: リテラルパス `/items/search` と動的パス `/items/:id` の競合境界。`search` が `:id` として誤解釈されないことを保証
  - **境界値での動作保証**: `/items/search` がリテラルパス（search_items）へ到達し、UUID パースエラーの 400（get_item_handler 経路）にならない
- **入力値**: `GET /items/search?media_type=anime&q=foo`（wiremock で正常応答をモック）
  - **境界値選択の根拠**: 要件 3「`/items/search` を `/items/:id` より前に登録」・note.md L48-50 に直接対応
  - **実際の使用場面**: 検索エンドポイント呼び出し時に毎回発生する基本経路
- **期待される結果**: `/items/:id`（UUID 必須）の経路へ落ちず、search ハンドラへ到達（200 系、または extractor 検証なら400 だが UUID 起因の400ではない）
  - **境界での正確性**: ルーティングがリテラル優先で解決される
  - **一貫した動作**: 登録順序を入れ替えても（あるいは将来 Axum バージョンが変わっても）誤マッチしない安全策の検証
- **テストの目的**: ルート登録順序が正しく、search が個別取得に吸われないことを確認する
  - **堅牢性の確認**: パスマッチングの優先順位への耐性
- 🔵 信頼性レベル: 要件 3 ルーティング制約・note.md L47-50・L192 より

### TC-0024-B03: MediaType 全8 variant が media_type として受理される（ユニット）

- **テスト名**: MediaType 境界（全列挙値）デシリアライズ網羅
  - **境界値の意味**: 許容集合の「全要素」と「集合外1件」の境界。8 variant すべてが受理され、それ以外が拒否される
  - **境界値での動作保証**: anime/movie/drama/manga/novel/game/academic_book/paper の8値すべてが `ItemSearchQuery` にデシリアライズ可能
- **入力値**: 各 `media_type=<variant>&q=x`（8パターン）＋ 集合外 `media_type=unknown`（1パターン）
  - **境界値選択の根拠**: 要件 2.1 の許容値8種・models/item.rs L15-24 に直接対応
  - **実際の使用場面**: 各メディア種別での検索呼び出し
- **期待される結果**: 8 variant はデシリアライズ成功し対応 enum 値になる。集合外1件はデシリアライズ失敗（Err）
  - **境界での正確性**: snake_case 文字列と enum の1対1対応
  - **一貫した動作**: 受理集合の内側（8値）と外側（unknown）で動作が一貫
- **テストの目的**: 許容列挙値の完全網羅と集合外拒否を DB 非依存で確認する
  - **堅牢性の確認**: enum 拡張・縮小時の取りこぼし検出
- 🔵 信頼性レベル: 要件 2.1 許容値・models/item.rs L15-24（既存 snake_case Deserialize）より

---

## 4. 開発言語・フレームワーク

- **プログラミング言語**: Rust（edition 2024）
  - **言語選択の理由**: 既存 mediavault-api が Rust + Axum 構成（Cargo.toml）。型安全・`Result` ベースのエラー伝播が本タスクのエラーマッピング検証に適する
  - **テストに適した機能**: `#[test]` / `#[tokio::test]`、`matches!` マクロ、enum match の網羅性検査
- **テストフレームワーク**: Rust 標準テスト + tokio + tower + wiremock
  - **フレームワーク選択の理由**: 既存テスト（response.rs / external_search.rs / routes/mod.rs）が同構成。ルーター駆動は `tower::ServiceExt::oneshot`、外部APIモックは `wiremock` 0.6（note.md L148）
  - **テスト実行環境**: ユニットは `cargo test -p mediavault-api`、実DB/E2Eは `docker compose up -d db` ＋ `cargo test -- --ignored`
- 🔵 信頼性レベル: note.md 第1章・第5章、Cargo.toml、既存テストパターンより

---

## 5. テストケース実装時の日本語コメント指針

各テストに以下の日本語コメントを必須とする（既存 response.rs / routes/mod.rs の規約踏襲）。

#### テストケース開始時

```rust
// 【テスト目的】: [このテストで確認することを明記]
// 【テスト内容】: [具体的にどの処理をテストするか]
// 【期待される動作】: [正常時/エラー時の結果]
// 🔵🟡🔴 信頼性レベル: [根拠資料]より
```

#### Given / When / Then

```rust
// 【テストデータ準備】: [なぜこのデータを用意するか]
// 【初期条件設定】: [テスト前の状態]
let app = build_router(test_app_state().await); // ルーター構築

// 【実際の処理実行】: [呼び出すハンドラ/メソッド]
let response = app.oneshot(request).await.unwrap();

// 【結果検証】: [何を検証するか]
// 【期待値確認】: [期待結果とその理由]
assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 〜であることを確認 🔵
```

#### セットアップ・クリーンアップ（E2E）

```rust
// 【テスト前準備】: wiremock サーバ起動・モック応答登録・必要なら TMDb キー DB 登録
// 【環境初期化】: 外部APIベースURLを wiremock のURLへ注入
// 【テスト後処理】: 登録した api_credentials 行のクリーンアップ（既存 cleanup_provider 踏襲）
```

---

## 6. 要件定義との対応関係

- **参照した機能概要**: item-search-requirements.md 第1章（GET /items/search の役割）
- **参照した入力・出力仕様**: 同 第2章（ItemSearchQuery DTO・ExternalSearchResult 出力・データフロー）
- **参照した制約条件**: 同 第3章（AppState 都度構築・ルート登録順序・新 ApiErrorCode variant・エラー集約・情報漏洩防止）
- **参照した使用例**: 同 第4章（4.1 正常系 TC-002-01/02、4.2 エラー・エッジケース）
- **参照した既存実装**:
  - `backend/mediavault-api/src/models/response.rs`（ApiError / ApiErrorCode / code_and_status・既存テストパターン）
  - `backend/mediavault-api/src/models/external_search.rs`（ExternalSearchResult / ExternalSearchError）
  - `backend/mediavault-api/src/routes/mod.rs`（build_router・oneshot 統合テスト・cleanup_provider・#[ignore] パターン）
  - `backend/mediavault-api/src/handlers/items.rs`（既存ハンドラパターン）

---

## 7. テストケース一覧表

| ID | 分類 | テスト層 | 概要 | 期待結果 | 信頼性 |
|---|---|---|---|---|---|
| TC-0024-N01 | 正常 | E2E統合 | anime → Jikan 検索 | 200 / provider=null | 🔵 |
| TC-0024-N02 | 正常 | E2E統合 | movie → TMDb 検索 | 200 / provider=tmdb | 🔵 |
| TC-0024-N03 | 正常 | E2E統合 | drama → TMDb 検索 | 200 | 🟡 |
| TC-0024-N04 | 正常 | ユニット | ItemSearchQuery デシリアライズ | Anime / "foo" | 🔵 |
| TC-0024-E01 | 異常 | ユニット | ApiKeyNotConfigured → 422 | 422 / API_KEY_NOT_CONFIGURED | 🔵 |
| TC-0024-E02 | 異常 | ユニット | Timeout → 502 | 502 / EXTERNAL_API_TIMEOUT | 🔵 |
| TC-0024-E03 | 異常 | ユニット | ApiError 全variant → 502 集約 | 502 全件 | 🟡 |
| TC-0024-E04 | 異常 | ルーター統合 | q 欠落 | 400 VALIDATION_ERROR | 🟡 |
| TC-0024-E05 | 異常 | ルーター統合 | media_type 欠落 | 400 VALIDATION_ERROR | 🟡 |
| TC-0024-E06 | 異常 | ルーター統合 | media_type 不正値 | 400 VALIDATION_ERROR | 🔵 |
| TC-0024-B01 | 境界 | E2E統合 | q 空文字透過 | 非400（サービス委譲） | 🟡 |
| TC-0024-B02 | 境界 | ルーター統合 | ルート誤マッチ防止 | search 到達（非UUID400） | 🔵 |
| TC-0024-B03 | 境界 | ユニット | MediaType 全8variant + 集合外 | 8受理 / 1拒否 | 🔵 |

**合計**: 13ケース（正常系4 + 異常系6 + 境界値3）

---

## 8. 品質判定

```
✅ 高品質:
- テストケース分類: 正常系・異常系・境界値を網羅（4/6/3）。要求された anime/movie 正常、422/502/400 異常、空文字/不正値境界をすべてカバー
- 期待値定義: 全ケースで HTTP ステータス・エラーコード文字列・provider 値等を具体的に明示
- 技術選択: Rust + #[tokio::test] + tower::oneshot + wiremock 確定（既存テストと整合）
- 実装可能性: 前提 TASK-0023 実装済み、対象ファイル・テスト配置・モック手段が特定済み
- 信頼性レベル分布: 🔵 8件 / 🟡 5件 / 🔴 0件（要件・既存実装に直接対応するものが多数）
```

**全体評価**: 高品質。`From` 変換のユニットテストでエラーマッピング（422/502）を DB 非依存に検証でき、Red フェーズで最小コストで失敗テストを作成可能。

---

## 9. tdd-red 着手前の再確認事項（要件 第7章 由来）

1. 🟡 不正クエリ時に統一 `ApiError`（`VALIDATION_ERROR` ボディ）を返すか素の Axum 400 か（TC-0024-E04/E05/E06 のボディ検証可否に影響）
2. 🟡 `From<ExternalSearchError> for ApiError` の配置（`models/response.rs` 推奨。TC-0024-E01/E02/E03 はこの配置を前提）
3. 🟡 新 `ApiErrorCode` variant 名（例: `ApiKeyNotConfigured` / `ExternalApiTimeout`）の確定
4. 🟡 E2E（TC-0024-N01〜N03, B01）は `#[ignore]` ＋ wiremock ＋ 実DB方針。外部APIベースURL注入手段の最終確認
5. 🔵 q 空文字は透過（TC-0024-B01）でサービス層判定（TASK-0023 踏襲）

---

## 次のステップ

次のお勧めステップ: `/tsumiki:tdd-red item-search TASK-0024` で Red フェーズ（失敗テスト作成）を開始します。
