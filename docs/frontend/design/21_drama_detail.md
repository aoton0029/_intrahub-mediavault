# 21. ドラマ 詳細

対応モック: `docs/frontend/ui/21_drama_detail.html`

本画面は `00_common.md` §5「詳細画面共通パターン」に準拠する。詳細は `20_movie_detail.md` の共通記述も参照。ここでは映画（基準）との差分のみ記載する。

## 差分サマリ

- ルート: `/media/:id`（`mediaType: drama`）。パンくずは「メディア / ドラマ」。編集ボタンあり
- 任意セクション有無: 種別固有情報 ✓（7項目、全画面中最多） / エピソード構成 ✓（複数シーズン） / スタッフ ✓ / 配信 ✓ — **8詳細画面中唯一、全任意セクションを持つ**
- セクション順序: 概要 → 種別固有情報 → **シーズン構成** → スタッフ → 関連作品 → 配信 → リソース
- ステータスラベルは「未着手/視聴中/視聴済」（内部値は`not_started`/`in_progress`/`completed`。`20_movie_detail.md`§6を参照）

## 種別固有情報（`prop-row` 7項目）

放送局 / 原語 / 放送開始日 / 放送終了日（放送中の場合は `.val.muted` で「放送中」表示）/ シーズン数 / 話数 / ジャンル

データ型はAPIレスポンスのフィールド名をそのまま使う（snake_case、変換層なし）:

```ts
interface DramaDetail {
  media_type: 'drama';
  detail: {
    networks: string[];
    original_language: string;
    first_air_date: string;
    last_air_date: string | null;   // nullは放送中 → 「放送中」表示
    number_of_seasons: number;
    number_of_episodes: number;
    genres: string[];
  } | null;
}
```

## シーズン構成（`GroupList`、複数グループ例）

モックは2シーズン分の `group-block` を表示（各シーズンのheaderに「(全n話)」を含む）。`GroupList` はグループを複数件レンダーできる設計とする。`ItemGroup`/`ItemEpisode`の型は `16_anime_detail.md` と同一。

## API連携（映画からの差分のみ）

- 種別固有情報フィールドが異なる以外は `20_movie_detail.md` §6と同一
- シーズン構成のAPIは `16_anime_detail.md` と同一（`group_type: season`、`ItemGroup`/`ItemEpisode`型も同一）

参照: [item-groups.md](../../backend/mediavault-api/item-groups.md), [item-episodes.md](../../backend/mediavault-api/item-episodes.md)（その他は `20_movie_detail.md` の参照リンクを参照）
