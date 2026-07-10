# 17. 学術書・専門書 詳細

対応モック: `docs/frontend/ui/17_academic_book_detail.html`

本画面は `00_common.md` §5「詳細画面共通パターン」に準拠するが、`rail-facts` がインタラクティブでなく **静的表示** である点が一般メディア詳細（16, 20-24）と異なる（下記参照）。

## 差分サマリ

- ルート: `/academic-books/:id`。パンくずは「学術書・専門書」（一覧のみ、サブカテゴリなし）。編集ボタンあり（遷移先: 学術書用編集フォーム）
- **`rail-facts` が非インタラクティブ**: `StatusSwitcher`ではなく単なる `<span class="meta-item">未着手</span>` 表示、`RatingStars` もクリック不可の読み取り専用、`FavoriteToggle` も `<span>`（クリック不可）。これは学術書・論文が「API検索から取り込むのではなく手動登録が主」という性質を反映したモックと推測される。実装時は `StatusSwitcher`/`RatingStars`/`FavoriteToggle` に `readOnly` propを追加して両モードに対応させる設計とする
- タグ/カテゴリに削除ボタン（×）が無い（`.tag-pill` のみ、`.tag-remove`/`.tag-add-trigger` が無い）。同様に読み取り専用の可能性が高いが、**【要確認】**一般メディア詳細と挙動を揃えるべきか、モック通り読み取り専用にすべきかはUX方針次第
- 登録方法の表示が `API(...)` ではなく「手動登録」（`FiEdit3` アイコン）
- 任意セクション有無: 種別固有情報 ✓（4項目） / エピソード・巻構成 ✗ / スタッフ ✗ / 配信 ✗
- セクション順序: 概要 → 種別固有情報 → 関連作品 → リソース
- リソースタブは「リンク」「ファイル」のみ（トレーラーなし）

## 種別固有情報（`prop-row` 4項目）

著者 / 出版社 / ISBN / NDL ID（未設定の場合 `.muted`）/ Google Books ID（フォーム上は「著者/出版社/ISBN」の3項目、詳細画面はNDL ID・Google Books IDも追加表示。**要確認**: フォーム側フィールドとの整合）

```ts
interface AcademicBookDetail extends ItemDetailBase {
  mediaType: 'academic_book';
  detail: {
    authors: string;
    publisher: string;
    isbn: string;
    ndlId?: string;
    googleBooksId?: string;
  };
}
```

## API連携

- 詳細取得: `GET /items/{id}`（`detail(academic_book)` + tags + categories + calibre_links）
- タグ/カテゴリ: 【要確認】読み取り専用なら追加/削除APIは呼ばない。一般メディアと同様の編集可能UIにする場合は `20_movie_detail.md` と同じ `POST/DELETE /items/{id}/tags`・`/items/{id}/categories`
- 関連作品: `20_movie_detail.md` と同一パターン
- リソース: `POST/DELETE /items/{id}/links`, `/items/{id}/files`（リンクラベル例: 「出版社ページ」）

参照: [items.md](../../backend/mediavault-api/items.md), [tags.md](../../backend/mediavault-api/tags.md), [categories.md](../../backend/mediavault-api/categories.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md)
