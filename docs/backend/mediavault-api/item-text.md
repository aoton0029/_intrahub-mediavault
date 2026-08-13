← [index](./index.md)

# Item Text API

`item_files` から抽出済みの全文テキストを、チャンク単位で取得する。`MediaVault-mcp` の `get_item_text` ツールの対応API（[mediavault-mcp/design/mastra-integration.md](../mediavault-mcp/design/mastra-integration.md)）であり、AIエージェントが作品の内容を材料として扱うための唯一の経路である。

**未実装。** 本ドキュメントは設計仕様であり、`item_file_texts` テーブル・ハンドラともにまだ存在しない。

テキストの抽出そのものは本APIの責務ではなく、`extract_text` ジョブ（[jobs.md](./jobs.md)）が非同期に行う。本APIは抽出結果を読み出すだけである。MediaVault は要約・embedding を生成しない。

---

## チャンク分割の規約

**最も重要な規約**: クライアントへ返すチャンクは、ファイル形式に依らず **0起点の連番 `index`** で識別する。PDFのページ、EPUBの章、動画のタイムスタンプといった形式固有の区切りは `index` に反映せず、表示用の `label` にのみ現れる。

この規約は `intrahub-mastra` が出典参照を `(itemId, fileId, chunkIndex)` の形で統一して保持するためのものである（[intrahub-mastra requirements.md](../../../../intrahub-mastra/docs/spec/knowledge-vault-generation/requirements.md) REQ-006）。形式ごとの差異を MediaVault 側で吸収し、クライアントには単一の構造だけを見せる。

- 抽出済みテキスト全体を `chunk_size` 文字ごとに分割し、先頭から 0, 1, 2, ... と採番する
- `label` は分割位置が形式固有の区切りに対応づけられる場合のみ設定する（例: `"p.42"` / `"第3章"`）。対応づけられない場合は `null`
- 同一の `(file_id, extraction_version, chunk_size)` に対して `index` と本文の対応は**不変**である

### `extraction_version`

抽出処理のバージョン識別子。抽出ロジックの変更やファイルの再抽出によってチャンク境界が変わると、クライアントが保存済みの `chunk_index` は**黙って別の箇所を指す**ようになる。`extracted_at` の変化だけでは境界がずれたかを判別できないため、本フィールドで明示する。

クライアントは保存済みの出典参照とレスポンスの `extraction_version` を比較し、不一致であれば参照を失効として扱える。

---

## データ型

### `ItemText`

| フィールド | 型 | 説明 |
|---|---|---|
| `item_id` | uuid | 対象 item |
| `file_id` | uuid | 抽出元の `item_files.id` |
| `extracted_at` | timestamp | 抽出が完了した日時 |
| `extraction_version` | string | 抽出処理のバージョン識別子 |
| `chunk` | `TextChunk` | 要求されたチャンク |

### `TextChunk`

| フィールド | 型 | 説明 |
|---|---|---|
| `index` | number | 0起点の連番 |
| `size` | number | 分割に使用した文字数（要求値と一致する） |
| `total_chunks` | number | 当該 `size` での全チャンク数 |
| `label` | string \| null | 表示用ラベル。形式固有の区切りに対応づく場合のみ |
| `text` | string | 本文。末尾チャンクは `size` 未満になりうる |

---

## GET /items/{id}/text

抽出済み全文の指定チャンクを取得する。

- **認証**: 不要（公開API）
- **クエリパラメータ**

  | 名前 | 型 | 必須 | 説明 |
  |---|---|---|---|
  | `file_id` | uuid | | 省略時は主ファイルを対象とする（下記「主ファイルの解決」） |
  | `chunk_index` | number | | 0起点。既定 `0` |
  | `chunk_size` | number | | 1チャンクの文字数。既定 `4000`、最大 `20000` |

- **成功レスポンス** (200): `ApiOk<ItemText>`

```json
{
  "success": true,
  "data": {
    "item_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
    "file_id": "f1a2b3c4-1e3e-4c9a-9c3e-2f6b1a2a0002",
    "extracted_at": "2026-08-10T09:30:00",
    "extraction_version": "pdf-v1",
    "chunk": {
      "index": 0,
      "size": 4000,
      "total_chunks": 12,
      "label": "p.1",
      "text": "本文の先頭4000文字..."
    }
  }
}
```

- **エラー**: 404 `ITEM_NOT_FOUND`, 404 `FILE_NOT_FOUND`, 400 `VALIDATION_ERROR`, 409 `AMBIGUOUS_FILE`, 422 `TEXT_NOT_EXTRACTED`

### 主ファイルの解決

`file_id` を省略した場合、対象 item の `item_files` のうち**抽出済みのもの**を候補とし、次のように扱う。

| 候補数 | 挙動 |
|---|---|
| 1件 | そのファイルを対象とする |
| 0件（ファイル自体が無い） | 404 `FILE_NOT_FOUND` |
| 0件（ファイルはあるが未抽出） | 422 `TEXT_NOT_EXTRACTED` |
| 2件以上 | 409 `AMBIGUOUS_FILE`。**推測で選ばない** |

`AMBIGUOUS_FILE` のレスポンスには候補一覧を含め、クライアントが `file_id` を指定して再試行できるようにする。

```json
{
  "success": false,
  "error": {
    "code": "AMBIGUOUS_FILE",
    "message": "複数のファイルが抽出済みです。file_id を指定してください",
    "candidates": [
      { "file_id": "f1a2b3c4-...", "label": "本編PDF", "file_type": "pdf" },
      { "file_id": "f9e8d7c6-...", "label": "付録", "file_type": "pdf" }
    ]
  }
}
```

> **注**: `ApiError` に `candidates` を持たせるのは本エンドポイントのみの拡張である。共通形式（[index.md](./index.md)）の `code` / `message` はそのまま維持する。

### エラーの使い分け

| コード | HTTP | 条件 |
|---|---|---|
| `ITEM_NOT_FOUND` | 404 | `item_id` が存在しない |
| `FILE_NOT_FOUND` | 404 | 指定された `file_id` が存在しない、または対象 item に属さない。`file_id` 省略時はファイルが1件もない |
| `TEXT_NOT_EXTRACTED` | 422 | ファイルは存在するが `extract_text` が未実行・未完了。**「テキストが空」ではなく「まだ抽出していない」ことを表す** |
| `AMBIGUOUS_FILE` | 409 | `file_id` 省略時に抽出済みファイルが2件以上 |
| `VALIDATION_ERROR` | 400 | `chunk_index` が `total_chunks` 以上、`chunk_size` が範囲外、UUID形式不正 |

`TEXT_NOT_EXTRACTED` と `FILE_NOT_FOUND` を区別することは必須である。クライアント（`MediaVault-mcp` 経由の AI エージェント）は前者を「抽出を依頼すれば解決する」、後者を「そもそも材料が無い」として扱い分ける。

---

## 全文抽出との関係

抽出は `extract_text` ジョブ（[jobs.md](./jobs.md)）が担う。payload は `{ "item_file_id": uuid, "path": string }` で既に定義済みであり、本APIのために新しい job_type を追加する必要はない。

| 状態 | `GET /items/{id}/text` の応答 |
|---|---|
| ジョブ未登録 | 422 `TEXT_NOT_EXTRACTED` |
| ジョブ `queued` / `running` | 422 `TEXT_NOT_EXTRACTED` |
| ジョブ `succeeded` | 200 |
| ジョブ `failed` | 422 `TEXT_NOT_EXTRACTED` |

本APIはジョブの状態を参照せず、抽出結果の有無だけで判定する。ジョブの進捗確認は `GET /api/v1/jobs` を使う。

---

## データモデルへの要求

抽出結果を保持する新テーブルが必要になる（[data-model.md](./data-model.md) 未反映）。

| テーブル | 主なカラム |
|---|---|
| `item_file_texts` | `id`, `item_file_id`（`item_files` への FK・UNIQUE）, `content`（抽出全文）, `extraction_version`, `extracted_at`, `created_at` / `updated_at` |

- `content` は分割せず全文で保持し、チャンク分割は**読み出し時に行う**。`chunk_size` をクエリで変えられるようにするため
- ラベル（`p.42` / 第3章）を返すには、形式固有の区切り位置を別途保持する必要がある。初期実装では `label` を常に `null` とし、区切り位置の保持は後続の課題とする
- 再抽出時は同一 `item_file_id` の行を置き換え、`extraction_version` を更新する

---

## 実装上の注意

- **1レスポンスに全文を詰め込まない。** `total_chunks` を返し、クライアントがループで取得する。`chunk_size` の上限 `20000` はこのための制約である
- `content` が巨大なファイルでは、チャンク切り出しを DB 側（`SUBSTRING`）で行い、全文をアプリケーションメモリへ載せない
- `total_chunks` は `ceil(char_length(content) / chunk_size)` で算出する。バイト長ではなく**文字数**で数える（日本語テキストで境界がずれるため）
