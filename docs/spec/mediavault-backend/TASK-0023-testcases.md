# TASK-0023 テストケース一覧: ExternalSearchServiceラッパー実装（media_type→provider振り分け）

**作成日**: 2026-06-25
**関連要件**: [TASK-0023-requirements.md](TASK-0023-requirements.md)
**関連タスク**: [TASK-0023.md](../../tasks/mediavault-backend/TASK-0023.md)
**関連ノート**: [note.md](note.md) TASK-0023セクション
**対象API**: `ExternalSearchService::search(media_type, query) -> Result<Vec<ExternalSearchResult>, ExternalSearchError>`

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・タスク仕様・既存実装（note.md記載）から確実な根拠があるテストケース
- 🟡 **黄信号**: 要件定義書・既存実装から妥当な推測によるテストケース
- 🔴 **赤信号**: 元の資料にない推測によるテストケース（本ドキュメントには無し）

## 0. テスト分類・配置方針

note.md「テスト規約（既存方針を継続）」L52-55・要件定義書 NFR-0023-04 に基づき、本タスクのテストは以下の2系統に分かれる。本タスクの主対象は **DB非依存のディスパッチロジック単体テスト**（モック/HTTPモックサーバー使用）であり、実DBキー取得確認は分離する。

| 種別 | 実行属性 | 実行コマンド | 対象 | 配置先 |
|---|---|---|---|---|
| ユニット（DB非依存・HTTPモック） | `#[tokio::test]` | `cargo test -p mediavault-api` | media_type→provider ディスパッチの正当性（対象プロバイダURLにのみリクエスト到達／他は非到達）、`ApiError`→`ExternalSearchError::ExternalApiError`変換、Jikanのキー取得スキップ、`ExternalSearchResult`変換、`ExternalSearchError`/`ExternalSearchResult`のシリアライズ/型定義 | `backend/mediavault-api/src/services/external_search.rs` / `backend/mediavault-api/src/models/external_search.rs` の `#[cfg(test)] mod tests` |
| 統合（実DB必要） | `#[tokio::test]` + `#[ignore]` | `cargo test -- --ignored`（事前に `docker compose up -d db`） | `find_by_provider`経由のDBキー取得→クライアント初期化のEnd-to-End、キー未登録時の`ApiKeyNotConfigured`（実DB経路）、キー必須プロバイダでの外部API非呼び出し確認 | `backend/mediavault-api/src/services/external_search.rs` の `#[cfg(test)] mod tests`（`#[ignore]`付与） |

**テスト手段の前提（第7章・note.md L29-32）**: `ApiClient::execute` がRPITIT（dyn非互換）のため `mockall::automock` の素トレイト適用が困難な可能性がある。本テストケースは原則 **(a) HTTPモックサーバー（`wiremock`等）を `new_with_base_url` で各クライアントへ注入し、「対象プロバイダURLにのみリクエストが到達／他には非到達」をHTTPレベルで検証**する方針を採る。`mockall`採用可否は tdd-red 着手前に確定する（要件定義書 第7章）。「executeのみが呼ばれる」検証は「対象プロバイダのモックサーバーにのみリクエストが到達し、他プロバイダのモックサーバーには到達しない」検証へ置換する。

**確定前提（実装着手前に固定すべき項目・要件定義書 次フェーズ引き渡しL252-255）**: マッピングは設計判断A（manga→Jikan）/B（game→IGDB固定）/C（anime→Jikanのみ・AniList対象外）を採用。`ExternalSearchResult.provider` のJikan表現（`Option<ApiProvider>` or DTO専用enum）は実装判断。本テストケースは provider表現に依存しない形でアサーションを記述し、確定方式に合わせて期待値を具体化する。

---

## 1. 正常系テストケース（基本的な動作）

### TC-002-01-A: media_type=Anime → Jikanのみへディスパッチ（ユニット・HTTPモック）

- **テスト名**: `search(Anime, query)` がJikanモックサーバーにのみリクエストを送り、他プロバイダには一切到達しない
  - **何をテストするか**: `MediaType::Anime` のとき第2章マッピング表どおりJikanクライアントの `execute` のみが実行されるか（REQ-0023-02・設計判断C）
  - **期待される動作**: JikanモックサーバーのみがHTTPリクエストを1回受信し、TMDb/IGDB/NDL/OpenLibrary用モックサーバーは受信0回。キー取得（`find_by_provider`）も発生しない
- **入力値**: `MediaType::Anime` + `query="鬼滅の刃"`、各プロバイダのモックサーバーURLを注入済み
  - **入力データの意味**: 要件定義書 シナリオ1（TC-002-01）の代表入力。anime→Jikanの🔵確定マッピングを表す
- **期待される結果**: `Ok(Vec<ExternalSearchResult>)` が返り、Jikanモックの受信回数==1、他全プロバイダモックの受信回数==0
  - **期待結果の理由**: REQ-0023-02「Animeの場合Jikanのみ呼び出し、他プロバイダを呼び出してはならない」に直接対応
- **テストの目的**: anime単一ディスパッチの正当性と「他プロバイダ非到達」を同時に保証する
  - **確認ポイント**: 隣接enum variant（Manga/Movie）へ誤ディスパッチしていないこと、Jikanがキー不要で動くこと
- 🔵 信頼性レベル: 要件定義書 REQ-0023-02・シナリオ1・TC-002-01、タスクファイル テストケース1 L75-79より

### TC-002-02-A: media_type=Movie → DBキーで初期化したTMDbへディスパッチ（統合・実DB）

- **テスト名**: `search(Movie, query)` が `find_by_provider(Tmdb)` のキーで初期化したTMDbクライアントの `execute` を呼ぶ
  - **何をテストするか**: `MediaType::Movie` でDBからTMDbキーを取得し、そのキーで初期化されたTMDbクライアントへディスパッチされるか（REQ-0023-03）
  - **期待される動作**: `find_by_provider(Tmdb)` が呼ばれ取得キーでTMDbクライアントが構築され、TMDbモック（または実TMDb）へリクエストが到達。他プロバイダは非到達
- **入力値**: `MediaType::Movie` + `query="タイトル"`、`api_credentials` に `provider=tmdb` キー登録済み
  - **入力データの意味**: 要件定義書 シナリオ2（TC-002-02）の代表入力。キー必須プロバイダの正常経路を表す
- **期待される結果**: `Ok(Vec<ExternalSearchResult>)` が返り、TMDbへリクエスト到達。注入キーがAuthStrategyとしてリクエストに反映されている
  - **期待結果の理由**: REQ-0023-03・REQ-0023-403（既存`find_by_provider`利用）に対応
- **テストの目的**: キー必須プロバイダのDBキー取得→クライアント初期化→ディスパッチのEnd-to-Endを保証する
  - **確認ポイント**: DBキーが実際にクライアントへ注入されること、他プロバイダ非到達
- 🔵 信頼性レベル: 要件定義書 REQ-0023-03・シナリオ2・TC-002-02、タスクファイル テストケース2 L81-85より

### TC-002-02-B: media_type=Drama → TMDbへディスパッチ（ユニット・HTTPモック）

- **テスト名**: `search(Drama, query)` がTMDbモックサーバーへ到達し、他プロバイダには到達しない
  - **何をテストするか**: `MediaType::Drama` がMovieと同一のTMDbへ写像されるか（REQ-0023-03・マッピング表）
  - **期待される動作**: TMDbモックのみ受信1回、他は0回
- **入力値**: `MediaType::Drama` + `query="タイトル"`、TMDbキー注入済み（モック）
  - **入力データの意味**: Movieと同一providerへ写像される第2のmedia_type。Drama/Movie両方がTMDbへ向かう網羅
- **期待される結果**: `Ok(...)`、TMDbモック受信==1、他==0
  - **期待結果の理由**: マッピング表 `Drama→TMDb`（🔵）に対応。MovieとDramaが同一providerでも個別に検証する必要があるため独立ケース化
- **テストの目的**: Drama→TMDbの写像を独立に保証する
  - **確認ポイント**: Movieとコードパスを共有しても、Drama単体で正しく到達すること
- 🔵 信頼性レベル: 要件定義書 マッピング表 L37・REQ-0023-03より

### TC-002-MANGA: media_type=Manga → Jikanへディスパッチ（キー取得スキップ・ユニット）

- **テスト名**: `search(Manga, query)` がJikanモックへ到達し、`find_by_provider` を一切呼ばない
  - **何をテストするか**: 設計判断Aにより `MediaType::Manga` がJikan（キー不要）へ写像され、キー取得をスキップするか（REQ-0023-04・REQ-0023-102）
  - **期待される動作**: Jikanモック受信==1、他プロバイダ受信==0、`find_by_provider` 呼び出し==0（DB非アクセス）
- **入力値**: `MediaType::Manga` + `query="ワンピース"`
  - **入力データの意味**: 設計判断A（types.rs L288 manga_details.jikan_id）の🟡マッピングを固定する代表入力。EDGE-0023-01
- **期待される結果**: `Ok(...)`、Jikanモック受信==1、OpenLibrary/NDLモック受信==0、キー取得スキップ
  - **期待結果の理由**: REQ-0023-04・REQ-0023-102（Jikanはキー取得スキップ）・設計判断Aに対応。タスク本文の「manga→OpenLibrary」ではなくJikanへ向かうことを固定する
- **テストの目的**: 設計判断A（manga→Jikan）とキー取得スキップを保証する
  - **確認ポイント**: OpenLibrary（タスク本文の旧候補）へ誤到達しないこと
- 🟡 信頼性レベル: 要件定義書 設計判断A・EDGE-0023-01・REQ-0023-04（types.rs L288からの妥当な推測）より

### TC-002-NOVEL: media_type=Novel → OpenLibraryへディスパッチ（ユニット・HTTPモック）

- **テスト名**: `search(Novel, query)` がOpenLibraryモックへ到達し、他プロバイダには到達しない
  - **何をテストするか**: `MediaType::Novel` がOpenLibrary（キー必須）へ写像されるか（REQ-0023-05・マッピング表）
  - **期待される動作**: OpenLibraryモック受信==1、他==0。`find_by_provider(OpenLibrary)` で取得したキーで初期化
- **入力値**: `MediaType::Novel` + `query="タイトル"`、OpenLibraryキー注入済み（モック）
  - **入力データの意味**: novel_details.openlibrary_id（types.rs L299）に基づく🔵マッピング。mangaとの分岐差（manga→Jikan / novel→OpenLibrary）を固定
- **期待される結果**: `Ok(...)`、OpenLibraryモック受信==1、Jikan/NDL受信==0
  - **期待結果の理由**: マッピング表 `Novel→OpenLibrary`（🔵）・REQ-0023-05に対応
- **テストの目的**: novel→OpenLibrary写像、およびmangaと別providerへ分岐することを保証する
  - **確認ポイント**: 設計判断Aで分離したmanga(Jikan)/novel(OpenLibrary)が混同されないこと
- 🔵 信頼性レベル: 要件定義書 マッピング表 L39・REQ-0023-05（types.rs L299）より

### TC-002-GAME: media_type=Game → IGDBへディスパッチ（Steam非到達・ユニット）

- **テスト名**: `search(Game, query)` がIGDBモックへ到達し、Steamには到達しない
  - **何をテストするか**: 設計判断Bにより `MediaType::Game` がIGDB（キー必須）へ固定写像され、Steamへは到達しないか（REQ-0023-05・EDGE-0023-02）
  - **期待される動作**: IGDBモック受信==1、Steamモック受信==0、他プロバイダ受信==0。`find_by_provider(Igdb)` で初期化
- **入力値**: `MediaType::Game` + `query="ゼルダの伝説"`、IGDBキー注入済み（モック）
  - **入力データの意味**: game_detailsが`steam_appid`/`igdb_id`両保持の真の曖昧点（note.md L39）に対し、IGDB固定を確定する代表入力。EDGE-0023-02
- **期待される結果**: `Ok(...)`、IGDBモック受信==1、Steamモック受信==0
  - **期待結果の理由**: 設計判断B・REQ-0023-05・REQ-0023-501（Steam切替は対象外）に対応。Steam非到達は本タスクの単一プロバイダ方針の中核
- **テストの目的**: game→IGDB固定とSteam非到達（隣接候補プロバイダへの誤到達防止）を保証する
  - **確認ポイント**: Steamモックへの到達が0であること（最重要・設計判断Bの回帰防止）
- 🟡 信頼性レベル: 要件定義書 設計判断B・EDGE-0023-02・note.md L39（IGDB固定の妥当な推測）より

### TC-002-ACADEMIC: media_type=AcademicBook → NDLへディスパッチ（ユニット・HTTPモック）

- **テスト名**: `search(AcademicBook, query)` がNDLモックへ到達し、他プロバイダには到達しない
  - **何をテストするか**: `MediaType::AcademicBook` がNDL（キー必須）へ写像されるか（REQ-0023-05・マッピング表）
  - **期待される動作**: NDLモック受信==1、他==0。`find_by_provider(Ndl)` で初期化
- **入力値**: `MediaType::AcademicBook` + `query="量子力学"`、NDLキー注入済み（モック）
  - **入力データの意味**: academic_book_details.ndl_id（types.rs L321）に基づく🔵マッピング。Paperと同一NDLへ向かうペアの一方
- **期待される結果**: `Ok(...)`、NDLモック受信==1、他==0
  - **期待結果の理由**: マッピング表 `AcademicBook→NDL`（🔵）・REQ-0023-05・EDGE-0023-03に対応
- **テストの目的**: academic_book→NDL写像を保証する
  - **確認ポイント**: 隣接variant Paper/Novelと混同せずNDLへ到達すること
- 🔵 信頼性レベル: 要件定義書 マッピング表 L41・EDGE-0023-03（types.rs L321）より

### TC-002-PAPER: media_type=Paper → NDLへディスパッチ（ユニット・HTTPモック）

- **テスト名**: `search(Paper, query)` がNDLモックへ到達し、他プロバイダには到達しない
  - **何をテストするか**: `MediaType::Paper` がNDL（キー必須）へ写像されるか（REQ-0023-05・マッピング表）
  - **期待される動作**: NDLモック受信==1、他==0
- **入力値**: `MediaType::Paper` + `query="機械学習"`、NDLキー注入済み（モック）
  - **入力データの意味**: paper_details.ndl_id（types.rs L334）に基づく🔵マッピング。AcademicBookと同一NDLへ向かうことを個別検証（8 variant全網羅のため独立ケース化）
- **期待される結果**: `Ok(...)`、NDLモック受信==1、他==0
  - **期待結果の理由**: マッピング表 `Paper→NDL`（🔵）・REQ-0023-05・EDGE-0023-03に対応
- **テストの目的**: paper→NDL写像をAcademicBookと独立に保証し、8 variant全てのディスパッチ網羅を完成させる
  - **確認ポイント**: AcademicBookとコードパスを共有してもPaper単体で正しく到達すること
- 🔵 信頼性レベル: 要件定義書 マッピング表 L42・EDGE-0023-03（types.rs L334）より

### TC-002-RESULT: 成功時にプロバイダModelが`ExternalSearchResult`へ変換される（ユニット）

- **テスト名**: `execute` の `ApiResponse<Model>` が `Vec<ExternalSearchResult>` にアダプタ変換される
  - **何をテストするか**: 成功レスポンスのプロバイダ固有Modelが共通DTO（media_type/provider/external_id/title/raw_data）へ正しく変換されるか（REQ-0023-06）
  - **期待される動作**: 返却の各 `ExternalSearchResult` が、入力 `media_type` と採用provider、元のexternal_id・title・raw_data（`ApiResponse.raw`由来）を保持する
- **入力値**: TMDbモックが既知のJSON（id/title等を含む）を返すよう設定 + `MediaType::Movie`
  - **入力データの意味**: アダプタ変換の正当性を確認する代表入力。ExternalSearchResult契約（要件定義書 第3章）の検証
- **期待される結果**: `result[0].media_type==Movie`、`external_id`==モックのid、`title`==モックのtitle、`raw_data`が生JSONを保持
  - **期待結果の理由**: REQ-0023-06（プロバイダModel→ExternalSearchResult変換）・要件定義書 第3章 出力仕様に対応
- **テストの目的**: アダプタ変換のフィールドマッピング正当性を保証する
  - **確認ポイント**: `provider`フィールドのJikan表現（Option or 専用enum）が確定方式どおりであること
- 🟡 信頼性レベル: 要件定義書 REQ-0023-06・第3章 出力仕様（ラップ形式は実装詳細未確定のため妥当な推測）より

---

## 2. 異常系テストケース（エラーハンドリング）

### TC-002-E01-A: キー必須プロバイダでキー未設定→ApiKeyNotConfigured（外部API非呼び出し・統合）

- **テスト名**: `search(Movie, query)` でTMDbキー未登録時に `ApiKeyNotConfigured(Tmdb)` を返し、外部API呼び出しが発生しない
  - **エラーケースの概要**: キー必須プロバイダ（TMDb）で `find_by_provider` が `None` を返した場合のディスパッチ前停止
  - **エラー処理の重要性**: キーが無い状態で外部APIを叩くと認証エラー/無駄な通信が発生するため、DBキー確認段階で早期returnする保証が必要
- **入力値**: `MediaType::Movie` + `query="タイトル"`、`api_credentials` に `tmdb` 行が存在しないクリーンな状態
  - **不正な理由**: キー必須プロバイダにキーが未登録（`find_by_provider(Tmdb)==None`）
  - **実際の発生シナリオ**: 初期セットアップ直後、TMDbキー未登録の状態で検索された場合
- **期待される結果**: `Err(ExternalSearchError::ApiKeyNotConfigured(ApiProvider::Tmdb))` が返り、TMDbモックサーバーへのリクエスト到達==0（外部API一切呼ばない）
  - **エラーメッセージの内容**: variant自体がprovider情報（`Tmdb`）を保持。HTTPステータス変換（422）は後続TASK-0024責務
  - **システムの安全性**: 外部APIへの無駄な/失敗確実なリクエストを送らないこと（最重要検証）
- **テストの目的**: REQ-0023-101（キー未設定→ApiKeyNotConfigured・外部API非呼び出し）を保証する
  - **品質保証の観点**: キー欠如時の早期停止が、後続のHTTP 422マッピングの前提を満たすこと
- 🔵 信頼性レベル: 要件定義書 REQ-0023-101・シナリオ3・TC-002-E01・テストケース3 L87-91より

### TC-002-E01-B: 各キー必須プロバイダで未設定時に対応providerのApiKeyNotConfiguredを返す（統合・パラメタライズド）

- **テスト名**: IGDB/NDL/OpenLibrary それぞれでキー未登録時に `ApiKeyNotConfigured(該当provider)` を返す
  - **エラーケースの概要**: TMDb以外のキー必須プロバイダ（IGDB/NDL/OpenLibrary）でも同一ロジックが機能するか
  - **エラー処理の重要性**: キー必須プロバイダ全てで一貫してキー確認→早期returnされる必要があるため
- **入力値**: `(Game, None)`→Igdb、`(Paper, None)`→Ndl、`(Novel, None)`→OpenLibrary の3組（いずれもキー未登録）
  - **不正な理由**: 各キー必須プロバイダのキーが未登録
  - **実際の発生シナリオ**: 特定プロバイダのキーのみ未設定で対応media_typeを検索した場合
- **期待される結果**: それぞれ `Err(ApiKeyNotConfigured(Igdb))` / `(Ndl)` / `(OpenLibrary)` を返し、いずれも外部API非到達
  - **エラーメッセージの内容**: 返却variantが正しいproviderを保持（GameはIgdb・設計判断B、NovelはOpenLibrary、PaperはNdl）
  - **システムの安全性**: 全キー必須プロバイダで外部API非呼び出しが一貫すること
- **テストの目的**: REQ-0023-101のキー必須プロバイダ網羅（TMDb単体ではなく全キー必須providerでの保証）
  - **品質保証の観点**: provider取り違え（例: Game時にApiKeyNotConfigured(Tmdb)等）がないことの回帰防止
- 🔵 信頼性レベル: 要件定義書 REQ-0023-101（キー必須プロバイダ列挙）・マッピング表・設計判断Bより

### TC-002-E02-A: 外部APIタイムアウト→ExternalApiError（panicしない・ユニット）

- **テスト名**: TMDbクライアントが `ApiError::Timeout` を返すとき `ExternalApiError` を返し、panicしない
  - **エラーケースの概要**: 外部API応答が遅延しタイムアウトした場合のエラー伝播
  - **エラー処理の重要性**: 外部API障害でサービスがpanic/クラッシュせず、Resultとして呼び出し元へ安全に伝播する必要があるため
- **入力値**: TMDbモックサーバーが応答遅延/接続不能で `ApiError::Timeout` を誘発する設定 + `MediaType::Movie` + キー登録済み
  - **不正な理由**: 外部API側の遅延（クライアント制御外の障害）
  - **実際の発生シナリオ**: TMDb API高負荷・ネットワーク遅延時
- **期待される結果**: `Err(ExternalSearchError::ExternalApiError(ApiError::Timeout))` が返り、panicやunwrap失敗が発生しない
  - **エラーメッセージの内容**: 内側の `ApiError` を保持。HTTPステータス変換（502・既存ExternalApiError流用）は後続TASK-0024責務
  - **システムの安全性**: `?` 演算子による伝播でプロセスが安全な状態を保つ（REQ-0023-103）
- **テストの目的**: REQ-0023-103（execute エラー時 ExternalApiError・非panic）を保証する
  - **品質保証の観点**: 外部依存障害がサービス全体を巻き込まないことの確認
- 🔵 信頼性レベル: 要件定義書 REQ-0023-103・シナリオ4・TC-002-E02・テストケース4 L93-97より

### TC-002-E02-B: 全ApiError variantがExternalApiErrorへ集約される（panicしない・ユニット・パラメタライズド）

- **テスト名**: `Http{status}`/`Auth`/`RateLimit{retry_after}`/`Parse`/`Network`/`Timeout` の6 variantいずれも `ExternalApiError` へラップされpanicしない
  - **エラーケースの概要**: api-client-lib の `ApiError` 全6 variantが漏れなく集約されるか（EDGE-0023-04）
  - **エラー処理の重要性**: 一部のApiError variantが取りこぼされてpanicや別エラーになると、502マッピングが破綻するため
- **入力値**: モックサーバーで各 variantを誘発（例: 500応答→Http、401応答→Auth/Http、不正JSON→Parse、接続断→Network、429+Retry-After→RateLimit）+ `MediaType::Movie`
  - **不正な理由**: 各種外部API障害（クライアント制御外）
  - **実際の発生シナリオ**: HTTP 5xx/認証失敗/レート制限/不正レスポンス/ネットワーク断など多様な外部障害
- **期待される結果**: 各ケースで `Err(ExternalSearchError::ExternalApiError(..))` を返し、いずれもpanicしない。内側variantが元のApiErrorと一致
  - **エラーメッセージの内容**: 6 variant全てが同一の `ExternalApiError` ラッパへ集約
  - **システムの安全性**: 想定外variantで未処理panicが発生しないこと
- **テストの目的**: EDGE-0023-04（全ApiError集約）・REQ-0023-103の網羅を保証する
  - **品質保証の観点**: ApiError variant追加時のmatch漏れ回帰防止（network/parse等の取りこぼし検知）
- 🔵 信頼性レベル: 要件定義書 EDGE-0023-04・REQ-0023-103、note.md L24（ApiError 6 variant）より

---

## 3. 境界値テストケース（最小値、最大値、隣接variant等）

### TC-002-B01: 空クエリ文字列が透過的に各プロバイダへ渡される（境界・ユニット）

- **テスト名**: `search(Anime, "")`（空文字クエリ）がバリデーションせず透過的にJikanへ渡される
  - **境界値の意味**: `query` 文字列長0の境界。要件定義書 L86 では空文字バリデーションはサービス層責務外（呼び出し元責務）で透過的に各プロバイダRequestへ渡す方針
  - **境界値での動作保証**: 空文字でもサービス層がエラー化せず、各プロバイダのRequestへそのまま渡されることを固定する
- **入力値**: `MediaType::Anime` + `query=""`
  - **境界値選択の根拠**: 要件定義書 第3章 入力仕様 L86「空文字バリデーションは要件未指定のため呼び出し元責務とし、サービス層では透過的に渡す」
  - **実際の使用場面**: ハンドラ層がバリデーションせず空クエリを渡してきた場合
- **期待される結果**: サービス層では `ValidationError` 等を発生させず、Jikanモックへ空クエリを含むリクエストが到達（またはプロバイダ側の応答をそのまま伝播）。panicしない
  - **境界での正確性**: 空文字を特別扱いせず透過的に処理すること
  - **一貫した動作**: 非空クエリと同じコードパスを通ること
- **テストの目的**: 空クエリ境界でサービス層がバリデーション責務を持たない方針（L86）を固定する
  - **堅牢性の確認**: 空文字で予期せぬpanic/早期returnが起きないこと。**サービス層で空文字を拒否すべきか否かはtdd-red着手前に要件L86の方針（透過）で確定すること**
- 🟡 信頼性レベル: 要件定義書 第3章 L86（空文字バリデーションは呼び出し元責務・透過処理）からの妥当な推測より

### TC-002-B02: 非常に長いクエリ文字列が透過的に処理される（境界・ユニット）

- **テスト名**: 極端に長い `query`（例: 10,000文字）でもサービス層がpanic/切り詰めせず透過処理する
  - **境界値の意味**: `query` 文字列長の上限境界。サービス層に長さ制約はなく、長大入力でも安定動作するか
  - **境界値での動作保証**: 長大クエリでサービス層がpanic・メモリ異常・独自切り詰めをしないことを固定する
- **入力値**: `MediaType::Anime` + `query=`（"あ"×10,000等の長大文字列）
  - **境界値選択の根拠**: 要件定義書にクエリ長上限の記載がなく、サービス層は透過（L86）。長大入力での堅牢性確認
  - **実際の使用場面**: 異常に長い検索文字列・攻撃的入力が渡された場合
- **期待される結果**: サービス層は長さ検証せず透過的にJikanへ渡す。panic・切り詰めなし。プロバイダ/HTTP層が拒否する場合はそのエラーが `ExternalApiError` として伝播
  - **境界での正確性**: 長大文字列で独自の切り詰め/加工をしないこと
  - **一貫した動作**: 通常長クエリと同じコードパスを通ること
- **テストの目的**: 長大クエリ境界でのサービス層の堅牢性（非panic・透過）を保証する
  - **堅牢性の確認**: 極端な入力長でメモリ/スタック異常が起きないこと
- 🟡 信頼性レベル: 要件定義書 第3章 L86（透過処理方針）からの妥当な推測（クエリ長上限の明記なし）より

### TC-002-B03: 全8 MediaType variantがちょうど1プロバイダへ一意写像される（境界・網羅性・ユニット）

- **テスト名**: 8 variant（Anime/Movie/Drama/Manga/Novel/Game/AcademicBook/Paper）すべてが第2章マッピング表どおり単一プロバイダへ写像される
  - **境界値の意味**: MediaType enum全列挙の網羅境界。8 variant漏れなく、かつ1 variant→1 providerの一意性が保たれるか（REQ-0023-01・REQ-0023-501）
  - **境界値での動作保証**: enumに将来variantが追加された際のmatch漏れ（非網羅）を検知し、全variantが必ずいずれかのproviderへ向かうことを固定する
- **入力値**: 8 variant × 期待provider の対応表 `[(Anime,Jikan),(Movie,Tmdb),(Drama,Tmdb),(Manga,Jikan),(Novel,OpenLibrary),(Game,Igdb),(AcademicBook,Ndl),(Paper,Ndl)]`
  - **境界値選択の根拠**: マッピング表（要件定義書 第2章）の全行。隣接variantへの取り違えを防ぐ最終網羅検証
  - **実際の使用場面**: 全メディア種別での検索ディスパッチ
- **期待される結果**: 各variantで期待provider「のみ」にリクエストが到達し、他provider到達==0。8件すべて成立
  - **境界での正確性**: 1 variantも未処理（fallthrough/panic）にならないこと
  - **一貫した動作**: Manga→Jikan（設計判断A）/Game→Igdb（設計判断B）を含め表どおりであること
- **テストの目的**: REQ-0023-01・REQ-0023-501（1 media_type→1 provider一意写像）を全variantで保証する
  - **堅牢性の確認**: 隣接enum variant（例: Manga↔Novel、AcademicBook↔Paper、Anime↔Movie）への誤ディスパッチが1件もないこと（dyn非互換enum/match実装の正当性確認）
- 🔵 信頼性レベル: 要件定義書 第2章 マッピング表・REQ-0023-01・REQ-0023-501・REQ-0023-402（enum/match実装）より

### TC-002-B04: 隣接enum variant誤ディスパッチ検証（Manga/Novel・AcademicBook/Paper・Anime/Movie 非混同・ユニット）

- **テスト名**: マッピングが分岐する隣接variantペアで、互いのプロバイダへ誤到達しない
  - **境界値の意味**: enum/match分岐の「隣り合う/紛らわしいvariant」境界。特に設計判断で分離したペアの取り違え境界
  - **境界値での動作保証**: dyn非互換のためenum/match手動分岐（REQ-0023-402）で実装する都合上、隣接variantのコピペミス・分岐順序ミスで誤プロバイダへ向かわないことを固定する
- **入力値**: 検証ペア — (a) `Manga`→Jikan であって OpenLibrary/NDL でない、(b) `Novel`→OpenLibrary であって Jikan/NDL でない、(c) `AcademicBook`/`Paper`→NDL であって OpenLibrary でない、(d) `Anime`→Jikan であって Movie系TMDb でない
  - **境界値選択の根拠**: 設計判断A（manga/novel分離）・NDL/OpenLibraryの隣接・要件定義書「dyn非互換dispatchが隣接variantで誤プロバイダを呼ばない」確認要求
  - **実際の使用場面**: match分岐の実装ミスが最も混入しやすい紛らわしいペア
- **期待される結果**: 各ペアで「期待provider受信==1、誤候補provider受信==0」。特にManga実行時OpenLibrary受信0、Novel実行時Jikan受信0、Paper実行時OpenLibrary受信0
  - **境界での正確性**: 紛らわしい隣接variantが分岐順序/コピペ起因で混線しないこと
  - **一貫した動作**: 設計判断A/Bの分離がmatch実装に正しく反映されていること
- **テストの目的**: dyn非互換enum/match実装（REQ-0023-402）における隣接variant誤ディスパッチを防止する
  - **堅牢性の確認**: 「adjacent enum variantで誤プロバイダを呼ばない」という本タスク中核の安全性要求を直接固定する
- 🔵 信頼性レベル: 要件定義書 REQ-0023-402・設計判断A/B・マッピング表より（隣接variant誤ディスパッチ防止はタスク指示の明示要求）

### TC-002-B05: Jikan系（Anime/Manga）はキー取得を一切行わない（境界・ユニット）

- **テスト名**: Anime/Manga 実行時に `find_by_provider` が一度も呼ばれない（キー取得スキップ境界）
  - **境界値の意味**: 「キー不要プロバイダ」と「キー必須プロバイダ」の処理分岐境界。Jikan系のみキー取得をスキップする
  - **境界値での動作保証**: キー不要provider（Jikan）でDBアクセスが発生しない＝DB未初期化でも検索可能であることを固定する
- **入力値**: `MediaType::Anime` および `MediaType::Manga`（モック化した `find_by_provider` 呼び出し回数を観測）
  - **境界値選択の根拠**: REQ-0023-102（Jikanはキー取得スキップ）。キー必須/不要の分岐境界の直接検証
  - **実際の使用場面**: APIキー未設定のクリーンな環境でanime/mangaを検索する場合（DB初期化前でも動作）
- **期待される結果**: Anime/Manga いずれも `find_by_provider` 呼び出し==0、かつJikanモック受信==1。DB接続不能でもApiKeyNotConfiguredにならず成功し得る
  - **境界での正確性**: Jikan系でDBキー取得経路に一切入らないこと
  - **一貫した動作**: 一方キー必須プロバイダ（TC-002-E01）では必ず `find_by_provider` を経由すること（対の検証）
- **テストの目的**: REQ-0023-102（Jikanキー取得スキップ）を境界として保証する
  - **堅牢性の確認**: キー不要providerが誤ってキー必須経路に入りApiKeyNotConfiguredを返さないこと
- 🔵 信頼性レベル: 要件定義書 REQ-0023-102・設計判断A/C・タスクファイル L60より

---

## 4. テストケース総覧（TC-ID対応表）

| TC-ID | 概要 | 種別 | 信頼性 | 要件対応 |
|---|---|---|---|---|
| TC-002-01-A | Anime→Jikanのみ・他非到達 | ユニット(HTTPモック) | 🔵 | REQ-0023-02, 設計判断C |
| TC-002-02-A | Movie→DBキーで初期化したTMDb | 統合 #[ignore] | 🔵 | REQ-0023-03, REQ-0023-403 |
| TC-002-02-B | Drama→TMDb・他非到達 | ユニット(HTTPモック) | 🔵 | REQ-0023-03 |
| TC-002-MANGA | Manga→Jikan・キー取得スキップ | ユニット(HTTPモック) | 🟡 | REQ-0023-04, 設計判断A, EDGE-0023-01 |
| TC-002-NOVEL | Novel→OpenLibrary・他非到達 | ユニット(HTTPモック) | 🔵 | REQ-0023-05 |
| TC-002-GAME | Game→IGDB・Steam非到達 | ユニット(HTTPモック) | 🟡 | REQ-0023-05, 設計判断B, EDGE-0023-02 |
| TC-002-ACADEMIC | AcademicBook→NDL・他非到達 | ユニット(HTTPモック) | 🔵 | REQ-0023-05, EDGE-0023-03 |
| TC-002-PAPER | Paper→NDL・他非到達 | ユニット(HTTPモック) | 🔵 | REQ-0023-05, EDGE-0023-03 |
| TC-002-RESULT | Model→ExternalSearchResult変換 | ユニット | 🟡 | REQ-0023-06, 出力仕様(第3章) |
| TC-002-E01-A | TMDbキー未設定→ApiKeyNotConfigured・外部API非呼出 | 統合 #[ignore] | 🔵 | REQ-0023-101 |
| TC-002-E01-B | IGDB/NDL/OpenLibraryキー未設定→各ApiKeyNotConfigured | 統合 #[ignore] | 🔵 | REQ-0023-101 |
| TC-002-E02-A | タイムアウト→ExternalApiError・非panic | ユニット(HTTPモック) | 🔵 | REQ-0023-103 |
| TC-002-E02-B | 全ApiError 6 variant→ExternalApiError集約・非panic | ユニット(HTTPモック) | 🔵 | REQ-0023-103, EDGE-0023-04 |
| TC-002-B01 | 空クエリ文字列の透過処理 | ユニット | 🟡 | 入力仕様L86 |
| TC-002-B02 | 非常に長いクエリ文字列の透過処理 | ユニット | 🟡 | 入力仕様L86 |
| TC-002-B03 | 全8 MediaType variant一意写像網羅 | ユニット(HTTPモック) | 🔵 | REQ-0023-01, REQ-0023-501 |
| TC-002-B04 | 隣接variant誤ディスパッチ防止 | ユニット(HTTPモック) | 🔵 | REQ-0023-402, 設計判断A/B |
| TC-002-B05 | Jikan系(Anime/Manga)キー取得スキップ | ユニット | 🔵 | REQ-0023-102 |

**集計**: 全18ケース（🔵13 / 🟡5 / 🔴0）。ユニット系15件・統合(#[ignore])系3件（TC-002-02-A, E01-A, E01-B）。

**カテゴリ別内訳**:
- 正常系（基本動作・ディスパッチ網羅）: 9ケース（TC-002-01-A, 02-A, 02-B, MANGA, NOVEL, GAME, ACADEMIC, PAPER, RESULT）
- 異常系（エラーハンドリング）: 4ケース（TC-002-E01-A, E01-B, E02-A, E02-B）
- 境界値: 5ケース（TC-002-B01, B02, B03, B04, B05）

> 正確な分類: 正常系9 / 異常系4 / 境界値5 = 計18ケース。
> 8 MediaType variant別ディスパッチは各1ケース（01-A=Anime, 02-A=Movie, 02-B=Drama, MANGA, NOVEL, GAME, ACADEMIC, PAPER の8件）で全網羅し、TC-002-B03で一括網羅性も二重担保する。

---

## 5. 開発言語・テストフレームワーク

- **プログラミング言語**: Rust（edition 2024）
  - **言語選択の理由**: 既存プロジェクト全体がRust + axum 0.8 + sqlx 0.8 + api-client-lib（ワークスペース内）で構築されており、本タスクも同一クレート（`mediavault-api`）内のサービス追加実装のため
  - **テストに適した機能**: `match`の網羅性検査（non-exhaustive match のコンパイルエラー）でMediaType全variant処理を静的に担保しつつ、テストで実ディスパッチ先を検証できる。`Result`/`?`/`#[non_exhaustive]`非依存のenum網羅がパニック防止と相性が良い
- **テストフレームワーク**: Rust標準テストハーネス（`cargo test`） + `tokio::test`（非同期 `search` 用） + HTTPモック（`wiremock` 等、`new_with_base_url` でクライアントへ注入）
  - **フレームワーク選択の理由**: `ApiClient::execute` がRPITIT（dyn非互換）で `mockall::automock` の素トレイト適用が困難な可能性があるため（note.md L29-32・要件定義書 第7章）、「対象プロバイダURLにのみリクエスト到達／他は非到達」をHTTPレベルで検証する `wiremock` 方式を主とする。`mockall` 採用可否はtdd-red前に確定し、可能なら呼び出し検証へ置換可
  - **テスト実行環境**: ユニット（ディスパッチ／エラー変換／変換アダプタ）はDB不要で `cargo test -p mediavault-api` にて即時実行（HTTPモックはローカルポート）。実DBキー取得確認（TC-002-02-A, E01-A, E01-B）は `docker compose up -d db` + `DATABASE_URL` を前提に `cargo test -- --ignored` で別実行する
  - **依存追加**: `wiremock`（または `mockall`）を `backend/mediavault-api/Cargo.toml` の `[dev-dependencies]` に新規追加する必要がある（note.md L31・要件定義書 次フェーズ引き渡しL257）
- 🔵 信頼性レベル: note.md L29-32・L52-55「テスト規約/mockall課題」・要件定義書 第7章・NFR-0023-04に直接対応

---

## 6. 要件定義との対応関係

- **参照した機能概要**: TASK-0023-requirements.md 第1章「機能の概要」（media_type→単一provider ディスパッチ・抽象化）
- **参照した入力・出力仕様**: 第3章（API契約 L65-81、入力 L83-86＝空文字透過、出力 ExternalSearchResult L88-103、データフロー L105-107）
- **参照した制約条件**: 第4章 ExternalSearchError仕様（ApiKeyNotConfigured/ExternalApiError・非panic）、第5章 機能要件（REQ-0023-01〜06, 101〜103, 401〜404, 501）、第7章 mockall課題
- **参照した使用例**: 第2章 マッピング表＋設計判断A/B/C、第8章 シナリオ1〜4（TC-002-01/02/E01/E02）・EDGE-0023-01〜04

## 7. 次フェーズへの引き渡し事項

- `tdd-red` 着手前に以下を確定すること（要件定義書 次フェーズ引き渡しL252-257より）:
  1. **テスト手段の確定**: `wiremock`（HTTPモックサーバー・URL到達検証）か `mockall`（RPITIT対応可否）か。困難なら「executeのみ呼ばれる」呼び出し検証を「対象URLのみ到達」検証へ置換（第7章）。→ 全ユニット系ケースの実装方式を決定
  2. **ディスパッチ実装方式**: `dyn ApiClient` 不可（dyn非互換・REQ-0023-402）のため enum/match で各プロバイダ型を直接構築。`match` の網羅性で8 variantを静的担保（TC-002-B03/B04 と整合）
  3. **`ExternalSearchResult.provider` のJikan表現**: `Option<ApiProvider>` か DTO専用enum か（要件定義書 第3章注記）。→ TC-002-RESULT の `provider` アサーション期待値を確定
  4. **空クエリ方針**: 要件L86の「サービス層透過（呼び出し元責務）」で確定。サービス層で拒否しない（TC-002-B01/B02 の期待値を透過で固定）
- `wiremock`/`mockall` を `backend/mediavault-api/Cargo.toml` の `[dev-dependencies]` に追加する（note.md L31）
- 設計判断A（manga→Jikan）・設計判断B（game→IGDB固定）の根拠をコミットログまたはnote.md追記に残すこと（要件定義書 次フェーズ引き渡し・タスク注意事項L114）
- 統合テスト（TC-002-02-A/E01-A/E01-B）のDBクリーンアップ方針（`api_credentials` は provider PRIMARY KEY固定衝突のため、テスト冒頭でDELETEまたはprovider分離）を実装時に決めること
