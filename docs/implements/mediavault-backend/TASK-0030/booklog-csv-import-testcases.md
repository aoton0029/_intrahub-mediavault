# TASK-0030 テストケース定義書: ブクログCSVインポート実装（POST /import/booklog）

**作成日**: 2026-06-27
**機能名**: booklog-csv-import（ブクログCSVインポート）
**タスクID**: TASK-0030
**要件名**: mediavault-backend
**前フェーズ成果物**: [booklog-csv-import-requirements.md](booklog-csv-import-requirements.md) / [note.md](note.md)
**親要件**: REQ-016・EDGE-002 ・ ストーリー5.1
**受け入れ基準**: TC-016-01 / TC-016-E01

---

## 0. 確定済み設計判断（要件定義書 6章「要実装判断事項」の解決）

本テストケース定義の前提として、要件定義書に残っていた未決事項のうち以下2点を**ユーザー判断により確定**した。テストケースはこの確定内容を前提に作成している。

| # | 項目 | 確定内容 | 影響 |
|---|---|---|---|
| 1 | **`consumed_date` の登録経路** | **作成パスを拡張する（選択肢a）**。`CreateItemRequest`（`backend/mediavault-api/src/models/item.rs`）に `consumed_date: Option<chrono::NaiveDate>` フィールドを追加し、`create_item_with_source`（`backend/mediavault-api/src/repositories/item_repository.rs`）の INSERT 文に `consumed_date` を bind する。これによりブクログCSVの「読了日」列が `items.consumed_date` に実際に永続化される。 | 🔵 確定。TC-N-04 / TC-N-05 / TC-DB-01 / TC-REG-01 が新設・対象 |
| 2 | **`media_type` 既定値** | **全インポート行で `MediaType::Novel` 固定**。ブクログCSVには media_type 列が無いため、`BooklogCsvRow → CreateItemRequest` 変換時に固定値 `MediaType::Novel` を付与する。 | 🔵 確定。TC-N-06 / TC-DB-02 が対象 |
| 3 | `source` 値（参考・既決） | `ItemSource::Manual` 固定（専用enum値は未存在）。要件定義書で既に確定済み。 | 🔵 |
| 4 | multipartフィールド名（参考） | `file` または `csv` のいずれかを受理（両対応）。 | 🟡 |

> **拡張に伴う既存影響（回帰観点）**: `CreateItemRequest` への必須でない `Option` フィールド追加であり、`#[serde(default)]` 相当で既存の手動作成（TASK-0009）デシリアライズは壊れない。`create_item`（薄いラッパー）経由の呼び出しは `consumed_date=None` となる。この回帰は TC-REG-01 で確認する。

---

## 1. 開発言語・フレームワーク

- **プログラミング言語**: Rust (Edition 2024)
  - **言語選択の理由**: 既存 `mediavault-api` クレートが Rust で実装されており、型安全な CSV デシリアライズ（serde + csv crate）と `Option`/`Result` によるエラー伝播がインポート処理の「行単位スキップ」に適している。
  - **テストに適した機能**: `#[test]`（同期ユニット）/ `#[tokio::test]`（非同期DB統合）、インライン `#[cfg(test)] mod tests`、`Result` ベースのアサーション。
- **テストフレームワーク**: 標準 `#[test]` / `#[tokio::test]` + 補助クレート
  - **フレームワーク選択の理由**: プロジェクト既存方針（`models/item.rs`・`item_repository.rs`・`handlers/items.rs` のインラインテスト）を継続。新規フレームワーク導入は不要。
  - **テスト実行環境**:
    - DB非依存ユニット: `cargo test -p mediavault-api`
    - DB必須統合: `#[tokio::test]` + `#[ignore]` → `cargo test -p mediavault-api -- --ignored`（Docker Compose の Postgres、`DATABASE_URL` 接続）
  - **補助クレート**: `csv = "1.3"`（要 Cargo.toml 追加）、`tempfile = "3"`（一時CSV生成）、`encoding_rs`（文字コードフォールバック構造のみ）
- 🔵 信頼性レベル: note.md「5. テスト関連情報」「1. 技術スタック」に直接対応

### テスト対象モジュール（新規）
- `backend/mediavault-api/src/import/booklog_csv.rs` — `BooklogCsvRow` 構造体・行単位パーサ・バリデーション（DB非依存ユニットの主対象）
- `backend/mediavault-api/src/models/import.rs` — `ImportSummary` / `ImportFailure` 型
- `backend/mediavault-api/src/handlers/import_booklog.rs` — multipart受け取り・全体オーケストレーション（統合の主対象）
- `backend/mediavault-api/src/models/item.rs` — `CreateItemRequest` に `consumed_date` 追加（拡張対象）
- `backend/mediavault-api/src/repositories/item_repository.rs` — `create_item_with_source` の INSERT 拡張（拡張対象）

---

## 2. テストケース一覧（サマリ）

| ID | 分類 | テスト名 | 種別 | 対応要件/基準 | 信頼性 |
|---|---|---|---|---|---|
| TC-N-01 | 正常系 | 正常CSV全行登録（ImportSummary成功サマリ） | 統合(DB) | TC-016-01, REQ-016 | 🔵 |
| TC-N-02 | 正常系 | 正常行デシリアライズ（BooklogCsvRow単体） | ユニット | 仮カラム定義 | 🟡 |
| TC-N-03 | 正常系 | 任意列が空でも正常登録（description/読了日/評価/ISBN空） | ユニット | 仮カラム定義 | 🟡 |
| TC-N-04 | 正常系 | CreateItemRequest が consumed_date を保持・デシリアライズ | ユニット | 設計判断#1 | 🔵 |
| TC-N-05 | 正常系 | 読了日→consumed_date がDBに永続化される | 統合(DB) | 設計判断#1 | 🔵 |
| TC-N-06 | 正常系 | media_type が Novel 固定で登録される | 統合(DB) | 設計判断#2 | 🔵 |
| TC-N-07 | 正常系 | source=Manual / ISBN→external_id が登録される | 統合(DB) | 既決#3, 要件定義3章 | 🔵 |
| TC-N-08 | 正常系 | 部分不正混在CSVで正常行のみ登録（EDGE-002 継続性） | 統合(DB) | TC-016-02, EDGE-002 | 🔵 |
| TC-E-01 | 異常系 | 作品名(title)空行のスキップ＋failure記録 | ユニット | EDGE-002, TC-016-02 | 🔵 |
| TC-E-02 | 異常系 | 読了日が不正形式の行スキップ（invalid date format） | ユニット | EDGE-002 | 🔵 |
| TC-E-03 | 異常系 | 評価が数値でない行スキップ（invalid rating） | ユニット | EDGE-002 | 🔵 |
| TC-E-04 | 異常系 | カラム数不足行のスキップ（deserialize失敗） | ユニット | EDGE-002 | 🔵 |
| TC-E-05 | 異常系 | ImportFailure に row_number と reason が記録される | ユニット | EDGE-002 | 🔵 |
| TC-E-06 | 異常系 | ファイル未添付 → 400 VALIDATION_ERROR | 統合 | TC-016-04 | 🔵 |
| TC-E-07 | 異常系 | 0バイトファイル → 400 VALIDATION_ERROR | 統合 | TC-016-04 | 🔵 |
| TC-E-08 | 異常系 | DB登録失敗時の扱い（行スキップ or 500、方針確認） | 統合(DB) | note.md L172 | 🟡 |
| TC-B-01 | 境界値 | ヘッダーのみ（データ0行）→ 200 success=0/failure=0 | 統合 | REQ-016 | 🟡 |
| TC-B-02 | 境界値 | 全行不正 → 200・success_count=0・failure_count=行数 | 統合 | TC-016-E01 | 🔵 |
| TC-B-03 | 境界値 | 1行のみ正常CSV（最小成功） | 統合(DB) | REQ-016 | 🟡 |
| TC-B-04 | 境界値 | row_number 採番（ヘッダー除外・1始まり） | ユニット | 要件定義2章 | 🔵 |
| TC-B-05 | 境界値 | 数百行CSVの完走（タイムアウトしない軽い確認） | 統合(DB) | 統合テスト要件 | 🟡 |
| TC-B-06 | 境界値 | title前後空白のみ（trim後空）→ title is empty 扱い | ユニット | 要件定義3章入力検証 | 🟡 |
| TC-REG-01 | 回帰 | create_item（manualラッパー）が consumed_date=None で従来通り動作 | 統合(DB) | 設計判断#1の回帰 | 🟡 |
| TC-DB-01 | DB拡張 | create_item_with_source が consumed_date を bind・RETURNING | 統合(DB) | 設計判断#1 | 🔵 |

**合計: 24件**（正常系 8 / 異常系 8 / 境界値 6 / 回帰 1 / DB拡張 1）

---

## 3. 正常系テストケース

### TC-N-01: 正常CSV全行登録（ImportSummary成功サマリ）
- **テスト名**: 仮カラム準拠の正常行のみのCSVをアップロードし全行が登録される
  - **何をテストするか**: `POST /import/booklog` に正常行のみのCSVを multipart で送信したとき、全行が `items` に登録され、`ImportSummary` が成功サマリを返すこと。
  - **期待される動作**: HTTP 200。`success_count` = データ行数、`failure_count` = 0、`failures` = 空配列。成功エンベロープ `{"success": true, "data": {...}}` でラップされる。
- **入力値**: ヘッダー `作品名,感想/レビュー,読了日,評価,ISBN` ＋ データ3行（例: `吾輩は猫である,面白い,2024-01-15,4.5,9784101010014`）を持つUTF-8 CSV。multipartフィールド名 `file`。
  - **入力データの意味**: 5列すべてが妥当な「典型的なブクログエクスポート相当」の正常データ。代表的な成功パスを確認する。
- **期待される結果**: `data.success_count == 3`, `data.failure_count == 0`, `data.failures.len() == 0`。`items` テーブルに3レコード存在。
  - **期待結果の理由**: REQ-016・TC-016-01「正常CSV全行登録」に直接対応。全行が有効なら全行登録が正しい。
- **テストの目的**: インポートの基本成功パス（multipart→パース→変換→登録→サマリ）を端から端まで確認。
  - **確認ポイント**: success_count とDB実レコード数の一致、レスポンスエンベロープ形状。
- 🔵 信頼性レベル: TC-016-01 / 要件定義4章「正常CSV全行登録」に直接対応

### TC-N-02: 正常行デシリアライズ（BooklogCsvRow単体）
- **テスト名**: 1データ行を BooklogCsvRow へ正しくデシリアライズできる
  - **何をテストするか**: `import/booklog_csv.rs` の `BooklogCsvRow`（`#[serde(rename)]` 付き）が仮カラムヘッダーから各フィールドへ正しくマッピングされること。
  - **期待される動作**: `title="吾輩は猫である"`, `description=Some("面白い")`, `consumed_date_raw="2024-01-15"`, `rating_raw="4.5"`, `external_id=Some("9784...")` が得られる。
- **入力値**: ヘッダー `作品名,感想/レビュー,読了日,評価,ISBN` ＋ 1データ行（全列値あり）。
  - **入力データの意味**: カラムマッピング分離設計（最重要制約）の検証。rename値が正しく効いているかを単体で固定する。
- **期待される結果**: 各フィールドが期待値どおり。csv crate のヘッダーベースデシリアライズが成功（Ok）。
  - **期待結果の理由**: 要件定義3章「カラムマッピングの分離（最重要設計制約）」。マッピングは `BooklogCsvRow` に閉じる。
- **テストの目的**: 実サンプル確定後に rename値のみ差分修正できる構造であることをユニットで担保。
  - **確認ポイント**: ヘッダー名（日本語・スラッシュ含む `感想/レビュー`）が正しく解釈されること。
- 🟡 信頼性レベル: 仮カラム定義（実サンプル未確認）に基づくため黄信号

### TC-N-03: 任意列が空でも正常登録される
- **テスト名**: title以外の任意列（感想/読了日/評価/ISBN）が空でも有効行として扱う
  - **何をテストするか**: 必須の `作品名` のみ値があり、その他4列が空文字の行が、不正行ではなく有効行（description=None等）として変換されること。
  - **期待される動作**: 行は成功扱い。`description=None`, `consumed_date=None`, `rating=None`, `external_id=None` の `CreateItemRequest` 相当へ変換される。
- **入力値**: `作品名=星の王子さま`、他4列すべて空文字の1行。
  - **入力データの意味**: 任意列の空は不正ではないという仕様（要件定義2章「任意」）の境界を明確化。
- **期待される結果**: 変換成功、`failures` に含まれない。空文字 → `None`（空のままの誤登録をしない）。
  - **期待結果の理由**: 仮カラム定義で description/読了日/評価/ISBN はすべて「任意」。
- **テストの目的**: 必須/任意の判定が正しく、任意列空を不正扱いしないことの確認。
  - **確認ポイント**: 空文字を `Some("")` ではなく `None` へ正規化していること。
- 🟡 信頼性レベル: 仮カラム定義の「任意」属性からの妥当な推測

### TC-N-04: CreateItemRequest が consumed_date を保持・デシリアライズ（設計判断#1）
- **テスト名**: 拡張後の CreateItemRequest が consumed_date フィールドを持ちデシリアライズできる
  - **何をテストするか**: `CreateItemRequest`（`models/item.rs`）に追加した `consumed_date: Option<chrono::NaiveDate>` が、JSONに `consumed_date` を含む場合はパースされ、含まない場合は `None`（`#[serde(default)]`）になること。
  - **期待される動作**: `{"media_type":"novel","title":"x","consumed_date":"2024-01-15"}` → `consumed_date == Some(NaiveDate 2024-01-15)`。`consumed_date` 省略JSON → `None`。
- **入力値**: (a) consumed_date を含むJSON、(b) consumed_date を省略したJSON（手動作成の既存形）。
  - **入力データの意味**: 設計判断#1の構造拡張がコンパイル・デシリアライズ両面で成立することを最小単位で固定。(b)は既存の手動作成リクエスト後方互換の確認。
- **期待される結果**: (a) `Some(2024-01-15)`、(b) `None`。両者ともデシリアライズ成功。
  - **期待結果の理由**: ブクログ「読了日」をDBへ反映するために `CreateItemRequest` 拡張が前提（要件定義3章「consumed_date のギャップ」を解決）。
- **テストの目的**: 既存DTOへのフィールド追加が後方互換（省略可）であることの保証。
  - **確認ポイント**: `#[serde(default)]` 付与により未指定が `None` になること（既存手動作成テストを壊さない）。
- 🔵 信頼性レベル: 設計判断#1（ユーザー確定）＋ `models/item.rs` L70-84 の実構造に基づく

### TC-N-05: 読了日→consumed_date がDBに永続化される（設計判断#1）
- **テスト名**: CSVの読了日が items.consumed_date に実際に保存される
  - **何をテストするか**: 読了日 `2024-01-15` を持つ行をインポートすると、登録された `items` レコードの `consumed_date` が `2024-01-15` になること。
  - **期待される動作**: インポート成功後、DBから取得した item の `consumed_date == Some(NaiveDate(2024,1,15))`。
- **入力値**: `作品名=ノルウェイの森,読了日=2024-01-15` を含む1行CSV。
  - **入力データの意味**: 「マッピング保持のみ（登録時未使用）」ではなく「実際にDBへ反映」を選んだ確定判断の検証。要件定義の最大の未決事項を直接カバー。
- **期待される結果**: 登録 item の `consumed_date == Some(2024-01-15)`。NULLや既定値でないこと。
  - **期待結果の理由**: `create_item_with_source` の INSERT に `consumed_date` を bind する拡張（設計判断#1）が効いていることの実証。
- **テストの目的**: パース→`CreateItemRequest.consumed_date`→INSERT bind→RETURNING の一気通貫を確認。
  - **確認ポイント**: 拡張前は NULL になっていた値が、拡張後はCSV値で保存される差分。
- 🔵 信頼性レベル: 設計判断#1（ユーザー確定）に基づく

### TC-N-06: media_type が Novel 固定で登録される（設計判断#2）
- **テスト名**: ブクログインポート行の media_type は常に novel になる
  - **何をテストするか**: CSVに media_type 列が無くても、登録された全 item の `media_type == MediaType::Novel` であること。
  - **期待される動作**: インポート成功後、登録 item の `media_type == Novel`。詳細テーブルは `novel_details` にレコード作成。
- **入力値**: 正常行2行（media_type列なし）のCSV。
  - **入力データの意味**: 「ブクログ＝書籍前提」で `Novel` 固定とする確定判断（設計判断#2）の検証。
- **期待される結果**: 全 item の `media_type == Novel`。`detail_table_name(Novel) == "novel_details"` に従い `novel_details` にも item_id が入る。
  - **期待結果の理由**: 仮カラムに media_type が無いため固定既定値が必要（要件定義3章・6章 #2）。
- **テストの目的**: 固定 media_type 付与ロジックと、それに連動する詳細テーブル振り分けの確認。
  - **確認ポイント**: 他のmedia_type（manga等）に誤って振り分けられないこと。
- 🔵 信頼性レベル: 設計判断#2（ユーザー確定）に基づく

### TC-N-07: source=Manual / ISBN→external_id が登録される
- **テスト名**: インポート行は source=manual・external_id=ISBN で登録される
  - **何をテストするか**: 登録 item の `source == ItemSource::Manual`、かつ `ISBN` 列値が `external_id` として保存されること。
  - **期待される動作**: 登録 item の `source == Manual`、`external_id == Some("9784101010014")`。
- **入力値**: `作品名=斜陽,ISBN=9784101010014` の1行CSV。
  - **入力データの意味**: 既決#3（source=Manual固定）と、`create_item_with_source` の `external_id` 引数経由でISBNを渡せる点（要件定義3章）の検証。
- **期待される結果**: `source == Manual`、`external_id == Some("9784101010014")`。
  - **期待結果の理由**: `ItemSource` enum は Api/Manual の2値のみで Booklog 専用値が無いため Manual 固定（要件定義3章既存コード整合制約）。
- **テストの目的**: source 固定値と external_id 反映経路（既存引数）の確認。
  - **確認ポイント**: ISBN空の場合は `external_id=None`（TC-N-03と整合）。
- 🔵 信頼性レベル: 要件定義3章「既存コードとの整合制約」・`item_repository.rs` L66-95 に基づく

### TC-N-08: 部分不正混在CSVで正常行のみ登録（EDGE-002 継続性）
- **テスト名**: 不正行を1行含むCSVで、不正行のみスキップし正常行は登録継続
  - **何をテストするか**: 正常行と不正行（title空）が混在するCSVで、正常行は登録され、不正行はスキップ＋failure記録されること（処理が中断しない）。
  - **期待される動作**: 例: 3行中2行正常・1行不正 → `success_count=2`, `failure_count=1`, `failures=[{row_number, reason}]`。正常2行はDBに存在。
- **入力値**: 行1=正常、行2=`作品名`空（不正）、行3=正常 のCSV。
  - **入力データの意味**: EDGE-002「一部行が形式不正の場合、不正行のみスキップし正常行の取込を継続」の中核シナリオ。
- **期待される結果**: success=2 / failure=1、不正行(row_number=2)が failures に記録、行3は不正行の後でも登録される。
  - **期待結果の理由**: EDGE-002・TC-016-02。1行の不正が後続行の処理を止めてはならない。
- **テストの目的**: 行単位スキップの継続性（不正行の後の正常行も登録）を確認。
  - **確認ポイント**: 不正行の後ろの正常行が確実に処理されること、row_number が正しいこと。
- 🔵 信頼性レベル: EDGE-002 / TC-016-02 に直接対応

---

## 4. 異常系テストケース

### TC-E-01: 作品名(title)空行のスキップ＋failure記録
- **テスト名**: 作品名が空の行はスキップされ failures に "title is empty" が記録される
  - **エラーケースの概要**: 必須カラム `作品名` が空文字の行を不正行として扱う。
  - **エラー処理の重要性**: title はDB必須かつアプリ必須。空titleの登録はデータ品質を破壊するため確実に排除する。
- **入力値**: `作品名`列が空文字、他列は妥当な値の1行。
  - **不正な理由**: title は必須（要件定義2章）。空はバリデーション違反。
  - **実際の発生シナリオ**: ブクログ側で作品名未設定のレコード、CSV編集時の事故。
- **期待される結果**: 当該行はスキップ。`failures` に `{ row_number, reason: "title is empty" }` を記録。例外を投げない。
  - **エラーメッセージの内容**: `"title is empty"`（要件定義2章の例示文言に一致）。
  - **システムの安全性**: 1行の不正で全体処理は落ちず、他行登録を継続。
- **テストの目的**: 必須カラム空のスキップ＋理由記録の確認。
  - **品質保証の観点**: EDGE-002 の最重要分岐。空title混入を防止。
- 🔵 信頼性レベル: 要件定義4章・note.md L61 の例示文言に直接対応

### TC-E-02: 読了日が不正形式の行スキップ（invalid date format）
- **テスト名**: 読了日が YYYY-MM-DD でない行はスキップされる
  - **エラーケースの概要**: `読了日` の日付パース失敗を不正行として扱う。
  - **エラー処理の重要性**: 不正な日付をそのままINSERTするとDB型エラー or 不正データ化するため、事前にスキップする。
- **入力値**: `作品名=x, 読了日=2024/13/40`（不正日付）の1行。
  - **不正な理由**: `読了日` は `YYYY-MM-DD` 形式制約（要件定義2章）。月13・日40は不正。
  - **実際の発生シナリオ**: ロケール差異・手編集による日付フォーマット崩れ。
- **期待される結果**: スキップ。`failures` に `{ row_number, reason: "invalid date format" }`。
  - **エラーメッセージの内容**: `"invalid date format"`（要件定義2章の例示）。
  - **システムの安全性**: title が有効でも日付不正なら登録しない（型安全優先）。
- **テストの目的**: 日付型変換失敗のスキップ＋理由記録。
  - **品質保証の観点**: DBへ不正日付が混入しないことを保証。
- 🔵 信頼性レベル: 要件定義4章「日付/数値の型変換失敗」・タスク実装詳細に対応

### TC-E-03: 評価が数値でない行スキップ（invalid rating）
- **テスト名**: 評価が数値変換できない行はスキップされる
  - **エラーケースの概要**: `評価` の `f32` パース失敗を不正行として扱う。
  - **エラー処理の重要性**: rating はDBで数値型。非数値の混入を排除する。
- **入力値**: `作品名=x, 評価=とても良い`（非数値）の1行。
  - **不正な理由**: `評価` は数値（`f32`）制約（要件定義2章）。
  - **実際の発生シナリオ**: ブクログの星表記やコメント混入。
- **期待される結果**: スキップ。`failures` に `{ row_number, reason: "invalid rating" }`。
  - **エラーメッセージの内容**: `"invalid rating"`（要件定義4章の例示）。
  - **システムの安全性**: 評価不正でも他行は影響を受けない。
- **テストの目的**: 数値型変換失敗のスキップ＋理由記録。
  - **品質保証の観点**: rating カラムの型整合を保証。
- 🔵 信頼性レベル: 要件定義4章「日付/数値の型変換失敗」に対応

### TC-E-04: カラム数不足行のスキップ（deserialize失敗）
- **テスト名**: 列数が不足してデシリアライズ失敗する行はスキップされる
  - **エラーケースの概要**: ヘッダー列数に満たない不正行を csv crate のデシリアライズ失敗として検知しスキップ。
  - **エラー処理の重要性**: 破損行でパニックせず処理継続するため。
- **入力値**: ヘッダーは5列だが、データ行が `斜陽,面白い`（2列のみ）の行。
  - **不正な理由**: 列数不足でフィールド対応が崩れる（要件定義4章「カラム数不足行」）。
  - **実際の発生シナリオ**: CSV破損、改行混入、エクスポート不具合。
- **期待される結果**: スキップ。`failures` に `{ row_number, reason }`（例: `"invalid row format"` 等のデシリアライズ理由）。
  - **エラーメッセージの内容**: 列不足を示す理由文字列。
  - **システムの安全性**: `?` 伝播やパニックではなく行スキップで処理継続。
- **テストの目的**: デシリアライズ失敗行のスキップ確認。
  - **品質保証の観点**: 破損CSV耐性。
- 🔵 信頼性レベル: 要件定義4章「カラム数不足行」に対応（reason文言は実装で確定＝黄寄りだが分類は青）

### TC-E-05: ImportFailure に row_number と reason が記録される
- **テスト名**: スキップ行ごとに row_number と reason が正確に記録される
  - **エラーケースの概要**: 複数の不正行があるとき、各行が独立した `ImportFailure` として記録されること。
  - **エラー処理の重要性**: 利用者がどの行をなぜ修正すべきか把握できることがインポートUXの要。
- **入力値**: データ行1=正常、行2=title空、行3=正常、行4=日付不正 の4行CSV。
  - **不正な理由**: 行2・行4がそれぞれ別理由で不正。
  - **実際の発生シナリオ**: 大規模CSVで複数種の不正が混在する典型。
- **期待される結果**: `failures.len()==2`、`failures` に `{row_number:2, reason:"title is empty"}` と `{row_number:4, reason:"invalid date format"}` を含む。`success_count==2`。
  - **エラーメッセージの内容**: 行ごとに固有の row_number と理由。
  - **システムの安全性**: 複数不正でも全体処理継続。
- **テストの目的**: `ImportFailure{row_number, reason}` の構造と複数件記録の確認（EDGE-002）。
  - **品質保証の観点**: 失敗の追跡可能性。
- 🔵 信頼性レベル: 要件定義2章 ImportFailure 仕様・EDGE-002 に直接対応

### TC-E-06: ファイル未添付 → 400 VALIDATION_ERROR
- **テスト名**: multipartにファイルフィールドが無い場合 400 を返す
  - **エラーケースの概要**: `file`/`csv` どちらのフィールドも無い multipart リクエスト。
  - **エラー処理の重要性**: 入力欠落を明確な4xxで弾き、500（サーバ起因）と区別する。
- **入力値**: ファイルフィールドを含まない multipart リクエスト。
  - **不正な理由**: 必須条件「ファイルが添付されること」違反（要件定義2章）。
  - **実際の発生シナリオ**: クライアント実装ミス、フィールド名タイプミス。
- **期待される結果**: HTTP 400、`{"success": false, "error": {"code": "VALIDATION_ERROR", ...}}`。DBへの登録は発生しない。
  - **エラーメッセージの内容**: ファイル未添付を示す分かりやすい文言。
  - **システムの安全性**: パース処理に入る前に早期リターン。
- **テストの目的**: ファイル未添付の入力検証（TC-016-04）。
  - **品質保証の観点**: 不正入力の明示的拒否。
- 🔵 信頼性レベル: 要件定義2章エラーレスポンス表・TC-016-04 に対応

### TC-E-07: 0バイトファイル → 400 VALIDATION_ERROR
- **テスト名**: 添付ファイルが0バイトの場合 400 を返す
  - **エラーケースの概要**: フィールドは存在するが中身が空（0バイト）。
  - **エラー処理の重要性**: 空ファイルは処理不能。ヘッダー0行とも区別して早期拒否する。
- **入力値**: `file` フィールドに0バイトの内容を添付した multipart。
  - **不正な理由**: 必須条件「0バイトでないこと」違反（要件定義2章）。
  - **実際の発生シナリオ**: 空CSVの誤アップロード、生成失敗ファイル。
- **期待される結果**: HTTP 400、`code == "VALIDATION_ERROR"`。登録なし。
  - **エラーメッセージの内容**: 空ファイルを示す文言。
  - **システムの安全性**: CSVパース前に弾く。
- **テストの目的**: 0バイト検証（TC-016-04）。
  - **品質保証の観点**: 空入力の確実な拒否。
- 🔵 信頼性レベル: 要件定義2章「0バイトでないこと」・TC-016-04 に対応

### TC-E-08: DB登録失敗時の扱い（行スキップ or 500 — 方針確認）
- **テスト名**: 行登録時にDBエラーが発生した場合の挙動を確認する
  - **エラーケースの概要**: パース・バリデーションは通ったが、`create_item_with_source` がDB起因で失敗するケース。
  - **エラー処理の重要性**: 行単位の不正（=スキップ対象）と、DB基盤障害（=500）の境界を明確化する。
- **入力値**: 接続不能な PgPool（`unreachable_pool()` 相当）に対し正常行CSVを投入、または制約違反を誘発する入力。
  - **不正な理由**: 入力自体は妥当だがDB側で失敗する。要件定義2章では「行単位の不正は500に含めない」「パース処理自体の致命的失敗→500」。
  - **実際の発生シナリオ**: DB接続断、コネクションプール枯渇。
- **期待される結果**: 実装方針に従い、(a) 当該行を failure（reason="db error" 等）として記録し処理継続、または (b) 致命的とみなし 500 INTERNAL_ERROR。**Greenフェーズ実装時にいずれかへ確定し、本ケースを確定挙動へ更新する**。最低限、未処理パニックは起こさないこと。
  - **エラーメッセージの内容**: db_error() の固定汎用文言（内部詳細は漏らさない、`item_repository.rs` db_error 準拠）。
  - **システムの安全性**: 内部スキーマ情報をクライアントへ漏らさない。
- **テストの目的**: DB障害時の安全な失敗（パニックしない・情報漏洩しない）の確認。
  - **品質保証の観点**: 行単位不正と基盤障害の責務分離。
- 🟡 信頼性レベル: note.md L172 / db_error 設計からの妥当な推測（最終方針はGreenで確定）

---

## 5. 境界値テストケース

### TC-B-01: ヘッダーのみ（データ0行）→ 200 success=0/failure=0
- **テスト名**: ヘッダー行のみでデータ行が無いCSVは 200 で空サマリを返す
  - **境界値の意味**: 「0バイト（=400）」と「中身はあるがデータ0行（=200空サマリ）」の境界を区別する。
  - **境界値での動作保証**: データ0行は正常な空インポートとして扱い、エラーにしない。
- **入力値**: 1行目ヘッダーのみ、データ行なしのCSV（非0バイト）。
  - **境界値選択の根拠**: 空ファイル(TC-E-07)との差を明確化する重要な境界。
  - **実際の使用場面**: 蔵書が空の利用者のエクスポート。
- **期待される結果**: HTTP 200、`success_count==0`, `failure_count==0`, `failures==[]`。登録なし。
  - **境界での正確性**: 0バイトでないため400にならず、データ0なので成功0。
  - **一貫した動作**: TC-016-E01（全行不正でも200）と整合する「常に200で集計返却」方針。
- **テストの目的**: データ0行の正常空サマリ確認。
  - **堅牢性の確認**: 空データでも例外を出さない。
- 🟡 信頼性レベル: 要件定義の「0バイト=400」「全行処理後にサマリ返却」からの妥当な推測

### TC-B-02: 全行不正 → 200・success_count=0・failure_count=行数
- **テスト名**: 全データ行が不正でも 200 で ImportSummary を返す
  - **境界値の意味**: success_count の下限（0）かつ failure_count が全行という極端ケース。
  - **境界値での動作保証**: 1行も登録できなくても例外で落ちない（TC-016-E01）。
- **入力値**: データ3行すべて `作品名` 空（全不正）のCSV。
  - **境界値選択の根拠**: 「全行スキップでも200」という受け入れ基準 TC-016-E01 の直接検証。
  - **実際の使用場面**: フォーマット完全不一致のCSV誤アップロード。
- **期待される結果**: HTTP 200、`success_count==0`, `failure_count==3`, `failures.len()==3`。
  - **境界での正確性**: failure_count == データ行数、success 0。
  - **一貫した動作**: 部分不正(TC-N-08)・全不正で同じサマリ構造。
- **テストの目的**: 全行不正でも200保証（例外回避）。
  - **堅牢性の確認**: イテレータ処理中にパニックしない（note.md L321-324）。
- 🔵 信頼性レベル: TC-016-E01 に直接対応

### TC-B-03: 1行のみ正常CSV（最小成功）
- **テスト名**: データ1行の正常CSVで1件登録される
  - **境界値の意味**: success_count の最小成功値（1）。
  - **境界値での動作保証**: 単一行でもヘッダー解釈・登録が成立する。
- **入力値**: ヘッダー＋正常データ1行のCSV。
  - **境界値選択の根拠**: 複数行(TC-N-01)に対する最小単位の境界。
  - **実際の使用場面**: 1冊だけ登録したい利用者。
- **期待される結果**: 200、`success_count==1`, `failure_count==0`。`items` に1件。
  - **境界での正確性**: row_number=1 の1件処理が正しい。
  - **一貫した動作**: 複数行と同じ集計ロジック。
- **テストの目的**: 最小成功ケースの確認。
  - **堅牢性の確認**: 単一行でのヘッダー/データ区別。
- 🟡 信頼性レベル: REQ-016 からの妥当な推測

### TC-B-04: row_number 採番（ヘッダー除外・1始まり）
- **テスト名**: failure の row_number はヘッダーを除いたデータ行基準で1始まり
  - **境界値の意味**: 行番号採番の基準点（ヘッダーを数えない、最初のデータ行=1）。
  - **境界値での動作保証**: 先頭データ行が不正なら row_number=1、最後の行も正しく採番。
- **入力値**: データ行1=不正(title空)、行2=正常、…最終行=不正 のCSV。
  - **境界値選択の根拠**: 要件定義2章「row_number は1始まり、ヘッダー行を除いたデータ行基準」の明示検証。オフバイワン防止。
  - **実際の使用場面**: 利用者がCSV上の行を特定して修正する際の正確性。
- **期待される結果**: 先頭データ不正行の row_number==1、最終データ不正行の row_number==データ行数。ヘッダーは加算されない。
  - **境界での正確性**: 1始まり・ヘッダー非カウント。
  - **一貫した動作**: 全テストで row_number 基準が一致。
- **テストの目的**: 行番号採番のオフバイワン境界確認。
  - **堅牢性の確認**: 先頭・末尾行での採番の正確性。
- 🔵 信頼性レベル: 要件定義2章 ImportFailure.row_number 定義に直接対応

### TC-B-05: 数百行CSVの完走（タイムアウトしない軽い確認）
- **テスト名**: 数百行規模のCSVがタイムアウトせず完走する
  - **境界値の意味**: 想定上限規模での処理完了（厳密性能検証は範囲外、完走のみ）。
  - **境界値での動作保証**: 行数増加で破綻しない。
- **入力値**: 正常行を約300行含むCSV。
  - **境界値選択の根拠**: 統合テスト要件「数百行で軽い完走確認」（要件定義3章）。
  - **実際の使用場面**: 中規模蔵書の一括移行。
- **期待される結果**: 200、`success_count==300`（全正常）でタイムアウトせず返却。
  - **境界での正確性**: 全行登録され件数一致。
  - **一貫した動作**: 小規模と同じ結果構造。
- **テストの目的**: 規模耐性の軽い確認（厳密NFRは対象外）。
  - **堅牢性の確認**: 大量行でのメモリ/時間の破綻なし。
- 🟡 信頼性レベル: 統合テスト要件からの妥当な推測（厳密検証は範囲外）

### TC-B-06: title前後空白のみ（trim後空）→ title is empty 扱い
- **テスト名**: 作品名が空白文字のみの行は空title扱いでスキップされる
  - **境界値の意味**: 「空文字」と「空白のみ文字列」の境界。視覚的に空のtitleを不正とみなすか。
  - **境界値での動作保証**: `"   "` のような実質空titleを登録しない。
- **入力値**: `作品名="   "`（半角/全角スペースのみ）の1行。
  - **境界値選択の根拠**: 要件定義3章「title空文字チェックで不正データを確実に排除」の境界明確化。
  - **実際の使用場面**: コピペ事故による空白title。
- **期待される結果**: trim後に空判定 → スキップ、`reason: "title is empty"`。**実装が trim を行うか否かは Greenで確定**し、本ケースを確定挙動へ更新する。
  - **境界での正確性**: 空白のみを空とみなす（推奨）。
  - **一貫した動作**: TC-E-01（純粋空文字）と同じ reason。
- **テストの目的**: title空判定の境界（trim有無）確認。
  - **堅牢性の確認**: 視覚的空title混入の防止。
- 🟡 信頼性レベル: 要件定義3章入力検証からの妥当な推測（trim方針はGreenで確定）

---

## 6. 回帰・DB拡張テストケース（設計判断#1の影響確認）

### TC-REG-01: create_item（manualラッパー）が consumed_date=None で従来通り動作
- **テスト名**: 既存の手動作成パスが consumed_date 追加後も壊れない
  - **何をテストするか**: `CreateItemRequest` に `consumed_date` を追加し `create_item_with_source` を拡張した後も、`create_item`（薄いラッパー、TASK-0009）が従来どおり成功し、`consumed_date=None` で登録されること。
  - **期待される動作**: `consumed_date` を含まない既存形の `CreateItemRequest` で `create_item` を呼ぶと成功し、登録 item の `consumed_date == None`。
- **入力値**: consumed_date 省略の `CreateItemRequest`（media_type=Anime 等、既存テスト相当）。
  - **入力データの意味**: 設計判断#1の拡張が既存機能を破壊しない（後方互換）ことの確認。
- **期待される結果**: 登録成功、`consumed_date == None`、その他フィールドは従来どおり。既存の TC-001 系統合テストが引き続きpassする。
  - **期待結果の理由**: 追加は `Option` ＋ `#[serde(default)]` のため未指定は None。`create_item` は manual/external_id=None 固定の挙動を維持。
- **テストの目的**: 既存作成パスへの回帰がないことを保証。
  - **確認ポイント**: INSERT 文への `consumed_date` 追加が既存呼び出しを壊さないこと。
- 🟡 信頼性レベル: `item_repository.rs` L53-56 のラッパー構造＋設計判断#1からの妥当な推測

### TC-DB-01: create_item_with_source が consumed_date を bind・RETURNING
- **テスト名**: create_item_with_source が consumed_date を受け取りDBへ保存・返却する
  - **何をテストするか**: 拡張後の `create_item_with_source` が `request.consumed_date` を INSERT に bind し、RETURNING で取得した item に反映されること。
  - **期待される動作**: `consumed_date=Some(2024-03-10)` を持つ `CreateItemRequest` を渡すと、返却 item の `consumed_date == Some(2024-03-10)`。
- **入力値**: `consumed_date=Some(NaiveDate(2024,3,10))`、source=Manual、external_id=Some("isbn") の `CreateItemRequest`。
  - **入力データの意味**: TC-N-05（API経由のE2E）に対し、リポジトリ層単体での bind 動作を直接固定する。
- **期待される結果**: 返却 item の `consumed_date == Some(2024-03-10)`、`source==Manual`、`external_id==Some("isbn")`。トランザクションでitems＋詳細テーブル両方INSERT。
  - **期待結果の理由**: INSERT 文に `consumed_date` カラムと bind を追加する設計判断#1の実装検証。
- **テストの目的**: リポジトリ層の consumed_date 永続化の単体確認（パーサ/ハンドラから切り離す）。
  - **確認ポイント**: INSERT のカラム列挙・bind 順序・RETURNING に consumed_date が含まれること。
- 🔵 信頼性レベル: 設計判断#1（ユーザー確定）＋ `item_repository.rs` L66-116 の実INSERT構造に基づく

---

## 7. テスト実装時の日本語コメント指針（既存方針を継続）

各テスト関数の冒頭に【テスト目的】【テスト内容】【期待される動作】と信頼性レベル（🔵/🟡/🔴）を、Given/When/Then 区分に対応するコメントを、各アサーションに【検証項目】【確認内容】を付与する（note.md「テスト規約」準拠）。

### ユニットテスト雛形（DB非依存・`import/booklog_csv.rs` 内）

```rust
#[test]
fn parse_row_skips_empty_title() {
    // 【テスト目的】: 作品名が空の行がスキップ＋reason記録されることを確認（TC-E-01）
    // 【テスト内容】: title空のCSV1行を行パーサへ渡す
    // 【期待される動作】: Err相当（ImportFailure）に reason="title is empty"
    // 🔵 信頼性レベル: EDGE-002 / 要件定義4章

    // 【テストデータ準備】: 必須title空・他列妥当の1行を用意する理由＝空title排除仕様の検証
    let header = "作品名,感想/レビュー,読了日,評価,ISBN";
    let row = ",面白い,2024-01-15,4.5,9784101010014";

    // 【実際の処理実行】: 行単位パーサ/バリデーションを呼び出す
    let result = parse_booklog_row(/* header, row, row_number=1 */);

    // 【結果検証】: スキップ理由が title is empty であること
    // 【検証項目】: reason 文言の一致 🔵
    assert_eq!(result.unwrap_err().reason, "title is empty");
}
```

### 統合テスト雛形（DB必須・`#[tokio::test]` + `#[ignore]`）

```rust
#[tokio::test]
#[ignore] // 【DB必須】: cargo test -p mediavault-api -- --ignored で実行
async fn import_persists_consumed_date() {
    // 【テスト目的】: 読了日が items.consumed_date に永続化されることを確認（TC-N-05・設計判断#1）
    // 🔵 信頼性レベル: ユーザー確定の設計判断#1

    // 【テスト前準備】: DBプール取得（test_pool()）と一時CSV作成
    let pool = test_pool().await;
    // 【テストデータ準備】: 読了日2024-01-15を持つ正常1行CSVを tempfile で作成

    // 【実際の処理実行】: multipartで POST /import/booklog 相当を実行

    // 【結果検証】: 登録itemのconsumed_dateがCSV値と一致
    // 【検証項目】: consumed_date == Some(2024-01-15) 🔵
    // 【品質保証】: 拡張前にNULLだった値がCSV値で保存される差分を保証
}
```

---

## 8. 要件定義との対応関係

- **参照した機能概要**: 要件定義書 1章（POST /import/booklog の概要）
- **参照した入力・出力仕様**: 要件定義書 2章（multipart入力、仮カラム定義、ImportSummary/ImportFailure、エラーレスポンス、row_number定義）
- **参照した制約条件**: 要件定義書 3章（カラムマッピング分離、既存コード整合：source=Manual / media_type既定値 / consumed_date・external_idギャップ、200保証）
- **参照した使用例**: 要件定義書 4章（正常全行登録、形式不正行スキップ、全行不正200、空ファイル400、型変換失敗、カラム数不足）
- **参照した受け入れ基準**: TC-016-01（TC-N-01）、TC-016-E01（TC-B-02）、TC-016-02/EDGE-002（TC-N-08・TC-E-01〜05）、TC-016-04（TC-E-06・TC-E-07）
- **解決した未決事項**: 要件定義書 6章 #1（consumed_date登録経路＝作成パス拡張へ確定：TC-N-04/05・TC-DB-01・TC-REG-01）、#2（media_type＝Novel固定：TC-N-06）
- **参照した既存コード**: `models/item.rs` L40-43（ItemSource）/ L70-84（CreateItemRequest）、`repositories/item_repository.rs` L53-116（create_item / create_item_with_source）

---

## 9. 品質判定

```
✅ 高品質:
- テストケース分類: 正常系8・異常系8・境界値6・回帰1・DB拡張1（計24）で網羅
- 期待値定義: 各ケースに具体的な期待値（success_count/failure_count/row_number/reason文言/DB値）を明記
- 技術選択: Rust + 標準test/tokio::test + csv/tempfile で確定（既存方針継続）
- 実装可能性: 既存 create_item_with_source 拡張・新規 import モジュールで実現可能
- 信頼性レベル: 🔵 が多数（コア要件・確定設計判断）、🟡 は実サンプル未確定/Green確定待ち項目に限定、🔴 0件
```

**信頼性レベル分布**: 🔵 15件 / 🟡 9件 / 🔴 0件

判定: **✅ 高品質**。確定済み設計判断（consumed_date拡張・media_type=Novel固定）をテストケースへ反映済み。🟡 は仮カラムフォーマット未確定（実サンプル入手後にrename差分修正）と一部 reason文言/trim方針/DBエラー方針の Green確定待ちに限定され、いずれもテスト構造自体には影響しない。

---

## 次のステップ

次のお勧めステップ: `/tsumiki:tdd-red mediavault-backend TASK-0030` でRedフェーズ（失敗テスト作成）を開始します。
