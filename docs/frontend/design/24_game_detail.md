# 24. ゲーム 詳細

対応モック: `docs/frontend/ui/24_game_detail.html`

本画面は `00_common.md` §5「詳細画面共通パターン」に準拠する。詳細は `20_movie_detail.md` の共通記述も参照。ここでは映画（基準）との差分のみ記載する。

## 差分サマリ

- ルート: `/media/:id`（`mediaType: game`）。パンくずは「一般メディア / ゲーム」。編集ボタンあり
- 任意セクション有無: 種別固有情報 ✓（5項目） / **エピソード・巻構成 ✗** / **スタッフ ✗** / **配信 ✗**
- セクション順序: 概要 → 種別固有情報 → 関連作品 → リソース（構成・スタッフ・配信セクションなし）
- ステータスラベルは「未着手/プレイ中/クリア済」
- リソースタブは「リンク」「ファイル」「トレーラー」の3種（movie/anime/dramaと同じ）
- 外部API識別情報例: `API(Steam) / external_id: {id}`
- 関連作品の `relation_type` に `dlc` が使われる実例あり（「エコーズ・オブ・ヴァリア: 追想の章」）
- 【将来検討事項】`detail.screenshots[]` のギャラリー表示に対応するUIコンポーネントは `_shared.css` に未定義のため、本モックの対象外（モックHTMLコメントに明記）

## 種別固有情報（`prop-row` 5項目）

プラットフォーム / 開発元 / 発売元 / Metacritic スコア / ジャンル

```ts
interface GameDetail extends ItemDetailBase {
  mediaType: 'game';
  detail: {
    platforms: string;       // 例: "PC / PS5"
    developers: string;
    publishers: string;
    metacriticScore?: number;
    genres: string;
  };
}
```

## API連携（映画からの差分）

- エピソード/巻構成・スタッフ・配信のAPIは呼ばない
- リソースは `POST/DELETE /items/{id}/links`, `/items/{id}/files`, `/items/{id}/trailers`（movie等と同じ3種）
- 関連作品の `relation_type` は `reference` に加え `dlc` の実データ例あり（DLC本体との紐付けに使用）

参照: [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md), [item-trailers.md](../../backend/mediavault-api/item-trailers.md), [item-relations.md](../../backend/mediavault-api/item-relations.md)
