# mediavault-mcp MCPツール仕様

**作成日**: 2026-08-07
**関連設計**: [architecture.md](architecture.md) / [dataflow.md](dataflow.md) / [interfaces.rs](interfaces.rs)
**関連要件定義**: [requirements.md](../spec/requirements.md)

> **注**: 本書は REST API 仕様書ではなく **MCPツール仕様書** である。MediaVault-mcp は独自の REST API を公開せず、MCP の Tools として機能を提供する。MediaVault-api の REST 仕様は [docs/backend/mediavault-api/](../../mediavault-api/) を参照。

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・設計文書・既存API仕様・ユーザヒアリングを参考にした確実な定義
- 🟡 **黄信号**: それらから妥当な推測による定義
- 🔴 **赤信号**: それらにない推測による定義

---

## 共通仕様

### エンドポイント 🔵

**信頼性**: 🔵 *REQ-001・[tech-stack.md](../tech-stack.md) より*

| パス | 用途 | 認証 |
|---|---|---|
| `POST /mcp` | Streamable HTTP による MCP セッション | **必須** |
| `GET /healthz` | コンテナ死活監視（MCPプロトコル外）| 不要 |

stdio トランスポートは第2段階（REQ-902）。

### 認証 🔵

**信頼性**: 🔵 *REQ-115・NFR-101 / NFR-102 より*

```http
Authorization: Bearer {MCP_AUTH_TOKEN}
```

- 静的トークン。MCPプロセス自身が `subtle` で定数時間比較する
- 欠落・不一致は **401** を返し、ツールは実行されない（MediaVault-api も呼ばれない）
- `MCP_AUTH_TOKEN` 未設定時はプロセスが起動しない（REQ-122）

### ツール結果の共通形式 🔵

**信頼性**: 🔵 *設計決定 D-01（ヒアリング2026-08-07 Q1）・REQ-146 / REQ-114 より*

**すべてのツールは MCP プロトコル上は成功を返し**、結果本体の `outcome` で状態を表現する。

```json
{
  "outcome": "success",
  "...": "ツール固有のフィールド",
  "error": null
}
```

| `outcome` | 意味 |
|---|---|
| `success` | 要求されたすべての操作が完了した |
| `partial` | 一部の操作が失敗または未処理のまま終了した（REQ-114） |
| `error` | 操作を実行できなかった |
| `ambiguous` | 対象が一意に確定せず、書き込みを行わなかった（REQ-142） |
| `not_found` | 指定された対象が存在しなかった（REQ-110） |

**エラー本体**:

```json
{
  "error": {
    "code": "ITEM_NOT_FOUND",
    "message": "指定されたアイテムが見つかりません",
    "retriable": false
  }
}
```

- `code` / `message` は MediaVault-api の値をそのまま保持する（REQ-146）
- MCP 自身が生成するコードは `MCP_` プレフィックス付き（`MCP_API_UNREACHABLE` など）
- `retriable` は同一入力での再試行が意味を持つかを示す 🟡

### 部分失敗の表現 🔵

**信頼性**: 🔵 *REQ-114・EDGE-003 / EDGE-004 より*

複数対象を扱うツール（`organize_item` / `create_item`）は、対象ごとに `OperationResult` を返す。

```json
{
  "outcome": "partial",
  "tags": [
    { "result": "applied", "target_id": "...", "target_name": "積読", "created_new": false },
    { "result": "already_applied", "target_id": "...", "target_name": "SF" },
    { "result": "not_resolved", "requested_name": "未読", "available_names": ["積読", "既読"] },
    { "result": "failed", "requested_name": "名作", "error": { "code": "...", "message": "...", "retriable": true } },
    { "result": "skipped", "requested_name": "後続" }
  ]
}
```

### ページネーション 🔵

**信頼性**: 🔵 *REQ-130 / REQ-143・`items.md` の keyset 仕様より*

- `limit`: **1..=50、既定 20**（MediaVault-api の最大100より厳しい）
- 51以上は丸めず **バリデーションエラー**（EDGE-101 の決定）🟡
- 継続取得は不透明な `next_cursor` 文字列で行う。keyset の内部構造（`after_created_at` / `after_id`）は隠蔽する 🟡

### ツールのメタデータ区分 🔵

**信頼性**: 🔵 *REQ-004・REQ-141 より*

| 区分 | ツール | annotation |
|---|---|---|
| 読み取り | `search_library`, `get_item_context`, `search_external_catalog`, `collection_overview`, `health` | `readOnlyHint: true` |
| 追記・更新 | `import_external_item`, `create_item`, `update_consumption`, `organize_item`, `relate_items`, `add_access_link` | `readOnlyHint: false`, `destructiveHint: false` |
| 破壊的 | **なし**（MVPでは公開しない） | — |

---

## ツール一覧

| # | ツール名 | 種別 | 関連US | 関連要件 |
|---|---|---|---|---|
| 1 | [`search_library`](#1-search_library) | 読み取り | US-01, US-09 | REQ-010 ~ REQ-014 |
| 2 | [`get_item_context`](#2-get_item_context) | 読み取り | US-02 | REQ-020 ~ REQ-022 |
| 3 | [`search_external_catalog`](#3-search_external_catalog) | 読み取り | US-03 | REQ-030 ~ REQ-032 |
| 4 | [`import_external_item`](#4-import_external_item) | 追記 | US-03 | REQ-033, REQ-112 |
| 5 | [`create_item`](#5-create_item) | 追記 | US-04 | REQ-040, REQ-041 |
| 6 | [`update_consumption`](#6-update_consumption) | 更新 | US-05 | REQ-050 ~ REQ-052 |
| 7 | [`organize_item`](#7-organize_item) | 追記 | US-06 | REQ-060, REQ-061, REQ-111 |
| 8 | [`relate_items`](#8-relate_items) | 追記 | US-07 | REQ-070 ~ REQ-072 |
| 9 | [`add_access_link`](#9-add_access_link) | 追記 | US-08 | REQ-080, REQ-081 |
| 10 | [`collection_overview`](#10-collection_overview) | 読み取り | US-09 | REQ-090 |
| 11 | [`health`](#11-health) | 読み取り | 全般 | REQ-100 |

---

## 1. search_library 🔵

**信頼性**: 🔵 *REQ-010 ~ REQ-014 / REQ-143・ヒアリング2026-08-07 Q3 / Q4 より*

**関連要件**: REQ-010, REQ-011, REQ-012, REQ-013, REQ-014, REQ-130, REQ-143

**ツール説明（AI向け）**:
> MediaVault に**既に登録済み**の作品を検索する。所蔵確認に使う。未登録の作品を外部から探す場合は `search_external_catalog` を使うこと。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `title` | string | — | 部分一致。本題・原題・別名を横断（PREP-02 前提）🔵 REQ-011 |
| `media_type` | enum | — | `anime`/`movie`/`drama`/`manga`/`novel`/`game`/`academic_book`/`paper` |
| `status` | enum | — | `not_started`/`in_progress`/`completed` |
| `tag` | string | — | タグ**名**。MCP が ID へ解決する |
| `category` | string | — | カテゴリ**名** |
| `is_favorite` | bool | — | |
| `year` | int | — | 公開年 🔵 ヒアリング Q3 |
| `sort` | enum | — | `created_desc`/`created_asc`/`updated_desc`/`title_asc`/`rating_desc`/`release_desc` 🔵 ヒアリング Q3 |
| `limit` | int | — | 1..=50、既定 20 |
| `cursor` | string | — | 前回の `next_cursor` |

**レスポンス**:

```json
{
  "outcome": "success",
  "source": "mediavault_library",
  "total_count": 3,
  "items": [
    {
      "item_id": "b6b6f9a0-...",
      "title": "作品A",
      "original_title": "Work A",
      "media_type": "anime",
      "release_year": 2023,
      "status": "in_progress",
      "rating": 8.5,
      "is_favorite": true,
      "tags": ["お気に入り原作"]
    }
  ],
  "next_cursor": "eyJhZnRlcl9pZCI6...",
  "applied_filters": { "title": "作品A", "media_type": "anime" },
  "error": null
}
```

- `source` は固定値。外部カタログ結果との取り違えを防ぐ（REQ-014, REQ-032）
- `description` / `details` は含めない。詳細は `get_item_context` で取得する（設計決定 D-04）

**主なエラー**:

| `outcome` | `code` | 条件 |
|---|---|---|
| `not_found` | `MCP_NAME_NOT_RESOLVED` | `tag` / `category` の名前が解決できない。`available_names` を併せて返す 🟡 |
| `error` | `MCP_INVALID_ARGUMENT` | `limit` が範囲外、検索語が空白のみ（EDGE-102） |
| `error` | `MCP_API_UNREACHABLE` | MediaVault-api へ到達できない（EDGE-001） |

---

## 2. get_item_context 🔵

**信頼性**: 🔵 *REQ-020 ~ REQ-022・ヒアリング（requirements Q5）・`items.md` GET /items/{id} 仕様より*

**関連要件**: REQ-020, REQ-021, REQ-022, NFR-002, NFR-005, EDGE-105

**ツール説明（AI向け）**:
> 1つの作品について、詳細・タグ・カテゴリ・ファイル・リンク・関連作品・スタッフ・キャストなどをまとめて取得する。作品について詳しく答える前に呼ぶこと。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `item_id` | uuid | ✔ | `search_library` で解決した ID |

**レスポンス**:

```json
{
  "outcome": "partial",
  "item": {
    "item_id": "b6b6f9a0-...",
    "title": "作品A",
    "media_type": "anime",
    "status": "in_progress",
    "tags": [{ "id": "...", "name": "..." }],
    "categories": [],
    "streaming_links": [{ "link_id": "...", "platform": "netflix", "url": "https://..." }],
    "detail": { "episodes": 12, "studios": ["..."] }
  },
  "relations": { "state": "loaded", "items": [ /* RelationView */ ] },
  "mylists":   { "state": "empty" },
  "groups":    { "state": "empty" },
  "cast":      { "state": "loaded", "items": [ /* CreditView */ ] },
  "staff":     { "state": "loaded", "items": [] },
  "files":     { "state": "failed", "error": { "code": "INTERNAL_ERROR", "message": "...", "retriable": true } },
  "links":     { "state": "loaded", "items": [] },
  "trailers":  { "state": "empty" },
  "error": null
}
```

**セクションの3状態**（REQ-021・EDGE-105）:

| `state` | 意味 |
|---|---|
| `loaded` | 取得成功、データあり |
| `empty` | 取得成功、**未登録** |
| `failed` | **取得失敗**（未登録とは異なる） |

1つでも `failed` があれば全体の `outcome` は `partial` になる。

**主なエラー**:

| `outcome` | `code` | 条件 |
|---|---|---|
| `not_found` | `ITEM_NOT_FOUND` | `GET /items/{id}` が404。他セクションの結果は返さない |

---

## 3. search_external_catalog 🔵

**信頼性**: 🔵 *REQ-030 ~ REQ-032 / REQ-117・`GET /items/search` 仕様より*

**関連要件**: REQ-030, REQ-031, REQ-032, REQ-117, NFR-202, EDGE-006

**ツール説明（AI向け）**:
> **外部の作品データベース**（Annict / TMDb / 楽天ブックス / Steam / NDL）を検索する。所蔵確認ではない。ここで見つかった作品は MediaVault に**まだ登録されていない**。登録するには `import_external_item` を呼ぶこと。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `media_type` | enum | ✔ | プロバイダの自動選択に使う |
| `q` | string | ✔ | 検索語 |

**レスポンス**:

```json
{
  "outcome": "success",
  "source": "external_catalog",
  "provider": "annict",
  "candidates": [
    {
      "external_id": "12345",
      "provider": "annict",
      "title": "作品A",
      "release_year": 2023,
      "media_type": "anime",
      "cover_image_url": "https://...",
      "summary": "あらすじ冒頭..."
    }
  ],
  "error": null
}
```

- **`item_id` フィールドを持たない**。所蔵品との取り違えを型レベルで防ぐ（REQ-032）
- `external_id` と `provider` は `import_external_item` へそのまま渡せる（NFR-003）

**主なエラー**:

| `outcome` | `code` | 条件 | `retriable` |
|---|---|---|---|
| `error` | `API_KEY_NOT_CONFIGURED` | プロバイダAPIキー未設定（422）🔵 REQ-117 | false |
| `error` | `EXTERNAL_API_TIMEOUT` | 外部APIタイムアウト（502）🔵 EDGE-006 | true |
| `error` | `EXTERNAL_API_ERROR` | 外部API障害（502） | true |

---

## 4. import_external_item 🔵

**信頼性**: 🔵 *REQ-033 / REQ-112・`POST /items/import` 仕様より*

**関連要件**: REQ-033, REQ-112, NFR-003

**ツール説明（AI向け）**:
> `search_external_catalog` で見つけた候補を MediaVault へ登録する。**利用者が候補を1つ選んでから**呼ぶこと。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `media_type` | enum | ✔ | |
| `external_id` | string | ✔ | 候補の `external_id` をそのまま |
| `provider` | string | — | 候補の `provider` をそのまま |

**レスポンス**:

```json
{
  "outcome": "success",
  "item": { /* ItemSummary */ },
  "already_existed": false,
  "error": null
}
```

- 既にインポート済みの場合、409 をエラーにせず `already_existed: true` + 既存 Item を返す（REQ-112）

**主なエラー**:

| `outcome` | `code` | 条件 |
|---|---|---|
| `not_found` | — | プロバイダ側で対象が見つからない（404） |
| `error` | `VALIDATION_ERROR` | `external_id` が空文字（400） |
| `error` | `API_KEY_NOT_CONFIGURED` | 422 |

---

## 5. create_item 🔵

**信頼性**: 🔵 *REQ-040 / REQ-041 / REQ-114・`POST /items` 仕様より*

**関連要件**: REQ-040, REQ-041, REQ-114, EDGE-004, EDGE-103

**ツール説明（AI向け）**:
> 外部データベースに存在しない作品（同人誌・個人資料・ローカルファイルなど）を手動で登録する。外部で見つかる作品は `search_external_catalog` → `import_external_item` を使うこと。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `media_type` | enum | ✔ | |
| `title` | string | ✔ | 空白のみは拒否（EDGE-103） |
| `original_title` / `description` / `homepage_url` | string | — | |
| `release_date` | date | — | `YYYY-MM-DD` |
| `rating` | number | — | |
| `is_favorite` | bool | — | |
| `file_paths` | string[] | — | 実データ領域の絶対パス 🔵 REQ-041 |
| `urls` | string[] | — | 参照URL 🔵 REQ-041 |
| `tags` | string[] | — | タグ**名** 🔵 REQ-041 |
| `mylists` | string[] | — | マイリスト**名** 🔵 REQ-041 |
| `create_if_missing` | bool | — | 既定 false 🔵 REQ-111 |

**レスポンス**（部分失敗の例）:

```json
{
  "outcome": "partial",
  "item": { "item_id": "new-uuid", "title": "同人誌A", "media_type": "manga" },
  "tags":    [{ "result": "failed", "requested_name": "同人", "error": { "code": "...", "retriable": true } }],
  "mylists": [{ "result": "skipped", "requested_name": "夏コミ" }],
  "files":   [{ "result": "applied", "target_id": "...", "target_name": "/srv/manga/...", "created_new": true }],
  "links":   [],
  "error": null
}
```

- **Item 作成に成功していれば `item` は必ず返る**。後続が失敗しても作成済み ID を失わない（EDGE-004）
- ロールバックしない（dataflow.md「データ整合性の保証」参照）

---

## 6. update_consumption 🔵

**信頼性**: 🔵 *REQ-050 ~ REQ-052 / REQ-110 / REQ-142・PRD §15.1 より*

**関連要件**: REQ-050, REQ-051, REQ-052, REQ-110, REQ-142, NFR-203, EDGE-106

**ツール説明（AI向け）**:
> 視聴・読了・プレイの状況を記録する。`item_id` は `search_library` で**先に一意に特定**しておくこと。日付は `YYYY-MM-DD` に変換してから渡すこと（「昨日」などの表現は受け付けない）。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `item_id` | uuid | ✔ | **UUID のみ**。タイトル指定は不可（REQ-142） |
| `status` | enum | — | `not_started`/`in_progress`/`completed` の3値のみ（EDGE-106） |
| `consumed_date` | date | — | `YYYY-MM-DD`（REQ-052） |
| `rating` | number | — | |
| `is_favorite` | bool | — | |

**レスポンス**:

```json
{
  "outcome": "success",
  "item_id": "b6b6f9a0-...",
  "title": "作品A",
  "changes": [
    { "field": "status", "before": "in_progress", "after": "completed" },
    { "field": "consumed_date", "before": null, "after": "2026-08-06" },
    { "field": "rating", "before": 7.0, "after": 8.0 }
  ],
  "error": null
}
```

- 更新前の値を取得するため内部で `GET /items/{id}` を1回追加で呼ぶ（REQ-051）
- 変化がなかったフィールドは `changes` に含めない 🟡

**主なエラー**:

| `outcome` | `code` | 条件 |
|---|---|---|
| `not_found` | `ITEM_NOT_FOUND` | 存在しない `item_id`。**更新は行われない**（REQ-110） |
| `error` | `MCP_INVALID_ARGUMENT` | `consumed_date` が日付形式でない（REQ-052） |

---

## 7. organize_item 🔵

**信頼性**: 🔵 *REQ-060 / REQ-061 / REQ-111 / REQ-113・ヒアリング2026-08-07 Q2 より*

**関連要件**: REQ-060, REQ-061, REQ-111, REQ-113, NFR-004, EDGE-003

**ツール説明（AI向け）**:
> タグ・カテゴリ・マイリストを**名前で**指定して作品に付与する。存在しない名前は既定では作成されず「候補なし」として返る。新規作成したい場合のみ `create_if_missing: true` を指定すること。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `item_id` | uuid | ✔ | |
| `tags` | string[] | — | タグ名 |
| `categories` | string[] | — | カテゴリ名 |
| `mylists` | string[] | — | マイリスト名 |
| `create_if_missing` | bool | — | **既定 false**（REQ-111・PRD §15.1） |

**レスポンス**:

```json
{
  "outcome": "success",
  "item_id": "b6b6f9a0-...",
  "title": "作品A",
  "tags": [
    { "result": "applied", "target_id": "...", "target_name": "積読", "created_new": true },
    { "result": "already_applied", "target_id": "...", "target_name": "SF" }
  ],
  "categories": [],
  "mylists": [
    { "result": "not_resolved", "requested_name": "夏休み", "available_names": ["2026年視聴", "積読リスト"] }
  ],
  "error": null
}
```

- `created_new` により新規作成分と既存分を区別する（REQ-061）
- `already_applied` により冪等性を明示する（REQ-113）
- 付与前に現状を取得して差分適用する（設計決定 D-03）

---

## 8. relate_items 🔵

**信頼性**: 🔵 *REQ-070 ~ REQ-072・PRD §15.1「関係種別」より*

**関連要件**: REQ-070, REQ-071, REQ-072, REQ-113, EDGE-005

> ⚠️ **前提**: MediaVault-api の `relation_type` ENUM 拡張（[prep.md](../spec/prep.md) PREP-01）が完了していること。未完了の場合、`adaptation` / `sequel` / `prequel` / `spinoff` は api 側で拒否される。

**ツール説明（AI向け）**:
> 2つの作品を関係づける。両方の `item_id` を `search_library` で先に特定しておくこと。関係には向きがあり、`item_id` が起点、`related_item_id` が終点になる。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `item_id` | uuid | ✔ | 関係の**起点** |
| `related_item_id` | uuid | ✔ | 関係の**終点** |
| `relation_type` | enum | ✔ | 下表の6種のみ |

**関係種別と向きの意味**（REQ-072）:

| 値 | `item_id`（起点） | `related_item_id`（終点） |
|---|---|---|
| `adaptation` | 原作 | 映像化・翻案作品 |
| `sequel` | 前作 | 続編 |
| `prequel` | 後の作品 | 前日譚 |
| `spinoff` | 本編 | スピンオフ |
| `dlc` | 本編 | DLC・追加コンテンツ |
| `reference` | 引用元 | 引用先 |

**レスポンス**:

```json
{
  "outcome": "success",
  "relation_id": "rel-uuid",
  "relation_type": "adaptation",
  "description": "「小説A」（原作）→「映画A」（映像化）",
  "item": { /* ItemSummary */ },
  "related_item": { /* ItemSummary */ },
  "already_related": false,
  "error": null
}
```

**主なエラー**:

| `outcome` | `code` | 条件 |
|---|---|---|
| `error` | `MCP_INVALID_ARGUMENT` | `item_id` と `related_item_id` が同一（EDGE-005）🟡 |
| `not_found` | `ITEM_NOT_FOUND` | いずれかの Item が存在しない |
| `success` | — | 409 `DUPLICATE_RELATION` → `already_related: true`（REQ-113） |

---

## 9. add_access_link 🔵

**信頼性**: 🔵 *REQ-080 / REQ-081 / REQ-116・ヒアリング（requirements Q4）より*

**関連要件**: REQ-080, REQ-081, REQ-116

**ツール説明（AI向け）**:
> 作品に URL を追加する。配信サービスは `kind: "streaming"`、予告編は `kind: "trailer"`、公式サイト・Jellyfin・Calibre-Web などは `kind: "link"` を使う。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `item_id` | uuid | ✔ | |
| `url` | string | ✔ | http / https のみ（REQ-116） |
| `kind` | enum | ✔ | `link` / `streaming` / `trailer` |
| `platform` | string | — | `kind=streaming` 時。対応5種以外は通常リンクへフォールバック |
| `label` | string | — | 通常リンクのラベル |

**対応プラットフォーム**: `netflix` / `amazon_prime` / `disney_plus` / `dmm_tv` / `apple_tv`

**レスポンス**（フォールバック時）:

```json
{
  "outcome": "success",
  "link_id": "link-uuid",
  "registered_as": "link",
  "fallback_from": "streaming",
  "already_registered": false,
  "error": null
}
```

- 非対応プラットフォーム（U-NEXT、Jellyfin など）は `item_links` へラベル付きで登録し、`fallback_from` で通知する（REQ-081）
- 409 `DUPLICATE_STREAMING_LINK` は `already_registered: true` として成功扱い 🟡

---

## 10. collection_overview 🔵

**信頼性**: 🔵 *REQ-090・ヒアリング（requirements Q3）より*

**関連要件**: REQ-090, REQ-143

> ⚠️ **前提**: `GET /api/v1/collection/overview`（[prep.md](../spec/prep.md) PREP-04）が実装済みであること。

**ツール説明（AI向け）**:
> コレクション全体の統計と最近の動きを取得する。「何を見るか」を相談される前に呼ぶと、候補を絞る条件を立てやすい。

**パラメータ**:

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `recent_limit` | int | — | 1..=50、既定 10 🟡 |

**レスポンス**:

```json
{
  "outcome": "success",
  "total_items": 181,
  "by_media_type": [
    { "key": "manga", "count": 87 },
    { "key": "anime", "count": 42 }
  ],
  "by_status": [
    { "key": "not_started", "count": 90 },
    { "key": "in_progress", "count": 21 },
    { "key": "completed", "count": 70 }
  ],
  "favorite_count": 34,
  "recently_added": [ /* ItemSummary[] */ ],
  "recently_updated": [ /* ItemSummary[] */ ],
  "error": null
}
```

---

## 11. health 🔵

**信頼性**: 🔵 *REQ-100・ヒアリング（requirements Q8）より*

**関連要件**: REQ-100, REQ-120, EDGE-001

**ツール説明（AI向け）**:
> MediaVault-api への到達性を確認する。他のツールが失敗したとき、原因が接続なのかデータなのかを切り分けるために使う。

**パラメータ**: なし

**レスポンス（api 到達時）**:

```json
{
  "outcome": "success",
  "mcp_version": "0.1.0",
  "api": { "reachable": true, "latency_ms": 12, "error": null }
}
```

**レスポンス（api 停止時）**:

```json
{
  "outcome": "success",
  "mcp_version": "0.1.0",
  "api": {
    "reachable": false,
    "latency_ms": null,
    "error": { "code": "MCP_API_UNREACHABLE", "message": "MediaVault-api へ接続できません", "retriable": true }
  }
}
```

- api が停止していても**ツール自体は成功**する。診断ツールとして機能させるため（EDGE-001）
- 内部APIキーの有効性、外部APIキーの設定状況は**確認しない**（ヒアリングで範囲を限定）

---

## MVP で公開しないツール 🔵

**信頼性**: 🔵 *REQ-141・PRD §10 / §13 / §15.1 より*

以下は `tools/list` に**含めない**:

- Item 削除、ファイル削除、関連解除、タグ・カテゴリ・マイリストの削除
- 物理ファイルの移動・リネーム
- APIキーの参照・更新
- 一括タグ付与、一括ステータス更新、重複統合

## 第2段階で追加するツール 🔵

**信頼性**: 🔵 *PRD §7.2・REQ-900 ~ REQ-902 より*

| ツール | 対応US | 前提 |
|---|---|---|
| `get_item_text` | US-10 | `GET /items/{id}/text` の新設 + 全文抽出の実装 |
| `enqueue_job` / `get_job` / `list_jobs` / `cancel_job` | US-11 | 汎用 jobs API の実装 |

stdio トランスポートも第2段階。Tool層・Service層は変更せず、`main.rs` / `server.rs` のみで対応する（REQ-902）。

---

## 関連文書

- **アーキテクチャ**: [architecture.md](architecture.md)
- **データフロー**: [dataflow.md](dataflow.md)
- **型定義**: [interfaces.rs](interfaces.rs)
- **設計ヒアリング**: [design-interview.md](design-interview.md)
- **要件定義**: [requirements.md](../spec/requirements.md)
- **受け入れ基準**: [acceptance-criteria.md](../spec/acceptance-criteria.md)
- **MediaVault-api 仕様**: [docs/backend/mediavault-api/](../../mediavault-api/)

## 信頼性レベルサマリー

- 🔵 青信号: 47件 (85%)
- 🟡 黄信号: 8件 (15%)
- 🔴 赤信号: 0件 (0%)

**品質評価**: ✅ **高品質**
