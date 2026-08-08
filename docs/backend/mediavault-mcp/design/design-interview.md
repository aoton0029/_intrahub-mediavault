# mediavault-mcp 設計ヒアリング記録

**作成日**: 2026-08-07
**ヒアリング実施**: step4 既存情報ベースの差分ヒアリング

## ヒアリング目的

[要件定義書](../spec/requirements.md)は EARS 68項目・信頼性 🔵84% で確定済みであり、要件レベルの曖昧さはほぼ解消されている。そのため本ヒアリングは、**要件を実装へ落とす際に複数の妥当な選択肢がある設計判断**に絞って実施した。

### 事前調査（既存実装の追加確認）

要件定義フェーズの調査に加え、設計に必要な以下を確認した。

| 確認対象 | 結果 |
|---|---|
| `backend/mediavault-api/src/models/response.rs` | `ApiOk<T>{success, data}` / `ApiError{success, error{code, message}}`。`ApiErrorCode` に30種以上のバリアントが定義済み |
| `backend/mediavault-api/src/middleware/api_key_auth.rs` | `Authorization` を `Bearer <key>` と生キーの両形式で受理。`std::env::var(...).unwrap_or_default()` + 空文字チェックの単純比較（定数時間比較ではない） |
| `backend/mediavault-api/src/models/item.rs` の `ListItemsQuery` | **ドキュメント記載より広い**。`year` / `date_field` / `sort`（`ItemSort`）が実装済みだが [items.md](../../mediavault-api/items.md) に未記載 |
| `item_status` ENUM | `not_started` / `in_progress` / `completed` の3値（再確認） |

この調査により、要件定義書 REQ-010 に列挙した条件より多くのフィルタが既に使えることが判明し、Q3 の質問につながった。

---

## 質問と回答

### 事前フェーズ: 設計規模・出力先・調査範囲

**質問日時**: 2026-08-07
**カテゴリ**: スコープ設定

- **設計規模** → **フル設計**
- **出力先** → **`docs/backend/mediavault-mcp/design/`**（spec と同じ階層にまとめる）
- **既存実装の詳細分析** → **必要**

**テンプレートからの調整**（対象が Rust 製 MCP サーバーであるため）:

| 既定 | 調整後 | 理由 |
|---|---|---|
| `interfaces.ts` | `interfaces.rs` | 実装言語が Rust |
| `database-schema.sql` | **生成しない** | MCP は DB を持たない（REQ-140 / NFR-303） |
| `api-endpoints.md` | `mcp-tools.md` | REST API を公開せず、MCP Tools として提供する |

---

### Q1: MCPツールがエラーを返すときの形式

**質問日時**: 2026-08-07
**カテゴリ**: アーキテクチャ / エラーハンドリング
**背景**: REQ-146「エラーコードとメッセージを失わない」と REQ-114「部分失敗を区別して返す」を同時に満たす方法が複数ありうる。MCP 仕様には `isError` によるツールエラー表現があるが、これは真偽値であり部分失敗を表現できない。形式が二重化すると AI 側の読み方が複雑になる。

**回答**: **構造化結果に統一**（常に `Ok` を返し、本体に `outcome` + `code` + `message` + `retriable` を入れる）

**信頼性への影響**:
- 設計決定 **D-01** として確定（🔵）
- `Outcome` 列挙型（`success` / `partial` / `error` / `ambiguous` / `not_found`）と `ToolError` 型が全ツール共通の外枠として確定
- REQ-114 と REQ-146 が同一スキーマで満たされ、[mcp-tools.md](mcp-tools.md) の全ツールでレスポンス形が統一された
- `retriable` フラグは要件にない追加項目のため 🟡 とした

---

### Q2: organize_item の冪等性の実現方法

**質問日時**: 2026-08-07
**カテゴリ**: データフロー / 冪等性
**背景**: REQ-113「既に付与済みなら重複を作らない」の実現方法が2案あった。(a) 付与前に現状を取得して差分だけ POST する、(b) そのまま POST して 409 を「既に付与済み」と解釈する。(b) は呼び出しが少ないが、`POST /items/{id}/tags/{tag_id}` が 409 を返す保証がドキュメント上なく、エンドポイントごとに検証が必要になる。

**回答**: **付与前に現状取得**（`GET /items/{id}` と `GET /items/{id}/mylists` で現状を得て差分適用）

**信頼性への影響**:
- 設計決定 **D-03** として確定（🔵）
- `OperationResult::AlreadyApplied` が確実に判別可能になり、REQ-061「新規作成分と既存分を区別」の実装が確定
- [dataflow.md](dataflow.md) 機能4のフローチャートが4分岐（already / attach / not_found / create→attach）で確定
- 追加コストは GET 1〜2回。単一ユーザー環境では許容と判断

---

### Q3: search_library への追加フィルタ公開

**質問日時**: 2026-08-07
**カテゴリ**: 技術選択 / スコープ
**背景**: 事前調査で `ListItemsQuery` に `year` / `date_field` / `sort` が実装済みであることが判明した。これらは要件定義 REQ-010 に列挙していないが、US-09（「未視聴のお気に入り映画」「最近追加したもの」）に直接役立つ。一方、ツールスキーマを広げすぎると AI が誤用しやすくなる。

**回答**: **`year` と `sort` を公開**（`date_field` は非公開）

**信頼性への影響**:
- 設計決定 **D-06** として確定（🔵）
- `SearchLibraryParams` に `year: Option<i32>` と `sort: Option<SortOrder>` を追加
- `date_field` を除外した判断は要件にない設計上の裁量のため 🟡
- api 側は実装済みのため追加コストはスキーマ定義のみ。[prep.md](../spec/prep.md) への追加タスクは発生しない

---

### Q4: MCPツールでの Item 表現（レスポンスサイズ制御）

**質問日時**: 2026-08-07
**カテゴリ**: パフォーマンス / データモデル
**背景**: PRD §11「1回のツール結果へ全量を詰め込まない」を満たす具体的な方法が未定義だった。`GET /items` は `ItemWithRefs`（`description` を含む全フィールド + タグ + カテゴリ）を返すため、20件返すと `description` だけで相当のトークンを消費する。

**回答**: **検索は要約形**（`ItemSummary` のみ。詳細は `get_item_context` で取得）

**信頼性への影響**:
- 設計決定 **D-04** として確定（🔵）
- `ItemSummary` 型が確定（id / title / original_title / media_type / release_year / status / rating / is_favorite / tags名）
- `release_date` を `release_year` に縮約し、タグを名前のみにした点は設計上の裁量のため 🟡
- REQ-012「同名作品を区別できる情報」を満たすため `original_title` を残した

---

## ヒアリング結果サマリー

### 確認できた事項

- MediaVault-api の `ApiOk` / `ApiError` の実際の形が確認でき、`ApiEnvelope<T>` を実態に合わせて定義できた
- 既存 `api_key_auth.rs` が定数時間比較を行っていないこと、未設定時に「常に401」となる実装であることを確認し、MCP 側では起動時失敗 + `subtle` 比較を採用する根拠が明確になった
- `ListItemsQuery` の実装がドキュメントより広いことが判明し、`search_library` の表現力を追加コストなく拡張できた

### 設計方針の決定事項

| ID | 決定 | 出典 |
|---|---|---|
| D-01 | エラーは構造化結果に統一（`Outcome` + `ToolError`） | Q1 |
| D-02 | 書き込み系ツールは UUID のみ受け取る | 要件 REQ-142（requirements フェーズで確定済み） |
| D-03 | 冪等性は「事前取得 + 差分適用」で担保 | Q2 |
| D-04 | 検索結果は `ItemSummary` へ縮約 | Q4 |
| D-05 | `get_item_context` は `futures::join!` で並列合成、`try_join!` は使わない | 要件 REQ-021 / REQ-022 からの導出 |
| D-06 | `search_library` に `year` / `sort` を公開、`date_field` は非公開 | Q3 |
| — | 名前→ID解決をキャッシュしない（呼び出し内メモ化のみ） | 設計上の判断（🟡） |
| — | 部分失敗時にロールバックしない | REQ-141（削除系非公開）との整合から導出 |

### 残課題

| 残課題 | ステータス | メモ |
|---|---|---|
| `limit` 上限超過時の扱い | **本設計で決定（拒否）** | [prep.md](../spec/prep.md) PREP-09 として挙がっていたもの。丸めるとAIに上限が伝わらないため、バリデーションエラーとする。要ユーザー確認 🟡 |
| `rating` の許容範囲 | 未解決 | PREP-10。MCP側では検証せず API のバリデーションに委ねる方針とした（EDGE-104） |
| `relate_items` の自己参照拒否 | **本設計で決定（拒否）** | EDGE-005。MCP側で `item_id == related_item_id` を弾く 🟡 |
| `import_external_item` の 409 時の既存Item特定手段 | 暫定 | `GET /items?media_type=X` の走査で `external_id` 一致を探す。PRD §8 の「重複候補検索API」が将来実装されれば切り替える 🟡 |
| リバースプロキシ経路での公開範囲 | 未解決 | PREP-11。Bearer 認証以外の追加防御の要否が未定 |
| `rmcp` 3.1 の実際のAPI形状 | 実装時に確認 | `StreamableHttpService` の axum 統合方法、`#[tool]` マクロの annotation 指定方法は実装着手時に SDK ドキュメントで確認する |

### 信頼性レベル分布

**ヒアリング前**（要件定義と既存実装のみから起こした場合の想定）:
- 🔵 青信号: 118件
- 🟡 黄信号: 42件
- 🔴 赤信号: 8件

**ヒアリング後**（全設計文書の実績）:
- 🔵 青信号: 168件 (+50)
- 🟡 黄信号: 42件 (±0)
- 🔴 赤信号: 0件 (−8)

🟡 の件数自体は変わらないが、内訳が「要件の解釈が定まらない箇所」から「要件から導出した実装レベルの具体化（タイムアウト値、フィールド縮約の粒度など）」へ移った。

## 関連文書

- **アーキテクチャ設計**: [architecture.md](architecture.md)
- **データフロー**: [dataflow.md](dataflow.md)
- **型定義**: [interfaces.rs](interfaces.rs)
- **MCPツール仕様**: [mcp-tools.md](mcp-tools.md)
- **要件定義**: [requirements.md](../spec/requirements.md)
- **準備タスク**: [prep.md](../spec/prep.md)
