# 01. 検索して追加（一般メディア）画面

対応: 設計書 §1〜§7

## 前提ファイル

- 参照:
  - `docs/frontend/design/12_general_media_search.md`
  - `docs/frontend/ui/12_general_media_search.html`, `docs/frontend/ui/_shared.css`
  - `docs/backend/mediavault-api/items.md`（`GET /items/search`, `POST /items/import` セクション）
  - `frontend/src/components/shared/MediaCard.tsx`（`variant="search-result"` を使用。`imported` / `actionLabel` / `onAction` props）
  - `frontend/src/components/shared/EmptyState.tsx`
  - `frontend/src/components/shared/FilterToolbar.tsx`（種別セレクト + 検索ボックス + ボタン用に利用可否を判断。ボタン押下起点の「検索」ボタンが必須のため、`FilterToolbar` にボタンが無ければモックDOM構造 `.filter-bar` に沿って画面側で直接組んでよい）
  - `frontend/src/hooks/useMediaListData.ts`（`buildQueryString` ヘルパーの参考実装）
  - `frontend/src/pages/MediaListPage.tsx`（`AppShell`/`Titlebar` 連携の参考実装。`/media` 一覧側の「＋ 作品を追加」リンクは既に `/media/search` を指しているため、本タスクではリンク先の画面を実装する）
  - `frontend/src/routes.tsx`
- 出力:
  - `frontend/src/pages/MediaSearchPage.tsx`
  - `frontend/src/hooks/useMediaSearch.ts`
  - `frontend/src/routes.tsx`（`/media/search` ルート追加のみ変更）
  - `frontend/src/pages/MediaSearchPage.test.tsx`
  - `frontend/tests/e2e/media-search.spec.ts`
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [x] UIに表示するアイテム（ラベル・見出し・ボタン文言・メッセージ等）は日本語を優先して使用する
- [x] アイコンは`react-icons`を積極的に使用する（見出し・検索アイコン・空状態アイコン等、視覚的な手がかりが有効な箇所には極力アイコンを添える。モックの🔍は`FiSearch`相当）
- [x] `MediaSearchPage` を `frontend/src/pages/MediaSearchPage.tsx` に実装する。ルート `/media/search`。`AppShell` 配下で `Titlebar title="検索して追加"` + `action={<Link to="/media/new">手動で入力する</Link>}` を表示する（設計書 §2）
- [x] `Titlebar` 直下に種別セレクト（`anime`/`movie`/`drama`/`manga`/`novel`/`game` の6種、ラベルは日本語: アニメ/映画/ドラマ/漫画/小説/ゲーム）+ 検索テキストボックス（placeholder「作品名で検索…」）+「検索」ボタン（`.btn.btn-accent`）を持つフィルタバーを実装する。モックDOM構造は `.filter-bar > .filter-select + .search-box + button.btn.btn-accent`
- [x] 種別セレクト変更時には自動再検索せず、「検索」ボタン押下時にのみ検索を実行する（設計書 §5、モックはボタン押下起点のため）
- [x] `useMediaSearch` フックを `frontend/src/hooks/useMediaSearch.ts` に実装する。`GET /items/search?media_type=...&q=...`（クエリパラメータ名は `title` ではなく `q` である点に注意）を呼び、レスポンス `SearchResultItem[]`（`id`/`media_type`/`provider`/`title`/`thumbnail_url`）を返す。`@tanstack/react-query` の `useQuery`（または `useMutation` 起点、検索ボタン押下時にfetchする形）を用い、`enabled`/手動トリガーいずれかの方式は実装側で判断してよい
- [x] 検索結果は `MediaGrid`相当（`.card-grid.is-compact`）に `MediaCard` の `variant="search-result"` で描画する。`title` はレスポンスの `title`、`badge` は種別の日本語ラベル（アニメ/映画/ドラマ/漫画/小説/ゲーム）を表示する。`year` と `originalTitle` はAPIレスポンスに含まれないため表示しない（設計書 §3、モックの `2025年`・`Symphonia of Stardust` 相当の行はAPI非対応につき実装しない）
- [x] 取り込み済み状態管理用に `ImportedIdSet`（`Set<string>`、`SearchResultItem.id` を保持）をローカル状態（`useState`）で持つ。取り込み済みIDのカードは `MediaCard` に `imported` を渡し、ボタンを disabled + 文言「取り込み済み」にする
- [x] 「取り込む」ボタン押下時、`POST /items/import`（body: `media_type`, `provider`（検索結果の`provider`をそのまま）, `external_id`（検索結果の`id`））を呼ぶ。201成功時・409 `ITEM_ALREADY_IMPORTED` エラー時のいずれも該当カードのIDを `ImportedIdSet` に追加して「取り込み済み」表示に切り替える
- [x] `GET /items/search` が 422 `API_KEY_NOT_CONFIGURED` を返した場合、検索結果グリッドの代わりに `EmptyState`（title「APIキーが設定されていません」、description「この種別の検索には {provider} のAPIキーが必要です。設定画面から登録してください。」、`{provider}` は種別に対応するプロバイダ名の日本語表示、action=`<Link to="/settings?tab=api">設定を開く</Link>`）を表示する。検索結果一覧とこの空状態は排他表示にする（設計書 §5・§7）
- [x] media_type → プロバイダ日本語表示名の対応（設計書 §6）: `anime`→Annict, `manga`/`novel`→楽天ブックス, `movie`/`drama`→TMDb, `game`→Steam（キー不要のためこの種別では422は基本発生しない）
- [x] 取り込み済みボタンは `.btn[disabled]` に `opacity: 0.5; pointer-events: none;` が当たる（`MediaCard`実装が既にdisabled属性で対応している場合はそのまま利用）
- [x] `frontend/src/routes.tsx` に `{ path: "media/search", element: <MediaSearchPage />, handle: { title: "検索して追加" } }` を `media` ルートの子として追加する

## テストリスト

- [x] `frontend/src/pages/MediaSearchPage.test.tsx`: 種別セレクト+検索語を入力し「検索」ボタン押下で `GET /items/search?media_type=...&q=...` が呼ばれ、結果が `MediaCard(variant="search-result")` として描画されることを確認する（`msw`でモック）
- [x] `frontend/src/pages/MediaSearchPage.test.tsx`: 「取り込む」ボタン押下で `POST /items/import` が呼ばれ、201成功時にそのカードが「取り込み済み」disabled表示に切り替わることを確認する
- [x] `frontend/src/pages/MediaSearchPage.test.tsx`: 「取り込む」ボタン押下で `POST /items/import` が 409 `ITEM_ALREADY_IMPORTED` を返した場合も同様に「取り込み済み」disabled表示に切り替わることを確認する
- [x] `frontend/src/pages/MediaSearchPage.test.tsx`: `GET /items/search` が 422 `API_KEY_NOT_CONFIGURED` を返した場合、検索結果グリッドが表示されず `EmptyState` のみが表示されることを確認する（排他表示）
- [x] `frontend/tests/e2e/media-search.spec.ts`: `yarn dev` 起動下で `/media/search` を実描画し、対応モック `docs/frontend/ui/12_general_media_search.html` と主要な構造要素（フィルタバー、`.card-grid.is-compact` とカード内の取り込みボタン、空状態表示）の有無・並び順が一致することを確認する

> Codexメモ: API検索はボタン押下起点に合わせて `useMutation` ベースで実装。
> モックHTMLの `originalTitle` / `year` / `provider` 行はAPI非対応のため非表示とし、検索前は補助的な初期 `EmptyState` を表示。
