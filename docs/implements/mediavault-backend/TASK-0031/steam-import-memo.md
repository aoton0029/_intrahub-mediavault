# TDD開発メモ: steam-import

## 概要

- 機能名: Steamライブラリインポート機能（`POST /import/steam`）
- 開発開始: 2026-06-27
- 現在のフェーズ: 完了（Refactorフェーズまで完了）

## 関連ファイル

- 元タスクファイル: `docs/tasks/mediavault-backend/TASK-0031.md`
- 要件定義: `docs/implements/mediavault-backend/TASK-0031/steam-import-requirements.md`
- テストケース定義: `docs/implements/mediavault-backend/TASK-0031/steam-import-testcases.md`
- Redフェーズ記録: `docs/implements/mediavault-backend/TASK-0031/steam-import-red-phase.md`
- 実装ファイル（未実装、Greenフェーズで実装）:
  - `backend/mediavault-api/src/import/steam_import.rs`
  - `backend/mediavault-api/src/handlers/import_steam.rs`
- 既存拡張ファイル:
  - `backend/mediavault-api/src/models/response.rs`（`ApiErrorCode::SteamApiKeyInvalid`追加済み）
  - `backend/mediavault-api/src/import/mod.rs`
  - `backend/mediavault-api/src/handlers/mod.rs`
  - `backend/mediavault-api/src/routes/mod.rs`
- テストファイル:
  - `backend/mediavault-api/src/import/steam_import.rs`（インラインテスト、17件）
  - `backend/mediavault-api/src/handlers/import_steam.rs`（インラインテスト、3件、`#[ignore]`）
  - `backend/mediavault-api/src/models/response.rs`（インラインテスト、1件）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2026-06-27

### テストケース

21件作成（目標10件を超過）。詳細は`steam-import-red-phase.md`参照。

- steam_id形式検証: 6件（DB非依存）
- SteamGameEntry→CreateItemRequest変換: 3件（DB非依存）
- import_steam_libraryユニット・統合: 8件（うち3件は`#[ignore]`実DB必要）
- ハンドラ統合: 3件（すべて`#[ignore]`実DB必要）
- ApiErrorCode単体: 1件（共通基盤のため実装済み・即時グリーン）

### テストコード

`backend/mediavault-api/src/import/steam_import.rs`・`backend/mediavault-api/src/handlers/import_steam.rs`を参照。

### 期待される失敗

- `validate_steam_id`・`steam_game_entry_to_create_item_request`・`import_steam_library`・`import_steam_handler`はすべて`todo!()`で未実装のため、呼び出し時にpanicする。
- `cargo build -p mediavault-api`は成功（型シグネチャ確定のためコンパイルエラーなし）。
- `cargo test -p mediavault-api steam`は1 passed（ApiErrorCode単体）・14 failed（todo!()によるpanic、または`unreachable_pool()`の接続タイムアウト）・6 ignored（実DB統合テスト）。

### 次のフェーズへの要求事項

Greenフェーズで以下を実装する:

1. `validate_steam_id`: 17桁数値文字列検証 + u64変換
2. `steam_game_entry_to_create_item_request`: appid/name(フォールバック対応)/playtime_forever → CreateItemRequest変換
3. `import_steam_library`: APIキー取得(DI)→Steam API呼び出し→重複チェック（スキップ＝集計外）→登録（1件ごと独立処理）→ImportSummary構築
4. `import_steam_handler`: リクエストデシリアライズ→usecase呼び出し→ImportSummaryレスポンス
5. テスト専用Steam APIベースURL注入経路の確定（ハンドラ統合テストでwiremock到達のため）

設計判断確定事項（note.md未確定事項#1〜#5）:
- 重複: スキップ＝集計外
- name=None: フォールバックタイトル`"Unknown (appid:{appid})"`でsuccess登録
- ImportFailure識別子: row_numberに配列インデックス（0始まり）を流用
- ImportSummaryカウント型: 既存u32のまま
- トランザクション分離: ゲーム1件ごとに独立処理

## Greenフェーズ（最小実装）

### 実装日時

2026-06-27

### 実装方針

確定済み設計判断（重複スキップ＝集計外、name=Noneフォールバック、row_number=配列インデックス、
u32カウント、ゲーム1件ごと独立処理）に従い、`todo!()`を実ロジックへ置き換えた。

- `validate_steam_id`: 17桁数値検証 + u64変換
- `SteamCredentialLookup::find`: Pool/Fixed経路でAPIキー解決（新規追加メソッド）
- `steam_game_entry_to_create_item_request`: appid/name(フォールバック)/playtime → CreateItemRequest変換
- `import_steam_library`: 検証→APIキー取得→Steam API呼び出し→重複チェック→登録→ImportSummary構築
- `map_steam_api_error`: api_client_lib::ApiError → ApiError（401/502）マッピング（新規追加ヘルパー）
- `import_steam_handler`: usecase呼び出し→ImportSummaryを200で返す

詳細は`steam-import-green-phase.md`参照。

### テスト結果

`cargo build -p mediavault-api`: 成功（エラー・警告なし）
`cargo test -p mediavault-api steam`: 15 passed / 0 failed / 6 ignored（実DB必要、Docker未起動のため未実行）

Redフェーズの`unreachable_pool()`ヘルパーが`PgPool::connect`（即時接続）のため到達不能ホストへの
接続で環境依存の30秒タイムアウトが発生していた問題を、`connect_lazy`へ変更して解消した
（テストのassert自体は変更なし）。

### 課題・改善点（Refactorフェーズで対応）

- `import_steam_library`関数がやや長い（検証・APIキー取得・クライアント構築・ループ処理を1関数に集約）。
  責務分割を検討する。
- `map_steam_api_error`のエラー分類網羅性レビュー。

## Refactorフェーズ（品質改善）

### 実施日時

2026-06-27

### 改善内容

Greenフェーズの課題（`import_steam_library`関数の責務分割の余地）に対応し、
`backend/mediavault-api/src/import/steam_import.rs`の`import_steam_library`を
以下3関数へ分割した（純粋なExtract Method、機能的変更なし）:

- `build_steam_client`: APIキー＋任意のベースURLからSteamClientを構築
- `fetch_owned_games`: APIキー解決→クライアント構築→`get_owned_games`呼び出し
- `register_single_game`: 1件分の重複チェック・変換・DB登録・ImportSummary反映

`import_steam_library`本体は「steam_id検証→ゲーム一覧取得→1件ずつ登録」の
オーケストレーションのみに整理された。TASK-0030（booklog_csv.rs）・TASK-0023
（external_search.rs）の日本語コメント規約（【ヘルパー関数】【単一責任】
【再利用性】【改善内容】等）に準拠したコメントを各関数に付与した。

### セキュリティレビュー結果

重大な脆弱性なし。APIキーはログ出力されず、入力検証順序（steam_id検証が
外部API呼び出し前）も維持されている。

### パフォーマンスレビュー結果

重大な性能課題なし。計算量O(n)・メモリ使用量はGreenフェーズと同一。

### テスト結果

`cargo build -p mediavault-api`: 成功（既存warningのみ、新規warningなし）
`cargo test -p mediavault-api steam`: 15 passed / 0 failed / 6 ignored（リファクタ前後で完全一致）
`cargo test -p mediavault-api`（全体）: 198 passed / 0 failed / 192 ignored

### 品質評価

高品質。テスト結果完全一致・セキュリティ/パフォーマンス課題なし・
責務分割目標達成・ファイルサイズ制限内（本体実装約290行）。

詳細は`steam-import-refactor-phase.md`参照。
