# 01. 学術書・専門書一覧画面（AcademicBookListPage）

対応: 設計書（`docs/frontend/design/03_academic_books.md`、実体は `02_general_media.md` 末尾「差分: 03_academic_books」）

依存: `docs/frontend/tasks/02_general_media/01_media_list_screen.md` 完了後に着手（`useMediaListData` / `MediaListPage` の実装パターンを流用する）。

## 前提ファイル

- 参照: `docs/frontend/design/03_academic_books.md`, `docs/frontend/design/02_general_media.md`, `docs/frontend/ui/03_academic_books.html`, `docs/frontend/ui/_shared.css`, `frontend/src/index.css`, `docs/backend/mediavault-api/items.md`
- 参照（既存実装、直接import対象）: `frontend/src/hooks/useMediaListData.ts`（`MediaListFilters`型・`fetchItemsPage`・`mapItemToMediaCard`のロジックを流用/拡張してよい）, `frontend/src/pages/MediaListPage.tsx`（フィルタ組み立て・`useSearchParams`同期・`useInfiniteScroll`利用パターンを流用してよい）, `frontend/src/components/shared/`（`FilterToolbar`, `MediaGrid`, `MediaCard`, `LoadMoreSentinel`, `useInfiniteScroll`）, `frontend/src/components/layout/AppShell.tsx`, `frontend/src/routes.tsx`
- 出力: `frontend/src/pages/AcademicBookListPage.tsx`, 必要に応じて `frontend/src/hooks/useMediaListData.ts` への軽微な拡張（`media_type`固定値を渡せるようにする等。既存の`MediaListFilters`型・呼び出し側の挙動は変更しない）
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [ ] UIに表示するアイテム（ラベル・見出し・ボタン文言・メッセージ等）は日本語を優先して使用する（`frontend/src/pages/MediaListPage.tsx`は現状英語ラベルだが、本タスクの新規実装分は設計書・モックに合わせて日本語で実装する）
- [ ] アイコンは`react-icons`を積極的に使用する（お気に入りチップの❤等、モックで絵文字表記の箇所は`react-icons/fi`等の対応するアイコンに置き換えてよい。`MediaListPage.tsx`の`FiHeart`利用を参考にする）
- [ ] `frontend/src/pages/AcademicBookListPage.tsx` を実装する。`useMediaListData`を`mediaType: 'academic_book'`固定で呼び出し、`FilterToolbar`・`MediaGrid density="compact"`・`LoadMoreSentinel`を配置する（`MediaListPage.tsx`と同一構造で、種別フィルタ関連のprops/UIのみ省く）
- [ ] `FilterToolbar`に`filterOptions`（種別セレクト）は渡さない（設計書差分：`media_type`が`academic_book`単一のため種別ドロップダウン無し）。`selectedFilter`/`onFilterChange`関連propsを省略した場合の`FilterToolbar`の挙動（種別セレクト自体が非表示になるか）を確認し、非表示にならない場合は`frontend/src/components/shared/FilterToolbar.tsx`側でoptional化されているか確認の上、必要なら[02_open_questions.md](02_open_questions.md)に記載する
- [ ] `FilterToolbar`の`chips`は「すべて」「❤ お気に入り」「🏷️ {tagName} ×」（選択中のみ）＋タグ`+`追加・カテゴリ`+`追加のみとし、`MediaListPage.tsx`の`media_type`を含まないフィルタ解除ロジックに合わせる
- [ ] `FilterToolbar`の`sortOptions`は「追加日順・更新日順・タイトル順・発売日順」の4種（「評価順」を含めない。設計書差分・§3【要確認】でバックエンド未実装のためAPIリクエストへの反映は行わない）
- [ ] `is_favorite` / `tag_id` / `category_id` / `title` を`useSearchParams`でURLクエリパラメータと同期する（`media_type`はURLパラメータとして扱わず`academic_book`固定）
- [ ] `useMediaListData`が返す`mediaCards`の`badge`表示名（「学術書」/「専門書」）の由来は現行APIレスポンス（`items.md`）に判別可能なフィールドが見当たらないため、暫定対応方針を決めて実装し、判断内容を[02_open_questions.md](02_open_questions.md)に記録する（例: 暫定で固定値「学術書」を表示する等）
- [ ] `frontend/src/routes.tsx` に `path: "academic-books"` のルートを追加し、`element={<AcademicBookListPage />}`、`handle: { title: "学術書・専門書" }`とする（「＋ 作品を追加」導線は`13_academic_book_search.md`の画面実装が本タスクの範囲外のため、遷移先ルートが未実装でも本タスクではプレースホルダの`Link`のみ設置しておいてよい旨を[02_open_questions.md](02_open_questions.md)に記載する）
- [ ] `frontend/src/index.css`: `_shared.css`に対応クラスが無い場合のみ追加してよい。既存クラスの値は変更しない

## テストリスト

- [ ] `AcademicBookListPage.test.tsx`: 初期表示で`FilterToolbar`（種別セレクトを含まない）・`MediaGrid`（`is-compact`）・`LoadMoreSentinel`が描画されること
- [ ] `AcademicBookListPage.test.tsx`: `useMediaListData`が`mediaType: 'academic_book'`固定で呼び出されること（モック/スパイで検証）
- [ ] `AcademicBookListPage.test.tsx`: `sortOptions`に「評価順」が含まれないこと
- [ ] `AcademicBookListPage.test.tsx`: お気に入りチップクリックで`is_favorite`パラメータがトグルされること
- [ ] `tests/e2e/academic-books.spec.ts`: `yarn dev`起動下で`AcademicBookListPage`を実描画し、`docs/frontend/ui/03_academic_books.html`と主要構造（`.filter-toolbar`内のチップ/ソートセレクト/検索欄の並び、種別セレクトが存在しないこと、`.card-grid.is-compact`の存在、カード内`.badge`/`.rating-stars-mini`の有無）が一致することをDOM構造アサーション（`getByRole`/`locator`）で確認する

> Codexメモ: (なし)
