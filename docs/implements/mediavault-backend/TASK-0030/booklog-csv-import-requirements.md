# TASK-0030 要件定義書: ブクログCSVインポート実装（POST /import/booklog）

**作成日**: 2026-06-27
**機能名**: booklog-csv-import（ブクログCSVインポート）
**タスクID**: TASK-0030
**要件名**: mediavault-backend
**関連タスク**: [TASK-0030.md](../../../tasks/mediavault-backend/TASK-0030.md)
**関連ノート**: [note.md](note.md)
**親要件**: [requirements.md](../../../spec/mediavault-backend/requirements.md) REQ-016・EDGE-002 ・ [user-stories.md](../../../spec/mediavault-backend/user-stories.md) ストーリー5.1
**受け入れ基準**: TC-016-01 / TC-016-E01（acceptance-criteria.md）
**設計文書**: [api-endpoints.md](../../../design/mediavault-backend/api-endpoints.md)「POST /import/booklog」・ [architecture.md](../../../design/mediavault-backend/architecture.md)（routes→handlers→parser→repositories）・ [dataflow.md](../../../design/mediavault-backend/dataflow.md)
**前提タスク**: TASK-0009（アイテム作成ロジック、完了済み）

**【信頼性レベル凡例】**:
- 🔵 **青信号**: タスク仕様・設計文書・既存コード（note.md記載）から確実な要件
- 🟡 **黄信号**: タスク仕様・設計文書から妥当な推測による要件
- 🔴 **赤信号**: 元資料に根拠が無い推測による要件

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

`POST /import/booklog` は、ブクログのエクスポートCSVファイルを `multipart/form-data` で受け取り、行単位で解析して有効行を `items` テーブルへ一括登録するインポートAPIである。形式不正な行はスキップして失敗理由を記録し、処理全体の結果サマリ `ImportSummary` を返す。

- **何をする機能か** 🔵: ブクログ蔵書データ（CSV）を一括取り込みし、各行を `CreateItemRequest` 相当のデータへ変換して既存のアイテム作成ロジック経由で登録する。*タスク概要 L16-17・REQ-016より*
- **解決する問題** 🔵: 利用者が既存の読書記録を手入力で再登録する手間を排除し、ブクログからの移行を可能にする。*user-stories.md ストーリー5.1「既存の読書記録を手入力せずに移行できる」より*
- **想定ユーザー** 🔵: 自分の蔵書・読書記録をMediaVaultへ移行したい利用者。*user-stories.md 5.1「私は利用者として」より*
- **システム内での位置づけ** 🔵: Phase 5「内部API・インポート」のインポート機能。前提TASK-0009のアイテム作成ロジック（`item_repository::create_item_with_source`）の上に乗る、HTTP公開のインポートエンドポイント。*依存タスク L19-21・architecture.md レイヤードアーキテクチャより*
- **対象外** 🟡:
  - ブクログCSVの実カラム形式の確定（prep.md記載のサンプル入手待ち。本タスクは仮カラム定義で実装し、入手後にマッピングのみ差分修正する）。*タスク概要 L17・注意事項 L110より*
  - Steamライブラリインポート（TASK-0031で別途実装、EDGE-002は共通）。
  - 厳密な性能要件検証（NFR対応タスクの範囲外。数百行での軽い完走確認のみ）。*統合テスト要件 L98より*

**参照したEARS要件**: REQ-016, EDGE-002
**参照したユーザストーリー**: ストーリー5.1（ブクログCSVから一括インポートする）
**参照した設計文書**: api-endpoints.md「POST /import/booklog」, architecture.md（routes→handlers→parser→repositories）, dataflow.md

---

## 2. 入力・出力の仕様（EARS機能要件・型定義ベース）

### エンドポイント 🔵 *api-endpoints.md「POST /import/booklog」（multipart）・タスク実装詳細1より*

```
POST /import/booklog
Content-Type: multipart/form-data
```

### 入力: multipartファイル 🔵🟡

| 項目 | 内容 | 信頼性 |
|---|---|---|
| 受け取り方式 | Axum `axum::extract::Multipart` エクストラクタ | 🔵 *api-endpoints.md / note.md L46-50より* |
| ファイルフィールド名 | `file` または `csv` | 🟡 *note.md L158「ファイルフィールド名: `file` または `csv`」より妥当な推測* |
| ファイル形式 | CSV（1行目はヘッダー行） | 🔵 *タスク実装詳細3 L52-54より* |
| 文字コード | UTF-8前提。Shift_JIS等の可能性に備え `encoding_rs` でのフォールバックを差し替え可能な設計とする（初期実装はUTF-8） | 🟡 *タスク注意事項 L111・note.md L311-314より妥当な推測* |
| 必須条件 | ファイルが添付され、かつ0バイトでないこと | 🔵 *タスク実装詳細1 L38より* |

### 仮カラムフォーマット定義（実サンプル確認待ち） 🟡

**信頼性**: 🟡 *prep.md「ブクログCSVサンプル準備」より実物未確認のため仮定義。一般的なブクログエクスポート仕様からの妥当な推測*

CSVヘッダー名 → アプリ側フィールドのマッピング（`import/booklog_csv.rs` の `BooklogCsvRow` に `#[serde(rename = "...")]` で定義し、実サンプル入手後はこの rename 値のみを差分修正する）:

| CSVカラム名（仮） | アプリ側フィールド | 必須/任意 | 型・制約 |
|---|---|---|---|
| `作品名` | title | **必須** | 文字列。空文字は不正行扱い |
| `感想/レビュー` | description | 任意 | 文字列 |
| `読了日` | consumed_date | 任意 | `YYYY-MM-DD` 形式。パース失敗は不正行扱い |
| `評価` | rating | 任意 | 数値（`f32`）。パース失敗は不正行扱い |
| `ISBN` | external_id（参考情報） | 任意 | 文字列 |

### 出力: ImportSummary（HTTP 200） 🔵 *api-endpoints.md「レスポンス（成功, 200）」・タスク実装詳細4より*

`models/import.rs` に定義する型。プロジェクト共通の成功レスポンスエンベロープ（`{"success": true, "data": ...}`）でラップする。*note.md L52-55より*

```jsonc
{
  "success": true,
  "data": {
    "success_count": 10,
    "failure_count": 2,
    "failures": [
      { "row_number": 3, "reason": "title is empty" },
      { "row_number": 7, "reason": "invalid date format" }
    ]
  }
}
```

| フィールド | 型 | 内容 |
|---|---|---|
| `success_count` | 整数 | 登録に成功した行数 |
| `failure_count` | 整数 | スキップした行数（= `failures.len()`） |
| `failures` | `ImportFailure[]` | スキップ行の詳細配列 |
| `ImportFailure.row_number` | 整数 | スキップ行番号（**1始まり、ヘッダー行を除いたデータ行基準**） |
| `ImportFailure.reason` | 文字列 | 不正理由（例 `"title is empty"`, `"invalid date format"`） |

### 出力: エラーレスポンス 🔵🟡

| ケース | ステータス | コード | 信頼性 |
|---|---|---|---|
| ファイル未添付 または 0バイト | `400` | `VALIDATION_ERROR` | 🔵 *タスク実装詳細1 L38・完了条件より* |
| パース処理自体の致命的失敗 | `500` | `INTERNAL_ERROR` | 🟡 *note.md L172より。行単位の不正はここに含めない* |

エラー形式: `{"success": false, "error": {"code": "...", "message": "..."}}`（`ApiError` 使用）。🔵 *note.md L52-55より*

### 入出力の関係性・データフロー 🔵

1. multipartからCSVバイト列を取得 → 存在・サイズ検証（0バイト→400）
2. 文字コードデコード（UTF-8、必要に応じフォールバック）
3. `csv` crate でヘッダー行を読み、データ行を1行ずつ `BooklogCsvRow` へデシリアライズ
4. 行別バリデーション（必須title空チェック／日付・数値の型変換）
5. 正常行 → `CreateItemRequest` 相当へ変換 → `item_repository` 経由で `items` へ登録（`source` 設定）→ `success_count++`
6. 不正行 → スキップ + `ImportFailure{row_number, reason}` 記録 → `failure_count++`
7. 全行処理後に `ImportSummary` を 200 で返却

**参照したEARS要件**: REQ-016, EDGE-002
**参照した設計文書**: api-endpoints.md「POST /import/booklog」, dataflow.md, interfaces相当（`models/item.rs` の `CreateItemRequest` / `ItemSource`）

---

## 3. 制約条件（非機能要件・既存コード・アーキテクチャ設計ベース）

### アーキテクチャ制約 🔵
- 層構成 `routes → handlers → (parser) → repositories → db/sqlx` を踏襲する。*architecture.md・note.md L22より*
- **カラムマッピングの分離（最重要設計制約）**: カラム名・型変換ロジックは `import/booklog_csv.rs` の `BooklogCsvRow` 構造体に閉じ込める。ハンドラ・レスポンス型・スキップロジックの構造は実サンプル確認後も変更不要とする。*タスク完了条件 L31・注意事項 L110より* 🔵
- 文字コードデコード処理は差し替え可能な形にしておく。*タスク注意事項 L111より* 🟡

### 既存コードとの整合制約（重要） 🔵
- 登録は既存の `item_repository::create_item_with_source(pool, request, source, external_id)` を再利用する。`create_item`（`source=Manual`・`external_id=None`固定の薄いラッパー）も存在する。*`repositories/item_repository.rs` L53-70より* 🔵
- `ItemSource` enum は現状 **`Api` と `Manual` の2値のみ**で、`Booklog` 専用値は存在しない。したがって本タスクの `source` は **`ItemSource::Manual`** を用いる（note.mlの未決事項はこれで確定）。専用値を新設する場合はDBの `item_source` enumマイグレーションが別途必要となり本タスク範囲外。*`models/item.rs` L40-43より* 🔵
- **`media_type` の扱い**: `CreateItemRequest` は `media_type`（必須）を要求するが、仮カラム定義に media_type 列が無い。ブクログ＝書籍前提のため `MediaType::Novel`（または書籍系の既定値）を固定で付与する想定。実装時に既定値を確定すること。*`models/item.rs` L71-84 / L15-24より* 🟡
- **`consumed_date` / `external_id` のギャップ（実装上の要確認事項）** 🟡:
  - `CreateItemRequest` 構造体には `consumed_date` フィールドが存在しない（`media_type, title, original_title, description, cover_image_url, release_date, homepage_url, rating, is_favorite, details` のみ）。一方 `Item` / DB / `UpdateItemRequest` には `consumed_date` がある。
  - `create_item_with_source` の INSERT も `consumed_date` をbindしていない（既定値依存）。
  - 仮マッピングの `読了日→consumed_date` / `ISBN→external_id` を**実際にDBへ反映するには、既存の作成パスの拡張（`CreateItemRequest` への `consumed_date` 追加 もしくは booklog専用の登録関数追加）が必要**。
  - 本タスクの最小スコープでは、`external_id` は `create_item_with_source` の引数で渡せるため反映可能。`consumed_date` は作成パス拡張が前提となるため、tdd-testcases / 実装方針確定時に「作成パスを拡張する」か「consumed_date は当面マッピング保持のみ（登録時は未使用）」かを決定すること。

### API制約 🔵
- 全行が形式不正でも **HTTP 200** で `ImportSummary` を返す（例外で落ちない）。*完了条件 L30・TC-016-E01より* 🔵
- パスルーティングは `/items/search`・`/items/import` 等の既存リテラルパスより後に配置し、動的パス（`:id`）への誤マッチを避ける。*note.md L368-371より* 🟡

### データベース制約 🔵
- 行ごとのINSERTは既存 repository のトランザクション処理（items本体＋詳細テーブル）を経由し原子性を確保する。*note.md L330より*

### セキュリティ・パフォーマンス要件 🟡
- 入力検証: title空文字チェック・型変換エラー処理で不正データを確実に排除する。*note.md L328より*
- 不正行の理由をログに記録する（運用時分析用、tracing）。*note.md L329より*
- 数百行規模のCSVでタイムアウトせず完了すること（軽い確認のみ、厳密検証は範囲外）。*統合テスト要件 L98より*

**参照したEARS要件**: REQ-016, EDGE-002
**参照した設計文書**: architecture.md, api-endpoints.md, `models/item.rs`, `repositories/item_repository.rs`

---

## 4. 想定される使用例（Edgeケース・データフローベース）

### 基本パターン（通常要件 REQ-016） 🔵
- **正常CSV全行登録（TC-016-01）**: 仮カラム準拠の正常行のみのCSVをアップロード → `success_count` がデータ行数と一致、`failure_count=0`、各行に対応する `items` レコードが登録される。

### エッジ・エラーケース 🔵🟡
- **形式不正行のスキップ（EDGE-002 / TC-016-02）** 🔵: `作品名`（title）が空の行を1行含むCSV → その行はスキップ、`failures` に `{row_number, reason="title is empty"}` を記録、他の正常行は登録継続。
- **全行不正でも例外にならない（TC-016-E01）** 🔵: 全行が形式不正なCSV → `200` を返し、`success_count=0`、`failure_count` がデータ行数と一致。
- **空ファイル/未添付（TC-016-04）** 🟡: 0バイトファイル または ファイル未添付のmultipart → `400 VALIDATION_ERROR`。
- **日付/数値の型変換失敗** 🔵: `読了日` が `YYYY-MM-DD` でない / `評価` が数値でない行 → スキップ + `reason`（例 `"invalid date format"`, `"invalid rating"`）記録。*タスク実装詳細3 L54より*
- **カラム数不足行** 🔵: 列数が不足しデシリアライズ失敗する行 → スキップ + 理由記録。*タスク実装詳細3 L54より*
- **文字コード（Shift_JIS）** 🟡: UTF-8デコード失敗時にフォールバック変換を試みる（初期実装ではUTF-8前提、差し替え可能構造を確保）。*タスク注意事項 L111より*

**参照したEARS要件**: EDGE-002
**参照した受け入れ基準**: TC-016-01, TC-016-E01
**参照した設計文書**: dataflow.md（インポート処理フロー）

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: ストーリー5.1「ブクログCSVから一括インポートする」（user-stories.md L195-205）
- **参照した機能要件**: REQ-016（ブクログCSV一括インポート機能の提供）
- **参照した非機能要件**: 明示的NFRは本タスクに直接紐づくものは無し（性能はNFR対応タスクへ委譲）
- **参照したEdgeケース**: EDGE-002（一部行が形式不正の場合、不正行のみスキップし正常行の取込を継続）
- **参照した受け入れ基準**: TC-016-01（正常CSV全行登録）、TC-016-E01（全行不正でも200 + ImportSummary）
- **参照した設計文書**:
  - **アーキテクチャ**: architecture.md（routes→handlers→parser→repositories の層構成、CSVパーサ分離方針）
  - **データフロー**: dataflow.md（インポート処理フロー：受け取り→デコード→行パース→検証→登録→サマリ）
  - **型定義/既存コード**: `models/item.rs`（`CreateItemRequest`, `ItemSource`, `MediaType`, `Item`）, `repositories/item_repository.rs`（`create_item` / `create_item_with_source`）, `handlers/item_files.rs`（multipart処理参考）
  - **API仕様**: api-endpoints.md「POST /import/booklog」（multipart, 200=ImportSummary, 400=VALIDATION_ERROR）
  - **新規追加予定**: `handlers/import_booklog.rs`, `import/booklog_csv.rs`, `import/mod.rs`, `models/import.rs`, `routes/mod.rs`（ルート追記）

---

## 6. 品質判定・実装上の留意点

### 信頼性レベル分布
| 章 | 🔵 | 🟡 | 🔴 |
|---|---|---|---|
| 1. 概要 | 4 | 2 | 0 |
| 2. 入出力 | 5 | 4 | 0 |
| 3. 制約 | 6 | 4 | 0 |
| 4. 使用例 | 4 | 3 | 0 |
| **合計** | **19** | **13** | **0** |

🔴（根拠なし推測）は0件。コア要件（multipart受け取り、行単位スキップ、ImportSummary、200保証、400検証、マッピング分離）はすべて🔵で確実。

### 要実装判断事項（tdd-testcases 以降で確定すべき項目）
1. **🟡 `consumed_date` の登録経路**: 既存 `CreateItemRequest` に `consumed_date` が無いため、(a) 作成パスを拡張する / (b) 当面マッピング保持のみで登録時未使用とする、のいずれかを選択する必要がある。
2. **🟡 `media_type` 既定値**: 仮カラムに media_type 列が無いため、書籍系既定値（`Novel` 等）を固定付与する。
3. **🟡 multipartフィールド名**: `file` / `csv` のどちらを正とするか（両対応も可）。
4. **🟡 文字コード**: 初期はUTF-8のみ。Shift_JIS対応は差し替え可能構造のみ確保。
5. **🟢 確定済**: `source = ItemSource::Manual`（専用enum値は未存在のため）。

### 品質判定: ⚠️ 要改善（軽微）
コア要件は明確かつ実装可能だが、(1) ブクログ実カラムフォーマット未確定（仮定義で進行・後日マッピングのみ差分修正）、(2) `consumed_date` の登録経路ギャップ、の2点に意思決定が残る。これらは「カラムマッピング分離」「作成パス拡張可否」の設計判断であり、実装着手は可能。

---

## 次のステップ

次のお勧めステップ: `/tsumiki:tdd-testcases mediavault-backend TASK-0030` でテストケースの洗い出しを行います。
