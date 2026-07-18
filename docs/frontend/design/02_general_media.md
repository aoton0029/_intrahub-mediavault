# 02. メディア（一覧）

対応モック: `docs/frontend/ui/02_general_media.html`（`03_academic_books.html` とほぼ同一構造、差分は本文末尾に記載）

## 1. 画面概要 / ルート

アニメ・映画・ドラマ・漫画・小説・ゲームのメディアをカードグリッドで一覧表示し、絞り込み・並び替え・検索を行う画面。ルート: `/media`。サイドバーの「メディア > すべて」がactive。

## 2. レイアウト構成

```
<AppShell>
  <Titlebar title="メディア" action={<Link to="/media/search">＋ 作品を追加</Link>} />
  <Content>
    <FilterToolbar>
      <FilterBar>
        <Chip active>すべて</Chip>
        <Chip>❤ お気に入り</Chip>
        <FilterSelect label="種別" options={[anime, movie, drama, manga, novel, game]} />
        <Chip active removable>🏷️ {tagName} ×</Chip>
        <ChipAdd>+ タグ</ChipAdd>
        <ChipAdd>+ カテゴリ</ChipAdd>
      </FilterBar>
      <SortSearchGroup>
        <SortSelect options={[追加日順, 更新日順, 評価順, タイトル順, 発売日順]} />
        <SearchBox placeholder="タイトルで検索…" />
      </SortSearchGroup>
    </FilterToolbar>

    <MediaGrid density="compact">
      <MediaCard variant="compact" … />  // rating-stars-mini付き
    </MediaGrid>

    <LoadMoreSentinel />
  </Content>
</AppShell>
```

## 3. 表示データ / Props型

```ts
interface MediaListFilters {
  isFavorite?: boolean;
  mediaType?: 'anime' | 'movie' | 'drama' | 'manga' | 'novel' | 'game';
  tagId?: string;
  categoryId?: string;
  title?: string;   // 部分一致検索
  sort?: 'created_at' | 'updated_at' | 'rating' | 'title' | 'release_date'; // 【要確認】バックエンド未実装
  page: number;
  limit: number;
}

interface MediaCardItem {
  id: string;
  mediaType: string;
  badgeLabel: string;   // 種別の日本語表示名
  title: string;
  isFavorite: boolean;
  ratingRounded?: number; // 0-5 の四捨五入値。rating-stars-mini の塗り分けに使用
}
```

## 4. 画面固有コンポーネント

なし（`00_common.md` の `FilterToolbar` / `MediaGrid` / `MediaCard` / `LoadMoreSentinel` のみで構成）

## 5. インタラクション仕様

- `is_favorite` / `media_type` / `tag_id` / `category_id` はクエリパラメータとして URL に同期（ブラウザバック・共有リンクに対応するため `useSearchParams` を使用）
- タグ・カテゴリの絞り込みは「選択中は`Chip active`＋×で解除、未選択時は`ChipAdd`から選択肢を開く」という単一選択のUIパターン（モックでは1件選択済みの状態のみを提示）
- ソートは `00_common.md` §6の通りバックエンド未実装のため、UIのみ先行実装し `sort` パラメータはPRの別タスクで有効化する
- `LoadMoreSentinel` は `useInfiniteScroll()` で次ページを追加取得。全件取得済み時はスピナーではなく「すべて読み込みました」等のテキストに切り替える終端状態を持つ

## 6. API連携

- `GET /items?media_type=...&is_favorite=...&tag_id=...&category_id=...&title=...&page=...&limit=...`（`sort` パラメータは【要確認】未実装）

参照: [items.md](../../backend/mediavault-api/items.md)

## 7. Tailwindスタイリング上の注意

- `.chip-add` は破線ボーダー（`border-style: dashed`）。既存タグ選択とは視覚的に区別する
- カード内 `.rating-stars-mini` は非インタラクティブ表示（詳細画面の `RatingStars` とは別コンポーネント）

---

## 差分: 03_academic_books（学術書・専門書一覧）

- ルート `/academic-books`、サイドバー「学術書・専門書」がactive、「＋ 作品を追加」の遷移先は `13_academic_book_search.md`
- `media_type` は `academic_book` 単一のため `FilterSelect（種別）` は無い（`FilterBar` は すべて/お気に入り/タグ/カテゴリのみ）
- ソート選択肢に「評価順」が無い（追加日順/更新日順/タイトル順/発売日順の4種）
- API: `GET /items?media_type=academic_book&is_favorite=...&tag_id=...&category_id=...&title=...&page=...&limit=...`
