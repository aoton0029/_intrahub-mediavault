# 03. 学術書・専門書（一覧）

対応モック: `docs/frontend/ui/03_academic_books.html`

本画面は `02_general_media.md` と同一構造のため、フルスペックの記述はそちらに譲り、ここでは差分のみを記載する（`02_general_media.md` 末尾の「差分: 03_academic_books」と同内容）。

## 差分サマリ

- ルート: `/academic-books`。サイドバー「学術書・専門書」がactive
- タイトルバーの「＋ 作品を追加」の遷移先は `13_academic_book_search.md`（検索・追加画面）
- `media_type` が `academic_book` 単一のため、`FilterBar` に種別ドロップダウン（`FilterSelect`）は無い。すべて/お気に入り/タグ（1件選択例）/カテゴリのみ
- ソート選択肢: 追加日順・更新日順・タイトル順・発売日順（「評価順」なし）
- カードグリッドは `is-compact`、バッジは「学術書」「専門書」の2種類の表示名がある
- 遅延ローディング領域あり（`LoadMoreSentinel`）

## API連携

- `GET /items?media_type=academic_book&is_favorite=...&tag_id=...&category_id=...&title=...&page=...&limit=...`

参照: [items.md](../../backend/mediavault-api/items.md)
