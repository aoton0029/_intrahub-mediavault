# 02. フェーズ1: 共通コンポーネント

依存: フェーズ0（[01_foundation.md](01_foundation.md)、トークン・テーマ）
参照: [design/00_common.md](../design/00_common.md) §3, §4, §5

アイコンは全て `react-icons/fi`（Feather Icons）を使用する。モックHTML内のコメント（例: `<!-- react-icons/fi: FiCheckCircle -->`）に厳密に従う。対応表は [00_common.md §4](../design/00_common.md#4-アイコンreact-icons) を参照。独自SVG・他アイコンセットの追加は行わない。

## タスク一覧

- [ ] **MediaCard / MediaGrid**
  - `<MediaCard variant="default" | "compact" | "search-result">`（`.cover`/`.badge`/`.fav`/`.title`/`.meta`/`.rating` を内包。`search-result`は「取り込み済み」時 `.btn[disabled]`）
  - `<MediaGrid density="default" | "compact">`（`MediaCard[]` をgrid配置）

- [ ] **LiteratureList / LiteratureRow**（書誌情報中心の行リスト。論文・学術書向け）

- [ ] **FilterToolbar**
  - `.chip`/`.chip-add`/`.filter-select`/`.sort-select`/`.search-box` を内包

- [ ] **LoadMoreSentinel + useInfiniteScroll()**
  - IntersectionObserverでページ追加読込

- [ ] **StatusSwitcher**
  - `value`/`onChange`、ポップオーバー開閉は `useState<boolean>` によるローカルstate
  - `useEffect` でdocumentへの外側クリックリスナー登録（コンポーネントスコープ）
  - 参照: [00_common.md §5-3](../design/00_common.md#5-インタラクション-react状態への変換方針)

- [ ] **RatingStars / RatingStarsMini**
  - `RatingStars`: `hoverValue`（プレビュー）と確定値 `value` を分離管理、`mouseleave` でプレビュー解除
  - `RatingStarsMini`: カード内の非インタラクティブ表示
  - 参照: [00_common.md §5-4](../design/00_common.md#5-インタラクション-react状態への変換方針)

- [ ] **FavoriteToggle**
  - `value`/`onChange` の制御コンポーネント。永続化（`PATCH /items/{id}`）は呼び出し元が担当
  - 参照: [00_common.md §5-2](../design/00_common.md#5-インタラクション-react状態への変換方針)

- [ ] **TagList**（`kind="tag" | "category"`）
  - `items`/`onAdd`/`onRemove`
  - `isAdding` state + 制御 `<input>`、`onKeyDown` でEnter確定/Escape取消、`onBlur` で遅延キャンセル（Enter確定とのクリック競合回避のため、元実装の `setTimeout(cancel, 100)` 相当の遅延またはクリック対象判定を検討）
  - 参照: [00_common.md §5-5, §5-6](../design/00_common.md#5-インタラクション-react状態への変換方針)

- [ ] **MylistCover**（`count={1-4}` `covers={string[]}`。収録作品数に応じたコラージュ）

- [ ] **Modal**（`open`/`onClose`/`title`。マイリスト作成・削除確認等の子要素を内包）

- [ ] **EmptyState**（`title`/`description`/`action?`。検索結果0件・APIキー未設定など）

- [ ] **ResourceTabs**（`tabs={links|files|trailers}`。CSSラジオタブではなく `useState<TabKey>` で選択管理）
  - 参照: [00_common.md §5-7](../design/00_common.md#5-インタラクション-react状態への変換方針)

- [ ] **FormSection / FormGrid / FormField / FormActions**
  - `.form-section-title` / `.form-grid` / `.form-field` / `.form-actions` に対応
  - react-hook-form + zod と組み合わせて使う想定（フォーム画面向け。11番の論文登録フォームで使用）

## 完了条件

各コンポーネントが単体で import して使える状態になっていること（親画面実装を待たない）。フェーズ3の画面実装タスクからはこれらのコンポーネント名をそのまま参照する。
