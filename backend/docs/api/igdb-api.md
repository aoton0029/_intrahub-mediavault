# API設計

## 基本方針
- IGDB の HTTP API（Apicalypse クエリ）に準拠
- ベースURL: `https://api.igdb.com/v4`
- 認証: Twitch OAuth2（Client Credentials） — 必須（`Client-ID` ヘッダー + `Authorization: Bearer <access_token>`）
- レスポンス形式: JSON

## エンドポイント一覧
| Method | Path | 説明 | 認証 |
|--------|------|------|------|
| POST | /games | ゲーム検索 / 詳細取得（Apicalypse） | 必要 |
| POST | /search | 横断検索（Characters, Collections, Games 等） | 必要 |
| POST | /companies | 企業情報検索 | 必要 |
| POST | /platforms | プラットフォーム情報取得 | 必要 |
| POST | /covers | カバー画像情報取得 | 必要 |
| POST | /screenshots | スクリーンショット取得 | 必要 |

---

## POST /games

### リクエスト
API はすべて `POST` を使用し、ボディに Apicalypse クエリをテキストで記述します。

例（検索）:
```bash
curl -X POST "https://api.igdb.com/v4/games" \
	-H "Client-ID: YOUR_CLIENT_ID" \
	-H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
	-H "Accept: application/json" \
	-d 'search "Halo"; fields id,name,cover.image_id,first_release_date,genres.name,platforms.name,summary,rating,total_rating; limit 20; where version_parent = null;'
```

### バリデーション
- body: Apicalypse クエリ文字列（必須）
- `fields` を明示的に指定することを推奨

### レスポンス（成功 200）
JSON 配列（ゲームオブジェクト）。例:
```json
[
	{
		"id": 1942,
		"name": "The Witcher 3: Wild Hunt",
		"cover": { "id": 89386, "image_id": "co1wyy" },
		"first_release_date": 1431993600,
		"genres": [{"id":12,"name":"Role-playing (RPG)"}],
		"platforms": [{"id":6,"name":"PC (Microsoft Windows)"}],
		"summary": "You are Geralt of Rivia...",
		"rating": 92.3,
		"total_rating": 93.6
	}
]
```

### エラーレスポンス
| ステータス | コード | 説明 |
|------------|--------|------|
| 400 | BAD_REQUEST | クエリ構文エラー等 |
| 401 | UNAUTHORIZED | 認証情報が無効 |
| 429 | RATE_LIMIT | レート制限超過 |

---

## POST /search

### リクエスト
横断検索（Characters, Collections, Games, Platforms, Themes 等）。ボディに Apicalypse クエリを記述。

例:
```bash
curl -X POST "https://api.igdb.com/v4/search" \
	-H "Client-ID: YOUR_CLIENT_ID" \
	-H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
	-d 'search "Sonic"; fields game.name,game.cover.image_id,platform.name,company.name; limit 20; where game != null;'
```

### バリデーション
- body: Apicalypse クエリ文字列（必須）

### レスポンス（成功 200）
検索対象ごとのオブジェクト配列を返す。

---

## POST /companies

### リクエスト
企業情報の検索。例:
```
POST https://api.igdb.com/v4/companies
Body:
search "Nintendo";
fields id,name,description,logo.image_id,developed,published,country,start_date,url;
limit 10;
```

### バリデーション
- body: Apicalypse クエリ文字列（必須）

### レスポンス（成功 200）
企業オブジェクトの配列を返す。

---

## POST /platforms

### リクエスト
プラットフォーム情報の取得。例:
```
POST https://api.igdb.com/v4/platforms
Body:
search "PlayStation";
fields id,name,abbreviation,platform_logo.image_id,generation,summary;
limit 10;
```

### バリデーション
- body: Apicalypse クエリ文字列（必須）

### レスポンス（成功 200）
プラットフォームオブジェクトの配列を返す。

---

## POST /covers

### リクエスト
カバー画像情報の取得。例:
```
POST https://api.igdb.com/v4/covers
Body:
fields image_id,url,game;
where game = 1942;
```

### バリデーション
- body: Apicalypse クエリ文字列（必須）

### レスポンス（成功 200）
```json
[
	{
		"id": 89386,
		"game": 1942,
		"image_id": "co1wyy",
		"url": "//images.igdb.com/igdb/image/upload/t_thumb/co1wyy.jpg"
	}
]
```

---

## POST /screenshots

### リクエスト
スクリーンショットの取得。例:
```
POST https://api.igdb.com/v4/screenshots
Body:
fields image_id,url,game,width,height;
where game = 1942;
```

### バリデーション
- body: Apicalypse クエリ文字列（必須）

### レスポンス（成功 200）
```json
[
	{
		"id": 9742,
		"game": 1942,
		"image_id": "mnljdjtrh44x4snmierh",
		"url": "//images.igdb.com/igdb/image/upload/t_thumb/mnljdjtrh44x4snmierh.jpg",
		"width": 1920,
		"height": 1080
	}
]
```

---

## 画像 URL
IGDB の画像は `image_id` を用い、サイズプレフィックスを指定して取得します。

例:
```
https://images.igdb.com/igdb/image/upload/{size}/{image_id}.jpg
```

利用可能なサイズ例:
- `t_thumb`, `t_cover_small`, `t_cover_big`, `t_screenshot_med`, `t_screenshot_big`, `t_1080p`, `t_original`

## 注意事項・運用上のヒント
- すべてのリクエストに `Client-ID` と `Authorization: Bearer <access_token>` を付与する必要があります。
- トークンの有効期限とレート制限に注意し、必要に応じてキャッシュや再取得ロジックを実装してください。
- Apicalypse クエリは柔軟だが、`fields` を必要最小限に絞ることでレスポンスサイズを抑えられます。

## 参考
- [IGDB API Documentation](https://api-docs.igdb.com/)
- アクセストークン取得: https://id.twitch.tv/oauth2/token

