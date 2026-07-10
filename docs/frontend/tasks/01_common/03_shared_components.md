# 03. 共通コンポーネント群

対応: 設計書 §3（共通コンポーネント一覧）, §4（アイコン）, §5-2〜5-7（インタラクション）

依存: [02_app_shell.md](02_app_shell.md) 完了後に着手。コンポーネントごとに独立性が高いため並行実装可。

## 前提ファイル

- 参照: `docs/frontend/ui/_shared.css`, `docs/frontend/ui/_shared.js`, `docs/frontend/ui/01_home.html`, `docs/frontend/ui/12_general_media_search.html`, `docs/frontend/ui/15_mylist_detail.html`
- 出力: `frontend/src/components/shared/` 配下（コンポーネント単位でファイル分割、`index.tsx`でre-export）
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

各行は設計書 §3 表の1エントリに対応する。`react-icons/fi` の割当は §4 の対応表と、モックHTML内の `<!-- react-icons/fi: FiXxx -->` コメントを一次情報として使う。

## タスク一覧: 表示系コンポーネント

- [x] `MediaCard`（`variant: "default" | "compact" | "search-result"`）— `.cover`/`.badge`/`FavoriteToggle`/`.title`/`.meta`/`RatingStarsMini`を内包。`search-result`は取り込み済み時に追加ボタンを`disabled`にする
- [x] `MediaGrid`（`density: "default" | "compact"`）— `MediaCard[]`をgrid配置
- [x] `LiteratureList` / `LiteratureRow` — 書誌情報中心の行リスト（論文・学術書）
- [x] `MylistCover`（`count: 1-4`, `covers: string[]`）— 収録作品数に応じたコラージュレイアウト
- [x] `PropertyList`（`items: {key,label,value}[]`）— `.prop-row`/`.prop-group`
- [x] `RelatedWorksList`（`items`）— `.result-row`、`FiGitBranch`使用
- [x] `EmptyState`（`title`, `description`, `action?`）

## タスク一覧: インタラクティブ系コンポーネント

- [x] `RatingStars`（`value`, `onChange`, `readOnly?`）— hover中のプレビュー値(`hoverValue`)と確定値`value`を分離管理、`mouseleave`でプレビュー解除、`FiStar`のfill/stroke切替で表現（設計書§5-4）
- [x] `RatingStarsMini`（`value`）— 非インタラクティブ表示
- [x] `FavoriteToggle`（`value`, `onChange`）— 制御コンポーネント、`FiHeart`使用、永続化（API呼び出し）は呼び出し元の責務（設計書§5-2）
- [x] `StatusSwitcher`（`value`, `onChange`）— `useState<boolean>`でポップオーバーopen管理 + `useEffect`で外側クリックリスナー登録（設計書§5-3）。`not_started`/`in_progress`/`done`を`--color-status-*`で色分け、アイコンは`FiCircle`/`FiPlayCircle`/`FiCheckCircle`、展開シェブロンは`FiChevronDown`
- [x] `TagList`（`kind: "tag" | "category"`, `items`, `onAdd`, `onRemove`）— `isAdding` state + 制御`<input>`、`onKeyDown`でEnter確定/Escape取消、`onBlur`は遅延キャンセル（設計書§5-5, 元実装は`setTimeout(cancel, 100)`でEnter確定とのクリック競合回避。React版での再現方法をCodexメモに記載）。アイコンは`FiTag`/`FiFolder`、追加は`FiPlus`、削除は`FiTrash2`
- [x] `Modal`（`open`, `onClose`, `title`）— `.modal-overlay`/`.modal`、マイリスト作成・削除確認等の子要素を`children`で受ける
- [x] `ResourceTabs`（`tabs: {links, files, trailers}`）— CSSラジオタブではなく`useState<TabKey>`でタブ切替（設計書§5-7）。アイコンは見出し`FiPaperclip`、リンクタブ`FiLink2`、ファイルタブ`FiPaperclip`、トレーラータブ`FiFilm`

## タスク一覧: フィルタ・検索系コンポーネント

- [x] `FilterToolbar` — `.chip`/`.chip-add`/`.filter-select`/`.sort-select`/`.search-box`を内包、検索アイコンは`FiSearch`、並び替えアイコンは近似の`FiArrowUpDown`系（設計書§4の【要確認】、最終選定をCodexメモに記載）
- [x] `LoadMoreSentinel` + `useInfiniteScroll()` — IntersectionObserverでページ追加読込

## タスク一覧: フォーム・設定系コンポーネント

- [x] `FormSection` / `FormGrid` / `FormField` / `FormActions` — `react-hook-form` + `zod`と組み合わせられる構成にする（バリデーションschemaは画面タスク側で定義、ここでは表示・エラー表示の型のみ）
- [x] `SettingsShell`（`tabs`）— CSSラジオタブではなくタブstateで実装
- [x] `ApiKeyCard`（`provider`, `keyMasked`, `onEdit`）— `.kv-card`、外部API連携アイコンは`FiLink`

## タスク一覧: 未確定アイコンの解決

- [x] 解除（マイリスト/関連作品）アイコン: モックはカスタムpath。`FiPackage`/`FiX`等から選定し、決定理由を[05_open_questions.md](05_open_questions.md)に記載してからコンポーネントに反映する
- [x] 並び替えアイコン: `react-icons`に厳密一致がない場合、最も近いソート系アイコンを選定し同様に記載する

## テストリスト

- [x] `RatingStars`: クリックで`onChange`が正しい値で呼ばれる／hover中に表示値がプレビューされ、`mouseleave`で確定値に戻る
- [x] `FavoriteToggle`: クリックで`onChange(!value)`が呼ばれる（`value`自体は変更しない＝制御コンポーネント）
- [x] `StatusSwitcher`: トリガークリックでポップオーバーが開閉する／外側クリックで閉じる／選択で`onChange`が呼ばれ閉じる
- [x] `TagList`: 追加ボタン→input表示→Enterで`onAdd`が呼ばれinputが閉じる／Escapeで取消される／`onRemove`が対象idで呼ばれる
- [x] `Modal`: `open=false`で非表示、`open=true`で表示、オーバーレイクリックまたは指定操作で`onClose`が呼ばれる
- [x] `ResourceTabs`: タブクリックで表示中のパネルが切り替わる
- [x] `MediaCard`: `variant="search-result"`かつ取り込み済み状態で追加ボタンが`disabled`になる
- [x] `MylistCover`: `count`が1/2/3/4それぞれで対応するレイアウトクラス（`.mylist-cover.n1`〜`.n4`相当）が付与される
- [x] `LoadMoreSentinel`: IntersectionObserverの交差コールバックで読込関数が呼ばれる（モックIntersectionObserverを使用）
- [x] `EmptyState`: `action`未指定時にアクションボタンが描画されない

> Codexメモ: タグ追加の blur 競合回避は元モックに合わせて `setTimeout(..., 100)` を維持した。
> Codexメモ: sort icon は `FiArrowDown`、解除系は `FiTrash2` を採用し、判断理由は `05_open_questions.md` に記録した。
