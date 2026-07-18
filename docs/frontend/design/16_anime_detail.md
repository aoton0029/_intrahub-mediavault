# 16. アニメ 詳細

対応モック: `docs/frontend/ui/16_anime_detail.html`

本画面は `00_common.md` §5「詳細画面共通パターン」に準拠する。詳細は `20_movie_detail.md` の共通記述も参照。ここでは映画（基準）との差分のみ記載する。

## 差分サマリ

- ルート: `/media/:id`（`mediaType: anime`）。パンくずは「メディア / アニメ」。**編集ボタンなし**（モックにタイトルバーaction無し）
- 任意セクション有無: **種別固有情報 ✗**（`PropertyList`セクションとしては非表示。ただし`GET /items/{id}`は`detail`オブジェクト自体は返す。概要から直接シーズン構成へ） / エピソード構成 ✓ / スタッフ ✓ / 配信 ✓
- セクション順序: 概要 → **シーズン構成** → スタッフ → 関連作品 → 配信 → リソース
- ステータスラベルは「未着手/視聴中/視聴済」。内部値（`status`）は`data-model.md`の`ItemStatus`定義により`not_started` / `in_progress` / `completed`の3値で確定（モックHTMLの`data-status="done"`は表示上の仮の値であり、実装時は`done`ではなく`completed`をAPI送受信値として使う）
- `GET /items/{id}`のレスポンスは`ItemDetail`（`Item`全フィールド + `detail` + `tags` + `categories` + `calibre_links` + `streaming_links`）。旧版の本節では`streaming_links`の記載が漏れていたため追記

## anime用 `detail` 形状

`GET /items/{id}`の`detail`フィールド（`PropertyList`としては非表示だが、レスポンスには含まれる。Annictの作品情報 + Jikan(MyAnimeList)のあらすじ・ジャンル等をマージしたもの）:

```ts
interface AnimeDetail {
  episodes: number | null;
  status: string | null;         // 例: "Finished Airing"
  season: string | null;         // 例: "2019-spring"
  year: number | null;
  studios: string[];
  source: string | null;         // 例: "Manga"
  duration: string | null;       // 例: "23 min per ep"
  trailer_url: string | null;
  genres: string[];
  rating: number | null;
  url: string | null;            // MyAnimeList URL
  alternative_titles: string[];
}
```

- 手動作成（`source: "manual"`）の場合は`detail`自体が`null`

## エピソード/シーズン構成（`GroupList`）

`ItemGroup`・`ItemEpisode`はAPIレスポンスのフィールドをそのまま使う（snake_case、変換層なし）:

```ts
interface ItemGroup {
  id: string;
  item_id: string;
  parent_item_id: string | null;
  group_type: 'season' | 'volume' | 'chapter';  // animeは常に 'season'
  group_name: string;       // 例: 「シーズン1」
  number: number | null;
  display_order: number;
  created_at: string;
  updated_at: string;
}

interface ItemEpisode {
  id: string;
  group_id: string;
  episode_number: number;
  title: string | null;
  original_title: string | null;
  air_date: string | null;
  description: string | null;
  created_at: string;
  updated_at: string;
}
```

- API: `GET/POST /items/{id}/groups`（`group_type: season`）、各グループの話数は `GET/POST /groups/{group_id}/episodes`（`volume` タイプには使用不可）
- グループ単位で「話数を追加」ボタン、リスト末尾に「シーズンを追加」ボタン

## API連携（映画からの差分のみ）

- 種別固有情報の `PropertyList` は表示しない（`detail`データ自体は取得済みだがUI上非表示）
- 詳細取得: `GET /items/{id}` → `ApiOk<ItemDetail>`（`Item`全フィールド + `detail`（上記形状） + `tags: TagRef[]` + `categories: CategoryRef[]` + `calibre_links` + `streaming_links: ItemStreamingLink[]`）
- ステータス変更: `PATCH /items/{id}/status`（`UpdateStatusRequest`: `status`必須, `consumed_date`任意。`status`値は`not_started`/`in_progress`/`completed`）→ `ApiOk<Item>`
- 評価・お気に入りなど: `PATCH /items/{id}`（`UpdateItemRequest`、全フィールドoptional。`media_type`/`source`/`external_id`は変更不可）→ `ApiOk<Item>`
- 配信: `GET/POST /items/{id}/streaming-links`, `DELETE /items/{id}/streaming-links/{link_id}`（`ItemStreamingLink`: `{id, item_id, platform, url, created_at}`、`platform`は`netflix`/`amazon_prime`/`disney_plus`/`dmm_tv`/`apple_tv`）
- スタッフ: `GET /items/{id}/staff` → `ApiOk<ItemStaff[]>`（取得）、`POST/DELETE /items/{id}/staff`（追加/解除）。`Staff = { id, external_id, name, image_url, created_at }`, `ItemStaff = { id, item_id, staff_id, role, character_name }`
- 関連作品: 取得 `GET /items/{id}/relations` → `ApiOk<ItemRelation[]>`、作成 `POST /item-relations { item_id, related_item_id, relation_type }`（`relation_type`は`reference`/`dlc`のみ）、解除 `DELETE /item-relations/{id}`（`ItemRelation.id`を使う。`item_id`ではない）。`ItemRelation = { id, item_id, related_item_id, relation_type, created_at }`
- マイリスト所属: 取得 `GET /items/{id}/mylists` → `ApiOk<Mylist[]>`（`Mylist = { id, name, created_at }`）、追加 `POST /mylists/{id}/items { item_id }`、解除 `DELETE /mylists/{id}/items/{item_id}`
- リソース: `POST/DELETE /items/{id}/links`, `/items/{id}/files`, `/items/{id}/trailers`。`ItemLink = { id, item_id, url, label, created_at }`, `ItemFile = { id, item_id, path, label, file_type: 'pdf'|'image'|'other', calibre_book_id, created_at }`, `ItemTrailer = { id, item_id, url, label, created_at }`（pdfのみ`PATCH /items/{id}/files/{file_id}/calibre-link`でCalibre連携）
- タグ/カテゴリのAPIは `20_movie_detail.md` §6と同一

参照: [items.md](../../backend/mediavault-api/items.md#detailsmedia_type別json形状), [item-groups.md](../../backend/mediavault-api/item-groups.md), [item-episodes.md](../../backend/mediavault-api/item-episodes.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [staff.md](../../backend/mediavault-api/staff.md), [mylists.md](../../backend/mediavault-api/mylists.md)（その他は `20_movie_detail.md` の参照リンクを参照）
