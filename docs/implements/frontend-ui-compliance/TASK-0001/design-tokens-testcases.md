# TASK-0001: デザイントークンの再定義（index.css）- TDDテストケース洗い出し

## 0. メタ情報

- **機能名**: design-tokens（デザイントークン再定義）
- **タスクID**: TASK-0001
- **要件名**: frontend-ui-compliance
- **参照ノート**: `docs/implements/frontend-ui-compliance/TASK-0001/note.md`
- **参照要件定義**: `docs/implements/frontend-ui-compliance/TASK-0001/design-tokens-requirements.md`

---

## 0.5 前提の確認（要件定義書で「要改善」とされた2点への対応方針の確定）

本タスクは **CSS変数（デザイントークン）の値定義のみ** を対象とし、値そのものを既存のテストフレームワーク（Vitest + Testing Library, jsdom）で直接アサーションすることは困難（jsdomはブラウザの実CSSカスケード計算・フォント読み込み・`@theme inline`のTailwindユーティリティ生成を再現しないため）。したがって **本タスクでは新規の自動テストコード（`*.test.ts(x)`）は作成しない** 🔵（`docs/tasks/frontend-ui-compliance/TASK-0001.md`単体テスト要件節に明記）。

以下、要件定義書（`design-tokens-requirements.md`）で🔴🟡の「要確認事項」とされていた2点について、実装時に採用する対応方針をこのステップで確定する。

### (A) `--radius` 名前衝突の対応方針 🟡→確定

- 既存 `:root` の `--radius: 0.625rem`（`frontend/src/index.css` 99行目、shadcn用）を **`--radius-shadcn: 0.625rem`にリネーム** する。
- 新規 `--radius: 6px`（`_shared.css`準拠）を追加する。
- `@theme inline` ブロック内（`frontend/src/index.css` 248-254行目）の `--radius-sm`〜`--radius-4xl` の計算式が参照している `var(--radius)` を **`var(--radius-shadcn)` に書き換える**（shadcnコンポーネントの角丸が新トークンの6pxベースに巻き込まれて意図せず変化することを防ぐため）。
  - これは「`@theme inline`のマッピング名変更（Tailwindユーティリティ・shadcnトークン連携）はTASK-0002」というスコープ制約に対する**必要最小限の例外**と位置づける。マッピング先の変数名を1文字も追加/削除せず、参照元シンボルのみを追随修正するもので、ユーティリティクラス構成自体は変更しないため、スコープ逸脱ではないと判断する（🟡本ステップでの判断、次工程レビューで妥当性再確認）。

### (B) `@theme inline`未定義参照リスク（`--border`衝突含む）の対応方針 🔴→確定

- `--border`は「shadcn系のborder色」と「`_shared.css`のborder色」が**同一名・同一用途（枠線色）**であるため、リネームせず**値のみを`#383838`に置換**する（`--radius`のような「別トングの計算式」に使われていないため衝突リスクが実質的にない）。
- 旧トークン（`--bg-base`, `--bg-surface`, `--bg-elevated`, `--text-primary`, `--text-secondary`, `--border-default`）は`:root`から**削除**し、新トークン名に置換する。これにより `@theme inline` の以下6行（`frontend/src/index.css` 199-204行目）が **未定義のCSS変数を参照する状態になる**：
  ```css
  --color-bg-base: var(--bg-base);
  --color-bg-surface: var(--bg-surface);
  --color-bg-elevated: var(--bg-elevated);
  --color-text-primary: var(--text-primary);
  --color-text-secondary: var(--text-secondary);
  --color-border-default: var(--border-default);
  ```
- 対応方針: 本タスクでは`@theme inline`ブロック自体の**構造変更（マッピング先の再設計）はTASK-0002で実施**するため、上記6行は**削除しない**。未定義変数を参照した場合、Tailwindはその軸（`bg-bg-base`等のユーティリティクラス）を生成しないか、`unset`相当として扱われる可能性があるが、**現在これらのユーティリティクラス（`bg-bg-base`, `text-text-primary`等）を使用しているコンポーネントの有無を実装時に確認**する（`grep -r "bg-bg-base\|text-text-primary\|border-border-default" frontend/src`）。
  - 使用箇所が無ければ実害なし（デッドコードの整理はTASK-0002の範疇）。
  - 使用箇所がある場合、本タスクの完了条件（回帰確認: `vitest run`全パス、目視確認）で検知できるため、既存コンポーネントテストの回帰実行と目視確認が実質的な安全網となる。
  - 🔴→🟡: 「未定義参照になるかどうか」自体は資料に明記がなかったが、上記grep調査と回帰テスト実行によって実装時に検知可能であることを確認した。この対応方針を要件の確定事項とする。

---

## 1. 検証観点の全体像

本タスクは自動アサーション可能な単体テストを持たないため、通常の正常系/異常系/境界値テストケースではなく、**完了条件（`docs/tasks/frontend-ui-compliance/TASK-0001.md`）に対応する検証観点**を「テストケース」として整理する。分類は以下の3種とする。

1. **回帰確認テスト**（既存の自動テストを実行して確認するもの）
2. **ビルド確認テスト**（コマンド実行で機械的に確認するもの）
3. **目視・手動確認テスト**（人間の目視・ブラウザ操作で確認するもの）

各項目に信頼性レベルを付与する。

---

## 2. 回帰確認テストケース（既存の自動テストを利用）

### TC-01: 既存コンポーネントテストの全件回帰実行
- **何をテストするか**: `index.css`のトークン置換によって、既存コンポーネントが参照しているTailwindユーティリティクラス名（`bg-*`, `text-*`, `border-*`等）の解決が壊れていないか
- **実行コマンド**: `yarn test`（`vitest run`、`frontend/vitest.config.ts`）
- **入力**: 変更後の`frontend/src/index.css`一式（トークン置換後）
- **期待される結果**: 既存の全テストスイートが失敗なく完了する（Exit code 0）。特に以下のファイルの回帰が無いこと:
  - `frontend/src/components/common/MediaCard.test.tsx`
  - `frontend/src/components/common/Sidebar.test.tsx`
  - `frontend/src/components/common/FilterBar.test.tsx`
  - `frontend/src/components/common/FilterBar.test.a11y.tsx`, `Sidebar.test.a11y.tsx`
  - `frontend/src/pages/HomePage.test.tsx`, `GeneralListPage.test.tsx`
- **期待結果の理由**: 本タスクはクラス名の追加・変更を伴わず値の置換のみであるため、DOM構造・クラス名を検証する既存テストは影響を受けないはずである
- **確認ポイント**: 失敗するテストが1件でもあれば、`@theme inline`未定義参照（2.(B)節）またはクラス名巻き込みの兆候として即座に調査する
- 🔵 信頼性レベル: 🔵（タスク定義「単体テスト要件」に明記の完了条件）

### TC-02: media_type別アクセントカラーのユニットテスト（既存があれば）への影響確認
- **何をテストするか**: `--accent-anime`等8色を参照するテスト・スナップショットが存在する場合、値が変更されていないこと
- **実行コマンド**: `yarn test`実行結果内でMediaCard関連テストのスナップショット差分の有無を確認
- **期待される結果**: media_type別アクセントカラーに関するテスト（存在する場合）で差分が出ないこと（REQ-002により値・変数名とも不変のため）
- **確認ポイント**: スナップショットテストがある場合のみ対象。無ければ本項目は目視確認（TC-08）に委ねる
- 🟡 信頼性レベル: 🟡（該当スナップショットテストの実在は未確認、note.mdのテスト配置パターンからの推測）

---

## 3. ビルド確認テストケース（コマンド実行で機械的に確認）

### TC-03: 型チェック込みビルドの成功確認
- **何をテストするか**: `index.css`編集後、TypeScriptの型チェックとViteビルドがエラーなく完了するか
- **実行コマンド**: `yarn build`（`tsc -b && vite build`）
- **期待される結果**: Exit code 0、ビルド成果物（`dist/`）が生成される。CSS構文エラー（重複プロパティ、閉じ括弧不足等）があればViteのCSSパイプラインでビルドエラーとして検出される
- **期待結果の理由**: タスク完了条件「`yarn build`がエラーなく完了する」に直接対応
- **確認ポイント**: `--radius`リネーム後の`@theme inline`側`var(--radius-shadcn)`参照ミスタイプ（変数名の不一致）はビルドエラーにはならず不可視のスタイル崩れとして現れる可能性があるため、TC-05（角丸目視）と併用する
- 🔵 信頼性レベル: 🔵（タスク定義完了条件に明記）

### TC-04: CSS変数定義の存在確認（軽量な静的チェック、実装時の目視/grepで実施）
- **何をテストするか**: `frontend/src/index.css`の`:root`ブロックに、完了条件に列挙された新トークンがすべて定義されているか
- **確認方法**: 実装後に`frontend/src/index.css`を目視、または`grep -E "\-\-(bg-app|bg-sidebar|bg-surface|bg-surface-hover|bg-input|border:|border-soft|text-primary|text-muted|text-faint|accent:|accent-strong|accent-soft|favorite|status-progress|status-done|status-none|danger|font-ui|font-display|font-mono|sidebar-w|properties-w|radius:|radius-shadcn)" frontend/src/index.css`で列挙し、完了条件の全項目（8項目）と突き合わせる
- **期待される結果**: 完了条件に列挙された全トークン名が`:root`内に存在する
- **確認ポイント**:
  - `--bg-base`, `--bg-surface`（旧#1a1d23）, `--bg-elevated`, `--text-secondary`, `--border-default`が削除されていること（旧`--bg-surface`は新`--bg-surface: #262626`に値のみ変わる想定のため、変数名としては存続するが値が変わっている点に注意）
  - `--radius-shadcn: 0.625rem`が存在し、`--radius: 6px`と共存していること
  - media_type別アクセントカラー8色（`--accent-anime`等）が**変更されず**存在していること（変数名・値ともに一致）
- 🔵 信頼性レベル: 🔵（完了条件チェックリストに1:1対応、自動テストではなく実装時の確認手順として位置づけ）

### TC-05: `@theme inline`未定義参照の実害有無確認（静的grep調査）
- **何をテストするか**: 削除される旧トークン名（`--bg-base`, `--bg-surface`旧値, `--bg-elevated`, `--text-primary`旧値, `--text-secondary`, `--border-default`）を参照するTailwindユーティリティクラス（`bg-bg-base`, `text-text-primary`, `border-border-default`等）が、コンポーネント側で使用されていないか
- **実行コマンド**: `grep -rn "bg-bg-base\|bg-bg-surface\|bg-bg-elevated\|text-text-primary\|text-text-secondary\|border-border-default" frontend/src`
- **期待される結果**: ヒットなし、またはヒットした場合は影響範囲を洗い出しTASK-0002に申し送るか本タスク内で対応するかを判断する記録を残す
- **確認ポイント**: ヒットがあった場合、TC-01の回帰テストが該当コンポーネントのレンダリングテストで検知できるかを再確認する
- 🟡 信頼性レベル: 🟡（0.5節(B)の対応方針に基づく新規調査観点、既存資料に明記なし）

---

## 4. 目視・手動確認テストケース

### TC-06: 背景色のダークグレー化 目視確認
- **何をテストするか**: `yarn dev`起動後、HomePage・ItemDetailPage・SettingsPageの背景色が`#1e1e1e`系の暗いグレーになっているか
- **入力値**: ブラウザで各ページを開いた状態
- **入力データの意味**: REQ-001の受け入れ基準（全体一覧・詳細・設定の各画面でダーク背景が表示される）を代表する3画面
- **期待される結果**: 背景色が旧`#0f1115`ではなく新`#1e1e1e`（サイドバーは`#161616`）になっている
- **期待結果の理由**: `--bg-app`, `--bg-sidebar`トークンが`_shared.css`準拠の値に置換されているため
- **確認ポイント**: ブラウザDevToolsのComputed Stylesで`--bg-app`等のCSS変数の実値を確認できるとなお良い（自動テストではなく手動確認）
- 🔵 信頼性レベル: 🔵（タスク完了条件・要件定義の使用例に明記）

### TC-07: 単一アクセント色の適用確認
- **何をテストするか**: ボタン等のプライマリアクセントに新しい単一アクセント色（`--accent: #8b6cf6`）が適用されているか
- **期待される結果**: 旧アクセント色ではなく紫系の新アクセント色（`#8b6cf6`系）が視認できる
- **確認ポイント**: 本タスクでは`@theme inline`のTailwindユーティリティ連携（`--color-accent`等）はTASK-0002対応のため、`var(--accent)`を直接使用している箇所のみ本タスク時点で反映される可能性がある点に留意（未反映箇所があってもTASK-0002待ちとして許容）
- 🟡 信頼性レベル: 🟡（`@theme inline`未対応のため本タスク時点での完全反映は保証されない、スコープ制約からの妥当な推測）

### TC-08: media_type別アクセントカラーの巻き込み事故有無 目視確認
- **何をテストするか**: 各MediaCardのバッジ等でmedia_type別アクセントカラー8色（`--accent-anime`等）が変更前と同じ色で表示されているか
- **期待される結果**: 8色すべてが変更前の値のまま表示される（例: anime=`#f97316`のオレンジ）
- **確認ポイント**: 新規`--accent`（単一アクセント色、紫系）と混同・上書きされていないこと
- 🔵 信頼性レベル: 🔵（REQ-002、タスク完了条件に明記）

### TC-09: フォント読み込みの確認
- **何をテストするか**: Google Fonts CDN経由でInter / Source Serif 4 / JetBrains Monoが読み込まれ、`--font-ui`等の変数が定義されているか
- **確認方法**: ブラウザDevToolsのNetworkタブでフォントリクエストの成否を確認、または`--font-ui`を直接使用している要素（あれば）のフォント表示を確認
- **期待される結果**: フォントリクエストが200で完了する。仮にネットワーク遮断等で失敗した場合もフォールバック（`-apple-system, sans-serif`等）でレイアウト崩壊しないこと
- **確認ポイント**: 本タスク時点では`@theme inline`の`--font-sans`（Geist Variable）は変更されないため、既存コンポーネントの表示フォントが直ちに変わるとは限らない（TASK-0002待ち）。本タスクでは変数定義の追加とCDN読み込み成功のみを確認範囲とする
- 🟡 信頼性レベル: 🟡（フォント導入手段がCDN方式である点は🟡推測、確認範囲の切り分けは本ステップでの判断）

### TC-10: Dockerビルドでの配信確認（該当する場合）
- **何をテストするか**: nginx配信用Dockerイメージのビルドが本CSS変更で失敗しないか
- **実行コマンド**: `docker build -f frontend/Dockerfile -t mediavault-frontend-test frontend`
- **期待される結果**: ビルド成功、`docker run`後に`curl http://localhost:8081/`でHTMLが返り、想定通りのスタイルが適用されたページが取得できる
- **確認ポイント**: 本タスクはCSSのみの変更で依存関係変更を伴わないため、影響は小さいと推測されるが、note.mdに手順が明記されているため実施しておく
- 🟡 信頼性レベル: 🟡（note.mdに「本タスクはCSSのみのため影響小」と記載、必須実施かは判断による）

---

## 5. 開発言語・フレームワーク

- **プログラミング言語**: CSS（変更対象）/ 確認スクリプトはシェル（bash, grep）
  - **言語選択の理由**: 本タスクの変更対象がCSSファイルであり、新規のTypeScript/JavaScriptロジックを伴わないため
  - **テストに適した機能**: 該当なし（CSS変数はロジックを持たないため、既存のVitestベースの回帰テストとコマンドラインでの静的確認を組み合わせる）
- **テストフレームワーク**: Vitest（既存回帰テストの実行用、`frontend/vitest.config.ts`）、Testing Library（コンポーネントレンダリング確認用、新規テスト追加なし）
  - **フレームワーク選択の理由**: 新規テストは作成しないが、既存の回帰確認の実行基盤として利用するため、プロジェクトに既に導入済みのVitestをそのまま用いる
  - **テスト実行環境**: ローカル開発環境（Node.js + Vite dev server）、CI環境（該当する場合）。ビルド確認は`yarn build`、E2E/目視確認は`yarn dev`のブラウザ確認
- 🔵 信頼性レベル: 🔵（note.md「技術スタック」「テスト関連情報」に明記の既存構成をそのまま利用する方針）

---

## 6. 要件定義との対応関係

- **参照した機能概要**: `design-tokens-requirements.md` 1節（旧トークンの`_shared.css`準拠置換）
- **参照した入力・出力仕様**: `design-tokens-requirements.md` 2節（置換前後のトークン一覧、`--radius`/`--border`衝突の詳細）
- **参照した制約条件**: `design-tokens-requirements.md` 3節（スコープ制約・単体テスト方針制約）
- **参照した使用例・エッジケース**: `design-tokens-requirements.md` 4節（変数名衝突、`@theme inline`との不整合、フォント読み込み失敗）
- **参照した完了条件**: `docs/tasks/frontend-ui-compliance/TASK-0001.md` 完了条件（全8項目）

---

## 7. 品質判定

- **テストケース分類**: 本タスクの性質上、通常の正常系/異常系/境界値の代わりに「回帰確認・ビルド確認・目視確認」の3分類で網羅（TC-01〜TC-10、計10項目）
- **期待値定義**: 各TCに具体的な期待結果（値・Exit code・grep結果）を明記済み
- **技術選択**: 確定（Vitest, `yarn build`, grep, ブラウザ目視 — 追加ライブラリなし）
- **実装可能性**: 現在の技術スタックで実現可能（新規テストコード作成不要という制約を踏まえた確認手順のみ）
- **要改善事項への対応**: 要件定義書で「要改善」とされていた(A)`--radius`名前衝突、(B)`@theme inline`未定義参照リスクについて、0.5節で具体的な対応方針（リネーム＋参照更新、grep調査によるリスク検知）を確定した
- **信頼性レベル分布**: 🔵7件、🟡5件（実装判断を要する箇所に集中、要件定義書の分布と同傾向）

**総合評価**: ✅ 高品質（検証観点が完了条件と1:1対応し、要改善だった2点の対応方針も本ステップで確定した）

---

## 次のお勧めステップ

`/tsumiki:tdd-red frontend-ui-compliance TASK-0001` でRedフェーズ（失敗テスト作成、ただし本タスクは新規自動テスト非作成方針のため、TC-04/TC-05の静的確認手順の実施記録を先に行う）を開始します。
</content>
</invoke>
