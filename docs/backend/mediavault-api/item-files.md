← [index](./index.md)

# Item Files API

## GET /items/{id}/files
指定アイテムに紐づくファイルを作成日時昇順で一覧取得する。
- **成功レスポンス** (200): `ApiOk<ItemFile[]>`

## 2つの登録経路と `path` の規約

ファイルの登録には2つの経路があり、`item_files.path` の意味が異なる。

| 経路 | エンドポイント | `path` | 実体の場所 | 実体の所有者 |
|---|---|---|---|---|
| リンク | `POST /items/{id}/files` | 指定された**絶対パス**をそのまま保存 | `/srv/anime`・`/srv/live-action`・`/srv/manga` | MediaVault ではない |
| アップロード | `POST /items/{id}/files/upload` | 保存先ベースディレクトリからの**相対パス** `{item_id}/{uuid}.{ext}` | MediaVault専用領域（`STORAGE_ROOT` 配下のアップロード領域／アイテムIDフォルダ） | MediaVault |

MediaVault は実データ領域へは書き込まない。リンク経路は実体をコピー・移動せず、パスを記録するだけである。

## POST /items/{id}/files
ファイルパス情報のみ登録（実体アップロードなし＝リンク）。`file_type` はクライアント指定ではなく、`path` の拡張子から自動分類される（下表参照）。
- **リクエストボディ** (`CreateItemFileRequest`): `path` (必須), `label` (optional)
- **成功レスポンス** (201): `ApiOk<ItemFile>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`

## POST /items/{id}/files/upload
実ファイルをアップロードして保存。ボディサイズ上限は本エンドポイントのみ100MBに拡張（`DefaultBodyLimit::max`）。`file_type` は元ファイル名の拡張子から自動分類される（下表参照）。
- **Content-Type**: `multipart/form-data`（`file`, `label` optional）
- **成功レスポンス** (201): `ApiOk<ItemFile>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`, 500 `FILE_STORAGE_WRITE_FAILED`

### 保存先の決定
保存先は `STORAGE_ROOT` + アップロード領域サブディレクトリ + **アイテムIDのフォルダ**。1アイテムのファイルは file_type を問わず同じフォルダにまとまる。

```
${STORAGE_ROOT}/
└── files/              # STORAGE_SUBDIR_FILES
     └── {item_id}/     # アイテムIDごとのフォルダ（APIが自動作成）
          └── {uuid}.{ext}
```

| 環境変数 | 既定値 | 用途 |
|---|---|---|
| `STORAGE_ROOT` | `/srv/mediavault` | MediaVault専用領域のルート |
| `STORAGE_SUBDIR_FILES` | `files` | アップロード領域のサブディレクトリ。`vault/2026` のようなネストも可 |

`STORAGE_SUBDIR_FILES` にルート外を指す値（絶対パス・`..` を含む）を与えた場合は無視されて既定値が使われる。アイテムIDはUUIDとして検証済みの値のみをフォルダ名に使う。

### 旧レイアウト（後方互換）
アイテムIDフォルダの導入前は `STORAGE_ROOT` 直下の file_type 別サブディレクトリ（`pdf/` `image/` `video/` `audio/` `archive/` `other/`、`STORAGE_SUBDIR_PDF` 等で上書き）にフラットに保存しており、`path` は `{uuid}.{ext}` だった。既存ファイルの移行は行わない。DELETE 時のみ、現行レイアウトに実体が無ければ旧レイアウトを探すフォールバックが働く（`file_storage::resolve_legacy_base_dir`）。新規アップロードが旧レイアウトへ書かれることはない。

### file_type 自動分類（拡張子・大文字小文字非依存）
| file_type | 拡張子 |
|---|---|
| `pdf` | pdf |
| `image` | jpg, jpeg, png, gif, webp, bmp, svg, avif, heic |
| `video` | mp4, mkv, avi, mov, wmv, webm, m4v, flv, ts |
| `audio` | mp3, flac, wav, aac, ogg, m4a, opus, wma |
| `archive` | zip, rar, 7z, tar, gz, cbz, cbr |
| `other` | 上記以外・拡張子なし |

従来の `file_type` フィールドがリクエストに含まれていても無視される（後方互換）。

## PATCH /items/{id}/files/{file_id}/calibre-link
PDFファイルとCalibre書籍IDを紐付ける。
- **リクエストボディ** (`UpdateCalibreLinkRequest`): `calibre_book_id` (必須)
- **成功レスポンス** (200): `ApiOk<ItemFile>`
- **エラー**: 404 `FILE_NOT_FOUND`, 400 `VALIDATION_ERROR`（対象が pdf 以外の file_type、または id 不正）

## DELETE /items/{id}/files/{file_id}
ファイルレコードを削除する。物理ファイルの扱いは登録経路によって異なる。
- **アップロード（`path` が相対パス）**: MediaVault専用領域の実体もクリーンアップする。現行レイアウトに実体が無ければ旧レイアウトを探す。アイテムIDフォルダが空になった場合はフォルダも削除する。
- **リンク（`path` が絶対パス）**: DBレコードのみ削除し、実データ領域の実体は残す。リンクの削除は「Item との紐付けを外す」操作であり、録画データ等を消す操作ではない。
- **成功レスポンス**: 204
- **エラー**: 404 `FILE_NOT_FOUND`, 400 `VALIDATION_ERROR`（id/file_idが不正なUUID形式の場合）
