# 01. 詳細画面ルートディスパッチ（MediaDetailPage）

対応: 設計書 §1（`docs/frontend/design/20_movie_detail.md`）、および [16_anime_detail/02_open_questions.md](../16_anime_detail/02_open_questions.md) で先送りされていた「`/media/:id` の振り分け方針」の解消

依存: [16_anime_detail/01_anime_detail_screen.md](../16_anime_detail/01_anime_detail_screen.md)（`AnimeDetailPage`）完了後に着手。（本タスク完了後に [02_movie_detail_screen.md](02_movie_detail_screen.md) に進む）

## 前提ファイル

- 参照: `docs/frontend/design/20_movie_detail.md`（§1のみ）, `docs/frontend/design/16_anime_detail.md`（§1のみ）, `docs/backend/mediavault-api/items.md`
- 参照（既存実装、直接import対象）: `frontend/src/routes.tsx`, `frontend/src/components/layout/AppShell.tsx`, `frontend/src/components/layout/Titlebar.tsx`, `frontend/src/pages/AnimeDetailPage.tsx`, `frontend/src/hooks/useAnimeDetailData.ts`
- 出力: `frontend/src/pages/MediaDetailPage.tsx`（新規、`media_type`による振り分けのみを担当するディスパッチャ）、`frontend/src/routes.tsx`（変更）、`frontend/src/components/layout/AppShell.tsx`（変更、動的タイトル/パンくず対応が必要な場合のみ）
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [x] UIに表示するアイテム（見出し・メッセージ等）は日本語を優先して使用する
- [x] `frontend/src/pages/MediaDetailPage.tsx` を新規作成する。`useParams`で`id`を取得し、`GET /items/{id}` を取得して `media_type` を判定し、`media_type: "anime"` なら既存の `AnimeDetailPage` の中身（または内部ロジック）を、`media_type: "movie"` なら [02_movie_detail_screen.md](02_movie_detail_screen.md) で実装する `MovieDetailPage` を描画する。`media_type` がそれ以外（drama/manga/novel/game/academic_book/paper）の場合は「この種別の詳細画面は未対応です」旨の `EmptyState` を表示する（`frontend/src/components/shared/EmptyState`を使用）
- [x] `ItemDetail` の取得が `MediaDetailPage`（振り分け用）と各詳細ページ用フック（`useAnimeDetailData`/`useMovieDetailData`）の両方で二重に発生する点をどう扱うか（例: 振り分け用は軽量に`item`のみ取得し`media_type`だけ見る、または各フックが内部で完結しmedia_type判定はレスポンスヘッダ相当の軽い先読みで済ませる等）は実装コストとのバランスで妥当な方法を選び、Codexメモに理由を記載する。過度な設計変更（例えば`useAnimeDetailData`/`useMovieDetailData`をリファクタして共通化する等）は本タスクの範囲外とし、行わない
- [x] パンくず・タイトルバーは `media_type` に応じて動的に変える必要がある（アニメ「一般メディア / アニメ」、映画「一般メディア / 映画」）。現状 `frontend/src/routes.tsx` の `path: "media/:id"` の `handle.breadcrumbs` は静的に「アニメ」固定になっているため、`AppShell`/`Titlebar`側でページコンポーネントから動的にタイトル/パンくずを上書きできる仕組み（例: `AppShell`が提供するcontext経由でページ側から`setPageChrome({ breadcrumbs, actions })`のように登録する、または`MediaDetailPage`が`AppShell`の`props.breadcrumbs`相当を渡せるようルート構成を`element={<AppShell><MediaDetailPage /></AppShell>}`的な形に変更する等）を検討し実装する。既存の`AppShell`の`matchedHandle`によるstatic fallback（他の静的ルートで使用中）は壊さないこと
- [x] タイトルバーの「編集する」ボタン（`btn-accent`、遷移先: 一般メディア編集フォーム）は映画詳細でのみ表示する。編集フォームのルートは未実装のため、遷移先パスは `/media/:id/edit` と仮定してリンクを実装し、実フォーム未実装である旨を [03_open_questions.md](03_open_questions.md) に記載する
- [x] `frontend/src/routes.tsx` の `path: "media/:id"` の `element` を `<AnimeDetailPage />` から `<MediaDetailPage />` に差し替える

## テストリスト

- [x] `MediaDetailPage.test.tsx`: `media_type: "anime"` のレスポンス時に既存アニメ詳細相当の内容が描画されること
- [x] `MediaDetailPage.test.tsx`: `media_type: "movie"` のレスポンス時に映画詳細（[02_movie_detail_screen.md](02_movie_detail_screen.md)実装後を前提としたスタブ/モック）が描画されること。本タスク単独で先行実装する場合は、`MovieDetailPage`をモックしてディスパッチロジックのみ検証してよい
- [x] `MediaDetailPage.test.tsx`: 未対応の`media_type`の場合に`EmptyState`が表示されること
- [ ] e2eテストは対応モックが単一画面に紐づかない横断的変更のため本タスクでは省略する（[02_movie_detail_screen.md](02_movie_detail_screen.md)側の e2e で映画詳細のレイアウト一致を確認する）

> Codexメモ: `MediaDetailPage` では `GET /items/{id}` を1回だけ行って `media_type` を判定し、各詳細ページ側の既存/新規フックはそのまま独立して再取得する構成にした。二重取得は残るが、既存フックの責務を崩さず最小差分で振り分けを導入できるため本タスクではこれを採用。
