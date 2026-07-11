# 20_movie_detail 実装タスク概要

`docs/frontend/design/20_movie_detail.md`（以下「設計書」）を実装に落とし込むための計画。対象は映画詳細画面（`/media/:id`、`mediaType: movie`）。`DetailLayout`・`DetailRail`・`DetailMain`・`PropertyList`・`StaffList`・`StreamingLinks`・`RailSection`・`StatusSwitcher`/`RatingStars`/`FavoriteToggle`/`TagList`・`detailSectionMatrix`（`movie`エントリ: `propertyList: true, groupList: false, staffList: true, streaming: true`）等の共通基盤は `docs/frontend/tasks/00_common/` で実装済み。先行実装済みの `docs/frontend/tasks/16_anime_detail/`（`AnimeDetailPage`/`useAnimeDetailData`）が実装パターンの参考になる。

`/media/:id` は現状 `AnimeDetailPage` 決め打ちルートになっており、映画詳細を追加するにあたり `media_type` に応じたディスパッチにする必要がある（[16_anime_detail/02_open_questions.md](../16_anime_detail/02_open_questions.md)で先送りされていた事項）。ディスパッチ導入とパンくず/タイトルバーの動的化を先行タスクとして切り出し、その後に映画詳細画面本体を実装する。

## タスクファイル構成

| ファイル | 内容 | 対応する設計書セクション |
|---|---|---|
| [01_detail_route_dispatch.md](01_detail_route_dispatch.md) | `/media/:id` を `media_type` に応じて `AnimeDetailPage`/`MovieDetailPage` に振り分ける `MediaDetailPage`（ディスパッチャ）の実装、パンくず/タイトルバーの動的化 | §1 |
| [02_movie_detail_screen.md](02_movie_detail_screen.md) | `MovieDetailPage`（`/media/:id`、`mediaType: movie`）本体、`ItemDetail`取得・スタッフ・関連作品・配信・リソース・タグ/カテゴリ/マイリストの各データ取得・更新フックの実装 | §2〜7 |
| [03_open_questions.md](03_open_questions.md) | 設計書中の【要確認】項目・編集フォーム未実装の扱いなど未決事項の追跡 | 全節 |

## Claude ⇄ Codex 運用ルール

このタスク群は Claude が計画・レビューを行い、実装は Codex に委譲する前提で書式を統一している。

- 各タスクファイルの「タスク一覧」「テストリスト」のチェックボックス `- [ ]` は、実装・テストが完了するたびに **Codexが** `- [x]` に変更する（Claude側では変更しない）。
- タスクを完了する際、対応するタスク見出しの直下にある `> Codexメモ:` 行に、実装上の判断・設計書との差分・未解決事項を1〜3行で追記する（無ければ `> Codexメモ: (なし)` のままでよい）。
- 【要確認】に該当する意思決定が必要になった場合は、実装を進めず [03_open_questions.md](03_open_questions.md) に追記し、Claudeのレビューを待つ。ブロッキングでない場合は妥当な仮決定を行い、その旨をメモに残して先に進んでよい。
- **実装範囲の限定**: 着手するタスクの「タスク一覧」に書かれた内容の実装のみを行う。参照してよいファイルは各タスクファイルの「前提ファイル」節に列挙されたものと、そこから直接importされるファイルに限定する。それ以外のファイルを探すための横断的なコードベース探索（grep/find等での関連ファイル探索、他タスクの実装状況の確認等）は行わない。前提ファイルの範囲で判断がつかない場合は、推測で探索を広げず [03_open_questions.md](03_open_questions.md) に記載するか、タスク側で仮決定してCodexメモに記載する。
- 依存関係: `01_detail_route_dispatch` → `02_movie_detail_screen` の順に着手する（映画詳細画面はディスパッチャ導入後の`/media/:id`ルート構成に組み込む前提のため）。

## 完了の定義（DoD）

各タスクは以下をすべて満たして完了とする。

- [ ] `yarn lint` が通る
- [ ] `yarn test` が通る（対象タスクのユニットテストを含む）
- [ ] `yarn build`（`tsc -b && vite build`）が型エラーなく通る
- [ ] 設計書の該当セクションで定義されたprops/クラス名/挙動と実装が一致している（差異があればCodexメモに記載）
- [ ] `docs/frontend/ui/20_movie_detail.html` + `_shared.css` の見た目・DOM構造とTailwind実装がセマンティックに一致している（対応モックがある場合）
- [ ] 対応モックがあるタスクは `yarn test:e2e` の実装⇔モック レイアウト一致テストが通る
