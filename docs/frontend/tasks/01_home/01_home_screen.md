# 01. ホーム画面本体（StatGrid / SectionHeading / HomePage）

対応: 設計書 §2, §3, §4, §5, §6, §7

依存: `docs/frontend/tasks/01_common/` 全タスク完了後に着手（`AppShell`, `MediaCard`, `MediaGrid` を利用するため）。

## 前提ファイル

- 参照: `docs/frontend/design/01_home.md`, `docs/frontend/ui/01_home.html`, `docs/frontend/ui/_shared.css`, `docs/backend/mediavault-api/items.md`
- 参照（共通実装、直接import対象）: `frontend/src/components/shared/`（`MediaCard`, `MediaGrid`）, `frontend/src/components/layout/AppShell.tsx`, `frontend/src/routes.tsx`
- 出力: `frontend/src/pages/HomePage.tsx`（または `frontend/src/pages/index.tsx` 配下に配置、既存の`routes.tsx`の配置慣習に合わせる）, `frontend/src/components/home/StatGrid.tsx`, `frontend/src/components/home/StatCard.tsx`, `frontend/src/components/home/SectionHeading.tsx`, `frontend/src/hooks/useHomeData.ts`（統計・一覧取得フック）
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [ ] `StatCard`（`label`, `value`, `isFavorite?`）を実装する（`.stat-card` / `isFavorite`時は`.is-favorite`で数値色を`--color-favorite`にする）
- [ ] `StatGrid`（`children` として `StatCard[]` を受ける、または `stats: HomeStats` を受けて内部で4枚組み立てる）を実装する（`.stat-grid`）
- [ ] `SectionHeading`（`title`, `seeAllHref`）を実装する（`.section-heading` + `.see-all` の `<Link>`）
- [ ] `frontend/src/hooks/useHomeData.ts` を実装する。`GET /items?limit=6`（最近追加）と `GET /items?status=in_progress&limit=6`（進行中）を並列取得し、`HomeStats`（`totalCount`/`inProgressCount`/`doneCount`/`favoriteCount`）は複数回の`GET /items`集計による暫定実装とする（【要確認】済み事項、[02_open_questions.md](02_open_questions.md) 参照）
- [ ] `HomePage` を実装し、`StatGrid` → `SectionHeading`+`MediaGrid`（最近追加、`limit=6`）→ `SectionHeading`+`MediaGrid`（進行中、`limit=6`）の順にレイアウトする
- [ ] `MediaCard` へ渡す props をAPIレスポンス（`ItemWithRefs`）から`MediaSummary`相当に変換するmapperを実装する（`rating`未設定時は`meta`ごと非表示になるよう`MediaCard`側の挙動に委譲し、ここでは`undefined`を渡すだけにする）
- [ ] 進行中セクションの`MediaCard`には`status_label`を`tag-pill`（色=`--color-status-progress`）として渡す
- [ ] カードのリンク先を`item.id`から`/media/:id`相当のパスとして生成する（モックのプレースホルダリンクを再現しない）
- [ ] `routes.tsx` にホーム（インデックスルート `/`）を登録する

## テストリスト

- [ ] `StatCard`: `isFavorite`指定時に対応するクラス/スタイルが付与されること
- [ ] `SectionHeading`: `seeAllHref`が`<Link>`の`to`に渡ること
- [ ] `useHomeData`: 最近追加・進行中それぞれのフェッチ結果が正しいkeyで返ること（`msw`等でモック）
- [ ] `HomePage`: 統計4項目・2つのセクション見出し・各`MediaGrid`が描画されること
- [ ] `HomePage`: 進行中セクションのカードに`status_label`のtag-pillが表示されること
- [ ] `HomePage`: `rating`未設定アイテムで評価metaが非表示になること
- [ ] `HomePage`: 「すべて見る」リンクが `/media`（進行中は `/media?status=in_progress`）へ遷移すること

> Codexメモ: (なし)
