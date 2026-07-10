# 00. 実装計画 全体像

`docs/frontend/design/00_common.md`（以下「共通設計」）および各画面設計書（`docs/frontend/design/01_home.md` 〜 `24_game_detail.md`、計18本）を実装に落とし込むための計画ドキュメント群。`frontend/` は React 19 + React Router v7 + TanStack Query + shadcn/ui(Tailwind v4) + react-hook-form/zod のスキャフォールドのみで、`src/` には `App.tsx`/`App.css` しかなく、ページ・コンポーネント・ルーティングは未実装の状態から着手する。

参照元:
- `docs/frontend/design/00_common.md`（共通設計、以下 §1〜§7 は本ドキュメントの章番号）
- `docs/frontend/design/*.md`（画面別設計、18本）
- `docs/frontend/ui/*.html`, `_shared.css`, `_shared.js`（モック）
- `docs/backend/mediavault-api/*.md`（API仕様、リソース単位）

このディレクトリのファイルはタスク一覧のみを扱い、コードは書かない。実装フェーズで各タスクファイルのチェックリストを実装順に消化していく。

---

## フェーズ構成

| フェーズ | ファイル | 内容 | 依存 |
|---|---|---|---|
| 0 | [01_foundation.md](01_foundation.md) | トークン定義・AppShell・テーマ切替・ルーティング骨格 | なし |
| 1 | [02_shared_components.md](02_shared_components.md) | 共通コンポーネント（MediaCard, FilterToolbar, StatusSwitcher 等） | フェーズ0 |
| 2 | [03_detail_layout.md](03_detail_layout.md) | 詳細画面共通パターン（DetailLayout/DetailRail 等） | フェーズ1 |
| 3 | [04_screens.md](04_screens.md) | 18画面それぞれの実装タスク | フェーズ0〜2（詳細画面はフェーズ2必須） |
| - | [05_open_questions.md](05_open_questions.md) | 未確定事項（【要確認】）の集約 | 随時更新 |

完了条件:
- フェーズ0: `AppShell`配下に空の`<Outlet>`ルートが表示され、テーマ切替・サイドバーナビが動作する
- フェーズ1: §3一覧表の全コンポーネントがStorybook等なしでも単体利用可能な状態で実装済み
- フェーズ2: 8詳細画面すべてで共有される`DetailLayout`系コンポーネントが実装済み
- フェーズ3: 18画面すべてが対応するモックと同等の見た目・挙動で実装済み

---

## 画面 → 設計書 → モック 対応表

| # | 画面名 | 設計書 | モック |
|---|---|---|---|
| 01 | ホーム | [design/01_home.md](../design/01_home.md) | [ui/01_home.html](../ui/01_home.html) |
| 02 | 一般メディア一覧 | [design/02_general_media.md](../design/02_general_media.md) | [ui/02_general_media.html](../ui/02_general_media.html) |
| 03 | 学術書一覧 | [design/03_academic_books.md](../design/03_academic_books.md) | [ui/03_academic_books.html](../ui/03_academic_books.html) |
| 04 | 論文一覧 | [design/04_papers.md](../design/04_papers.md) | [ui/04_papers.html](../ui/04_papers.html) |
| 06 | マイリスト一覧 | [design/06_mylists.md](../design/06_mylists.md) | [ui/06_mylists.html](../ui/06_mylists.html) |
| 11 | 論文登録フォーム | [design/11_paper_form.md](../design/11_paper_form.md) | [ui/11_paper_form.html](../ui/11_paper_form.html) |
| 12 | 一般メディア検索 | [design/12_general_media_search.md](../design/12_general_media_search.md) | [ui/12_general_media_search.html](../ui/12_general_media_search.html) |
| 13 | 学術書検索 | [design/13_academic_book_search.md](../design/13_academic_book_search.md) | [ui/13_academic_book_search.html](../ui/13_academic_book_search.html) |
| 15 | マイリスト詳細 | [design/15_mylist_detail.md](../design/15_mylist_detail.md) | [ui/15_mylist_detail.html](../ui/15_mylist_detail.html) |
| 16 | アニメ詳細 | [design/16_anime_detail.md](../design/16_anime_detail.md) | [ui/16_anime_detail.html](../ui/16_anime_detail.html) |
| 17 | 学術書詳細 | [design/17_academic_book_detail.md](../design/17_academic_book_detail.md) | [ui/17_academic_book_detail.html](../ui/17_academic_book_detail.html) |
| 18 | 論文詳細 | [design/18_paper_detail.md](../design/18_paper_detail.md) | [ui/18_paper_detail.html](../ui/18_paper_detail.html) |
| 19 | 設定 | [design/19_settings.md](../design/19_settings.md) | [ui/19_settings.html](../ui/19_settings.html) |
| 20 | 映画詳細 | [design/20_movie_detail.md](../design/20_movie_detail.md) | [ui/20_movie_detail.html](../ui/20_movie_detail.html) |
| 21 | ドラマ詳細 | [design/21_drama_detail.md](../design/21_drama_detail.md) | [ui/21_drama_detail.html](../ui/21_drama_detail.html) |
| 22 | マンガ詳細 | [design/22_manga_detail.md](../design/22_manga_detail.md) | [ui/22_manga_detail.html](../ui/22_manga_detail.html) |
| 23 | 小説詳細 | [design/23_novel_detail.md](../design/23_novel_detail.md) | [ui/23_novel_detail.html](../ui/23_novel_detail.html) |
| 24 | ゲーム詳細 | [design/24_game_detail.md](../design/24_game_detail.md) | [ui/24_game_detail.html](../ui/24_game_detail.html) |

※ 番号は共通設計・モックの命名に合わせた欠番あり（05, 07〜10, 14 は存在しない）。全18画面。
