# 03_academic_books 実装タスク概要

`docs/frontend/design/03_academic_books.md`（以下「設計書」、実体は `docs/frontend/design/02_general_media.md` 末尾の「差分: 03_academic_books」）を実装に落とし込むための計画。対象は学術書・専門書一覧画面（`/academic-books`）。`AppShell`・`FilterToolbar`・`MediaGrid`・`MediaCard`・`LoadMoreSentinel`・`useInfiniteScroll`等の共通基盤、および`useMediaListData`（`docs/frontend/tasks/02_general_media/`で実装済み）を再利用し、`media_type=academic_book`固定・種別フィルタ無し・ソート選択肢4種という差分のみを実装する単一タスクとする。

## タスクファイル構成

| ファイル | 内容 | 対応する設計書セクション |
|---|---|---|
| [01_academic_book_list_screen.md](01_academic_book_list_screen.md) | `AcademicBookListPage`（`/academic-books`）本体の実装 | 02_general_media.md §1〜§7、差分節 |
| [02_open_questions.md](02_open_questions.md) | 設計書中の【要確認】項目・バッジ表示名の由来など未決事項の追跡 | 差分節 |

## Claude ⇄ Codex 運用ルール

このタスク群は Claude が計画・レビューを行い、実装は Codex に委譲する前提で書式を統一している。

- 各タスクファイルの「タスク一覧」「テストリスト」のチェックボックス `- [ ]` は、実装・テストが完了するたびに **Codexが** `- [x]` に変更する（Claude側では変更しない）。
- タスクを完了する際、対応するタスク見出しの直下にある `> Codexメモ:` 行に、実装上の判断・設計書との差分・未解決事項を1〜3行で追記する（無ければ `> Codexメモ: (なし)` のままでよい）。
- 【要確認】に該当する意思決定が必要になった場合は、実装を進めず [02_open_questions.md](02_open_questions.md) に追記し、Claudeのレビューを待つ。ブロッキングでない場合は妥当な仮決定を行い、その旨をメモに残して先に進んでよい。
- **実装範囲の限定**: 着手するタスクの「タスク一覧」に書かれた内容の実装のみを行う。参照してよいファイルは各タスクファイルの「前提ファイル」節に列挙されたものと、そこから直接importされるファイルに限定する。それ以外のファイルを探すための横断的なコードベース探索（grep/find等での関連ファイル探索、他タスクの実装状況の確認等）は行わない。前提ファイルの範囲で判断がつかない場合は、推測で探索を広げず [02_open_questions.md](02_open_questions.md) に記載するか、タスク側で仮決定してCodexメモに記載する。

## 完了の定義（DoD）

各タスクは以下をすべて満たして完了とする。

- [ ] `yarn lint` が通る
- [ ] `yarn test` が通る（対象タスクのユニットテストを含む）
- [ ] `yarn build`（`tsc -b && vite build`）が型エラーなく通る
- [ ] 設計書の該当セクションで定義されたprops/クラス名/挙動と実装が一致している（差異があればCodexメモに記載）
- [ ] `docs/frontend/ui/03_academic_books.html` + `_shared.css` の見た目・DOM構造とTailwind実装がセマンティックに一致している
- [ ] `yarn test:e2e` の実装⇔モック レイアウト一致テストが通る
