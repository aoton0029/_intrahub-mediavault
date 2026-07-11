# 16_anime_detail 実装タスク概要

`docs/frontend/design/16_anime_detail.md`（以下「設計書」）を実装に落とし込むための計画。対象はアニメ詳細画面（`/media/:id`、`mediaType: anime`）。`DetailLayout`・`DetailRail`・`DetailMain`・`GroupList`/`EpisodeRow`・`StaffList`・`StreamingLinks`・`RailSection`・`StatusSwitcher`/`RatingStars`/`FavoriteToggle`/`TagList`・`detailSectionMatrix`（`anime`エントリ: `propertyList: false, groupList: true, staffList: true, streaming: true`）等の共通基盤は `docs/frontend/tasks/00_common/` で実装済みであることが前提。画面固有コンポーネントは無く、既存共通コンポーネントの組み合わせ＋データ取得・更新フックの実装のみで完結するため、単一タスクとする。

## タスクファイル構成

| ファイル | 内容 | 対応する設計書セクション |
|---|---|---|
| [01_anime_detail_screen.md](01_anime_detail_screen.md) | `AnimeDetailPage`（`/media/:id`）本体、`ItemDetail`取得・シーズン/エピソード・スタッフ・関連作品・配信・リソース・タグ/カテゴリ/マイリストの各データ取得・更新フックの実装 | 設計書 全節 |
| [02_open_questions.md](02_open_questions.md) | 設計書中の【要確認】項目・マイリスト所属取得APIの扱いなど未決事項の追跡 | 設計書 全節 |

## Claude ⇄ Codex 運用ルール

このタスク群は Claude が計画・レビューを行い、実装は Codex に委譲する前提で書式を統一している。

- 各タスクファイルの「タスク一覧」「テストリスト」のチェックボックス `- [ ]` は、実装・テストが完了するたびに **Codexが** `- [x]` に変更する（Claude側では変更しない）。
- タスクを完了する際、対応するタスク見出しの直下にある `> Codexメモ:` 行に、実装上の判断・設計書との差分・未解決事項を1〜3行で追記する（無ければ `> Codexメモ: (なし)` のままでよい）。
- 【要確認】に該当する意思決定が必要になった場合は、実装を進めず [02_open_questions.md](02_open_questions.md) に追記し、Claudeのレビューを待つ。ブロッキングでない場合は妥当な仮決定を行い、その旨をメモに残して先に進んでよい。
- **実装範囲の限定**: 着手するタスクの「タスク一覧」に書かれた内容の実装のみを行う。参照してよいファイルは各タスクファイルの「前提ファイル」節に列挙されたものと、そこから直接importされるファイルに限定する。それ以外のファイルを探すための横断的なコードベース探索（grep/find等での関連ファイル探索、他タスクの実装状況の確認等）は行わない。前提ファイルの範囲で判断がつかない場合は、推測で探索を広げず [02_open_questions.md](02_open_questions.md) に記載するか、タスク側で仮決定してCodexメモに記載する。

## 完了の定義（DoD）

各タスクは以下をすべて満たして完了とする。

- [x] `yarn lint` が通る
- [x] `yarn test` が通る（対象タスクのユニットテストを含む）
- [x] `yarn build`（`tsc -b && vite build`）が型エラーなく通る
- [x] 設計書の該当セクションで定義されたprops/クラス名/挙動と実装が一致している（差異があればCodexメモに記載）
- [x] `docs/frontend/ui/16_anime_detail.html` + `_shared.css` の見た目・DOM構造とTailwind実装がセマンティックに一致している
- [x] `yarn test:e2e` の実装⇔モック レイアウト一致テストが通る
