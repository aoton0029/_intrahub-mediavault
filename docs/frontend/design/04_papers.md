# 04. 論文・文献（一覧）

対応モック: `docs/frontend/ui/04_papers.html`

## 1. 画面概要 / ルート

論文・文献を書誌情報中心の行リストで一覧表示する画面。カードグリッドではなく `LiteratureList` を使う点が `02_general_media.md` / `03_academic_books.md` との最大の違い。ルート: `/papers`。サイドバー「論文・文献」がactive。

## 2. レイアウト構成

```
<AppShell>
  <Titlebar title="論文・文献" action={<Link to="/papers/new">＋ 作品を追加</Link>} />
  <Content>
    <FilterBar>
      <Chip active>すべて</Chip>
      <Chip>❤ お気に入り</Chip>
      <Chip>🏷️ タグで絞り込み</Chip>
      <Chip>📁 カテゴリで絞り込み</Chip>
      <SortSelectInline options={[追加日順, 更新日順, タイトル順, 発売日順]} />  {/* .search-box の中にselectを埋め込む特殊パターン */}
      <SearchBox placeholder="タイトルで検索…" />
    </FilterBar>

    <LiteratureList>
      <LiteratureRow
        title favorite byline={[journalName, doi]} tagList? />
    </LiteratureList>

    <LoadMoreSentinel />
  </Content>
</AppShell>
```

## 3. 表示データ / Props型

```ts
interface PaperListItem {
  id: string;
  title: string;
  doi: string;
  isFavorite: boolean;
  tags?: string[];        // 一部のみ表示（例: 「積読」）
}
```

## 4. 画面固有コンポーネント

- `SortSelectInline`: `.search-box` の見た目のまま `<select>` を内包する特殊パターン（`00_common.md` の `sort-select` とは別実装。論文一覧のみで使用）

## 5. インタラクション仕様

- タグ/カテゴリの絞り込みチップはこの画面では「未選択」表示のみがモックにあり（`02_general_media.md` と異なり選択済み状態の例は無い）。クリックで選択肢を開くUIとして実装
- `LiteratureRow` はクリックで `18_paper_detail.md` へ遷移

## 6. API連携

- `GET /items?media_type=paper&is_favorite=...&tag_id=...&category_id=...&title=...&page=...&limit=...`（モックHTMLコメントに準拠）

参照: [items.md](../../backend/mediavault-api/items.md)

## 7. Tailwindスタイリング上の注意

- `.byline .sep` は中黒（・）区切り。DOIは `font-mono`
- `.fav-mark.is-active` のみ `--color-favorite` を適用、非アクティブ時は `--color-text-faint`
