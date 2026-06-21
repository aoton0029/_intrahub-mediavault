# API設計 — Open Library

## 基本方針
- RESTful API（HTTP GET, JSON）
- ベースURL: `https://openlibrary.org`
- 認証: 不要（公開 API）
- レスポンス形式: JSON
- Rate Limit: 公式の明確な制限はないが、過度なアクセスは避ける（遅延推奨）

## エンドポイント一覧
| Method | Path | 説明 | 認証 |
|--------|------|------|------|
| GET | /search.json | 書籍・著者・件名での全文検索 | 不要 |
| GET | /isbn/{isbn}.json | ISBN による書籍取得（Works にリダイレクト） | 不要 |
| GET | /books/{olid}.json | Editions の詳細取得 | 不要 |
| GET | /works/{olid}.json | Works（作品）の詳細取得 | 不要 |
| GET | /authors/{olid}.json | 著者の詳細取得 | 不要 |

---

## GET /search.json

### リクエスト
```
GET https://openlibrary.org/search.json?q={query}&fields={fields}&limit={limit}&page={page}
```

### パラメーター
- `q`: 全文検索クエリ（任意）
- `title`: タイトル検索（任意、`q` と組み合わせ可）
- `author`: 著者名検索（任意）
- `isbn`: ISBN 検索（任意）
- `fields`: 返すフィールドをカンマ区切りで指定（任意）
- `limit`: 結果数（デフォルト 10）
- `page`: ページ番号（デフォルト 1）

### リクエスト例
```bash
curl "https://openlibrary.org/search.json?q=三体&fields=key,title,author_name,first_publish_year,isbn,cover_i&limit=10"
```

### レスポンス（成功 200）
```json
{
  "numFound": 42,
  "start": 0,
  "docs": [
    {
      "key": "/works/OL26308W",
      "title": "三体",
      "author_name": ["劉慈欣"],
      "first_publish_year": 2006,
      "isbn": ["9787536692930"],
      "cover_i": 8739161
    }
  ]
}
```

---

## GET /isbn/{isbn}.json

### リクエスト
```
GET https://openlibrary.org/isbn/{isbn}.json
```

### 説明
ISBN（10桁または13桁）から書籍の Edition オブジェクトを返す。Works へのリダイレクトを含む場合がある。

### リクエスト例
```bash
curl "https://openlibrary.org/isbn/9784152091352.json"
```

### レスポンス（成功 200）
```json
{
  "key": "/books/OL12345M",
  "title": "三体",
  "isbn_13": ["9784152091352"],
  "publishers": ["早川書房"],
  "publish_date": "2019",
  "number_of_pages": 512,
  "works": [{"key": "/works/OL26308W"}]
}
```

---

## GET /works/{olid}.json

### リクエスト
```
GET https://openlibrary.org/works/{olid}.json
```

### 説明
指定した作品（Work）の詳細を取得する。

### リクエスト例
```bash
curl "https://openlibrary.org/works/OL26308W.json"
```

### レスポンス（抜粋）
```json
{
  "key": "/works/OL26308W",
  "title": "三体",
  "description": "...",
  "subjects": ["Science fiction", "Astrophysics"],
  "authors": [{"author": {"key": "/authors/OL7677748A"}}]
}
```

---

## GET /authors/{olid}.json

### リクエスト
```
GET https://openlibrary.org/authors/{olid}.json
```

### 説明
著者の詳細情報（名前、略歴、写真等）を取得する。

### リクエスト例
```bash
curl "https://openlibrary.org/authors/OL7677748A.json"
```

### レスポンス（成功 200）
```json
{
  "key": "/authors/OL7677748A",
  "name": "劉慈欣",
  "bio": "...",
  "photos": [1234567]
}
```

---

## 書影 URL

`cover_i`（整数 ID）または `isbn` / `olid` を用いてカバー画像を取得できる。

```
https://covers.openlibrary.org/b/{type}/{value}-{size}.jpg
```

- `type`: `id`, `isbn`, `olid`, `oclc`, `lccn` など
- `size`: `S`（小）、`M`（中）、`L`（大）

例:
```
https://covers.openlibrary.org/b/id/8739161-M.jpg
https://covers.openlibrary.org/b/isbn/9784152091352-L.jpg
```

---

## エラーレスポンス

| ステータス | 説明 |
|------------|------|
| 404 | 指定リソースが存在しない |
| 410 | Gone（廃止されたエンドポイント） |
| 5xx | サーバーエラー |

---

## 使用上の注意・Tips
- `search.json` の `fields` を明示して不要なデータ転送を避ける。
- 書影は `cover_i` が存在する場合のみ取得できる（null の場合は 404）。フォールバックを用意すること。
- `isbn/{isbn}.json` は Works にリダイレクトされる場合がある（HTTP 302 または JSON 内の `works` キー参照）。
- レート制限の明示はないが、バッチ処理時はリクエスト間に遅延を入れること（推奨: 200ms 以上）。
- `number_of_pages` は Edition ごとに異なる場合がある。

## 参考リンク
- [Open Library API Documentation](https://openlibrary.org/developers/api)
- [Open Library Covers API](https://openlibrary.org/dev/docs/api#anchor-cover)
- [Open Library Search API](https://openlibrary.org/dev/docs/api#anchor-search)
