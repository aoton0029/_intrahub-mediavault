# mediavault-mcp × intrahub-mastra 連携エンドポイント設計

**作成日**: 2026-08-11
**最終更新**: 2026-08-13（intrahub-mastra 側 2026-08-13 ヒアリング結果の反映）
**関連設計**: [architecture.md](architecture.md) / [mcp-tools.md](mcp-tools.md) / [api-tool-mapping.md](api-tool-mapping.md) / [interfaces.rs](interfaces.rs)
**関連PRD**: [MediaVault-mcp PRD](../PRD.md) §7.2・§8 / [intrahub-mastra PRD](../../../../../intrahub-mastra/docs/PRD.md)
**関連要件（mastra側）**: [requirements.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/requirements.md) / [acceptance-criteria.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/acceptance-criteria.md)

> **注**: 本書は `intrahub-mastra`（Knowledge Vault生成Agent群）が MediaVault-mcp をMCPクライアントとして利用するために必要な、第2段階ツール `get_item_text` の詳細設計と、既存MVPツールの利用範囲を定義する。`mcp-tools.md` §「第2段階で追加するツール」を具体化するものであり、MVPツール（`search_library`, `get_item_context` 等）の仕様はそのまま流用する。エンドポイント単位の露出可否は [api-tool-mapping.md](api-tool-mapping.md) を参照。

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・設計文書・既存API仕様・PRDを参考にした確実な定義
- 🟡 **黄信号**: それらから妥当な推測による定義
- 🔴 **赤信号**: それらにない推測による定義

---

## 1. 前提: intrahub-mastraが必要とする操作 🔵

**信頼性**: 🔵 *intrahub-mastra PRD §9・§11・§13・§15 より*

`mediaResearchAgent`（Read Only）が `generateItemKnowledgeWorkflow` の中で呼び出すツールは以下の3つに限定される。

| ステップ | ツール | 用途 |
|---|---|---|
| 1 | `search_library` | トピック・作品名から対象Itemを解決する（US-01相当） |
| 2 | `get_item_context` | Item本体・関連作品・スタッフ・ファイル一覧などを取得する |
| 3 | `get_item_text` | ファイルの抽出済み全文を取得し、`ResearchResult` 生成の材料にする（**本書で新設**） |

書き込み系ツール（`create_item` / `update_consumption` / `organize_item` / `relate_items` / `add_access_link`）は `mediaResearchAgent` には一切公開しない。これはMediaVault-mcp側の権限制御ではなく、Mastra側でAgentに渡すツール集合を絞ることで実現する（intrahub-mastra PRD §8原則6・§15）。MediaVault-mcp側は将来的にトークンのスコープ分離（read-onlyトークン）を検討可能だが、MVPでは同一トークンで運用する（§4.1）。

mastra 側が課している禁止事項は次の3点であり、MediaVault-mcp はこれらと矛盾しない範囲で機能を提供する。

| mastra 側の要求 | 内容 | MediaVault-mcp への含意 |
|---|---|---|
| REQ-401 | `mediaResearchAgent` に渡すのは Read Only ツール（`search_library` / `get_item_context` / `get_item_text`）のみ | 上記3ツールが `readOnlyHint: true` であり続けること |
| REQ-402 | `create_item` / `update_consumption` / `organize_item` / `relate_items` / `add_access_link` を**いかなる Agent にも**渡さない | 書き込みツールを read-only 相当に見せかけない。annotation を正確に付ける |
| REQ-410 | MediaVault 内の視聴・読了状況、評価、タグ、関連、Item を知識生成の副作用として変更しない | 読み取りツールが副作用を持たないこと（`get_item_context` は参照のみで、閲覧履歴等を記録しない） |

## 2. get_item_text 🔵

**信頼性**: 🔵 *PRD §7.2 US-10・§8 `GET /api/v1/items/{id}/text` より*

### 目的

Itemに紐づくファイルの抽出済み全文を、ページまたはチャンク単位で取得する。MCP自身は要約・embeddingを生成しない（原則7）。

### 前提条件

- MediaVault-apiに `GET /api/v1/items/{item_id}/text` を新設する。**REST仕様は [item-text.md](../../mediavault-api/item-text.md) に確定済み**。
- 全文抽出処理（PDF・画像からのテキスト抽出）は抽出リソース（[extraction.md](../../mediavault-api/extraction.md)）と Extractor worker が担う。抽出未実行のファイルは `not_extracted` を返す。

### D-08: チャンクは0起点の連番のみ 🔵

**信頼性**: 🔵 *[intrahub-mastra requirements.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/requirements.md) REQ-006（ヒアリング2026-08-13）より*

mastra は `sourceRefs` を `(itemId, fileId, chunkIndex)` の連番インデックス形式で統一し、**PDFページ・EPUB章・固定文字数チャンクの差は MediaVault 側が吸収する**ことを前提にしている。したがって:

- `chunk.index` は**ファイル形式に依らず0起点の連番**。ページ番号や章番号を流用しない
- 形式固有の区切りは `chunk.label`（`"p.42"` / `"第3章"`）という**任意の付属情報**としてのみ現れる。mastra はこれを構造として解釈しない
- **`extraction_version` を新設する**。再抽出でチャンク境界がずれると、保存済み Knowledge Note の `sourceRefs` が**黙って別の箇所を指す**。`extracted_at` の変化だけでは境界のずれを判別できないため、抽出処理のバージョン識別子を返し、mastra 側が参照の失効を検出できるようにする

これは `intrahub-mastra` 側 [prep.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/prep.md) の未チェック項目「`get_item_text` が連番インデックスを返す前提で MediaVault 側と合意する」に対する回答である。

### 入力スキーマ 🟡

**信頼性**: 🟡 *`get_item_context` のfile一覧構造・PRD US-10「ページまたはチャンク単位」から妥当推測*

```json
{
  "item_id": "string (required)",
  "file_id": "string (optional, 省略時は主ファイルを対象とする)",
  "chunk": {
    "index": "number (optional, 0起点。既定 0)",
    "size": "number (optional, 既定4000文字。最大20000)"
  }
}
```

- `file_id` を省略した場合、Itemに抽出済みファイルが複数あり主ファイルを一意に決められないときは `ambiguous` を返し、`file_id` の候補一覧を提示する（api の `AMBIGUOUS_FILE` レスポンスの `candidates` を透過する）。
- `chunk` を省略した場合は先頭チャンク（`index: 0`）を返す。

### 出力スキーマ 🟡

```json
{
  "outcome": "success",
  "item_id": "string",
  "file_id": "string",
  "extracted_at": "string (ISO8601) | null",
  "extraction_version": "string",
  "chunk": {
    "index": 0,
    "size": 4000,
    "total_chunks": 12,
    "label": "p.1",
    "text": "string"
  },
  "error": null
}
```

`label` は形式固有の区切りに対応づけられる場合に `"p.1-3"` のような範囲表記を返し、それ以外は `null`（[item-text.md](../../mediavault-api/item-text.md)）。

| `outcome` | 意味 | api 側の対応 |
|---|---|---|
| `success` | 指定チャンクを返した | 200 |
| `not_found` | `item_id` / `file_id` が存在しない | 404 `ITEM_NOT_FOUND` / `FILE_NOT_FOUND` |
| `not_extracted` | ファイルは存在するが全文抽出が未実行（`request_extraction` での抽出依頼を促すメッセージを含む） | 422 `TEXT_NOT_EXTRACTED` |
| `ambiguous` | `file_id` が未指定でItemに抽出済みファイルが複数ある | 409 `AMBIGUOUS_FILE` |
| `error` | MediaVault-api接続エラー、`chunk.index` 範囲外等 | `MCP_API_UNREACHABLE` / 400 `VALIDATION_ERROR` |

**`not_found` と `not_extracted` の区別は必須**である。前者は「そもそも材料が無い」、後者は「抽出を依頼すれば解決する」であり、mastra 側の対処が異なる（NFR-031）。

### D-09: ツール自体の可視性 🔵

**信頼性**: 🔵 *[acceptance-criteria.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/acceptance-criteria.md) TC-004-E02 より*

mastra の受け入れ基準 TC-004-E02 は、実装前には `get_item_text` を**MCPツール不在**として扱うことを求めていた。現在は抽出機能が実装済みのためツールを常時公開するが、接続先の不整合でツールが見つからない場合も `errors` に記録し、`metadata_only` へ暗黙にフォールバックしない。

したがって全文抽出が未実装の間、`get_item_text` を **`tools/list` に出さない**（環境変数によるフィーチャーフラグで制御）。公開したうえで常に `not_extracted` を返す方式では、mastra 側が「ツールが無い」と「このファイルは未抽出」を区別できず、`mode: metadata_only` へのフォールバック判断（REQ-101）を誤る。

### D-07: シリーズ名の提供 🔵

**信頼性**: 🔵 *[requirements.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/requirements.md) REQ-016a・EDGE-006（ヒアリング2026-08-13）より*

mastra は Knowledge Note の配置先 `topic` を「MediaVault Item の作品名（シリーズがある場合はシリーズ名）」から決定し、**LLM の推測による `topic` 生成を禁止**する（REQ-016a）。したがって MediaVault 側が決定的な規則でシリーズ名を返す必要がある。

`get_item_context` に `series` セクションを追加する（規則の詳細は [api-tool-mapping.md](api-tool-mapping.md) §4 D-07）。

```json
"series": { "state": "loaded", "item_id": "a1b2...", "title": "作品Aシリーズ" }
```

`GET /items/{id}/groups` の `parent_item_id` から親 Item を引いて解決し、解決できない場合は `state: "empty"` を返す。**`group_name`（"Season 1" 等）や `sequel` / `prequel` 関係からシリーズ名を推測しない。** `empty` を受け取った mastra は EDGE-006 により既定の未分類階層へ配置し `warnings` に記録する。MediaVault 側は「分からない」を正確に返すことが責務であり、埋め合わせをしない。

### 非機能要求 🔵

**信頼性**: 🔵 *PRD §11 より*

- 1回のレスポンスに全文を詰め込まない。`total_chunks` を返し、Agent側がループで取得する。
- `mediaResearchAgent` は取得したチャンクを `ResearchResult.sources[].fileId` / `chunks[].extractedAt` に記録し、出典追跡を維持する（intrahub-mastra PRD §13）。

### annotations 🔵

**信頼性**: 🔵 *mcp-tools.md 既存ツールの分類方式に準拠*

`get_item_text` は読み取り専用ツールであり、`readOnlyHint: true` を付与する。

## 3. MediaVault-apiへの要求（追記） 🔵

**信頼性**: 🔵 *PRD §8 の既存要求を本連携向けに具体化*

| API | 要求 | 状況 |
|---|---|---|
| `GET /api/v1/items/{id}/text` | `file_id` 省略時の対象解決、`chunk.index`/`chunk.size` による取得、`extracted_at` / `extraction_version` の返却、0起点連番インデックスの保証 | ✅ **実装済み**（[item-text.md](../../mediavault-api/item-text.md)） |
| 抽出リソース | 公開APIで依頼・状態確認・キャンセルし、Extractor worker の完了後に `get_item_text` で本文を取得できること | ✅ **実装済み**（[extraction.md](../../mediavault-api/extraction.md)） |
| `GET /items/{id}/groups` | シリーズ名解決のため `parent_item_id` を返すこと（D-07） | ✅ **実装済み**（[item-groups.md](../../mediavault-api/item-groups.md)） |
| 別名・原題を含む検索 | `search_library` が別名でも所蔵を解決できること | ✅ **実装済み**。`title` は本題・原題・`details.alternative_titles` を横断OR部分一致（[items.md](../../mediavault-api/items.md)） |

PRD §8 の要求のうち本連携に関わる api 機能は実装済みである（[api-tool-mapping.md](api-tool-mapping.md) §6）。

## 4. 認証・接続経路 🔵

**信頼性**: 🔵 *PRD §10・mcp-tools.md「認証」より*

- `intrahub-mastra` はStreamable HTTPで `POST /mcp` に接続し、`Authorization: Bearer {MCP_AUTH_TOKEN}` を使用する。既存のMVP認証方式をそのまま利用し、mastra専用の認証方式は設けない。
- `intrahub-mastra` はミニPC上の内部ネットワークから接続する常駐エージェントであるため、PRD §10の「内部経路」に該当する。リバースプロキシを経由しない接続を前提とする。
- `MCP_AUTH_TOKEN` は `intrahub-mastra` の環境変数として別途管理し、リポジトリに含めない。

### 4.1 D-10: read-only トークンスコープ（第2段階）🔵

**信頼性**: 🔵 *intrahub-mastra PRD §15・MediaVault-mcp NFR-101 より*

現状、書き込みツールからの保護は**完全にクライアント側にある**。`intrahub-mastra` PRD §15 は「MCPトークン自体が書き込み権限を持つ間は、Mastra側のツール選別を必須の防御線とする」としており、MediaVault-mcp 側の設定ミスや、同じトークンを使う別クライアントの接続では防げない。

第2段階の要求として、`MCP_AUTH_TOKEN` に加えて `MCP_READONLY_TOKEN` を設ける。read-only トークンで接続したセッションでは `tools/list` を `readOnlyHint: true` のツールに限定し、書き込みツールの呼び出しはツール実行前に拒否する。これにより mastra 側の許可リスト（REQ-401）と MediaVault-mcp 側のトークンスコープが**二重の防御線**になる。

MVP では現行どおり単一トークンで運用する。

## 5. 対象外（本書のスコープ外） 🔵

**信頼性**: 🔵 *intrahub-mastra PRD §5・MediaVault-mcp PRD §13 より*

- `request_extraction` / `get_extraction_status` / `cancel_extraction` の詳細設計（実装済み。[mcp-tools.md](mcp-tools.md)・[extraction.md](../../mediavault-api/extraction.md) を参照）
- Knowledge Vault側の `vault-mcp`（`search_notes` / `create_note` 等）の設計。これは `intrahub-mastra` 側のリポジトリ・別ドキュメントで扱う。
- MediaVault-mcpからのナレッジ取得・保存ツールの追加（両PRDの原則により提供しない）
- **`list_citations` / `add_citation`**: 第2段階で MediaVault-mcp に実装するが（[api-tool-mapping.md](api-tool-mapping.md) §3）、mastra 側 REQ-401 の許可リストが3ツール限定のため `mediaResearchAgent` には**渡さない**。`list_citations` は `readOnlyHint: true` であり REQ-402 にも抵触しないため、mastra 側が許可リストを拡張すれば追加できる。MediaVault-mcp 側からは要求しない

## 6. mastra 側 fetch budget との整合 🔵

**信頼性**: 🔵 *[requirements.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/requirements.md) NFR-001 / NFR-002 / NFR-003 より*

mastra は1回の Workflow 実行に次の取得上限を設けている。MediaVault-mcp 側はこれをループ制御可能な形で支える。

| 対象 | mastra の既定上限 | MediaVault-mcp 側の対応 |
|---|---|---|
| Item数 | 5 | `search_library` の `limit` は 1..=50。mastra 側で絞る |
| Itemあたりのファイル数 | 3 | `get_item_context` の `files` セクションが全件を返すため、mastra 側で先頭3件を選ぶ |
| チャンク数（実行合計） | 50 | `get_item_text` が `total_chunks` を返すため、mastra は取得前に必要回数を見積もれる |
| 入力累計トークン | 120,000 | 既定 `chunk.size` 4000文字が上限見積もりの単位になる |

**MediaVault-mcp 側は上限を強制しない。** 上限超過を無言で切り捨てないこと（NFR-003）は mastra 側の責務であり、mcp が勝手に打ち切ると mastra が `warnings` を出せなくなる。mcp は `total_chunks` のような**判断材料を正確に返す**ことに徹する。

なお `get_item_context` の `citations` セクションが件数のみを返すのも同じ理由による（[api-tool-mapping.md](api-tool-mapping.md) §3 D-12）。引用本文をレスポンスへ含めると、`get_item_context` 1回あたりのトークン量が Item 依存で予測不能になり、上記の見積もりが成立しなくなる。

## 関連文書

- [MediaVault-mcp PRD](../PRD.md)
- [api-tool-mapping.md](api-tool-mapping.md) — 全エンドポイントの露出可否と決定事項 D-07〜D-12
- [mcp-tools.md](mcp-tools.md) — 既存MVPツール（`search_library` / `get_item_context` 等）の詳細仕様
- [item-text.md](../../mediavault-api/item-text.md) — `GET /items/{id}/text` の REST 仕様
- [intrahub-mastra PRD](../../../../../intrahub-mastra/docs/PRD.md) — 本連携を利用するAgent/Workflow設計
- [intrahub-mastra requirements.md](../../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/requirements.md) — REQ-006 / REQ-016a / REQ-401 / NFR-001 / NFR-031 の出典
