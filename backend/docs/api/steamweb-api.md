# API設計

## 基本方針
- RESTful（読み取り中心の公開Web API）
- ベースURL: `https://api.steampowered.com`, `https://store.steampowered.com`
- 認証: API Key（クエリパラメータ `key`）
- レスポンス形式: JSON
- Rate Limit: 公式に明確な制限表記はないが、過度なリクエストは避ける（キャッシュ推奨）

## エンドポイント一覧
| Method | Path | 説明 | 認証 |
|--------|------|------|------|
| GET | https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/ | 指定ユーザーの所持ゲーム一覧を取得 | 必須（API Key） |
| GET | https://store.steampowered.com/api/appdetails?appids=${appId} | 指定アプリの詳細情報を取得 | 不要 |
| GET | https://store.steampowered.com/api/storesearch/?term={query}&cc=JP&l=english | ストア内でゲームを検索 | 不要 |

---

## GET https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/

### リクエスト
```bash
curl -s "https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key=YOUR_API_KEY&steamid=76561197960435530&include_appinfo=1&include_played_free_games=1&format=json"
```

### バリデーション
- `key`: 必須・string（取得したSteam APIキー）
- `steamid`: 必須・uint64（対象ユーザーのSteamID64）
- `include_appinfo`: 任意・int（1でアプリ情報を含める）
- `include_played_free_games`: 任意・int（1で無料ゲームも含める）

### レスポンス（成功 200）
```json
{
  "response":{
    "game_count":2,
    "games":[
      {
        "appid":570,
        "name":"Dota 2",
        "playtime_forever":12345,
        "img_icon_url":"a1b2c3d4e5f6g7h8i9j0",
        "img_logo_url":"0j9i8h7g6f5e4d3c2b1a",
        "has_community_visible_stats":true
      }
    ]
  }
}
```

---

## GET https://store.steampowered.com/api/appdetails?appids={appId}

### リクエスト
```bash
curl -s "https://store.steampowered.com/api/appdetails?appids=1"
```
### バリデーション
- `appids`: 必須・int（対象アプリのID）

### レスポンス（成功 200）
```json
{
  "1":{
    "success":true,
    "data":{
      "type":"game",
      "name":"Half-Life",
      "steam_appid":1,
      "required_age":0,
      "is_free":false,
      "detailed_description":"<strong>Half-Life</strong> is a science fiction first-person shooter developed by Valve and published by Sierra Studios. It was released in 1998 for Microsoft Windows.",
      "about_the_game":"<strong>Half-Life</strong> is a science fiction first-person shooter developed by Valve and published by Sierra Studios. It was released in 1998 for Microsoft Windows.",
      "short_description":"A science fiction first-person shooter developed by Valve and published by Sierra Studios.",
      "header_image":"https://cdn.akamai.steamstatic.com/steam/apps/1/header.jpg?t=1596561600",
      ...
    }
  }
}
```
---

## GET https://store.steampowered.com/api/storesearch/?term={query}&cc=JP&l=english

### リクエスト
```bash
curl -s "https://store.steampowered.com/api/storesearch/?term=half-life&cc=JP&l=english"
```

### バリデーション
- `term`: 必須・string（検索クエリ）
- `cc`: 任意・string（国コード、例: JP）
- `l`: 任意・string（言語コード、例: english）

### レスポンス（成功 200）
```json
{
  "total":1,
  "results":[
    {
      "id":1,
      "type":"game",
      "name":"Half-Life",
      "url":"https://store.steampowered.com/app/1/HalfLife/",
      ...
    }
  ]
}
```

---
## 注意事項・運用メモ
- `steamid` は 64-bit の SteamID（例: 76561197960265728）。Vanity URL を渡された場合は `ResolveVanityURL` を使って変換する。
- `include_appinfo=1` を指定すると `name` が返るため、UI 表示が容易になる。
- Valve のドメインでホスティングされる画像を直接参照する場合は Hotlink 利用ポリシーやキャッシュを検討する。
- 一部ゲームは画像ハッシュが空の場合や、ユーザーの公開範囲により情報が不足する可能性がある。

---

## 参考リンク
- https://developer.valvesoftware.com/wiki/Steam_Web_API#GetOwnedGames_(v0001)
