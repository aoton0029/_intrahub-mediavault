# API設計 - Annict API

## 基本方針
- RESTful API
- ベースURL: `https://api.annict.com`
- 認証: アクセストークン（`access_token` クエリパラメータ、または `Authorization: Bearer <token>` ヘッダ）
- レスポンス形式: JSON
- ページネーション: `page` / `per_page`（デフォルト25件、最大50件）。レスポンスに `total_count` / `next_page` / `prev_page` を含む
- `fields` パラメータで返却フィールドを絞り込み可能（例: `fields=id,title`）

## エンドポイント一覧
| Method | Path | 説明 | 認証 |
|--------|------|------|------|
| GET | /v1/works | 作品を検索・取得する | 必要 |
| GET | /v1/episodes | エピソードを検索・取得する | 必要 |
| GET | /v1/series | シリーズを検索・取得する | 必要 |
| GET | /v1/characters | キャラクターを検索・取得する | 必要 |
| GET | /v1/people | 人物を検索・取得する | 必要 |
| GET | /v1/organizations | 団体（制作会社等）を検索・取得する | 必要 |
| GET | /v1/casts | キャスト（キャラクターと声優の紐付け）を検索・取得する | 必要 |
| GET | /v1/staffs | スタッフ（作品と人物/団体の紐付け）を検索・取得する | 必要 |

---

## GET /v1/works

### メソッド
GET

### URL
https://api.annict.com/v1/works

### 説明
Annictに登録されているアニメ作品情報を取得する。

### パラメータ
- `access_token` (string): 認証トークン（必須）
- `fields` (string): 返却フィールドの絞り込み（例: `id,title`）
- `filter_ids` (string): 作品IDで絞り込み（例: `1,2,3`）
- `filter_season` (string): 放送シーズンで絞り込み（例: `2016-spring`）
- `filter_title` (string): タイトルキーワードで絞り込み
- `page` (integer): ページ番号
- `per_page` (integer): 1ページあたりの件数（デフォルト25、最大50）
- `sort_id` (string): IDでソート（`asc` / `desc`）
- `sort_season` (string): シーズンでソート（`asc` / `desc`）
- `sort_watchers_count` (string): 視聴者数でソート（`asc` / `desc`）

### レスポンスフィールド
- `id`: 作品ID
- `title` / `title_kana`: タイトル / 読み仮名
- `media` / `media_text`: メディア種別（tv, ova, movie, web, other）
- `season_name`: 放送シーズン
- `released_on`: 公開日
- `official_site_url`: 公式サイトURL
- `twitter_username`: 公式Twitterアカウント
- `episodes_count`: 話数
- `watchers_count`: 視聴者数
- `images.recommended_url`: 推奨画像URL

### リクエスト例
```bash
curl "https://api.annict.com/v1/works?access_token=YOUR_TOKEN&filter_title=SHIROBAKO"
```

### レスポンス例
```json
{
  "works": [
    {
      "id": 4168,
      "title": "SHIROBAKO",
      "media": "tv",
      "season_name": "2014-autumn",
      "episodes_count": 24,
      "watchers_count": 1254
    }
  ],
  "total_count": 1,
  "next_page": null,
  "prev_page": null
}
```

---

## GET /v1/episodes

### メソッド
GET

### URL
https://api.annict.com/v1/episodes

### 説明
Annictに登録されているエピソード情報を取得する。

### パラメータ
- `access_token` (string): 認証トークン（必須）
- `fields` (string): 返却フィールドの絞り込み
- `filter_ids` (string): エピソードIDで絞り込み（例: `1,2,3`）
- `filter_work_id` (integer): 作品IDで絞り込み
- `page` (integer): ページ番号
- `per_page` (integer): 1ページあたりの件数（デフォルト25、最大50）
- `sort_id` (string): IDでソート（`asc` / `desc`）
- `sort_sort_number` (string): 話数順でソート（`asc` / `desc`）

### レスポンスフィールド
- `id`: エピソードID
- `number` / `number_text`: 話数 / 表示用話数
- `sort_number`: ソート用の話数
- `title`: サブタイトル
- `records_count` / `record_comments_count`: 記録数 / コメント付き記録数
- `work`: 紐づく作品情報（Worksと同様のオブジェクト）
- `prev_episode` / `next_episode`: 前後のエピソード情報

### リクエスト例
```bash
curl "https://api.annict.com/v1/episodes?access_token=YOUR_TOKEN&filter_work_id=4168"
```

### レスポンス例
```json
{
  "episodes": [
    {
      "id": 45,
      "number": null,
      "number_text": "第2話",
      "sort_number": 2,
      "title": "殺戮の夢幻迷宮",
      "records_count": 0,
      "record_comments_count": 0,
      "work": { "id": 4168, "title": "SHIROBAKO" },
      "prev_episode": null,
      "next_episode": null
    }
  ],
  "total_count": 1
}
```

---

## GET /v1/series

### メソッド
GET

### URL
https://api.annict.com/v1/series

### 説明
Annictに登録されているシリーズ（作品をまとめる単位）情報を取得する。

### パラメータ
- `access_token` (string): 認証トークン（必須）
- `fields` (string): 返却フィールドの絞り込み（例: `id,name`）
- `filter_ids` (string): シリーズIDで絞り込み（例: `1,2,3`）
- `filter_name` (string): シリーズ名で絞り込み
- `page` (integer): ページ番号
- `per_page` (integer): 1ページあたりの件数（デフォルト25、最大50）
- `sort_id` (string): IDでソート（`asc` / `desc`）

### レスポンスフィールド
- `id`: シリーズID
- `name`: 日本語名
- `name_ro`: ローマ字表記
- `name_en`: 英語名

### リクエスト例
```bash
curl "https://api.annict.com/v1/series?access_token=YOUR_TOKEN&filter_ids=65"
```

### レスポンス例
```json
{
  "series": [
    {
      "id": 65,
      "name": "ソードアート・オンライン",
      "name_ro": "Sword Art Online",
      "name_en": "Sword Art Online"
    }
  ],
  "total_count": 1,
  "next_page": null,
  "prev_page": null
}
```

---

## GET /v1/characters

### メソッド
GET

### URL
https://api.annict.com/v1/characters

### 説明
Annictに登録されているキャラクター情報を取得する。

### パラメータ
- `access_token` (string): 認証トークン（必須）
- `fields` (string): 返却フィールドの絞り込み（例: `id,name`）
- `filter_ids` (string): キャラクターIDで絞り込み（例: `1,2,3`）
- `filter_name` (string): キャラクター名で絞り込み
- `page` (integer): ページ番号
- `per_page` (integer): 1ページあたりの件数（デフォルト25、最大50）
- `sort_id` (string): IDでソート（`asc` / `desc`）

### レスポンスフィールド
- `id` / `name` / `name_kana` / `name_en`: ID・名前・読み・英語名
- `nickname` / `nickname_en`: 愛称
- `birthday` / `birthday_en` / `age` / `age_en`: 誕生日・年齢
- `blood_type` / `blood_type_en`: 血液型
- `height` / `height_en` / `weight` / `weight_en`: 身長・体重
- `nationality` / `nationality_en`: 国籍
- `occupation` / `occupation_en`: 職業
- `description` / `description_en` / `description_source` / `description_source_en`: 説明・出典
- `favorite_characters_count`: お気に入り数
- `series`: 紐づくシリーズ情報（`id`, `name`, `name_ro`, `name_en`）

### リクエスト例
```bash
curl "https://api.annict.com/v1/characters?access_token=YOUR_TOKEN&filter_ids=7510"
```

### レスポンス例
```json
{
  "characters": [
    {
      "id": 7510,
      "name": "アスナ",
      "name_en": "Asuna",
      "series": { "id": 65, "name": "ソードアート・オンライン" }
    }
  ],
  "total_count": 1,
  "next_page": null,
  "prev_page": null
}
```

---

## GET /v1/people

### メソッド
GET

### URL
https://api.annict.com/v1/people

### 説明
Annictに登録されている人物（声優・監督等）情報を取得する。

### パラメータ
- `access_token` (string): 認証トークン（必須）
- `fields` (string): 返却フィールドの絞り込み（例: `id,name`）
- `filter_ids` (string): 人物IDで絞り込み（例: `1,2,3`）
- `filter_name` (string): 人物名で絞り込み
- `page` (integer): ページ番号
- `per_page` (integer): 1ページあたりの件数（デフォルト25、最大50）
- `sort_id` (string): IDでソート（`asc` / `desc`）

### レスポンスフィールド
- `id` / `name` / `name_kana` / `name_en`: ID・名前・読み・英語名
- `nickname` / `nickname_en`: 愛称
- `gender_text`: 性別
- `url` / `url_en`: 公式サイト
- `wikipedia_url` / `wikipedia_url_en`: Wikipediaリンク
- `twitter_username` / `twitter_username_en`: Twitterアカウント
- `birthday` / `blood_type` / `height` / `prefecture`: 誕生日・血液型・身長・出身地
- `favorite_people_count` / `casts_count` / `staffs_count`: お気に入り数・出演数・スタッフ参加数

### リクエスト例
```bash
curl "https://api.annict.com/v1/people?access_token=YOUR_TOKEN&filter_ids=7118"
```

### レスポンス例
```json
{
  "people": [
    {
      "id": 7118,
      "name": "水瀬いのり",
      "name_en": "Minase, Inori",
      "birthday": "1995-12-02",
      "casts_count": 58,
      "prefecture": { "id": 13, "name": "東京都" }
    }
  ],
  "total_count": 1
}
```

---

## GET /v1/organizations

### メソッド
GET

### URL
https://api.annict.com/v1/organizations

### 説明
Annictに登録されている団体（アニメ制作会社等）情報を取得する。

### パラメータ
- `access_token` (string): 認証トークン（必須）
- `fields` (string): 返却フィールドの絞り込み
- `filter_ids` (string): 団体IDで絞り込み
- `filter_name` (string): 団体名で絞り込み
- `page` (integer): ページ番号
- `per_page` (integer): 1ページあたりの件数（デフォルト25、最大50）
- `sort_id` (string): IDでソート（`asc` / `desc`）

### レスポンスフィールド
- `id` (integer): 団体ID
- `name` / `name_kana` / `name_en` (string): 名称
- `url` / `url_en` (string): 公式サイトURL
- `wikipedia_url` / `wikipedia_url_en` (string): Wikipediaリンク
- `twitter_username` / `twitter_username_en` (string): Twitterアカウント
- `favorite_organizations_count` (integer): お気に入り数
- `staffs_count` (integer): 参加作品数

### リクエスト例
```bash
curl "https://api.annict.com/v1/organizations?access_token=YOUR_TOKEN&filter_ids=3"
```

### レスポンス例
```json
{
  "organizations": [
    {
      "id": 3,
      "name": "P.A.WORKS",
      "name_en": "P.A.WORKS",
      "url": "http://www.pa-works.jp/",
      "twitter_username": "PAWORKS_info",
      "favorite_organizations_count": 81,
      "staffs_count": 23
    }
  ],
  "total_count": 1,
  "next_page": null,
  "prev_page": null
}
```

---

## GET /v1/casts

### メソッド
GET

### URL
https://api.annict.com/v1/casts

### 説明
Annictに登録されているキャスト情報（作品・キャラクター・人物の紐付け）を取得する。

### パラメータ
- `access_token` (string): 認証トークン（必須）
- `fields` (string): 返却フィールドの絞り込み
- `filter_ids` (string): キャストIDで絞り込み（例: `1,2,3`）
- `filter_work_id` (integer): 作品IDで絞り込み
- `page` (integer): ページ番号
- `per_page` (integer): 1ページあたりの件数（デフォルト25、最大50）
- `sort_id` (string): IDでソート（`asc` / `desc`）
- `sort_sort_number` (string): ソート番号でソート（`asc` / `desc`）

### レスポンスフィールド
- `id`: キャストID
- `name` / `name_en`: 声優名（日本語 / 英語）
- `sort_number`: ソート番号
- `work`: 紐づく作品情報
- `character`: 紐づくキャラクター情報
- `person`: 紐づく人物情報

### リクエスト例
```bash
curl "https://api.annict.com/v1/casts?access_token=YOUR_TOKEN&filter_work_id=5459"
```

### レスポンス例
```json
{
  "casts": [
    {
      "id": 43414,
      "name": "東山奈央",
      "name_en": "Touyama, Nao",
      "sort_number": 10,
      "work": { "id": 5459, "title": "ゆるキャン△" },
      "character": { "id": 32268, "name": "志摩リン" },
      "person": { "id": 1411, "name": "東山奈央" }
    }
  ],
  "total_count": 1,
  "next_page": null,
  "prev_page": null
}
```

---

## GET /v1/staffs

### メソッド
GET

### URL
https://api.annict.com/v1/staffs

### 説明
Annictに登録されているスタッフ情報（作品と人物・団体の紐付け）を取得する。

### パラメータ
- `access_token` (string): 認証トークン（必須）
- `fields` (string): 返却フィールドの絞り込み
- `filter_ids` (string): スタッフIDで絞り込み
- `filter_work_id` (integer): 作品IDで絞り込み
- `page` (integer): ページ番号
- `per_page` (integer): 1ページあたりの件数（デフォルト25、最大50）
- `sort_id` (string): IDでソート（`asc` / `desc`）
- `sort_sort_number` (string): ソート番号でソート（`asc` / `desc`）

### レスポンスフィールド
- `id` (integer): スタッフID
- `name` / `name_en` (string): 名前
- `role_text` (string): 主な役割（監督、アニメーション制作等）
- `role_other` / `role_other_en` (string): その他の役割
- `sort_number` (integer): ソート番号
- `work` (object): 紐づく作品情報
- `person` / `organization` (object): 紐づく人物または団体情報（いずれか一方）

### リクエスト例
```bash
curl "https://api.annict.com/v1/staffs?access_token=YOUR_TOKEN&filter_work_id=4308"
```

### レスポンス例
```json
{
  "staffs": [
    {
      "id": 35319,
      "name": "京都アニメーション",
      "role_text": "アニメーション制作",
      "sort_number": 200,
      "work": { "id": 4308, "title": "響け！ユーフォニアム" },
      "organization": { "id": 74, "name": "京都アニメーション" }
    }
  ],
  "total_count": 1,
  "next_page": null,
  "prev_page": null
}
```

---

## 使用上の注意・Tips
- アクセストークンを直接リポジトリに含めない（`.env` またはシークレット管理を利用）。
- `per_page` は最大50件までのため、全件取得にはページネーション（`page` / `next_page`）の実装が必要。
- `staffs` の `person` / `organization` はどちらか一方のみ設定される。
- `fields` パラメータでレスポンスサイズを絞り込むことでAPI呼び出し効率を上げられる。
- レート制限に注意し、大量アクセス時はリトライ／バックオフを実装する。

## 参考リンク
- [Annict Developers - REST API v1](https://developers.annict.com/docs/rest-api/v1)
