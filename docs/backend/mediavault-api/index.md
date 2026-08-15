# MediaVault API 設計

## 基本方針
- RESTful API（Rust / Axum / sqlx / PostgreSQL）
- ベースURL:
  - 公開API: `/api/v1`
  - 内部API: `/api/v1/internal`
- 認証:
  - 公開API（`/api/v1/*`）: **認証なし**。単一ユーザー・セルフホスト用途のためログイン機構は持たない。
  - 内部API（`/api/v1/internal/*`）: `api_key_auth` ミドルウェアが全ルートに適用される。`Authorization` ヘッダに `INTERNAL_API_KEY` 環境変数の値（生の値、または `Bearer <key>` 形式）を渡す必要がある。キー未設定・不一致は `401 UNAUTHORIZED`。
- レスポンス形式: JSON

---

## 共通レスポンス形式

### 成功時（`ApiOk<T>`）
```json
{ "success": true, "data": { /* T */ } }
```
特記のない限り HTTP `200`。作成系は `201`、削除系は `204 No Content`（ボディなし）。

### ページネーション付き成功時（`PaginatedOk<T>`、keyset/カーソル方式）
```json
{
  "success": true,
  "data": [ /* T[] */ ],
  "pagination": {
    "limit": 20,
    "has_more": true,
    "next_after_created_at": "2026-07-01T12:00:00",
    "next_after_id": "b2b5c1a0-0000-0000-0000-000000000000"
  }
}
```
`has_more=false`の場合、`next_after_created_at`/`next_after_id`は`null`。件数の総数（`total`）は返さない。

### エラー時（`ApiError`）
```json
{ "success": false, "error": { "code": "ITEM_NOT_FOUND", "message": "..." } }
```

### エラーコード一覧

| コード | HTTPステータス | 説明 |
|---|---|---|
| VALIDATION_ERROR | 400 | リクエストの値が不正（UUID形式不正、必須項目欠如など） |
| UNAUTHORIZED | 401 | 内部APIキー不一致・未設定 |
| ITEM_NOT_FOUND | 404 | 指定した item が存在しない |
| UNPROCESSABLE_ENTITY | 422 | 汎用の処理不能エラー |
| INTERNAL_ERROR | 500 | サーバ内部エラー（DB接続失敗など） |
| EXTERNAL_API_ERROR | 502 | 外部API呼び出し全般のエラー |
| DUPLICATE_TAG_NAME | 409 | タグ名が重複 |
| TAG_NOT_FOUND | 404 | 指定した tag が存在しない |
| DUPLICATE_CATEGORY_NAME | 409 | カテゴリ名が重複 |
| CATEGORY_NOT_FOUND | 404 | 指定した category が存在しない |
| MYLIST_NOT_FOUND | 404 | 指定した mylist が存在しない |
| DUPLICATE_RELATION | 409 | 同一の item 関連がすでに存在 |
| GROUP_NOT_FOUND | 404 | 指定した item group が存在しない |
| INVALID_GROUP_TYPE_FOR_EPISODES | 400 | `volume` タイプの group に episode を作成しようとした |
| DUPLICATE_EPISODE_NUMBER | 409 | 同一 group 内で episode_number が重複 |
| STAFF_NOT_FOUND | 404 | 指定した staff が存在しない |
| CAST_NOT_FOUND | 404 | 指定した cast が存在しない |
| INVALID_PROVIDER | 400 | `provider` パスパラメータが未対応の値 |
| API_KEY_NOT_CONFIGURED | 422 | 外部検索に必要なAPIキーが未登録 |
| EXTERNAL_API_TIMEOUT | 502 | 外部APIの呼び出しタイムアウト・失敗 |
| ITEM_ALREADY_IMPORTED | 409 | 既に同一ソースからインポート済み |
| FILE_STORAGE_WRITE_FAILED | 500 | アップロードファイルの保存に失敗 |
| FILE_NOT_FOUND | 404 | 指定した item file が存在しない |
| STEAM_API_KEY_INVALID | 401 | Steam Web API キーが無効 |
| CITATION_NOT_FOUND | 404 | 指定した citation が存在しない |
| TEXT_NOT_EXTRACTED | 422 | ファイルは存在するが全文抽出が未実行（[item-text.md](./item-text.md)） |
| AMBIGUOUS_FILE | 409 | `file_id` 省略時に抽出済みファイルが複数あり一意に決められない（[item-text.md](./item-text.md)） |
| DUPLICATE_STREAMING_LINK | 409 | 同一アイテムに同一プラットフォームの配信URLが登録済み |
| UNSUPPORTED_BACKUP_VERSION | 400 | 未対応のバックアップスキーマバージョン |
| IMPORT_JOB_NOT_FOUND | 404 | 指定したBooklogインポートジョブが存在しない |
| EXTRACTION_NOT_FOUND | 404 | 指定ファイルに抽出が存在しない |
| EXTRACTION_ALREADY_FINISHED | 409 | 終端状態の抽出をキャンセルしようとした |
| UNSUPPORTED_FILE_TYPE | 422 | 文字抽出に対応していないファイル種別 |
| INVALID_LEASE_TOKEN | 409 | worker の lease token が不一致または失効済み |

### ページネーション正規化
`limit` クエリパラメータは以下のルールで補正される（`normalize_limit`）:
- `limit` 未指定 → `20`（デフォルト）
- `limit < 1` → `20`
- `limit > 100` → `100`

`after_created_at` / `after_id` は両方指定された場合のみ有効なカーソルとして扱われる。片方のみ指定された場合は無視され、先頭ページとして扱われる（400にはしない）。

---

## 主要Enum

| Enum | 値 |
|---|---|
| `media_type` | anime, movie, drama, manga, novel, game, academic_book, paper |
| `item_status` | （例: unwatched/watching/completed 等、item のステータス管理に使用） |
| `item_source` | アイテムの取得経路（手動登録／外部API取込／CSVインポート等） |
| `group_type` | season, volume, chapter |
| `relation_type` | adaptation, sequel, prequel, spinoff, dlc, reference |
| `file_type` | pdf, image, other |
| `api_provider` | tmdb, igdb, ndl, steam, open_library, ani_list（jikanは認証不要のため対象外）<br>⚠️ `PUT /settings/api-keys/{provider}` が受理するのは tmdb, igdb, ndl, steam, **annict**, **rakuten** であり、本Enumと一致しない（[settings.md](./settings.md)）。要整理 |
| `locator_type` | page, timestamp, location, chapter, none（citation の付加情報の種類） |

---

## エンドポイント一覧（公開API `/api/v1`）

| Method | Path | 説明 | 詳細 |
|--------|------|------|------|
| GET | /health | ヘルスチェック | [health.md](./health.md) |
| GET | /collection/overview | コレクション全体統計（media_type別/status別件数・お気に入り・最近追加/更新） | [collection.md](./collection.md) |
| GET | /items | アイテム一覧取得（フィルタ・ページネーション） | [items.md](./items.md) |
| GET | /items/counts-by-media-type | メディア種別ごとの件数集計 | [items.md](./items.md) |
| POST | /items | アイテム新規作成 | [items.md](./items.md) |
| GET | /items/search | 外部API横断検索 | [items.md](./items.md) |
| POST | /items/import | 外部検索結果からアイテムをインポート | [items.md](./items.md) |
| GET | /items/{id} | アイテム詳細取得 | [items.md](./items.md) |
| PATCH | /items/{id} | アイテム更新 | [items.md](./items.md) |
| DELETE | /items/{id} | アイテム削除 | [items.md](./items.md) |
| PATCH | /items/{id}/status | ステータス更新 | [items.md](./items.md) |
| GET | /tags | タグ一覧取得 | [tags.md](./tags.md) |
| POST | /tags | タグ作成 | [tags.md](./tags.md) |
| DELETE | /tags/{id} | タグ削除 | [tags.md](./tags.md) |
| POST | /items/{id}/tags/{tag_id} | アイテムにタグ付与 | [tags.md](./tags.md) |
| DELETE | /items/{id}/tags/{tag_id} | アイテムからタグ削除 | [tags.md](./tags.md) |
| GET | /categories | カテゴリ一覧取得 | [categories.md](./categories.md) |
| POST | /categories | カテゴリ作成 | [categories.md](./categories.md) |
| DELETE | /categories/{id} | カテゴリ削除 | [categories.md](./categories.md) |
| POST | /items/{id}/categories/{category_id} | アイテムにカテゴリ付与 | [categories.md](./categories.md) |
| DELETE | /items/{id}/categories/{category_id} | アイテムからカテゴリ削除 | [categories.md](./categories.md) |
| GET | /mylists | マイリスト一覧取得 | [mylists.md](./mylists.md) |
| POST | /mylists | マイリスト作成 | [mylists.md](./mylists.md) |
| GET | /items/{id}/mylists | アイテムが属するマイリスト一覧取得 | [mylists.md](./mylists.md) |
| POST | /mylists/{id}/items | マイリストにアイテム追加 | [mylists.md](./mylists.md) |
| DELETE | /mylists/{id}/items/{item_id} | マイリストからアイテム削除 | [mylists.md](./mylists.md) |
| GET | /items/{id}/relations | アイテム関連一覧取得 | [item-relations.md](./item-relations.md) |
| POST | /item-relations | アイテム関連作成 | [item-relations.md](./item-relations.md) |
| DELETE | /item-relations/{id} | アイテム関連削除 | [item-relations.md](./item-relations.md) |
| POST | /items/{id}/groups | グループ作成（season/volume/chapter） | [item-groups.md](./item-groups.md) |
| GET | /items/{id}/groups | グループ一覧取得 | [item-groups.md](./item-groups.md) |
| POST | /groups/{group_id}/episodes | エピソード作成 | [item-episodes.md](./item-episodes.md) |
| GET | /groups/{group_id}/episodes | エピソード一覧取得 | [item-episodes.md](./item-episodes.md) |
| POST | /staff | スタッフ作成 | [staff.md](./staff.md) |
| GET | /items/{id}/staff | アイテムのスタッフ紐付け一覧取得 | [staff.md](./staff.md) |
| POST | /items/{id}/staff | アイテムにスタッフ紐付け | [staff.md](./staff.md) |
| DELETE | /items/{id}/staff/{item_staff_id} | アイテムのスタッフ紐付け削除 | [staff.md](./staff.md) |
| POST | /cast | キャスト作成 | [cast.md](./cast.md) |
| GET | /items/{id}/cast | アイテムのキャスト紐付け一覧取得 | [cast.md](./cast.md) |
| POST | /items/{id}/cast | アイテムにキャスト紐付け | [cast.md](./cast.md) |
| DELETE | /items/{id}/cast/{item_cast_id} | アイテムのキャスト紐付け削除 | [cast.md](./cast.md) |
| GET | /items/{id}/files | アイテムファイル一覧取得 | [item-files.md](./item-files.md) |
| POST | /items/{id}/files | アイテムファイル情報登録 | [item-files.md](./item-files.md) |
| POST | /items/{id}/files/upload | アイテムファイルアップロード（multipart） | [item-files.md](./item-files.md) |
| PATCH | /items/{id}/files/{file_id}/calibre-link | Calibre連携ID更新 | [item-files.md](./item-files.md) |
| DELETE | /items/{id}/files/{file_id} | アイテムファイル削除 | [item-files.md](./item-files.md) |
| GET | /items/{id}/text | 抽出済み全文のチャンク取得 | [item-text.md](./item-text.md) |
| POST | /items/{id}/files/{file_id}/extraction | 文字抽出を冪等に要求 | [extraction.md](./extraction.md) |
| GET | /items/{id}/files/{file_id}/extraction | 最新の抽出状態・進捗を取得 | [extraction.md](./extraction.md) |
| POST | /items/{id}/files/{file_id}/extraction/cancel | 最新の抽出をキャンセル | [extraction.md](./extraction.md) |
| GET | /items/{id}/links | 外部リンク一覧取得 | [item-links.md](./item-links.md) |
| POST | /items/{id}/links | 外部リンク追加 | [item-links.md](./item-links.md) |
| DELETE | /items/{id}/links/{link_id} | 外部リンク削除 | [item-links.md](./item-links.md) |
| POST | /items/{id}/streaming-links | 配信URL追加（Netflix/AmazonPrime/DisneyPlus/DmmTv/AppleTv） | [item-streaming-links.md](./item-streaming-links.md) |
| GET | /items/{id}/streaming-links | 配信URL一覧取得 | [item-streaming-links.md](./item-streaming-links.md) |
| DELETE | /items/{id}/streaming-links/{link_id} | 配信URL削除 | [item-streaming-links.md](./item-streaming-links.md) |
| POST | /items/{id}/images | 画像URL追加 | [item-images.md](./item-images.md) |
| GET | /items/{id}/images | 画像URL一覧取得 | [item-images.md](./item-images.md) |
| DELETE | /items/{id}/images/{image_id} | 画像URL削除 | [item-images.md](./item-images.md) |
| GET | /items/{id}/trailers | 予告編リンク一覧取得 | [item-trailers.md](./item-trailers.md) |
| POST | /items/{id}/trailers | 予告編リンク追加 | [item-trailers.md](./item-trailers.md) |
| DELETE | /items/{id}/trailers/{trailer_id} | 予告編リンク削除 | [item-trailers.md](./item-trailers.md) |
| GET | /items/{id}/citations | 引用一覧取得 | [citations.md](./citations.md) |
| POST | /items/{id}/citations | 引用作成 | [citations.md](./citations.md) |
| PATCH | /citations/{id} | 引用更新 | [citations.md](./citations.md) |
| DELETE | /citations/{id} | 引用削除 | [citations.md](./citations.md) |
| PUT | /settings/api-keys/{provider} | 外部APIキー登録・更新 | [settings.md](./settings.md) |
| POST | /import/booklog | Booklog CSVインポート | [import.md](./import.md) |
| POST | /import/steam | Steamライブラリインポート | [import.md](./import.md) |

内部API（`/api/v1/internal/*`）の一覧・詳細は [internal-api.md](./internal-api.md) を参照。

## カテゴリ別詳細

- [data-model.md](./data-model.md) — レスポンスに登場するstructのフィールド一覧
- [health.md](./health.md) — Health
- [collection.md](./collection.md) — Collection（コレクション全体統計）
- [items.md](./items.md) — Items
- [tags.md](./tags.md) — Tags
- [categories.md](./categories.md) — Categories
- [mylists.md](./mylists.md) — Mylists
- [item-relations.md](./item-relations.md) — Item Relations
- [item-groups.md](./item-groups.md) — Item Groups（season / volume / chapter）
- [item-episodes.md](./item-episodes.md) — Item Episodes
- [staff.md](./staff.md) — Staff
- [cast.md](./cast.md) — Cast
- [item-files.md](./item-files.md) — Item Files
- [item-text.md](./item-text.md) — Item Text（抽出済み全文のチャンク取得）
- [extraction.md](./extraction.md) — Extraction（抽出リソース・worker連携）
- [item-links.md](./item-links.md) — Item Links
- [item-streaming-links.md](./item-streaming-links.md) — Item Streaming Links
- [item-images.md](./item-images.md) — Item Images
- [item-trailers.md](./item-trailers.md) — Item Trailers
- [citations.md](./citations.md) — Citations（作品・論文からの引用）
- [settings.md](./settings.md) — Settings
- [import.md](./import.md) — Import
- [internal-api.md](./internal-api.md) — 内部API（`/api/v1/internal/*`）

DBスキーマとレスポンスモデルの概要は [data-model.md](./data-model.md) を参照。
