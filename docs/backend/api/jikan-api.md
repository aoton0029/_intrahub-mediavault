# API設計

## 基本方針
- RESTful API（Jikan はパブリックな外部 API への参照）
- ベースURL: `https://api.jikan.moe/v4`
- 認証: 不要（公開 API）。商用利用や高頻度アクセスは利用規約を確認すること。
- レスポンス形式: JSON

## エンドポイント一覧
| Method | Path | 説明 | 認証 |
|--------|------|------|------|
| GET | /anime | アニメの検索（クエリ指定） | 不要 |
| GET | /anime/{id}/full | 指定アニメの詳細取得（フル） | 不要 |
| GET | /anime/{id}/episodes | 指定アニメのエピソード一覧取得 | 不要 |
| GET | /anime/{id}/episodes/{episode} | 指定アニメの特定エピソード取得 | 不要 |
| GET | /anime/{id}/staff | 指定アニメのスタッフ情報取得 | 不要 |
| GET | /anime/{id}/pictures | 指定アニメの画像一覧取得 | 不要 |
| GET | /anime/{id}/videos | 指定アニメの動画（トレーラーなど）取得 | 不要 |
| GET | /manga | 漫画の検索（クエリ指定） | 不要 |
| GET | /manga/{id}/full | 指定漫画の詳細取得（フル） | 不要 |
| GET | /characters | キャラクター検索 | 不要 |
| GET | /producers | プロデューサー（会社）検索 | 不要 |
| GET | /seasons | 季節一覧取得 / `/{year}/{season}` で年・季節指定 | 不要 |

---

## GET /anime

### リクエスト
```
GET https://api.jikan.moe/v4/anime?q={query}&page={page}&limit={limit}&type={type}
```

### パラメーター
- `q`: 検索クエリ（必須）
- `page`: ページ番号（任意）
- `limit`: 1ページあたりの結果数（任意）
- `type`: タイプ（"tv", "movie", "ova", "special", "ona", "music", "cm", "pv", "tv_special"）（任意）

### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime?q=naruto&page=1"
```

### レスポンス（成功 200、概要）
```json
{
  "data": [ /* アニメオブジェクトの配列 */ ],
  "pagination": { /* ページ情報: has_next_page, current_page, items_per_page など */ }
}
```

---

## GET /anime/{id}/full

### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/full
```

### 説明
指定 ID のアニメに関する詳細情報を取得する。`images`, `trailer`, `titles`, `episodes`, `producers`, `genres` など多くのフィールドを含む。

### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/full"
```

### レスポンス（抜粋）
```json
{
  "data": {
    "mal_id": 1,
    "title": "...",
    "images": { /* ネストされた画像情報 */ },
    "trailer": { /* トレーラー情報 */ },
    "episodes": 220,
    "producers": [ /* 製作会社配列 */ ],
    "genres": [ /* ジャンル配列 */ ]
  }
}
```

---

## GET /anime/{id}/episodes

### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/episodes
```

### 説明
指定アニメのエピソード一覧を取得する。ページネーションを返す場合がある。

### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/episodes"
```

### レスポンス（概要）
```json
{
  "data": [ /* エピソードオブジェクトの配列 */ ],
  "pagination": { /* ページ情報 */ }
}
```

---

## GET /anime/{id}/pictures

### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/pictures
```

### 説明
指定アニメのスチルやプロモ画像などの一覧を取得する。

### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/pictures"
```

### レスポンス（概要）
```json
{
  "data": [ /* 画像オブジェクトの配列 */ ]
}
```

---

## GET /characters

### リクエスト
```
GET https://api.jikan.moe/v4/characters?q={query}&page={page}
```

### 説明
キャラクター名で検索し、画像やプロフィールを取得できる。

### パラメーター
- `q`: 検索クエリ
- `page`: ページ番号

### リクエスト例
```bash
curl "https://api.jikan.moe/v4/characters?q=Sakura&page=1"
```

### レスポンス（概要）
```json
{
  "data": [ /* キャラクターオブジェクト */ ],
  "pagination": { /* ページ情報 */ }
}
```

---

## GET /producers

### リクエスト
```
GET https://api.jikan.moe/v4/producers?q={query}&page={page}
```

### 説明
プロデューサー（会社）を検索し、関連アニメ情報を取得する。

### リクエスト例
```bash
curl "https://api.jikan.moe/v4/producers?q=Aniplex&page=1"
```

### レスポンス（概要）
```json
{
  "data": [ /* プロデューサーオブジェクト */ ],
  "pagination": { /* ページ情報 */ }
}
```

---

## GET /seasons

### リクエスト
```
GET https://api.jikan.moe/v4/seasons
GET https://api.jikan.moe/v4/seasons/{year}/{season}
```

### 説明
指定年・季節のアニメ一覧を取得する。引数なしで現在または利用可能な季節一覧を取得できる場合がある。

### リクエスト例
```bash
curl "https://api.jikan.moe/v4/seasons/2020/summer"
```

### レスポンス（概要）
```json
{
  "data": [ /* アニメオブジェクトの配列 */ ],
  "pagination": { /* ページ情報 */ }
}
```

---

## GET /staff

### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/staff
```

### 説明
指定アニメのスタッフ情報を取得する。

### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/staff"
```

### レスポンス（概要）
```json
{
  "data": [
    {
      "person": {
        "mal_id": 0,
        "url": "string",
        "images": {
          "jpg": {
            "image_url": "string"
          }
        },
        "name": "string"
      },
      "positions": [
        "string"
      ]
    }
  ]
}
```

---

## GET /anime/{id}/videos
### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/videos
```
### 説明
指定アニメの動画（トレーラーなど）を取得する。
### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/videos"
```
### レスポンス（概要）
```json
{
  "data": {
    "promo": [
      {
        "title": "string",
        "trailer": {
          "youtube_id": "string",
          "url": "string",
          "embed_url": "string",
          "images": {
            "image_url": "string",
            "small_image_url": "string",
            "medium_image_url": "string",
            "large_image_url": "string",
            "maximum_image_url": "string"
          }
        }
      }
    ],
    "episodes": [
      {
        "mal_id": 0,
        "url": "string",
        "title": "string",
        "episode": "string",
        "images": {
          "jpg": {
            "image_url": "string"
          }
        }
      }
    ],
    "music_videos": [
      {
        "title": "string",
        "video": {
          "youtube_id": "string",
          "url": "string",
          "embed_url": "string",
          "images": {
            "image_url": "string",
            "small_image_url": "string",
            "medium_image_url": "string",
            "large_image_url": "string",
            "maximum_image_url": "string"
          }
        },
        "meta": {
          "title": "string",
          "author": "string"
        }
      }
    ]
  }
}
```
---
## /anime/{id}/episodes
### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/episodes
```
### 説明
指定アニメのエピソード一覧を取得する。ページネーションを返す場合がある。
### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/episodes"
```
### レスポンス（概要）
```json
{
  "data": [
    {
      "mal_id": 0,
      "url": "string",
      "title": "string",
      "title_japanese": "string",
      "title_romanji": "string",
      "aired": "string",
      "score": null,
      "filler": true,
      "recap": true,
      "forum_url": "string"
    }
  ],
  "pagination": {
    "last_visible_page": 0,
    "has_next_page": true
  }
}
```
---
## /anime/{id}/episodes/{episode}
### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/episodes/{episode}
```
### 説明
指定アニメの特定エピソードを取得する。
### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/episodes/1"
```
### レスポンス（概要）
```json
{
  "data": {
    "mal_id": 0,
    "url": "string",
    "title": "string",
    "title_japanese": "string",
    "title_romanji": "string",
    "duration": 0,
    "aired": "string",
    "filler": true,
    "recap": true,
    "synopsis": "string"
  }
}
```
---
## /anime/{id}/staff
### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/staff
```
### 説明
指定アニメのスタッフ情報を取得する。
### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/staff"
```
### レスポンス（概要）
```json
{
  "data": [
    {
      "person": {
        "mal_id": 0,
        "url": "string",
        "images": {
          "jpg": {
            "image_url": "string"
          }
        },
        "name": "string"
      },
      "positions": [
        "string"
      ]
    }
  ]
}
```
---
## /anime/{id}/pictures
### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/pictures
```
### 説明
指定アニメのスチルやプロモ画像などの一覧を取得する。
### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/pictures"
```
### レスポンス（概要）
```json
{
  "data": [
    {
      "images": {
        "jpg": {
          "image_url": "string"
        }
      }
    }
  ]
}
```
---
## /manga
### リクエスト
```
GET https://api.jikan.moe/v4/manga?q={query}&page={page}&limit={limit}&type={type}
```
### 説明
漫画の検索エンドポイント。クエリ、ページネーション、タイプ指定が可能
### パラメーター
- `q`: 検索クエリ（必須）
- `page`: ページ番号（任意）
- `limit`: 1ページあたりの結果数（任意）
- `type`: タイプ（"manga", "novel", "lightnovel", "oneshot", "doujin", "manhwa", "manhua"）（任意）
- `start_date`: 開始日（任意）
- `end_date`: 終了日（任意）
### リクエスト例
```bash
curl "https://api.jikan.moe/v4/manga?q=one_piece&page=1"
```
### レスポンス（概要）
```json
{
  "data": [ /* 漫画オブジェクトの配列 */ ],
  "pagination": { /* ページ情報 */ }
}
```
---
## /anime/{id}/full
### リクエスト
```
GET https://api.jikan.moe/v4/anime/{id}/full
```
### 説明
指定 ID のアニメに関する詳細情報を取得する。`images`, `trailer`, `titles`, `episodes`, `producers`, `genres` など多くのフィールドを含む。
### リクエスト例
```bash
curl "https://api.jikan.moe/v4/anime/1/full"
```
### レスポンス（抜粋）
```json
{
  "data": {
    "mal_id": 0,
    "url": "string",
    "images": {
      "jpg": {
        "image_url": "string",
        "small_image_url": "string",
        "large_image_url": "string"
      },
      "webp": {
        "image_url": "string",
        "small_image_url": "string",
        "large_image_url": "string"
      }
    },
    "approved": true,
    "titles": [
      {
        "type": "string",
        "title": "string"
      }
    ],
    "title": "string",
    "title_english": "string",
    "title_japanese": "string",
    "title_synonyms": [
      "string"
    ],
    "type": "Manga",
    "chapters": 0,
    "volumes": 0,
    "status": "Finished",
    "publishing": true,
    "published": {
      "from": "string",
      "to": "string",
      "prop": {
        "from": {
          "day": 0,
          "month": 0,
          "year": 0
        },
        "to": {
          "day": 0,
          "month": 0,
          "year": 0
        },
        "string": "string"
      }
    },
    "score": 0.1,
    "scored_by": 0,
    "rank": 0,
    "popularity": 0,
    "members": 0,
    "favorites": 0,
    "synopsis": "string",
    "background": "string",
    "authors": [
      {
        "mal_id": 0,
        "type": "string",
        "name": "string",
        "url": "string"
      }
    ],
    "serializations": [
      {
        "mal_id": 0,
        "type": "string",
        "name": "string",
        "url": "string"
      }
    ],
    "genres": [
      {
        "mal_id": 0,
        "type": "string",
        "name": "string",
        "url": "string"
      }
    ],
    "explicit_genres": [
      {
        "mal_id": 0,
        "type": "string",
        "name": "string",
        "url": "string"
      }
    ],
    "themes": [
      {
        "mal_id": 0,
        "type": "string",
        "name": "string",
        "url": "string"
      }
    ],
    "demographics": [
      {
        "mal_id": 0,
        "type": "string",
        "name": "string",
        "url": "string"
      }
    ],
    "relations": [
      {
        "relation": "string",
        "entry": [
          {
            "mal_id": 0,
            "type": "string",
            "name": "string",
            "url": "string"
          }
        ]
      }
    ],
    "external": [
      {
        "name": "string",
        "url": "string"
      }
    ]
  }
}
```

## 使用上の注意・Tips
- レート制限に注意。大量取得時はページネーション、遅延、指数バックオフを実装すること。
- 画像はネストされたフィールド（例: `images.jpg.image_url`）なので存在チェックを行う。
- 一部エンドポイントは `pagination` オブジェクトを返す。次ページの有無は `has_next_page` を確認する。
- 公開 API のため応答のフィールドや挙動が予告なく変更され得る。重要用途ではキャッシュとフェールセーフ設計を推奨。

## 参考リンク
- [Jikan API Documentation](https://docs.api.jikan.moe/)
