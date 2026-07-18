# 23. 小説 詳細

対応モック: `docs/frontend/ui/23_novel_detail.html`

本画面は `00_common.md` §5「詳細画面共通パターン」に準拠する。構造は `22_manga_detail.md`（漫画）と完全に同一パターンのため、詳細はそちらを参照。ここでは映画（基準）との差分のみ記載する。

## 差分サマリ

- ルート: `/media/:id`（`mediaType: novel`）。パンくずは「メディア / 小説」。編集ボタンあり
- `doc-original` は発行年のみ
- 任意セクション有無: 種別固有情報 ✓（4項目） / **巻構成 ✓** / スタッフ ✗ / 配信 ✗
- セクション順序: 概要 → 種別固有情報 → 巻構成 → 関連作品 → リソース
- ステータスラベルは「未着手/読書中/読了」（内部値は`not_started`/`in_progress`/`completed`。`20_movie_detail.md`§6を参照）
- リソースタブは「リンク」「ファイル」のみ（トレーラータブなし）
- 外部API識別情報例: `API(楽天ブックス) / external_id: {id}`

## 種別固有情報（`prop-row` 4項目）

著者 / 出版社 / ISBN / シリーズ名（`MangaDetail` と同型）

データ型はAPIレスポンスのフィールド名をそのまま使う（snake_case、変換層なし）:

```ts
interface NovelDetail {
  media_type: 'novel';
  detail: {
    authors: string;
    publisher: string;
    isbn: string;
    series_name?: string;
  } | null;
}
```

## 巻構成（`GroupList`、`group_type: volume`）

未刊行の巻は `.muted` 表示（例: 「第三部(未刊行・予約中)」）。`22_manga_detail.md` と同一パターン（`ItemGroup`型も同一）。

## API連携

`22_manga_detail.md` §API連携と同一（配信・スタッフAPIは呼ばない、リソースはリンク・ファイルのみ、関連作品・マイリスト所属は`20_movie_detail.md`§6と同一）。

参照: `22_manga_detail.md` の参照リンクを参照（[item-groups.md](../../backend/mediavault-api/item-groups.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [mylists.md](../../backend/mediavault-api/mylists.md)）
