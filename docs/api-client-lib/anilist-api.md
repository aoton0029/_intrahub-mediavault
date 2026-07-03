# API設計 — AniList

## 基本方針
- GraphQL API
- エンドポイント: `https://graphql.anilist.co`
- 認証: Bearer Token（オプション。ユーザー認証が必要な操作のみ）
- レスポンス形式: JSON
- Rate Limit: 90 リクエスト/分（認証なし）、90/分（認証あり）

## 主要操作一覧

| 操作 | 種別 | 説明 | 認証 |
|------|------|------|------|
| Media Search | Query | タイトル/ジャンルでアニメ・マンガを検索 | 不要 |
| Media Details | Query | 指定 ID のアニメ詳細取得 | 不要 |
| MediaList Export | Query | ユーザーのメディアリスト（視聴履歴）取得 | 必要 |

---

## Query: Media Search（アニメ検索）

### リクエスト
```bash
curl -X POST https://graphql.anilist.co \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query ($search: String, $page: Int, $perPage: Int) { Page(page: $page, perPage: $perPage) { media(search: $search, type: ANIME) { id title { romaji native english } coverImage { large } startDate { year month day } episodes genres studios { nodes { name } } status } } }",
    "variables": { "search": "進撃の巨人", "page": 1, "perPage": 20 }
  }'
```

### パラメーター（variables）
- `search`: 検索クエリ（任意）
- `type`: `ANIME` または `MANGA`（デフォルト ANIME）
- `page`: ページ番号（デフォルト 1）
- `perPage`: 件数（デフォルト 20、最大 50）
- `genre_in`: ジャンルフィルタ（配列）
- `season` / `seasonYear`: 放映季・年フィルタ

### レスポンス（成功 200）
```json
{
  "data": {
    "Page": {
      "media": [
        {
          "id": 16498,
          "title": {
            "romaji": "Shingeki no Kyojin",
            "native": "進撃の巨人",
            "english": "Attack on Titan"
          },
          "coverImage": { "large": "https://s4.anilist.co/file/anilistcdn/media/anime/cover/large/bx16498.jpg" },
          "startDate": { "year": 2013, "month": 4, "day": 7 },
          "episodes": 25,
          "genres": ["Action", "Drama", "Fantasy", "Mystery"],
          "studios": { "nodes": [{ "name": "Wit Studio" }] },
          "status": "FINISHED"
        }
      ]
    }
  }
}
```

---

## Query: Media Details（詳細取得）

### リクエスト
```bash
curl -X POST https://graphql.anilist.co \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query ($id: Int) { Media(id: $id, type: ANIME) { id title { romaji native english } description episodes duration startDate { year month day } endDate { year month day } status season seasonYear genres tags { name } studios { nodes { name } } coverImage { extraLarge } averageScore } }",
    "variables": { "id": 16498 }
  }'
```

### レスポンス（抜粋）
```json
{
  "data": {
    "Media": {
      "id": 16498,
      "title": { "romaji": "Shingeki no Kyojin", "native": "進撃の巨人", "english": "Attack on Titan" },
      "description": "Several hundred years ago, humans were nearly exterminated by giants...",
      "episodes": 25,
      "duration": 24,
      "status": "FINISHED",
      "season": "SPRING",
      "seasonYear": 2013,
      "genres": ["Action", "Drama", "Fantasy"],
      "tags": [{ "name": "Military" }, { "name": "Post-Apocalyptic" }],
      "studios": { "nodes": [{ "name": "Wit Studio" }] },
      "coverImage": { "extraLarge": "https://s4.anilist.co/..." },
      "averageScore": 84
    }
  }
}
```

---

## Query: MediaList Export（ユーザーリスト取得）

### 認証
Bearer Token（AniList OAuth2）が必要。ユーザーが AniList にログインし、アクセストークンをプラグイン設定に保存する。

### リクエスト
```bash
curl -X POST https://graphql.anilist.co \
  -H "Authorization: Bearer {access_token}" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query ($userName: String) { MediaListCollection(userName: $userName, type: ANIME) { lists { name entries { id status score(format: POINT_10_DECIMAL) startedAt { year month day } completedAt { year month day } media { id title { romaji native } episodes coverImage { large } } } } } }",
    "variables": { "userName": "YOUR_USERNAME" }
  }'
```

### レスポンス（抜粋）
```json
{
  "data": {
    "MediaListCollection": {
      "lists": [
        {
          "name": "Completed",
          "entries": [
            {
              "id": 12345,
              "status": "COMPLETED",
              "score": 9.0,
              "startedAt": { "year": 2023, "month": 4, "day": 7 },
              "completedAt": { "year": 2023, "month": 7, "day": 1 },
              "media": {
                "id": 16498,
                "title": { "romaji": "Shingeki no Kyojin" },
                "episodes": 25
              }
            }
          ]
        }
      ]
    }
  }
}
```

---

## ステータスマッピング（AniList → プラグイン内部）

| AniList status | プラグイン status | 備考 |
|---|---|---|
| CURRENT | in_progress | 視聴中 |
| PLANNING | wishlist | 視聴予定 |
| COMPLETED | completed | 完了 |
| DROPPED | dropped | 切り捨て |
| PAUSED | on_hold | 一時停止 |
| REPEATING | in_progress | 再視聴中 |

---

## エラーレスポンス

```json
{
  "errors": [
    {
      "message": "Not Found.",
      "status": 404,
      "locations": [{ "line": 2, "column": 3 }]
    }
  ]
}
```

| ステータス | 説明 |
|------------|------|
| 400 | GraphQL クエリ構文エラー |
| 401 | 認証エラー（Bearer トークン不正） |
| 404 | 指定リソースが存在しない |
| 429 | レート制限超過 |

---

## 使用上の注意・Tips
- `description` フィールドには HTML タグが含まれる場合があるため、DOMParser 等でプレーンテキストに変換すること。
- `score` は AniList のスケール設定（10点/100点等）に依存する。`format: POINT_10_DECIMAL` を指定して統一する。
- `averageScore` は 0〜100 の整数。`ratings` マップに `anilist: averageScore / 10` として格納する。
- GraphQL ではフィールドを必要最小限に絞り、パフォーマンスとデータ転送量を最適化する。
- `MediaListCollection` は認証なしでも公開設定のユーザーは取得可能。プライベート設定の場合は 401 が返る。

## 参考リンク
- [AniList GraphQL Explorer](https://anilist.co/graphiql)
- [AniList API Documentation](https://anilist.gitbook.io/anilist-apiv2-docs/)
- [AniList API GitHub](https://github.com/AniList/ApiV2-GraphQL-Docs)
