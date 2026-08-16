← [index](./index.md)

# Theme Songs API

映像作品（anime / movie / drama）のOP・ED・挿入歌などを管理するAPI。曲（`theme_songs`）はアイテムから独立したマスタとして持ち、`item_theme_songs`を介してアイテムと多対多で紐づく。同じ曲を1期・2期・劇場版それぞれに紐づけても曲レコードは1件で済み、作品ごとに異なる`theme_type`（1期ではOP、劇場版では挿入歌）を持たせられる。

アーティスト・作曲・作詞は正規化せず`theme_songs`の列として持つ（`artists`マスタは作らない）。`item_staff.role`が自由文字列であるのと同じく、表記の揺れは許容する。

映像作品向けの機能だが、`item_streaming_links`と同様にDB・APIとも`media_type`による制約はかけない。

## theme_type の意味

`item_theme_songs.theme_type`は、その曲が**その作品において**どう使われているかを表す。同じ曲でも紐づける作品ごとに異なる値を取れる。

| 値 | 意味 |
|---|---|
| `op` | オープニングテーマ |
| `ed` | エンディングテーマ |
| `insert` | 挿入歌 |
| `image` | イメージソング（本編未使用のタイアップ曲など） |
| `character` | キャラクターソング |
| `theme` | 主題歌（OP/EDの区別がない作品。映画向け） |
| `other` | 上記に当てはまらないもの |

上記7値以外は 400 `VALIDATION_ERROR` で拒否される。

## ThemeSong

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | |
| `title` | string | 曲名。空白のみは400 |
| `artist` | string \| null | 歌手・アーティスト名。複数名は1つの文字列にまとめる |
| `composer` | string \| null | 作曲 |
| `lyricist` | string \| null | 作詞 |
| `arranger` | string \| null | 編曲 |
| `note` | string \| null | 補足（バージョン違い、フル尺の情報など） |
| `created_at` / `updated_at` | timestamp | |

曲名は別作品で重複しうるため一意制約は持たない。

## ThemeSongLink

1つの曲に複数の配信・視聴リンクを持たせる。

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | |
| `theme_song_id` | UUID | |
| `link_type` | enum | `youtube` / `spotify` / `apple_music` / `amazon_music` / `niconico` / `official` / `other` |
| `url` | string | |
| `label` | string \| null | 任意の表示名（「TVサイズ」「MV」など） |
| `sort_order` | int | 表示順（デフォルト0） |
| `created_at` | timestamp | |

同一`link_type`の複数登録は許す（公式MVとTVサイズなど）。同一曲に同一URLの重複のみ 409 で拒否する。

## ItemThemeSong

アイテムと曲の紐付け。レスポンスには曲本体とそのリンクをネストして返す（テーマソング欄の描画に必ず要るため）。

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | 紐付けのID。削除時に使う |
| `item_id` | UUID | |
| `theme_type` | enum | 上記「theme_type の意味」参照 |
| `display_order` | int | 同一`theme_type`内の並び（OP1 / OP2）。デフォルト0 |
| `created_at` | timestamp | |
| `theme_song` | ThemeSongWithLinks | 曲本体 + `links: ThemeSongLink[]` |

---

## GET /theme-songs

曲を一覧・検索する。

- **クエリパラメータ** (`ListThemeSongsQuery`):
  - `q` (string, optional) — 曲名・`artist`の部分一致（大文字小文字を区別しない）
  - `limit` (number, optional) — `normalize_limit`で正規化（未指定・1未満→20、100超→100）
- **成功レスポンス** (200): `ApiOk<ThemeSongWithLinks[]>`（`title`昇順、同名は`created_at`昇順）

## POST /theme-songs

曲を作成する。リンクを同時に登録でき、曲とリンクは単一トランザクションで作成される。

- **リクエストボディ** (`CreateThemeSongRequest`):
  - `title` (必須, string) — 空白のみは400
  - `artist` / `composer` / `lyricist` / `arranger` / `note` (optional, string)
  - `links` (optional, `{ link_type, url, label?, sort_order? }[]`) — `link_type`・`url`は必須。`url`が空文字は400
- **成功レスポンス** (201): `ApiOk<ThemeSongWithLinks>`
- **エラー**: 400 `VALIDATION_ERROR`（`title`空、不正な`link_type`、`url`空）, 409 `DUPLICATE_THEME_SONG_LINK`（`links`内でURLが重複）

```json
{
  "success": true,
  "data": {
    "id": "6f1a3c20-0000-0000-0000-000000000000",
    "title": "残酷な天使のテーゼ",
    "artist": "高橋洋子",
    "composer": "佐藤英敏",
    "lyricist": "及川眠子",
    "arranger": null,
    "note": null,
    "links": [
      {
        "id": "8b2d4e10-0000-0000-0000-000000000000",
        "theme_song_id": "6f1a3c20-0000-0000-0000-000000000000",
        "link_type": "youtube",
        "url": "https://www.youtube.com/watch?v=xxxxxxxxxxx",
        "label": "MV",
        "sort_order": 0,
        "created_at": "2026-08-16T10:00:00"
      }
    ],
    "created_at": "2026-08-16T10:00:00",
    "updated_at": "2026-08-16T10:00:00"
  }
}
```

## GET /theme-songs/{id}

曲の詳細。リンクに加えて、その曲が使われているアイテムの一覧を含む。「1期ではOP、劇場版では挿入歌」を1リクエストで確認できる。

- **成功レスポンス** (200): `ApiOk<ThemeSongDetail>` — `ThemeSongWithLinks`の全フィールド + `items: ThemeSongItemRef[]`（`{ item_id, title, media_type, theme_type }`）
- **エラー**: 404 `THEME_SONG_NOT_FOUND`, 400 `VALIDATION_ERROR`（UUID形式不正）

## PATCH /theme-songs/{id}

- **リクエストボディ** (`UpdateThemeSongRequest`): `title` / `artist` / `composer` / `lyricist` / `arranger` / `note`（いずれも optional。指定されたフィールドのみ更新）
- **成功レスポンス** (200): `ApiOk<ThemeSongWithLinks>`
- **エラー**: 404 `THEME_SONG_NOT_FOUND`, 400 `VALIDATION_ERROR`（`title`に空白のみを指定）

## DELETE /theme-songs/{id}

曲を削除する。`theme_song_links`とすべてのアイテムへの紐付け（`item_theme_songs`）はCASCADEで削除される。アイテム自体は削除されない。

- **成功レスポンス**: 204
- **エラー**: 404 `THEME_SONG_NOT_FOUND`

## GET /theme-songs/{id}/links

指定曲のリンクを`sort_order`昇順（同順位は作成日時昇順）で一覧取得する。

- **成功レスポンス** (200): `ApiOk<ThemeSongLink[]>`
- **エラー**: 404 `THEME_SONG_NOT_FOUND`

## POST /theme-songs/{id}/links

- **リクエストボディ** (`CreateThemeSongLinkRequest`): `link_type` (必須), `url` (必須), `label` (optional), `sort_order` (optional, デフォルト 0)
- **成功レスポンス** (201): `ApiOk<ThemeSongLink>`
- **エラー**: 404 `THEME_SONG_NOT_FOUND`, 400 `VALIDATION_ERROR`（不正な`link_type`、`url`が空文字）, 409 `DUPLICATE_THEME_SONG_LINK`

## DELETE /theme-songs/{id}/links/{link_id}

- **成功レスポンス**: 204
- **エラー**: 404 `THEME_SONG_NOT_FOUND`

---

## GET /items/{id}/theme-songs

指定アイテムのテーマソングを`theme_type`のenum順（op → ed → insert → image → character → theme → other）、次に`display_order`昇順、次に作成日時昇順で一覧取得する。

- **成功レスポンス** (200): `ApiOk<ItemThemeSong[]>`
- **エラー**: 404 `ITEM_NOT_FOUND`

## POST /items/{id}/theme-songs

既存の曲をアイテムに紐づける。曲の新規作成は`POST /theme-songs`で行う（UIは「既存曲を検索 → なければ作成 → 紐付け」の2ステップ）。

- **リクエストボディ** (`CreateItemThemeSongRequest`): `theme_song_id` (必須), `theme_type` (必須), `display_order` (optional, デフォルト 0)
- **成功レスポンス** (201): `ApiOk<ItemThemeSong>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 404 `THEME_SONG_NOT_FOUND`, 400 `VALIDATION_ERROR`（不正な`theme_type`）, 409 `DUPLICATE_ITEM_THEME_SONG`（同一 `item_id` / `theme_song_id` / `theme_type` の組み合わせ）

同じ曲を同じアイテムに異なる`theme_type`で紐づけることはできる（1期でOP、後半でEDになった曲など）。

## DELETE /items/{id}/theme-songs/{item_theme_song_id}

紐付けのみを解除する。曲レコード（`theme_songs`）は削除されない。

- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`

## ItemDetail拡張

`GET /items/{id}`のレスポンス(`ItemDetail`)に`theme_songs: ItemThemeSong[]`が含まれる。テーマソングを持たないアイテムでは空配列。
