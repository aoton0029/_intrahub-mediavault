# 22. 漫画 詳細

対応モック: `docs/frontend/ui/22_manga_detail.html`

本画面は `00_common.md` §5「詳細画面共通パターン」に準拠する。詳細は `20_movie_detail.md` の共通記述も参照。ここでは映画（基準）との差分のみ記載する。

## 差分サマリ

- ルート: `/media/:id`（`mediaType: manga`）。パンくずは「一般メディア / 漫画」。編集ボタンあり
- `doc-original` は原題ではなく発行年のみ（例: 「2022年」。原題フィールドが無いメディアタイプ）
- 任意セクション有無: 種別固有情報 ✓（4項目） / **巻構成 ✓**（エピソードではなく「巻」単位） / **スタッフ ✗** / **配信 ✗**
- セクション順序: 概要 → 種別固有情報 → 巻構成 → 関連作品 → リソース（スタッフ・配信セクションなし）
- ステータスラベルは「未着手/読書中/読了」
- リソースタブは「リンク」「ファイル」のみ（**トレーラータブなし**、`resource-tabs` に `tab-trailers` を含めない）
- 外部API識別情報例: `API(楽天ブックス) / external_id: {id}`

## 種別固有情報（`prop-row` 4項目）

著者 / 出版社 / ISBN / シリーズ名

```ts
interface MangaDetail extends ItemDetailBase {
  mediaType: 'manga';
  detail: {
    authors: string;
    publisher: string;
    isbn: string;
    seriesName?: string;
  };
}
```

## 巻構成（`GroupList`、`group_type: volume`）

`volume` タイプは話数（episodes）を持たず、各巻自体が1つのグループ項目として `episode-row` に相当する行になる（未刊行の巻は `.muted` クラスでグレー表示）。

```ts
interface VolumeGroup {
  id: string;
  groupType: 'volume';
  volumes: { number: number; title: string; unpublished?: boolean }[];
}
```

- API: `GET/POST /items/{id}/groups`（`group_type: volume`）

## API連携（映画からの差分）

- 配信・スタッフのAPIは呼ばない（セクション自体が存在しない）
- リソースの `POST/DELETE /items/{id}/links`, `/items/{id}/files` のみ（`/items/{id}/trailers` は呼ばない）

参照: [item-groups.md](../../backend/mediavault-api/item-groups.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md)
