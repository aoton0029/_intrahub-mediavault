# API設計

## 基本方針
- RESTful API
- ベースURL: `/api/v1`（注: TMDb 実際のベースは https://api.themoviedb.org/3）
- 認証: API Key（v3: `api_key` クエリパラメータ）または Bearer トークン（v4: `Authorization: Bearer <token>` ヘッダ）
- レスポンス形式: JSON

## エンドポイント一覧
| Method | Path | 説明 | 認証 |
|--------|------|------|------|
| GET | /search/movie | 映画を検索する | 必要 |
| GET | /movie/{movie_id} | 映画の詳細を取得する（/credits, /images, /videos などのサブエンドポイントあり） | 必要 |
| GET | /search/tv | テレビシリーズを検索する | 必要 |
| GET | /tv/{series_id} | テレビシリーズの詳細を取得する | 必要 |
| GET | /tv/{series_id}/season/{season_number} | シーズン情報・エピソード一覧を取得する | 必要 |
| GET | /tv/{series_id}/images | 画像一覧を取得する | 必要 |
| GET | /tv/{series_id}/credits | キャスト・クレジットを取得する | 必要 |

---

## GET /search/movie

### メソッド
GET

### URL
https://api.themoviedb.org/3/search/movie

### 説明
タイトル等で映画を検索する。

### パラメータ
- `api_key` or `Authorization` header
- `query` (string): 検索ワード（必須）
- `language` (string): レスポンスの言語（例: `ja-JP`）
- `page` (int): ページ番号（デフォルト: 1）
- `region` (string): ISO 3166-1 コード
- `year` / `primary_release_year` (int): 公開年フィルタ

### リクエスト例
```bash
curl -X GET "https://api.themoviedb.org/3/search/movie?api_key=YOUR_API_KEY&query=Godzilla&page=1&language=ja-JP"
```

### レスポンス例
```json
{
	"page": 1,
	"results": [ /* movie objects */ ],
	"total_pages": 2,
	"total_results": 39
}
```

---

## GET /movie/{movie_id}

### メソッド
GET

### URL
https://api.themoviedb.org/3/movie/{movie_id}

### 説明
指定した映画の詳細を取得する。サブエンドポイントとして `/credits`, `/images`, `/videos` などがある。

### リクエスト例
```bash
curl -X GET "https://api.themoviedb.org/3/movie/550?api_key=YOUR_API_KEY&language=ja-JP"
```

---

## GET /search/tv

### メソッド
GET

### URL
https://api.themoviedb.org/3/search/tv

### 説明
テレビシリーズを検索する。

### パラメータ
- `query`, `first_air_date_year`, `language`, `page`, `year`

### リクエスト例
```bash
curl "https://api.themoviedb.org/3/search/tv?api_key=YOUR_API_KEY&query=Naruto&language=ja-JP"
```

---

## GET /tv/{series_id}

### メソッド
GET

### URL
https://api.themoviedb.org/3/tv/{series_id}

### 説明
シリーズ全体の情報を取得する。

---

## GET /tv/{series_id}/season/{season_number}

### メソッド
GET

### URL
https://api.themoviedb.org/3/tv/{series_id}/season/{season_number}

### 説明
シーズン情報・エピソード一覧を取得する。

---

## GET /tv/{series_id}/images

### メソッド
GET

### URL
https://api.themoviedb.org/3/tv/{series_id}/images

### 説明
作品に関連する画像一覧を取得する。

---

## GET /tv/{series_id}/credits

### メソッド
GET

### URL
https://api.themoviedb.org/3/tv/{series_id}/credits

### 説明
キャスト・クレジット情報を取得する。

---

## 画像 URL
TMDb の画像は `poster_path` / `backdrop_path` 等にパスが返る。完全な URL は以下のベースを使用する:

```
https://image.tmdb.org/t/p/{size}{file_path}
```

代表的なサイズ:
- ポスター: `w92`, `w154`, `w185`, `w342`, `w500`, `w780`, `original`
- 背景: `w300`, `w780`, `w1280`, `original`

---

## 使用上の注意・Tips
- API Key を直接リポジトリに含めない（`.env` またはシークレット管理を利用）。
- レスポンスの `poster_path` は `null` の場合があるため存在チェックを行う。
- v4 (Bearer) と v3 (`api_key`) の認証方式の差に注意。v4 は一部エンドポイントで必要。
- レート制限・地域差（`region` パラメータ）に注意。大量アクセス時はリトライ／バックオフを実装する。

## 参考リンク
- [TMDb API Documentation](https://developer.themoviedb.org/docs/getting-started)
- https://image.tmdb.org/t/p/

