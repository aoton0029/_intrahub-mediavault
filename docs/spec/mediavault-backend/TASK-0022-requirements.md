# TASK-0022 要件定義書: api_credentials（外部APIキー管理）CRUD実装

**作成日**: 2026-06-25
**関連タスク**: [TASK-0022](../../tasks/mediavault-backend/TASK-0022.md)
**関連ノート**: [note.md](note.md) TASK-0022セクション
**親要件**: [requirements.md](requirements.md) REQ-015 / NFR-202 ・ [acceptance-criteria.md](acceptance-criteria.md) TC-015-01・TC-015-02・TC-015-03
**API仕様**: [api-endpoints.md](../../design/mediavault-backend/api-endpoints.md) `PUT /settings/api-keys/:provider`

**【信頼性レベル凡例】**:
- 🔵 **青信号**: タスク仕様・設計文書・既存コード（note.md記載）から確実な要件
- 🟡 **黄信号**: タスク仕様・設計文書から妥当な推測による要件
- 🔴 **赤信号**: 推測による要件（本ドキュメントには無し）

---

## 1. 機能の概要

`PUT /settings/api-keys/:provider` は、外部メタデータ取得API（TMDb / IGDB / NDL / Steam / OpenLibrary / AniList）のAPIキーを `api_credentials` テーブルに **upsert（登録または更新）** するエンドポイントである。🔵 *タスク概要・api-endpoints.md L375-388より*

- **何をする機能か**: パスパラメータ `:provider` で示された外部API事業者のAPIキーを、リクエストボディ `{ "api_key": "xxxxx" }` の値でDBに永続化する。レコードが無ければ作成、あれば `api_key` と `updated_at` を更新する。🔵 *完了条件 L24-26より*
- **解決する問題**: 外部APIキーをコード変更・再デプロイなしにDB経由で更新可能にする（NFR-202）。🔵 *requirements.md NFR-202より*
- **想定ユーザー**: 本システムの管理者（設定画面/設定APIの利用者）。🟡 *api-endpoints.md「外部APIキー管理」分類より妥当な推測*
- **システム内での位置づけ**: Phase3「外部API連携」の前提タスク。後続 TASK-0023 の `ExternalSearchService` が `find_by_provider` でキーをDBから取得して外部APIを呼び出す土台となる。🔵 *タスク概要・依存タスク L19-21・note.md L33より*
- **対象外**: Jikanはキー不要のため `ApiProvider` enumに含まれず、`/settings/api-keys/jikan` は `INVALID_PROVIDER` 扱いとなる。暗号化は本フェーズ対象外（平文保存）。🔵 *REQ-015・注意事項 L96/L98より*

**参照したEARS要件**: REQ-015, NFR-202
**参照した設計文書**: api-endpoints.md（外部APIキー管理セクション）, architecture.md（レイヤードアーキテクチャ方針）

## 2. 入力・出力の仕様

### 入力 🔵 *types.rs L86-94/L419-421・api-endpoints.md L384-386より*

- **パスパラメータ** `:provider`（`String`）: `ApiProvider` enum にデシリアライズ可能な snake_case 文字列のみ許可。
  - 許可値: `tmdb` / `igdb` / `ndl` / `steam` / `open_library` / `ani_list`（enum定義は `#[sqlx(type_name = "api_provider", rename_all = "snake_case")]` / `#[serde(rename_all = "snake_case")]`）。
  - 上記以外（`jikan` を含む）はバリデーションエラー。
- **リクエストボディ** DTO `UpdateApiKeyRequest`（タスク内命名。設計上の `UpsertApiCredentialRequest`（types.rs L419-421）と同義）:
  ```rust
  #[derive(Debug, Deserialize)]
  pub struct UpdateApiKeyRequest {
      pub api_key: String,
  }
  ```
  - `api_key`: 必須の `String`（DBカラムは `VARCHAR(500) NOT NULL`）。🔵 *database-schema.sql L351より*

### 出力 🔵 *完了条件 L27・note.md L15/L32より*

- **正常系**: HTTP `200`。ボディは更新後の `ApiCredential { provider: ApiProvider, api_key: String, updated_at: NaiveDateTime }` を既存の `ApiOk<T>` 形式で返す。
  - `api_key` を平文でレスポンスに含めるかは実装判断に委ねられる（ログ出力時はマスキング検討）。🟡 *注意事項 L97・note.md L32より妥当な推測*
- **異常系**: 既存の `ApiError`/`ApiErrorCode` 形式に準拠（`{ code, message }`）。

### データフロー 🔵 *architecture.md レイヤード方針・note.md L11/L22より*

`routes/settings.rs` → `handlers::settings::update_api_key`（provider文字列→enum変換・DTO受領）→ `repositories::api_credential_repository::upsert_api_credential`（`sqlx::query!` でupsert）→ DB `api_credentials`。

**参照したEARS要件**: REQ-015
**参照した設計文書**: types.rs L86-94（ApiProvider）/ L236-240（ApiCredential）/ L419-421（UpsertApiCredentialRequest）, database-schema.sql L348-353

## 3. 機能要件（EARS記法）

### 通常要件

- REQ-0022-01: システムは `PUT /settings/api-keys/:provider` リクエストを受理し、`:provider` を `ApiProvider` enum に変換できる場合、`{ "api_key": ... }` の値で `api_credentials` に upsert しなければならない。🔵 *完了条件 L24より*
- REQ-0022-02: 対象 `provider` のレコードが存在しない場合、システムは新規レコードを作成しなければならない（TC-015-01）。🔵 *完了条件 L24・TC-015-01より*
- REQ-0022-03: 対象 `provider` のレコードが既存の場合、システムは `api_key` と `updated_at` を更新しなければならない（TC-015-03）。🔵 *完了条件 L25・TC-015-03より*
- REQ-0022-04: システムは upsert を `INSERT ... ON CONFLICT (provider) DO UPDATE SET api_key = $2 ...` の単一SQLで実行しなければならない（`provider` がPRIMARY KEY）。🔵 *実装詳細3 L49・database-schema.sql L350より*
- REQ-0022-05: システムは更新後の `ApiCredential` をHTTP 200で返さなければならない。🔵 *完了条件 L27より*
- REQ-0022-06: システムは後続タスク向けに `find_by_provider(provider: ApiProvider) -> Result<Option<ApiCredential>, ...>` を提供しなければならない。🔵 *実装詳細5 L57-60より*

### 条件付き要件

- REQ-0022-101: `:provider` が `ApiProvider` enum に存在しない文字列の場合、システムは `INVALID_PROVIDER`（400）を返し、DBへの書き込みを一切行ってはならない（TC-015-02）。🔵 *完了条件 L26・TC-015-02・api-endpoints.md L388より*
- REQ-0022-102: `INVALID_PROVIDER` エラーコードは `ApiErrorCode` enum に新規バリアントとして追加しなければならない（現状未定義）。🔵 *note.md L18-19（response.rs L50-65に未存在）より*

### 状態要件

- REQ-0022-201: `updated_at` はDBトリガー `trg_api_credentials_updated_at`（共通 `update_updated_at_column()`）がBEFORE UPDATEで自動設定するため、アプリ側で明示更新する場合も最終的にトリガー値が優先される。🔵 *note.md L14・database-schema.sql L375-376より*

### 制約要件

- REQ-0022-401: システムはAPIキーを平文で保存し、本タスクで暗号化を実装してはならない（暗号化はフェーズ対象外）。🔵 *REQ-015/NFR-202・注意事項 L96より*
- REQ-0022-402: システムはDBエラー発生時、SQL・DB内部情報をクライアントへ漏らさず、既存の共通DBエラー変換（`repositories/db_error_utils.rs` の `db_error` 相当）を経由して `InternalError`（500）へ正規化しなければならない。🔵 *note.md L22より*
- REQ-0022-403: 新規DB層は既存リポジトリ命名規約 `repositories/*_repository.rs`（例: `repositories/api_credential_repository.rs`）に倣わなければならない（タスクファイル記載の `db/api_credentials.rs` ではなく既存規約を優先）。🔵 *note.md L9より*

## 4. 非機能要件

- NFR-0022-01: 本エンドポイントは既存のレスポンス規約（`ApiError`/`ApiErrorCode`/`ApiOk<T>`）に準拠し、独自のレスポンス形式を新設してはならない。🔵 *note.md エラーハンドリング規約より*
- NFR-0022-02: APIキーをコード変更・再デプロイなしにDB経由で更新可能にしなければならない（NFR-202）。🔵 *requirements.md NFR-202より*
- NFR-0022-03: SQLは `sqlx::query!`（コンパイル時検証）+ バインドパラメータのみを用い、文字列結合によるSQL構築を行ってはならない。🔵 *実装詳細3・既存リポジトリ方針より*
- NFR-0022-04: `api_key` をログ出力する場合はマスキングを検討する（要件に明記がないため実装判断）。🟡 *注意事項 L97より妥当な推測*

## 5. 想定される使用例（Edgeケース含む）

### 基本パターン
- 設定画面/CLIから各providerのAPIキーを初回登録 → 後で同providerにキー再登録（ローテーション）。🔵 *TC-015-01/03より*

### Edgeケース・エラー処理
- EDGE-0022-01: 未知のprovider文字列（例 `unknown_provider`, `jikan`, 大文字 `TMDB` 等 snake_case不一致）→ `INVALID_PROVIDER`（400）、DB無変更。🔵 *TC-015-02・REQ-015（Jikan対象外）より*
- EDGE-0022-02: リクエストボディに `api_key` が欠落 → serdeデシリアライズ失敗により既存のボディパースエラー処理（400系）に従う。🟡 *DTO必須フィールドからの妥当な推測*
- EDGE-0022-03: 同一providerへの連続upsertで、2回目以降は既存行のUPDATEとなり重複行は生成されない（`provider` がPRIMARY KEY）。🔵 *database-schema.sql L350・TC-015-03より*
- EDGE-0022-04: DB接続不能・SQL実行失敗 → `db_error` 経由で `INTERNAL_ERROR`（500）、内部情報非漏洩。🔵 *note.md L22より*

**参照したEARS要件**: REQ-015, NFR-202, TC-015-01/02/03

## 6. Given/When/Then シナリオ（単体テスト要件と対応）

### シナリオ1: TMDbキー新規登録（TC-015-01） 🔵
- **Given**: `api_credentials` に `tmdb` のレコードが存在しない
- **When**: `PUT /settings/api-keys/tmdb` に `{ "api_key": "valid-tmdb-key" }` を送信
- **Then**: 200が返り、`provider=tmdb, api_key=valid-tmdb-key` のレコードが作成される

### シナリオ2: 不正provider指定（TC-015-02） 🟡 *acceptance-criteria信頼性🟡指定より*
- **Given**: enumに存在しない文字列 `unknown_provider`
- **When**: `PUT /settings/api-keys/unknown_provider` に `{ "api_key": "x" }` を送信
- **Then**: `400 INVALID_PROVIDER` が返り、DBへの書き込みは発生しない

### シナリオ3: キー更新が以後反映される（TC-015-03） 🔵
- **Given**: `provider=tmdb` のレコードが既存（`api_key=old-key`）
- **When**: `PUT /settings/api-keys/tmdb` に `{ "api_key": "new-key" }` を送信
- **Then**: 200が返り、`find_by_provider(Tmdb)` が `api_key=new-key` を返し、`updated_at` も更新される

## 7. 完了基準

- [ ] `PUT /settings/api-keys/:provider` が実装され、`api_key` 本文で `api_credentials` に upsert される。
- [ ] 既存レコードがある場合 `api_key` と `updated_at` が更新される（TC-015-03）。
- [ ] 不正な `provider` 文字列で `400 INVALID_PROVIDER` を返す（TC-015-02）。`ApiErrorCode::InvalidProvider` が新規追加されている。
- [ ] 正常系で更新後 `ApiCredential` を `200` で返す。
- [ ] `find_by_provider` が実装され後続タスクから参照可能。
- [ ] 単体テスト TC-015-01/02/03 がすべて成功する。
- [ ] DB層が `repositories/api_credential_repository.rs` 規約に従い、`handlers/settings.rs`・`routes/settings.rs` が新規作成され `build_router` に登録されている。

## 8. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-015
- **参照した非機能要件**: NFR-202
- **参照したEdgeケース**: 本ドキュメントで新設（EDGE-0022-01〜04）
- **参照した受け入れ基準**: TC-015-01 / TC-015-02 / TC-015-03
- **参照した設計文書**:
  - **型定義**: types.rs L86-94（ApiProvider）/ L236-240（ApiCredential）/ L419-421（UpsertApiCredentialRequest）
  - **データベース**: database-schema.sql L348-353（api_credentials）/ L375-376（updated_atトリガー）
  - **API仕様**: api-endpoints.md L375-388（PUT /settings/api-keys/:provider）
  - **既存コード現況**: note.md TASK-0022セクション（repositories規約・response.rs L50-65のApiErrorCode未定義・db_error_utils.rs）

## 9. 信頼性レベルサマリー

| カテゴリ | 🔵 | 🟡 | 🔴 | 合計 |
|---|---|---|---|---|
| 機能要件（通常） | 6 | 0 | 0 | 6 |
| 機能要件（条件付き） | 2 | 0 | 0 | 2 |
| 機能要件（状態） | 1 | 0 | 0 | 1 |
| 機能要件（制約） | 3 | 0 | 0 | 3 |
| 非機能要件 | 3 | 1 | 0 | 4 |
| Edgeケース | 3 | 1 | 0 | 4 |
| シナリオ | 2 | 1 | 0 | 3 |

**全体評価**: 高品質（赤信号なし。黄信号は想定ユーザー・api_keyレスポンス/ログ扱い・ボディ欠落時の挙動など実装時に既存コード/設計判断で確定すべき細部）

---

## 次フェーズへの引き渡し事項

- `tdd-testcases` フェーズでは、本ドキュメントのシナリオ1〜3を中核とし、EDGE-0022-01（snake_case不一致/Jikan）・EDGE-0022-02（api_key欠落）・EDGE-0022-04（DBエラー→500）を追加テストケースとして洗い出すこと。
- `tdd-red` 着手前に、(1) `ApiErrorCode::InvalidProvider`（400）の追加箇所（response.rs L50-65付近）、(2) provider文字列→enum変換の実装方式（`serde_json` 経由 or `match` 文）、(3) `api_key` をレスポンス/ログに含めるか、を確定すること。
- DB層ファイルはタスクファイル記載の `db/api_credentials.rs` ではなく既存規約 `repositories/api_credential_repository.rs` に倣うこと（note.md L9）。
