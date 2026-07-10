# 03. フェーズ2: 詳細画面共通パターン

依存: フェーズ1（[02_shared_components.md](02_shared_components.md)。特に `TagList`/`StatusSwitcher`/`RatingStars`/`FavoriteToggle` をレール内で使用）
参照: [design/00_common.md](../design/00_common.md) §6

8詳細画面（16, 17, 18, 20, 21, 22, 23, 24）が準拠する正準構成。詳細は [00_overview.md の対応表](00_overview.md#画面--設計書--モック-対応表)参照。

## タスク一覧

- [ ] **DetailLayout**（`.detail-layout`。grid: `.detail-rail` + `.detail-main`。`rail={...}` `main={...}` を受け取る）

- [ ] **DetailRail**（`.detail-rail`、左カラム・sticky）
  - `.doc-cover`（表紙）
  - `.doc-title`（h1） + `.doc-original`（原題、任意）
  - `.rail-facts`: `StatusSwitcher`（`not_started`/`in_progress`/`done`、色は `--color-status-*`）、`RatingStars`、`FavoriteToggle`、登録日等 `.meta-item`、外部API ID等 `.meta-item.muted`
  - `.rail-divider`
  - `RailSection` ×3: タグ（`TagList kind="tag"`）/ カテゴリ（`TagList kind="category"`）/ マイリスト（所属リスト + 解除ボタン + 追加リンク）

- [ ] **DetailSection**（`.doc-section`。`icon`/`title` を受け取る汎用セクション枠）

- [ ] **PropertyList**（`.prop-row`/`.prop-group`。種別固有情報の key-value 表示。フィールドは画面ごとに異なる）

- [ ] **GroupList / EpisodeRow**（`.group-block`/`.group-header`/`.episode-row`。シーズン・巻構成。anime/drama/manga/novel向け）

- [ ] **StaffList**（スタッフセクション `.prop-list-item`。anime/movie/drama向け）

- [ ] **StreamingLinks**（配信セクション `.prop-list-item`。anime/movie/drama向け）

- [ ] **RelatedWorksList**（関連作品 `.result-row`。全詳細画面共通）

- [ ] **ResourceTabs の詳細画面向け組み込み**（リンク/ファイル/トレーラー。academic_book/paperでは「出版社ページ」等ラベルが変わる点に注意。コンポーネント自体は [02_shared_components.md](02_shared_components.md) で実装済み）

## メイン（DetailMain）正準セクション順序

1. 概要 — 全画面共通
2. 種別固有情報（`PropertyList`） — anime以外
3. エピソード/巻構成（`GroupList`） — anime, drama, manga, novel
4. スタッフ（`StaffList`） — anime, movie, drama
5. 関連作品（`RelatedWorksList`） — 全画面共通
6. 配信（`StreamingLinks`） — anime, movie, drama
7. リソース（`ResourceTabs`） — 全画面共通

## 画面別の任意セクション有無マトリクス

| 画面 | 種別固有情報 | 構成(Group) | スタッフ | 配信 |
|---|---|---|---|---|
| anime (16) | ✗ | ✓ | ✓ | ✓ |
| movie (20) | ✓(6項目) | ✗ | ✓ | ✓ |
| drama (21) | ✓(7項目) | ✓ | ✓ | ✓ |
| manga (22) | ✓(4項目) | ✓ | ✗ | ✗ |
| novel (23) | ✓(4項目) | ✓ | ✗ | ✗ |
| game (24) | ✓(5項目) | ✗ | ✗ | ✗ |
| academic_book (17) | ✓(4項目) | ✗ | ✗ | ✗ |
| paper (18) | ✓(5項目) | ✗ | ✗ | ✗ |

各画面固有のフィールド一覧・API推測は個別設計書（[04_screens.md](04_screens.md)から辿る）に記載。

## 完了条件

`DetailLayout` + `DetailRail` + 空の `DetailMain` セクション群を組み合わせたプレースホルダ詳細画面が1つ表示できる状態（実データ結線は不要）。
