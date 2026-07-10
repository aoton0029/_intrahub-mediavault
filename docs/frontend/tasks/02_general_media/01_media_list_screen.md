# 01. 一般メディア一覧画面（MediaListPage）

対応: 設計書 §1, §2, §3, §5, §6, §7

## 前提ファイル

- 参照: `docs/frontend/design/02_general_media.md`, `docs/frontend/ui/02_general_media.html`, `docs/frontend/ui/_shared.css`, `frontend/src/index.css`, `docs/backend/mediavault-api/items.md`, `docs/backend/mediavault-api/tags.md`, `docs/backend/mediavault-api/categories.md`
- 参照（共通実装、直接import対象）: `frontend/src/components/shared/`（`FilterToolbar`, `MediaGrid`, `MediaCard`, `LoadMoreSentinel`, `useInfiniteScroll`）, `frontend/src/components/layout/AppShell.tsx`, `frontend/src/routes.tsx`, `frontend/src/hooks/useHomeData.tsx`（`mapItemToMediaCard`・`buildQueryString`・`ItemWithRefs`型の実装パターンを流用してよい）
- 出力: `frontend/src/pages/MediaListPage.tsx`, `frontend/src/hooks/useMediaListData.ts`（一覧取得・無限スクロール用フック）
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [ ] `frontend/src/hooks/useMediaListData.ts` を実装する。`MediaListFilters`（`isFavorite?`, `mediaType?: 'anime'|'movie'|'drama'|'manga'|'novel'|'game'`, `tagId?`, `categoryId?`, `title?`）を引数に取り、`GET /items?media_type=...&is_favorite=...&tag_id=...&category_id=...&title=...&limit=20&after_created_at=...&after_id=...`（`items.md` §GET /items のkeysetページネーション仕様）でページ取得する。`sort`パラメータはバックエンド未実装のためリクエストに含めない（設計書 §3, §5【要確認】）
- [ ] 無限スクロール取得のページ結合ロジックを実装する（`useInfiniteQuery`または手動state管理。`pagination.has_more`が`false`になったら取得終了とし、終端フラグを返す）
- [ ] 取得した`ItemWithRefs`を`MediaCardProps`（`badge`は`MEDIA_TYPE_LABELS`相当のマッピング、`variant="compact"`、`href="/media/{id}"`、`rating`未設定時は`undefined`）へ変換するmapperを実装する（`useHomeData.tsx`の`mapItemToMediaCard`と同等のロジックを`useMediaListData.ts`内に実装してよい。共通化はこのタスクの範囲外）
- [ ] `frontend/src/pages/MediaListPage.tsx` を実装し、`Titlebar`相当（`AppShell`が担うため本ページでは`h1`は持たず、ルート側`handle.title`を使う）、`FilterToolbar`、`MediaGrid density="compact"`、`LoadMoreSentinel`の順にレイアウトする
- [ ] `is_favorite` / `media_type` / `tag_id` / `category_id` / `title` を `useSearchParams`（react-router-dom）でURLクエリパラメータと同期する（設計書 §5）
- [ ] `FilterToolbar`に渡す`chips`を組み立てる: 「すべて」チップ（`is_favorite`・`tag_id`・`category_id`いずれも未指定の時にactive、クリックで全フィルタ解除）、「❤ お気に入り」チップ（`is_favorite`のトグル）、タグ選択中チップ（`🏷️ {tagName} ×`形式、`GET /tags`で取得した`name`を解決して表示、クリックで`tag_id`を解除）を実装する。タグ未選択時は`chip-add`（`+ タグ`、クラスに`chip-add`を使い破線ボーダーを適用）を表示し、クリックで選択肢を開くUIは設計書に詳細記載が無いため、`open`状態を持つ簡易ドロップダウン（`GET /tags`一覧から選択）で実装してよい（実装方針が設計書から一意に決まらない場合は[02_open_questions.md](02_open_questions.md)に仮決定として記録する）。カテゴリも同様のパターンで`GET /categories`を用いて実装する
- [ ] `FilterToolbar`の`filterOptions`（種別セレクト）に `['すべて', 'アニメ', '映画', 'ドラマ', '漫画', '小説', 'ゲーム']` を渡し、選択値を`media_type`クエリパラメータ（`anime`/`movie`/`drama`/`manga`/`novel`/`game`）にマッピングする。既存`FilterToolbar`は`filterOptions: string[]`のみを受け取り値とラベルの分離を持たないため、日本語ラベル⇔APIの`media_type`値のマッピング表をこのファイル内に実装する
- [ ] `FilterToolbar`の`sortOptions`に `['追加日順', '更新日順', '評価順', 'タイトル順', '発売日順']` を渡す。バックエンド未実装のため選択してもAPIリクエストには反映しない（UIのみ先行実装。設計書 §5, §3【要確認】）
- [ ] `FilterToolbar`の`searchValue`/検索欄入力を`title`クエリパラメータに同期する（`onChange`のデバウンス処理を含めてよい。既存`FilterToolbar`は`searchValue`のみを受け取り`onChange`を持たないため、コンポーネント自体に`onSearchChange`等のprops追加が必要な場合は`frontend/src/components/shared/FilterToolbar.tsx`を拡張してよい。既存の`chips`/`filterOptions`/`sortOptions`propsの型・挙動は変更しない）
- [ ] `useInfiniteScroll`を用いて`LoadMoreSentinel`のトリガー要素を配置し、交差時に次ページを取得する。全件取得済み時は`LoadMoreSentinel`に`loading={false}`かつ`text="すべて読み込みました"`を渡す
- [ ] `routes.tsx` の `path: "media"` のプレースホルダ（`<div>一般メディア一覧のプレースホルダ</div>`）を `<MediaListPage />` に置き換える
- [ ] `frontend/src/index.css`: `_shared.css`に定義が無いクラスが必要になった場合のみ追加してよい。既存クラス（`.filter-toolbar`, `.chip`, `.chip-add`, `.card-grid`等）の値は変更しない

## テストリスト

- [ ] `useMediaListData.test.ts`: フィルタパラメータ（`media_type`/`is_favorite`/`tag_id`/`category_id`/`title`）が正しくクエリ文字列に反映されること（`msw`等でモック）
- [ ] `useMediaListData.test.ts`: `pagination.has_more=true`のレスポンスに対し次ページ取得で`after_created_at`/`after_id`が前回レスポンス値で送信されること
- [ ] `useMediaListData.test.ts`: `pagination.has_more=false`で取得が終端になること
- [ ] `MediaListPage.test.tsx`: 初期表示で`FilterToolbar`・`MediaGrid`（`is-compact`）・`LoadMoreSentinel`が描画されること
- [ ] `MediaListPage.test.tsx`: 種別セレクトを変更すると`useSearchParams`の`media_type`が更新されること
- [ ] `MediaListPage.test.tsx`: お気に入りチップクリックで`is_favorite`パラメータがトグルされること
- [ ] `MediaListPage.test.tsx`: タグ選択中チップの`×`クリックで`tag_id`パラメータが解除されること
- [ ] `MediaListPage.test.tsx`: 全件取得済み時に「すべて読み込みました」テキストが表示されること
- [ ] `tests/e2e/media-list.spec.ts`: `yarn dev`起動下で`MediaListPage`を実描画し、`docs/frontend/ui/02_general_media.html`と主要構造（`.filter-toolbar`内のチップ/セレクト/検索欄の並び、`.card-grid.is-compact`の存在、カード内`.badge`/`.title`/`.rating-stars-mini`の有無）が一致することをDOM構造アサーション（`getByRole`/`locator`）で確認する

> Codexメモ: (なし)
