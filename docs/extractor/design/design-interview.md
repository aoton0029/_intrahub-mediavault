# MediaVault Extractor 設計ヒアリング記録

**作成日**: 2026-08-14
**ヒアリング実施**: step4 既存情報ベースの差分ヒアリング

## ヒアリング目的

[requirements.md](../spec/requirements.md)（125要件）と [acceptance-criteria.md](../spec/acceptance-criteria.md)（115テストケース）で機能要件は固まっていたが、実装方式に落とすうえで以下を確定する必要があった。

1. 要件段階で「設計フェーズで決める」とした残課題7件（[interview-record.md](../spec/interview-record.md) §残課題）
2. 既存 Rust 実装との整合方法
3. 型定義の言語（本件に TypeScript は登場しない）

### ヒアリング前の既存実装調査

`backend/mediavault-api/src/` を調査し、設計を既存パターンへ寄せるための事実を確認した。

| 調査項目 | 実測結果 | 設計への影響 |
|---|---|---|
| ルーター構成 | `src/main.rs:64` で公開は `.nest("/api/v1", ...)`、内部は `.merge(...)` でルート直下 | 内部APIの移設は main.rs の1行で完結する（D-6） |
| エラーコード | `src/models/response.rs:52` の `ApiErrorCode` enum + `code_and_status()` の一箇所集約 | 新エラーコード6件はここへの追加のみ |
| 認証 | `src/middleware/api_key_auth.rs:15`。`Bearer <key>` と生キーの両形式を受理 | そのまま再利用。実装不要 |
| repository パターン | 18個すべてが `db_error()` + `sqlx::query_as` の同一構造 | 新規2本も同じ形で書ける |
| DB制約検出 | `src/repositories/db_error_utils.rs` に `is_unique_violation`（23505） | 部分UNIQUE違反による冪等化にそのまま使える |
| migration | `20260623000001_init_schema` 1本のみ。`update_updated_at_column()` は既に定義済み | 新規 migration は追加ファイル。トリガー関数は再利用 |
| 日時型 | 全テーブル `TIMESTAMP`（TIMESTAMPTZ ではない）、Rust 側は `chrono::NaiveDateTime` | 新テーブルも同じ型を使う |
| 201 の返し方 | `src/handlers/item_files.rs:46` で `(StatusCode::CREATED, Json(ApiOk::new(x)))` | `ApiOk` の `IntoResponse` は常に200のため明示が必要 |
| `FileType` | `src/models/item_file.rs:16` に6値の sqlx ENUM が実装済み | 抽出対象判定に再利用（`Pdf` / `Image` のみ許可） |

この調査により、設計の大半を「既存パターンの適用」として確定でき、推測に頼る範囲が狭まった。

---

## 質問と回答

### Q1: 抽出レコードは履歴を残すか？

**質問日時**: 2026-08-14
**カテゴリ**: データモデル
**背景**: 要件段階（REQ-044）で「未完了の抽出は1ファイル1件」を部分UNIQUE index で担保することは決まっていた。しかしこの制約は `queued` / `running` / `cancelling` のみを縛るため、終端状態の行が複数残ることを**許す**。要件はこの点に言及しておらず、「再抽出時に既存行を UPDATE で上書きする」設計も同じ制約を満たす。両者はテーブルサイズと追跡可能性のトレードオフであり、DBスキーマを書く前に決める必要があった。

**選択肢**:
- 履歴を残す（再抽出のたびに新行、GET は最新1件）
- 1ファイル1行に固定（UPDATE で上書き）

**回答**: **履歴を残す**

**信頼性への影響**:
- [architecture.md](architecture.md) D-2 として確定（🔵）
- `GET .../extraction` の取得方法が「`created_at DESC LIMIT 1`」に確定し、[api-endpoints.md](api-endpoints.md) の該当項目が 🟡 → 🔵 へ
- index `idx_item_file_extractions_file_created (item_file_id, created_at DESC)` が必要になり追加（🔵）
- claim 用 index を**部分index**にする設計が導かれた。終端状態の履歴行が増えても claim の走査量が増えないようにするため（🔵）
- [user-stories.md](../spec/user-stories.md) ストーリー4.1（OCR方式の変遷追跡）・4.2（抽出方式ごとの比較）が実現可能であることが確定
- 一方 `item_file_texts` は `item_file_id` UNIQUE のまま。「抽出履歴は残すが抽出結果は現行のみ」という非対称が明示された

---

### Q2: 複数ページにまたがるチャンクの `label` は？

**質問日時**: 2026-08-14
**カテゴリ**: データモデル / API仕様
**背景**: 要件段階（REQ-042・ヒアリングQ5）で `boundaries` を MVP から保存することは決めたが、**チャンクへのラベル割り当て規則**は未定だった。`chunk_size` の既定値 4000 文字は通常数ページ分に相当するため、「1チャンク = 複数ページ」が例外ではなく常態である。[item-text.md](../../backend/mediavault-api/item-text.md) の既存記述「分割位置が形式固有の区切りに対応づけられる場合のみ設定する」を厳密に読むと、実際にはほぼ常に `null` になってしまい、boundaries を保存する意味が失われる。

**選択肢**:
- 範囲表記 `"p.1-3"`
- 先頭ページのみ `"p.1"`
- 完全一致時のみ、他は `null`

**回答**: **範囲表記 `"p.1-3"`**

**信頼性への影響**:
- [architecture.md](architecture.md) D-5 として確定（🔵）
- `compose_chunk_label()` のアルゴリズムが確定し、[interfaces.rs](interfaces.rs) に実装込みで記述できた（🔵）
- [item-text.md](../../backend/mediavault-api/item-text.md) の「分割位置が形式固有の区切りに対応づけられる場合のみ」という記述が**改訂対象**として追加確定
- 「チャンク末尾の引用でページがずれる」という先頭ページ案の欠陥が回避された
- 補助関数 `numeric_suffix()`（`"p.42"` → `"42"`）が必要になった。`"p.1-p.3"` ではなく `"p.1-3"` とするため（🟡）
- REQ-413（`index` は0起点連番のまま）との両立が明示され、ページ情報は `label` にのみ現れるという規約が維持された

---

### Q3: 既存 `/internal/*` 5本の移設方法は？

**質問日時**: 2026-08-14
**カテゴリ**: アーキテクチャ
**背景**: [prep.md](../spec/prep.md) §確認事項で判断待ちとしていた項目。要件（REQ-029）は `/api/v1/internal/*` への統一を決めていたが、旧パスを暫定 alias として残すかは未定だった。既存実装を調べたところ、`src/main.rs:64` が内部ルーターを `.merge()` でルート直下に付けており、変更は1行で済むことが判明した。

**選択肢**:
- 即時切替（旧パス削除）
- 旧パスを alias として残す

**回答**: **即時切替**

**信頼性への影響**:
- [architecture.md](architecture.md) D-6 として確定（🔵）
- 実装方法が具体化: `internal.rs` のパス文字列は変更せず、`main.rs` で内部ルーターを公開ルーターへ `merge` してから `/api/v1` 配下へ `nest` する。`merge` は Router ごとのレイヤーを保持するため `api_key_auth` は内部ルートにのみ適用され続ける（🔵）
- alias 削除の後始末タスクが不要になり、実装スコープが縮小
- mediavault-mcp 側の内部APIキー判定ロジックが**修正不要**であることが確定（元々 `/api/v1/internal/*` を前提にしていたため、api 側を合わせることで不整合が解消する）
- [prep.md](../spec/prep.md) §確認事項の1件が解消

---

### Q4: OCRフォールバックの判定基準は？

**質問日時**: 2026-08-14
**カテゴリ**: 技術選択
**背景**: PRD FR-004 は「テキストが存在しない、または**品質基準を満たさない**ページだけOCRを実行する」と書いているが、品質基準の定義がない。[tech-stack.md](../tech-stack.md) §「このファイルで決めていないこと」にも残課題として挙がっていた。文字数0のみを条件にすると、文字化けPDFや透かしテキストだけのスキャンPDFを取りこぼす。

**選択肢**:
- 文字密度の閾値＋環境変数で調整可能
- 文字数0のページのみ
- 設計フェーズでは決めず、境界だけ定義する

**回答**: **文字密度の閾値＋設定可能**

**信頼性への影響**:
- [architecture.md](architecture.md) D-7 として確定（🔵）
- `needs_ocr()` のアルゴリズム（A4基準で正規化した文字数）が確定し、[interfaces.py](interfaces.py) に実装込みで記述できた（🔵）
- 環境変数 `EXTRACTOR_OCR_FALLBACK_MIN_CHARS_PER_PAGE` が追加された。既定値 50 は暫定（🟡）で、実データでのチューニングが前提
- [tech-stack.md](../tech-stack.md) §決めていないこと の1件が解消（判定方式は確定、閾値の実測は残る）
- REQ-106 が「実装可能な粒度」に到達した

---

### Q5: 作業規模と出力先

**質問日時**: 2026-08-14
**カテゴリ**: スコープ
**背景**: kairo-design の必須確認事項。要件定義を `docs/extractor/spec/` に置いたため、設計も同階層に置くほうが相対リンクが短く保てる。

**回答**: **フル設計 / `docs/extractor/design/`**

**信頼性への影響**: 成果物構成が確定。architecture / dataflow / database-schema / api-endpoints / interfaces（2言語）/ design-interview の6ファイル。

---

### Q6: 型定義ファイルの形式は？

**質問日時**: 2026-08-14
**カテゴリ**: 技術選択
**背景**: kairo-design のテンプレートは `interfaces.ts`（TypeScript）を前提としているが、本件は Rust（api）と Python（worker）の2言語にまたがり、TypeScript は一切登場しない。フロントエンドは抽出機能に関与しない。

**選択肢**:
- Rust + Python の2ファイル
- 内部APIの JSON スキーマ1本に集約
- Rust のみ

**回答**: **Rust + Python の2ファイル**

**信頼性への影響**:
- [interfaces.rs](interfaces.rs)（42項目）と [interfaces.py](interfaces.py)（38項目）を作成
- 単なる型宣言に留めず、既存パターン（`sqlx::Type` / `sqlx::FromRow` / `parse_xxx_request`）へ合わせた形で書けたため、実装時にほぼそのまま写せる
- worker 側は Protocol 境界（`OcrEngine` / `Extractor` / `ProgressReporter` / `ExtractorApiClient`）を明示でき、REQ-069（yomitoku 固有型を境界外へ出さない）とテスト戦略（TC-060-07: fake への差し替え）が型で表現された

---

### Q7: 既存実装の詳細分析を行うか？

**質問日時**: 2026-08-14
**カテゴリ**: スコープ
**背景**: 設計を既存コードのパターンへ寄せるか、要件と設計文書のみから起こすかの判断。

**回答**: **必要**

**信頼性への影響**:
- 冒頭の調査表9項目が得られ、設計の多くが「推測」から「既存パターンの適用」へ変わった
- 特に大きかったのは D-6（main.rs の実測により移設方法が1行の変更と判明）と、エラーコード追加箇所の特定
- [architecture.md](architecture.md) の信頼性が 🔵 93% に到達した主因

---

### Q8: claim レスポンスのファイル参照形式は？

**質問日時**: 2026-08-14（要件定義フェーズで先行実施）
**カテゴリ**: アーキテクチャ / セキュリティ
**背景**: `item_files.path` はリンク経路（絶対パス）とアップロード経路（`STORAGE_ROOT` 相対）の2系統があり、意味が異なる。この差をどこで吸収するかで、api と worker の結合度が変わる。

**回答**: **root 種別 + 相対パス**

**信頼性への影響**:
- [architecture.md](architecture.md) D-3 として確定（🔵）
- `FileRef { root: "storage"|"library", relative_path }` が [interfaces.rs](interfaces.rs) / [interfaces.py](interfaces.py) の両方に定義された
- api が worker のマウントレイアウトを知る必要がなくなり、コンテナ間の結合が下がった
- worker 側の `resolve_file_ref()` に検証手順（事前拒否 → resolve → **resolve後に** `is_relative_to` 判定 → 開く）が確定し、NFR-103（symlink 経由の脱出防止）が実装可能な形になった

---

## ヒアリング結果サマリー

### 確認できた事項

- 抽出は履歴を残し、`GET` は最新1件を返す（Q1）
- `label` は範囲表記。boundaries を保存する意味が実際に生きる形になった（Q2）
- 内部APIは即時切替。実装は main.rs の1行 + パス文字列そのまま（Q3・既存実装調査）
- OCRフォールバックは文字密度の閾値。環境変数で調整可能（Q4）
- 型定義は Rust + Python。TypeScript は不要（Q6）
- ファイル参照は root 種別 + 相対パス（Q8）
- 既存実装の9パターンを踏襲でき、新規に発明する部分がほぼない（Q7）

### 設計方針の決定事項

| # | 決定 | 文書 |
|---|---|---|
| D-1 | 抽出リソースはファイル従属。冪等性は部分UNIQUE index | [architecture.md](architecture.md) |
| D-2 | 抽出は履歴を残す。`item_file_texts` は常に1行 | 同上 |
| D-3 | ファイル参照は root 種別 + 相対パス | 同上 |
| D-4 | complete は単一トランザクションで結果保存と成功遷移を確定 | 同上 |
| D-5 | `label` は範囲表記 `"p.1-3"` | 同上 |
| D-6 | 内部APIは `/api/v1/internal/*` へ即時移設 | 同上 |
| D-7 | OCRフォールバックは文字密度の閾値 | 同上 |
| D-8 | 抽出方式は `method` + ページ数の併記 | 同上 |

### 実装上の重要な帰結

1. **冪等性の実装が3行になった**: INSERT → `is_unique_violation` で検出 → 既存行を SELECT。既存 `db_error_utils` をそのまま使える
2. **「キャンセル後に成功確定しない」がクエリ条件そのもの**: complete の `WHERE state = 'running'` により `cancelling` は0行になる。特別な分岐が不要
3. **「古い worker が上書きしない」も同様**: `WHERE lease_token = $2` で自然に弾かれる
4. **エラーログの振り分けが自動**: 既存 `ApiError::into_response` が 5xx→ERROR / 4xx→WARN を行うため、新エラーコードも追加実装なしで乗る

### 残課題

設計フェーズでも確定せず、実装または実測に委ねる項目。すべて [prep.md](../spec/prep.md) §確認事項に対応する。

1. **`EXTRACTOR_OCR_FALLBACK_MIN_CHARS_PER_PAGE` の値**（既定 50 は暫定）— 実データでのチューニングが必要
2. **`max_attempts` の既定値**（3 を仮置き）
3. **lease 期間と heartbeat 間隔**（300s / 30s を仮置き）— CPU OCR の実測後に確定
4. **`MAX_CONTENT_CHARS` の値**（500万文字を仮置き）— 蔵書の最大文字数を確認して決める
5. **`EXTRACTOR_JOB_TIMEOUT_SEC`**（3600s を仮置き）— NFR-003 の実測待ち
6. **MVP対象形式に `archive`（cbz/cbr）を含めるか** — 現設計は pdf / image のみ
7. **lease 切れ sweeper の実行タイミング**（claim 時に併せて実行 / 定期実行）— [database-schema.sql](database-schema.sql) §sweeper にクエリのみ記載
8. **GPU（`cuda`）運用を行うか** — MVP は CPU 既定で進められるため Phase 6 まで先送り可能

いずれも**設計を止める性質の課題ではない**。1〜5は環境変数・定数の値のみで、構造には影響しない。

### 信頼性レベル分布

**ヒアリング前**（要件定義と既存設計文書のみから設計を起こした場合の想定）:
- 🔵 青信号: 約 60%
- 🟡 黄信号: 約 30%（残課題7件、既存実装との整合方法）
- 🔴 赤信号: 約 10%（リソースの履歴保持、label 規則、型定義の言語）

**ヒアリング後**（設計文書6ファイルの実測）:

| ファイル | 🔵 | 🟡 | 🔴 | 項目数 |
|---|---|---|---|---|
| [architecture.md](architecture.md) | 27 (93%) | 2 (7%) | 0 | 29 |
| [dataflow.md](dataflow.md) | 16 (89%) | 2 (11%) | 0 | 18 |
| [api-endpoints.md](api-endpoints.md) | 20 (83%) | 4 (17%) | 0 | 24 |
| [database-schema.sql](database-schema.sql) | 32 (78%) | 9 (22%) | 0 | 41 |
| [interfaces.rs](interfaces.rs) | 34 (81%) | 8 (19%) | 0 | 42 |
| [interfaces.py](interfaces.py) | 29 (76%) | 9 (24%) | 0 | 38 |
| **合計** | **158 (82%)** | **34 (18%)** | **0 (0%)** | **192** |

**要件トレーサビリティ**: [requirements.md](../spec/requirements.md) の全125要件が設計文書のいずれかで扱われていることを確認済み（[architecture.md](architecture.md) §要件トレーサビリティ）。設計文書から参照されている要件IDのうち、requirements.md / acceptance-criteria.md に存在しないもの（宛先のないID）は0件。

8問の回答と既存実装調査により 🔴 が消滅した。残る 🟡 34件のうち約半数は「未確定の既定値」であり、上記残課題1〜5に集約される。

---

## 関連文書

- **アーキテクチャ設計**: [architecture.md](architecture.md)
- **データフロー**: [dataflow.md](dataflow.md)
- **DBスキーマ**: [database-schema.sql](database-schema.sql)
- **API仕様**: [api-endpoints.md](api-endpoints.md)
- **型定義（api）**: [interfaces.rs](interfaces.rs)
- **型定義（worker）**: [interfaces.py](interfaces.py)
- **要件定義**: [requirements.md](../spec/requirements.md)
- **要件ヒアリング記録**: [interview-record.md](../spec/interview-record.md)
- **準備タスク**: [prep.md](../spec/prep.md)
