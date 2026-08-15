# mediavault-mcp API→MCPツール対応表

**作成日**: 2026-08-13
**関連設計**: [mcp-tools.md](mcp-tools.md) / [architecture.md](architecture.md) / [mastra-integration.md](mastra-integration.md) / [interfaces.rs](interfaces.rs)
**関連要件定義**: [requirements.md](../spec/requirements.md)
**関連PRD**: [MediaVault-mcp PRD](../PRD.md) §7・§8・§13

> **注**: 本書は MediaVault-api の**全エンドポイントを網羅**し、それぞれを MCP ツールへ露出するか否かと、その根拠を確定させる。ツールごとの入出力スキーマは [mcp-tools.md](mcp-tools.md) が正典であり、本書は重複させない。MediaVault-api の REST 仕様は [docs/backend/mediavault-api/](../../mediavault-api/) を参照。

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・設計文書・既存API仕様・ユーザヒアリングを参考にした確実な定義
- 🟡 **黄信号**: それらから妥当な推測による定義
- 🔴 **赤信号**: それらにない推測による定義

---

## 1. 露出方針 🔵

**信頼性**: 🔵 *PRD §6 原則1 / §7 / §13・REQ-141・[mcp-tools.md](mcp-tools.md)「ツールのメタデータ区分」より*

MediaVault-mcp は REST API をそのまま複製せず、利用者の目的に沿った操作へまとめる（PRD §6 原則1）。したがってエンドポイントとツールは 1:1 に対応しない。全エンドポイントを次の4区分に分類する。

| 区分 | 意味 | 判断根拠 |
|---|---|---|
| **E** (Exposed) | MCPツールとして直接的な目的を担う | PRD §7.1 MVP / §7.2 第2段階 |
| **I** (Internal) | ツールから内部的に呼ぶが、単独ツールにはしない（ID解決・集約の部品） | PRD §6 原則1「目的単位のツールにする」 |
| **N** (Not exposed) | 意図的に露出しない。`tools/list` に現れず、MCP からは到達できない | PRD §13・REQ-141 |
| **F** (Future) | 将来候補。MVP・第2段階では出さない | PRD §7.3 |

**区分の集計**（全76本 = 既存75本 + 新設 `GET /items/{id}/text` 1本）:

| 区分 | 件数 | 割合 |
|---|---|---|
| E | 19 | 25% |
| I | 21 | 28% |
| N | 35 | 46% |
| F | 1 | 1% |

**N が全体の46%を占める**のは設計上の意図である。MediaVault-api は単一ユーザー・セルフホスト前提で**公開APIに認証を持たない**（[index.md](../../mediavault-api/index.md)）ため、削除・ファイル操作・シークレット更新まで含めて全操作が無防備に並んでいる。MCP はここから「AIに委ねてよい操作」だけを選び取る層として機能する。

---

## 2. エンドポイント対応表 🔵

**信頼性**: 🔵 *[docs/backend/mediavault-api/](../../mediavault-api/) 各ファイルの見出しを機械的に列挙し、PRD §7・§13 と照合して分類*

> **注**: 分類の網羅性は `mediavault-api/*.md` の `## <METHOD> <PATH>` 見出しを正とする。[index.md](../../mediavault-api/index.md) のエンドポイント一覧は11本欠落しており、根拠には使わない。

### 2.1 health / collection 🔵

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| GET | `/health` | E | `health` | REQ-100。api の依存サービス状態を透過する |
| GET | `/collection/overview` | E | `collection_overview` | REQ-090。media_type別・status別件数と最近追加/更新を返す |

### 2.2 items 🔵

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| GET | `/items` | E | `search_library` | REQ-010〜014。`title` は本題・原題・別名を横断ORで部分一致（[items.md](../../mediavault-api/items.md)）。`include_total=true` を指定して総件数を返す |
| GET | `/items/counts-by-media-type` | I | `collection_overview` | 件数の一部。単独ツールにする価値がない |
| POST | `/items` | E | `create_item` | REQ-040, REQ-041 |
| GET | `/items/search` | E | `search_external_catalog` | REQ-030〜032。**ローカル検索と明確に分離する**（PRD §6 原則2） |
| POST | `/items/import` | E | `import_external_item` | REQ-033, REQ-112 |
| GET | `/items/{id}` | I | `get_item_context` | 集約の中核。`detail` / `tags` / `categories` / `streaming_links` / `images` / `calibre_links` を含む |
| PATCH | `/items/{id}` | E | `update_consumption` | REQ-050〜052。`rating` / `is_favorite` / `consumed_date` を担当 |
| DELETE | `/items/{id}` | **N** | — | REQ-141・PRD §13 |
| PATCH | `/items/{id}/status` | E | `update_consumption` | REQ-050。`status` を担当。`PATCH /items/{id}` と1ツールに統合する |

### 2.3 tags / categories / mylists 🔵

タグ・カテゴリ・マイリストは api 上いずれも ID 指定であり、**MCP が名前から解決する**（PRD §8）。この解決処理そのものはツールにせず、`organize_item` と `search_library` の内部に閉じる。

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| GET | `/tags` | I | `organize_item` / `search_library` | 名前→ID解決。解決できない名前は `not_resolved` として候補一覧を返す（REQ-111） |
| POST | `/tags` | I | `organize_item` | `create_if_missing: true` のときのみ呼ぶ（REQ-060） |
| DELETE | `/tags/{id}` | **N** | — | REQ-141 |
| POST | `/items/{id}/tags/{tag_id}` | I | `organize_item` | 付与本体 |
| DELETE | `/items/{id}/tags/{tag_id}` | **N** | — | REQ-141 |
| GET | `/categories` | I | `organize_item` / `search_library` | 同上 |
| POST | `/categories` | I | `organize_item` | `create_if_missing` 指定時のみ |
| DELETE | `/categories/{id}` | **N** | — | REQ-141 |
| POST | `/items/{id}/categories/{category_id}` | I | `organize_item` | 付与本体 |
| DELETE | `/items/{id}/categories/{category_id}` | **N** | — | REQ-141 |
| GET | `/mylists` | I | `organize_item` | 名前→ID解決 |
| POST | `/mylists` | I | `organize_item` | `create_if_missing` 指定時のみ |
| GET | `/items/{id}/mylists` | I | `get_item_context` | `mylists` セクション |
| POST | `/mylists/{id}/items` | I | `organize_item` | 追加本体 |
| DELETE | `/mylists/{id}/items/{item_id}` | **N** | — | REQ-141 |

### 2.4 item-relations 🔵

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| GET | `/items/{id}/relations` | I | `get_item_context` | `relations` セクション |
| POST | `/item-relations` | E | `relate_items` | REQ-070〜072。`relation_type` は adaptation / sequel / prequel / spinoff / dlc / reference の6値が**すでに実装済み**（[item-relations.md](../../mediavault-api/item-relations.md)）。PRD §8 の「値拡張が必要」は解消済み |
| DELETE | `/item-relations/{id}` | **N** | — | REQ-141。「関連解除」は明示的に非公開（[mcp-tools.md](mcp-tools.md)） |

### 2.5 item-groups / item-episodes 🔵

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| POST | `/items/{id}/groups` | **N** | — | season / volume / chapter の構造投入は `import_external_item` と worker の責務。AI からの手動投入に対応する US が存在しない |
| GET | `/items/{id}/groups` | I | `get_item_context` | `groups` セクション。加えて**シリーズ名解決の一次情報**（下記 §4 D-07） |
| POST | `/groups/{group_id}/episodes` | **N** | — | 同上 |
| GET | `/groups/{group_id}/episodes` | **F** | （将来 `list_episodes`） | **`get_item_context` には含めない**。グループ件数ぶんの追加呼び出しになり（N+1）、シーズン×話数でレスポンスが線形に膨らむため NFR-002 / NFR-005 に抵触する。エピソード単位の情報が必要になった時点で専用の読み取りツールを設計する |

### 2.6 staff / cast 🔵

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| POST | `/staff` | **N** | — | 人物マスタの登録。US-01〜US-12 に該当なし |
| GET | `/items/{id}/staff` | I | `get_item_context` | `staff` セクション |
| POST | `/items/{id}/staff` | **N** | — | 同上 |
| DELETE | `/items/{id}/staff/{item_staff_id}` | **N** | — | REQ-141 |
| POST | `/cast` | **N** | — | US に該当なし |
| GET | `/items/{id}/cast` | I | `get_item_context` | `cast` セクション |
| POST | `/items/{id}/cast` | **N** | — | 同上 |
| DELETE | `/items/{id}/cast/{item_cast_id}` | **N** | — | REQ-141 |

### 2.7 item-files 🔵

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| GET | `/items/{id}/files` | I | `get_item_context` / `get_item_text` | `files` セクション、および `get_item_text` の `file_id` 解決 |
| POST | `/items/{id}/files` | **N** | — | PRD §13「物理ファイル変更ツールを公開しない」 |
| POST | `/items/{id}/files/upload` | **N** | — | multipart アップロード。同上 |
| PATCH | `/items/{id}/files/{file_id}/calibre-link` | **N** | — | 外部システム連携IDの書き換え。同上 |
| DELETE | `/items/{id}/files/{file_id}` | **N** | — | REQ-141 |
| GET | `/items/{id}/text` | E | `get_item_text` | **新設**（第2段階）。仕様は [item-text.md](../../mediavault-api/item-text.md) |

### 2.8 item-links / streaming-links / trailers / images 🔵

3種のリンクは `add_access_link` 1ツールに統合し、`link_kind` で振り分ける（REQ-080, REQ-081）。

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| GET | `/items/{id}/links` | I | `get_item_context` | `links` セクション |
| POST | `/items/{id}/links` | E | `add_access_link` | `link_kind: "link"` |
| DELETE | `/items/{id}/links/{link_id}` | **N** | — | REQ-141 |
| GET | `/items/{id}/streaming-links` | I | （呼ばない） | **`GET /items/{id}` の `streaming_links` に含まれる**ため、`get_item_context` では別途呼ばない。呼び出し回数削減（NFR-005） |
| POST | `/items/{id}/streaming-links` | E | `add_access_link` | `link_kind: "streaming"` |
| DELETE | `/items/{id}/streaming-links/{link_id}` | **N** | — | REQ-141 |
| GET | `/items/{id}/trailers` | I | `get_item_context` | `trailers` セクション |
| POST | `/items/{id}/trailers` | E | `add_access_link` | `link_kind: "trailer"` |
| DELETE | `/items/{id}/trailers/{trailer_id}` | **N** | — | REQ-141 |
| GET | `/items/{id}/images` | I | （呼ばない） | **`GET /items/{id}` の `images` に含まれる**ため別途呼ばない |
| POST | `/items/{id}/images` | **N** | — | 画像管理は UI の責務。読み取りは `get_item_context` で提供済み |
| DELETE | `/items/{id}/images/{image_id}` | **N** | — | REQ-141 |

### 2.9 citations 🔵

第2段階で `list_citations` / `add_citation` を新設する。詳細は §3。

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| GET | `/items/{id}/citations` | E | `list_citations` ＋ `get_item_context`（件数のみ） | §3 D-11 / D-12 |
| POST | `/items/{id}/citations` | E | `add_citation` | §3 D-11 |
| PATCH | `/citations/{citation_id}` | **N** | — | 既存の引用文の上書きは実質的に破壊的（§3 D-11） |
| DELETE | `/citations/{citation_id}` | **N** | — | REQ-141 |

### 2.10 text / extraction 🔵

第2段階（REQ-900・REQ-901・PRD §7.2）。全文取得と抽出操作は実装済みであり、すべて公開APIを使う。

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| GET | `/api/v1/items/{id}/text` | E | `get_item_text` | REQ-900。抽出済み本文を0起点のチャンク単位で取得する |
| POST | `/api/v1/items/{id}/files/{file_id}/extraction` | E | `request_extraction` | REQ-901。未完了の抽出はDB制約により冪等に1件へ収束する |
| GET | `/api/v1/items/{id}/files/{file_id}/extraction` | E | `get_extraction_status` | REQ-901。状態・進捗・試行回数を取得する |
| POST | `/api/v1/items/{id}/files/{file_id}/extraction/cancel` | E | `cancel_extraction` | REQ-901。**キャンセルは削除ではなく状態遷移**のため N にしない |

### 2.11 settings / import 🔵

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| PUT | `/settings/api-keys/{provider}` | **N** | — | シークレットの参照・更新（[mcp-tools.md](mcp-tools.md)「MVPで公開しないツール」）。加えて**レスポンスボディに `api_key` が平文で含まれる**（[settings.md](../../mediavault-api/settings.md)）ため、露出すれば MCP クライアントへ鍵が漏れる。NFR-103 に直接抵触する |
| POST | `/import/booklog` | **N** | — | multipart CSV の一括登録。件数上限がなく、AI 経由の実行は「曖昧な対象へ書き込まない」（PRD §6 原則3）と両立しない |
| POST | `/import/steam` | **N** | — | Steam ライブラリ全件の一括登録。同上。PRD §7.3「一括操作」に相当し将来検討 |

### 2.12 internal API 🔵

| Method | Path | 区分 | 対応MCPツール | 根拠・備考 |
|---|---|---|---|---|
| POST | `/internal/items` | **N** | — | 公開API `POST /items` と同一の目的。二重露出を避ける |
| GET | `/internal/items/search` | **N** | — | 公開API `GET /items` と同一 |
| PATCH | `/internal/items/{id}` | **N** | — | 公開API `PATCH /items/{id}` と同一 |
| POST | `/internal/items/{id}/groups` | **N** | — | 公開API版と同じく N（§2.5） |
| POST | `/internal/groups/{group_id}/episodes` | **N** | — | 同上 |
| POST | `/internal/items/{id}/files` | **N** | — | 公開API版と同じく N（§2.7） |

> **MCP は内部APIを使わない。** 抽出の依頼・状態確認・キャンセルを含め、MCP が呼ぶのは公開APIのみである。worker 専用の内部APIと `INTERNAL_API_KEY` は MCP の責務外である。

---

## 3. citations ツール（第2段階）🟡

**信頼性**: 🟡 *[citations.md](../../mediavault-api/citations.md) の仕様と、ユーザヒアリング2026-08-13（citations 公開の決定）より。対応する US・REQ は本設計と同時に新設するため 🟡*

**関連要件**: US-12, REQ-903, REQ-904

`Citation` は `quote_text` に加えて `locator_type`（page / timestamp / location / chapter / none）と対応する位置フィールドを持ち、**出典位置を型付きで保持できる**。これは intrahub-mastra の `ResearchResult.claims[].sourceRefs` および Knowledge Note frontmatter の `sources`（[intrahub-mastra PRD](../../../../../intrahub-mastra/docs/PRD.md) §13.1・§14）と直接対応する構造である。

導入は**第2段階**とする。MVP の11ツールは [tasks/overview.md](../tasks/overview.md) の TASK-0001〜0026 で計画済みであり、そこへ割り込ませない。

### D-11: 露出範囲は「追記のみ、既存を壊さない」🟡

| 操作 | 区分 | ツール | annotation |
|---|---|---|---|
| `GET /items/{id}/citations` | E | `list_citations` | `readOnlyHint: true` |
| `POST /items/{id}/citations` | E | `add_citation` | `readOnlyHint: false`, `destructiveHint: false` |
| `PATCH /citations/{citation_id}` | **N** | — | — |
| `DELETE /citations/{citation_id}` | **N** | — | — |

`PATCH` を N とする理由: **`quote_text` はユーザーが書いた本文**であり、上書きは実質的に破壊的である。`update_consumption` が扱う `status` / `rating` / `consumed_date` のような機械的フィールドとは性質が異なり、「破壊的ツールを公開しない」（REQ-141・PRD §15.1）の趣旨に照らして非公開が妥当と判断した。引用の訂正は UI の責務とする。実運用で訂正需要が確認された場合は `update_citation` として再検討する（§5）。

### list_citations 🟡

**ツール説明（AI向け）**:
> ある作品に記録済みの引用を一覧する。作品の内容について語る前に、利用者自身が残した引用があるか確認すること。

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `item_id` | uuid | ✔ | `search_library` で解決した ID |
| `limit` | number | | 1..=50、既定20 |
| `cursor` | string | | 前回レスポンスの `next_cursor` |

**API 側にページネーションが存在しない**（`GET /items/{id}/citations` は作成日時昇順で全件返す）ため、**MCP 側で切り出す**。`next_cursor` はオフセットを不透明化した文字列とし、既存のページネーション規約（[mcp-tools.md](mcp-tools.md)「ページネーション」）に揃える。全件を取得しているため `total_count` は常に返せる。

```json
{
  "outcome": "success",
  "item_id": "b6b6f9a0-...",
  "total_count": 12,
  "citations": [
    {
      "citation_id": "c1b6f9a0-...",
      "quote_text": "人は見たいものしか見ようとしない。",
      "note": "第3章の議論のまとめとして引用",
      "locator_type": "page",
      "page_number": 128,
      "timestamp_seconds": null,
      "location_number": null,
      "chapter": null,
      "created_at": "2026-07-01T12:00:00"
    }
  ],
  "next_cursor": null,
  "error": null
}
```

位置情報は api の値を**そのまま透過**し、「p.128」のような表示文字列へ整形しない（REQ-146 の趣旨）。

**主なエラー**:

| `outcome` | `code` | 条件 |
|---|---|---|
| `not_found` | `ITEM_NOT_FOUND` | `item_id` が存在しない |
| `error` | `MCP_INVALID_ARGUMENT` | `limit` が 1..=50 の範囲外 |

### add_citation 🟡

**ツール説明（AI向け）**:
> 作品から引用を記録する。**同じ引用を繰り返し登録しないこと**（重複は検出されない）。引用元の位置が分かる場合は `locator_type` と対応する位置を必ず指定すること。

| 名前 | 型 | 必須 | 説明 |
|---|---|---|---|
| `item_id` | uuid | ✔ | 対象 Item |
| `quote_text` | string | ✔ | 引用本文 |
| `locator_type` | enum | ✔ | `page` / `timestamp` / `location` / `chapter` / `none` |
| `note` | string | | 引用に対する所感・文脈 |
| `page_number` | number | | `locator_type: "page"` のとき必須 |
| `timestamp_seconds` | number | | `locator_type: "timestamp"` のとき必須 |
| `location_number` | number | | `locator_type: "location"` のとき必須 |
| `chapter` | string | | `locator_type: "chapter"` のとき必須 |

**MCP 側で `locator_type` と位置フィールドの整合を検証する**（REQ-904）。api は「対応する値を指定することを推奨するが、必須バリデーションはしない（未指定は null のまま保存）」（[citations.md](../../mediavault-api/citations.md)）ため、`locator_type: "page"` なのに `page_number` が無い引用が黙って保存される。人間が UI から入力する分には許容できても、**AI からの入力を受ける MCP でこれを緩くすると出典不明の引用が蓄積し、`sourceRefs` の追跡可能性が損なわれる**。不整合は api を呼ばずに `MCP_INVALID_ARGUMENT` で弾く。`locator_type: "none"` のときのみ、全位置フィールドが未指定であることを要求する。

**冪等性 — 既存決定 D-03 の明示的な例外**: [design-interview.md](design-interview.md) の D-03 は「冪等性は事前取得 + 差分適用で担保する」と定めるが、**citations にはこの方式を適用できない**。`organize_item` が扱うタグ・カテゴリは名前が一意キーになるため「既に付与済みか」を事前取得で判定できるのに対し、引用には一意キーがなく、api にも重複検出がない。同一の引用を2回記録することが利用者の意図である場合すら区別できない。

したがって `add_citation` は**冪等にできない操作**として扱い、D-03 と PRD §6 原則6「冪等にできる操作は冪等に」の適用外であることをここに記録する。ツール説明で重複登録を明示的に戒め、判断を呼び出し元へ委ねる。

**D-02 との整合**: `add_citation` は書き込み系ツールだが、`item_id` を UUID でのみ受け取り名前解決を行わないため、D-02「書き込み系ツールは UUID のみ受け取る」を満たす。

**主なエラー**:

| `outcome` | `code` | 条件 |
|---|---|---|
| `not_found` | `ITEM_NOT_FOUND` | `item_id` が存在しない |
| `error` | `MCP_INVALID_ARGUMENT` | `locator_type` と位置フィールドが不整合 |
| `error` | `VALIDATION_ERROR` | api 側のバリデーション失敗（`quote_text` 空など） |

### D-12: `get_item_context` の `citations` は件数のみ 🟡

`quote_text` は長さ・件数とも上限がないため、他セクションと同じ `items` 形式にすると `get_item_context` のレスポンスサイズが Item 依存で予測不能になり、NFR-002 に抵触する。件数のみを返し、本文は `list_citations` へ誘導する。

```json
"citations": { "state": "loaded", "count": 12 }
```

`state` は既存の3状態規約（`loaded` / `empty` / `failed`）に従う。ただし**`items` を持たない唯一のセクション**になるため、[interfaces.rs](interfaces.rs) では `SectionView<T>` と別型（`CountSectionView`）として定義する。

### mastra との関係 🔵

**信頼性**: 🔵 *[intrahub-mastra requirements.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/requirements.md) REQ-401 / REQ-402 より*

mastra 側 requirements は citations を要求していない。REQ-401 の許可リストは `search_library` / `get_item_context` / `get_item_text` の3ツールに限定されるため、**`mediaResearchAgent` には `list_citations` も `add_citation` も渡さない**。`list_citations` は `readOnlyHint: true` であり REQ-402（書き込みツール全面禁止）にも抵触しないため、mastra 側が許可リストを拡張すれば追加できる。この余地を記録するに留め、MediaVault-mcp 側からは要求しない。

---

## 4. mastra 連携で必要な追加保証 🔵

**信頼性**: 🔵 *[intrahub-mastra requirements.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/requirements.md)（2026-08-13 更新）より*

[mastra-integration.md](mastra-integration.md) は 2026-08-11 作成であり、mastra 側の 2026-08-13 ヒアリング結果を反映していない。本節で差分を確定させる。

### D-07: シリーズ名の解決規則（mastra REQ-016a）🔵

mastra は Knowledge Note の配置先 `topic` を「MediaVault Item の作品名（シリーズがある場合はシリーズ名）」から決定し、**LLM の推測による `topic` 生成を禁止**する（REQ-016a）。したがって MediaVault 側が**決定的な規則**でシリーズ名を返す必要がある。

`get_item_context` に `series` セクションを追加し、次の順序で解決する。

1. `GET /items/{id}/groups` の `parent_item_id` が非 null → その ID で `GET /items/{parent_item_id}` を引き、`title` をシリーズ名とする
2. `parent_item_id` がすべて null → シリーズを持たないと判断し `state: "empty"`。**`group_name`（"Season 1" 等）をシリーズ名に流用しない**
3. `relations` に `sequel` / `prequel` が存在しても**シリーズ名の推測には使わない**（推測禁止の要求に反するため）
4. 解決不能・取得失敗 → `state: "failed"`

```json
"series": { "state": "loaded", "item_id": "a1b2...", "title": "作品Aシリーズ" }
```

`state: "empty"` を受け取った mastra は EDGE-006 により既定の未分類階層へ配置し `warnings` に記録する。**MediaVault 側は「分からない」を正確に返すことが責務**であり、埋め合わせをしない。

**既存決定 D-05 への影響**: [design-interview.md](design-interview.md) の D-05 は「`get_item_context` は `futures::join!` で並列合成する」と定めるが、`series` は **`GET /items/{id}/groups` の結果を見てから親 Item を引く**ため、単一ラウンドの並列合成に収まらない。実装は2段構成になる。

1. 第1ラウンド: 既存の並列取得（`GET /items/{id}` と各セクション）に `GET /items/{id}/citations` を加えて `join!`
2. 第2ラウンド: `groups` の `parent_item_id` が非 null のときのみ `GET /items/{parent_item_id}` を1回

第2ラウンドは条件付きで、多くの Item では発生しない。`try_join!` を使わない方針（D-05）は維持し、親 Item の取得失敗は `series` セクションを `failed` にするだけで他セクションに影響させない。

**`citations` の件数取得は第1ラウンドに含める**。`GET /items/{id}/citations` は他セクションと同じく item_id だけで引けるため、並列化を妨げない。

### D-08: チャンク連番インデックスの吸収（mastra REQ-006）🔵

mastra は `sourceRefs` を `(itemId, fileId, chunkIndex)` の連番インデックス形式で統一し、**PDFページ・EPUB章・固定文字数チャンクの差は MediaVault 側が吸収する**ことを前提にしている（REQ-006）。

- `get_item_text` は media 種別に依らず **0起点の連番 `chunk.index`** のみを構造として返す
- 表示用ラベルは `chunk.label`（`"p.42"` / `"第3章"` 等）として**任意の付属情報**とする。mastra はこれを構造として解釈しない
- **`extraction_version` を新設する**。再抽出でチャンク境界がずれると、保存済み Knowledge Note の `sourceRefs` が**黙って別の箇所を指す**。`extracted_at` の変化だけでは境界がずれたか判別できないため、抽出処理のバージョン識別子を返し、mastra 側が `sourceRefs` の失効を検出できるようにする

api 側仕様は [item-text.md](../../mediavault-api/item-text.md) を参照。

### D-09: `get_item_text` の可視性（mastra TC-004-E02）🔵

mastra の受け入れ基準 TC-004-E02 は「`get_item_text` 非提供を**ツール不在**として検出し、暗黙のフォールバックをしない」ことを求める。

全文抽出が未実装の間、`get_item_text` を **`tools/list` に出さない**（環境変数によるフィーチャーフラグで制御）。公開したうえで常に `not_extracted` を返す方式では、mastra 側が「ツールが無い」と「このファイルは未抽出」を区別できず、`mode: metadata_only` へのフォールバック判断（REQ-101）を誤る。

### D-10: read-only トークンスコープ（第2段階）🔵

mastra 側は現在「MCPトークン自体が書き込み権限を持つ間は、Mastra側のツール選別を必須の防御線とする」（[intrahub-mastra PRD](../../../../../intrahub-mastra/docs/PRD.md) §15）としており、**防御が完全にクライアント側にある**。MediaVault-mcp 側の設定ミスや別クライアントの接続では防げない。

第2段階の要求として、`MCP_AUTH_TOKEN` に加えて `MCP_READONLY_TOKEN` を設け、read-only トークンで接続したセッションでは `tools/list` を `readOnlyHint: true` のツールに限定する。MVP では現行どおり単一トークンで運用する（NFR-101）。

### 失敗クラスの対応（mastra NFR-031）🔵

NFR-031 は7つの失敗を区別して返すことを求めるが、**そのうち MediaVault 側が原因となるのは3つだけ**である。残り4つは Knowledge Vault 側の失敗であり、MediaVault-mcp の責務外。

| NFR-031 の失敗 | 責務 | mcp `outcome` | `code` |
|---|---|---|---|
| MediaVault MCP 接続失敗 | **mcp** | （MCPプロトコル層。ツール結果に現れない） | — |
| MediaVault API 到達失敗 | **mcp** | `error` | `MCP_API_UNREACHABLE` |
| 抽出未実行（`not_extracted`） | **mcp** | `not_extracted` | — |
| Vault 書き込み失敗 | Vault | — | — |
| 競合検出（mastra REQ-105） | Vault | — | — |
| 保護による拒否（mastra REQ-202） | Vault | — | — |
| スキーマ検証失敗 | mastra | — | — |

MediaVault-mcp が担う3クラスはいずれも**互いに区別可能**である。特に「api 到達失敗」と「抽出未実行」は原因も対処も異なる（前者は復旧待ち、後者は抽出ジョブの依頼）ため、`MCP_API_UNREACHABLE` と `not_extracted` で明確に分ける。

加えて、mcp が返しうる MediaVault 側のその他の失敗:

| 状況 | `outcome` | `code` |
|---|---|---|
| Item 不在 | `not_found` | `ITEM_NOT_FOUND` |
| ファイル特定不能 | `ambiguous` | — |
| 引数不正 | `error` | `MCP_INVALID_ARGUMENT` |
| 内部認証失敗 | `error` | `MCP_INTERNAL_AUTH_FAILED` |
| レスポンス解釈失敗 | `error` | `MCP_DECODE_FAILED` |

いずれも既存の `outcome` / `McpErrorCode`（[interfaces.rs](interfaces.rs)）で表現でき、**新規エラーコードの追加は不要**。`not_extracted` のみ `outcome` の値を1つ増やす（D-01「MCPプロトコル上は常に成功し `outcome` で状態を表現する」とは矛盾しない）。

---

## 5. 将来候補（F）🔵

**信頼性**: 🔵 *PRD §7.3 より*

| 対象 | 想定ツール | 根拠 |
|---|---|---|
| `GET /groups/{group_id}/episodes` | `list_episodes` | §2.5。エピソード単位の情報が必要になった時点で設計する |
| `PATCH /citations/{citation_id}` | `update_citation` | §3 D-11。引用文の訂正需要が実運用で確認された場合に再検討 |
| 重複候補検索API（未実装） | 重複統合支援 | PRD §7.3 |
| 一括タグ付与・一括ステータス更新 | — | PRD §7.3 |
| `POST /import/booklog` / `POST /import/steam` | — | §2.11。一括登録として PRD §7.3 の範囲 |

---

## 6. MediaVault-api への残存要求 🔵

**信頼性**: 🔵 *[docs/backend/mediavault-api/](../../mediavault-api/) の現行仕様を確認した結果より*

PRD §8「MediaVault-apiへの要求候補」のうち、**大半はすでに実装済み**である。

| PRD §8 の要求 | 状況 | 確認先 |
|---|---|---|
| 別名・原題を含む検索 | ✅ **実装済み**。`title` は本題・原題・`details.alternative_titles` を横断OR部分一致 | [items.md](../../mediavault-api/items.md) |
| 検索結果の該当件数 | ✅ **実装済み**。`include_total=true` で `pagination.total` | [items.md](../../mediavault-api/items.md) |
| `relation_type` の値拡張 | ✅ **実装済み**。6値すべて対応 | [item-relations.md](../../mediavault-api/item-relations.md) |
| `GET /collection/overview` | ✅ **実装済み** | [collection.md](../../mediavault-api/collection.md) |
| `GET /items/{id}/context` 集約API | ⏸ **保留**。PRD §8 の「初期実装ではMCPが複数API呼び出しでよい」判断を維持する。性能・レスポンスサイズに問題が出た時点で再検討 | — |
| `GET /items/{id}/text` | ✅ **実装済み** | [item-text.md](../../mediavault-api/item-text.md) |
| 抽出リソース3本 | ✅ **実装済み** | [extraction.md](../../mediavault-api/extraction.md) |

**MCP が必要とする api 側機能に真の欠落はない。**

その他、本調査で判明した api 側ドキュメントの不整合（設計への影響はないが記録する）:

- [index.md](../../mediavault-api/index.md) のエンドポイント一覧が**11本欠落**している（`GET /items/counts-by-media-type`, `GET /tags`, `GET /categories`, `GET /mylists`, `GET /items/{id}/mylists`, `GET /items/{id}/relations`, `GET /items/{id}/links`, `GET /items/{id}/trailers`, `GET /items/{id}/staff`, `GET /items/{id}/files`, `DELETE /items/{id}/files/{file_id}`）
- `index.md` の `api_provider` enum（tmdb / igdb / ndl / steam / open_library / ani_list）と [settings.md](../../mediavault-api/settings.md) の受理集合（annict / rakuten を含む）が食い違う。MCP は §2.11 のとおり settings を露出しないため設計に影響しない

---

## 7. タスク化が必要な項目 🟡

**信頼性**: 🟡 *[tasks/overview.md](../tasks/overview.md) の現行計画との差分から*

本書で確定した設計のうち、既存タスク（TASK-0001〜0026）でカバーされないもの。

> **2026-08-13 実装状況**: 下記のうち **D-07 / D-12 / `list_citations` / `add_citation` / D-10 は実装済み**（`backend/mediavault-mcp`）。公開ツールは11個から**13個**になった。残りは MediaVault-api 側の未実装機能に依存する。

**実装済み**:

| 項目 | 実装箇所 |
|---|---|
| D-07 `series` セクション | `src/services/context.rs` の `resolve_series`。`parent_item_id` から親 Item を1回だけ引く第2ラウンド |
| D-12 `citations` 件数セクション | `src/result/mod.rs` の `CountSection`。第1ラウンドの `join!` に含める |
| `list_citations` / `add_citation` | `src/services/citations.rs` / `src/tools/citations.rs` |
| D-10 read-only トークン | `MCP_READONLY_TOKEN`。`src/auth.rs` の `TokenScope` と `src/server.rs` の `list_tools` / `call_tool` |

**未実装**:

| 項目 | 前提 |
|---|---|
| stdio トランスポート | REQ-902（api 側には依存しない） |

---

## 8. mastra 側へ伝達すべき事項 🔵

**信頼性**: 🔵 *[intrahub-mastra prep.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/prep.md) の未チェック項目より*

`intrahub-mastra` 側の `prep.md` に「`get_item_text` が連番インデックスを返す前提で MediaVault 側と合意する」が未チェックで残っている。本書 **D-08 がその回答**である。加えて次を伝達する（`intrahub-mastra` リポジトリの文書更新は当該リポジトリの管轄であり、本書では行わない）。

| 事項 | 内容 |
|---|---|
| 連番インデックス（REQ-006） | D-08 で合意。`chunk.index` は0起点連番、ページ・章は `chunk.label` の任意情報 |
| `extraction_version` | MediaVault 側から**新規提案**。再抽出による `sourceRefs` 失効を検出するために mastra 側でも保持を検討されたい |
| シリーズ名（REQ-016a） | D-07 で `get_item_context` の `series` として提供。解決不能時は `state: "empty"` を返すため、EDGE-006 の未分類階層で受けること |
| `get_item_text` の可視性（TC-004-E02） | D-09。抽出機能の実装完了に伴い `tools/list` へ公開済み |
| `list_citations` | 第2段階で MediaVault-mcp に実装するが、REQ-401 の許可リストが3ツール限定のため渡さない。必要なら mastra 側で許可リストを拡張されたい |

---

## 関連文書

- **MCPツール仕様**: [mcp-tools.md](mcp-tools.md) — 各ツールの入出力スキーマの正典
- **mastra連携**: [mastra-integration.md](mastra-integration.md)
- **アーキテクチャ**: [architecture.md](architecture.md)
- **型定義**: [interfaces.rs](interfaces.rs)
- **要件定義**: [requirements.md](../spec/requirements.md)
- **PRD**: [MediaVault-mcp PRD](../PRD.md)
- **MediaVault-api 仕様**: [docs/backend/mediavault-api/](../../mediavault-api/)
- **全文取得API**: [item-text.md](../../mediavault-api/item-text.md)

## 信頼性レベルサマリー

- 🔵 青信号: 21件 (81%)
- 🟡 黄信号: 5件 (19%)
- 🔴 赤信号: 0件 (0%)

🟡 はいずれも本書と同時に新設する citations 関連（US-12 / REQ-903 / REQ-904）と、既存タスク計画との差分に由来する。要件定義への反映後に 🔵 へ昇格する。

**品質評価**: ✅ **高品質**
