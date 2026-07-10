# 16. アニメ 詳細

対応モック: `docs/frontend/ui/16_anime_detail.html`

本画面は `00_common.md` §5「詳細画面共通パターン」に準拠する。詳細は `20_movie_detail.md` の共通記述も参照。ここでは映画（基準）との差分のみ記載する。

## 差分サマリ

- ルート: `/media/:id`（`mediaType: anime`）。パンくずは「一般メディア / アニメ」。**編集ボタンなし**（モックにタイトルバーaction無し）
- 任意セクション有無: **種別固有情報 ✗**（概要から直接シーズン構成へ） / エピソード構成 ✓ / スタッフ ✓ / 配信 ✓
- セクション順序: 概要 → **シーズン構成** → スタッフ → 関連作品 → 配信 → リソース
- ステータスラベルは「未着手/視聴中/視聴済」
- 外部API識別情報例: `API(Jikan) / external_id: {id}`

## エピソード/シーズン構成（`GroupList`）

```ts
interface EpisodeGroup {
  id: string;
  groupType: 'season';
  label: string;          // 例: 「シーズン1」
  episodes: { number: number; title: string }[];
}
```

- API: `GET/POST /items/{id}/groups`（`group_type: season`）、各グループの話数は `GET/POST /groups/{group_id}/episodes`（`volume` タイプには使用不可）
- グループ単位で「話数を追加」ボタン、リスト末尾に「シーズンを追加」ボタン

## API連携（映画からの差分のみ）

- 種別固有情報の `PropertyList` は表示しない
- その他（ステータス/評価/お気に入り/タグ/カテゴリ/マイリスト/関連作品/配信/リソース/スタッフ）のAPIは `20_movie_detail.md` §6と同一

参照: [item-groups.md](../../backend/mediavault-api/item-groups.md), [item-episodes.md](../../backend/mediavault-api/item-episodes.md)（その他は `20_movie_detail.md` の参照リンクを参照）
