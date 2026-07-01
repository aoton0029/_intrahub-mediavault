# TDDテストケース定義書: アクセシビリティ・レスポンシブ対応 (TASK-0034)

**機能名**: accessibility-responsive（アクセシビリティ・レスポンシブ対応）
**タスクID**: TASK-0034
**要件名**: frontend-collection-ui
**出力ファイル**: `docs/implements/frontend-collection-ui/TASK-0034/accessibility-responsive-testcases.md`

## 信頼性レベル指示

- 🔵 **青信号**: 要件定義書・タスクファイル・既存実装を参考にしてほぼ推測していない
- 🟡 **黄信号**: 要件定義書・既存実装から妥当な推測
- 🔴 **赤信号**: 元資料にない推測

> 一次情報: `docs/tasks/frontend-collection-ui/TASK-0034.md`（単体テスト要件TC-1〜6・UI/UX要件）、`docs/implements/frontend-collection-ui/TASK-0034/accessibility-responsive-requirements.md`、および対象ソース実地調査（`ItemFormPage.tsx` / `SettingsPage.tsx` / `Sidebar.tsx` / `FilterBar.tsx` / `ConfirmDialog.tsx`）。本タスク専用の `note.md` は未生成のため上記を一次情報とした。

---

## 開発言語・フレームワーク

- **プログラミング言語**: TypeScript (React 18.3+ / TSX)
  - **言語選択の理由**: フロントエンドが React + TypeScript + Vite で統一されており（`frontend/CLAUDE.md`）、型安全に DOM 属性・role を検証できる。🔵
  - **テストに適した機能**: JSX/TSX を直接レンダリングして aria 属性・role・ラベル関連付けをアサートできる。
- **テストフレームワーク**: Vitest + @testing-library/react + @testing-library/user-event（+ jest-dom マッチャ）
  - **フレームワーク選択の理由**: 既存の単体テスト（`Sidebar.test.tsx`, `FilterBar.test.tsx`, `ConfirmDialog.test.tsx` 等）が Vitest + Testing Library で書かれており一貫性を保つ。🔵
  - **テスト実行環境**: `yarn test`（jsdom 環境）。Radix UI Dialog の Esc/フォーカストラップも jsdom + user-event で検証可能。
- 🔵 信頼性レベル: `frontend/CLAUDE.md` の開発コマンドおよび既存テスト構成より確実。

---

## 実装対象と現状（実地調査サマリ）

| 対象 | 実ファイルパス | 現状 | 本タスクでの想定作業 |
|---|---|---|---|
| 手動追加・編集フォーム | `frontend/src/pages/ItemFormPage.tsx` | shadcn/ui `Form`/`FormLabel`/`FormControl` 使用済み。`aria-describedby` は `FormMessage` 経由で標準提供見込み | 確認・テスト追加 |
| APIキー登録タブ | `frontend/src/pages/SettingsPage.tsx` | ⚠️ `ProviderRow` の `<Label>` に `htmlFor` が無く `Input` の `id` も無い（ラベル未関連付けの実欠陥） | **修正**（`htmlFor`/`id` 付与または `aria-label`）＋テスト |
| インポートタブ | `frontend/src/pages/SettingsPage.tsx` | `booklog-file` / `steam-id` は `Label htmlFor`/`id` 関連付け済み | 確認・テスト追加 |
| サイドバー | `frontend/src/components/common/Sidebar.tsx` | `NavLink`（ネイティブ `<a>`）+ `nav aria-label`。モバイル `Sheet`/ハンバーガーは未実装 | **新規追加**（モバイル対応）＋テスト |
| フィルタUI | `frontend/src/components/common/FilterBar.tsx` | `label htmlFor`/`id` 関連付け済み、チェックボックスもラベル内包 | 確認・テスト追加 |
| 削除確認ダイアログ | `frontend/src/components/common/ConfirmDialog.tsx` | shadcn/ui `Dialog`（Radix）で Esc/フォーカストラップ標準提供 | 確認・テスト追加 |

> ⚠️ タスク記載の `components/layout/Sidebar.tsx` / `FilterBar.tsx` は誤りで、実体は `components/common/` 配下（requirements.md 要確認事項1 と一致）。🔵

---

# 1. 正常系テストケース（基本的な動作）

### TC-01: 手動追加フォームのタイトル入力にラベルが関連付けられている（TASK-0034 TC-1）

- **テスト名**: ItemFormPage（手動追加）のタイトル入力が `getByLabelText` で取得できる
  - **何をテストするか**: `FormLabel`「タイトル」と対応する入力が `htmlFor`/`id`（または `aria-labelledby`）で関連付いていること
  - **期待される動作**: スクリーンリーダーがフォーカス時にラベルを読み上げられる
- **入力値**: `<ItemFormPage mode="create" group="general" />` をレンダリング
  - **入力データの意味**: 手動追加モードが NFR-202 のラベル付け主対象（TASK-0034 TC-1）
- **期待される結果**: `screen.getByLabelText(/タイトル/)` が `<input>`（またはtextarea）を1件返す
  - **期待結果の理由**: `FormLabel`+`FormControl` が Radix `Label`/`id` により関連付けられるため（`ItemBaseFields` 経由）
- **テストの目的**: フォームラベルの機能的関連付けの確認
  - **確認ポイント**: プレースホルダ代替ではなく実ラベルで取得できること
- 🔵 信頼性レベル: TASK-0034 TC-1 と `ItemFormPage.tsx` 実装（shadcn/ui Form 使用）より確実

### TC-02: APIキー登録フォームの各入力にラベルが関連付けられている（TASK-0034 TC-2）

- **テスト名**: SettingsPage APIキータブで各プロバイダ入力が `getByLabelText`（またはaria-label）で取得できる
  - **何をテストするか**: TMDB/IGDB/NDL/Steam/Open Library/AniList 各行の APIキー入力にアクセシブルネームがあること
  - **期待される動作**: 各サービス名に対応する入力欄を支援技術で識別できる
- **入力値**: `<SettingsPage />` をレンダリングし APIキータブ（デフォルト表示）を対象。編集状態に入り入力欄を表示させる（`maskedKey` 無しなら初期から `<Input>` 表示）
  - **入力データの意味**: REQ-403（APIキーは設定UI経由）に基づく主要フォーム
- **期待される結果**: 各行の入力が `getByLabelText('TMDB')` 等、またはアクセシブルネーム（`aria-label`）で取得できる
  - **期待結果の理由**: ラベルが入力に関連付いていれば取得可能
  - ⚠️ **現状は未関連付け**（`<Label>` に `htmlFor` 無し・`<Input>` に `id` 無し）のため、このテストは Red となる想定。Green フェーズで `htmlFor`/`id` または `aria-label` を付与して通す。
- **テストの目的**: APIキーフォームのラベル関連付けの確認と欠陥修正の駆動
  - **確認ポイント**: 6プロバイダすべてで取得可能なこと
- 🔵 信頼性レベル: TASK-0034 TC-2 より。現状欠陥は `SettingsPage.tsx` 実地調査より確実

### TC-03: サイドバー各リンクが link ロールで取得でき Tab フォーカス可能（TASK-0034 TC-3）

- **テスト名**: Sidebar の全ナビ項目が `getAllByRole('link')` で取得できネイティブ `<a>` である
  - **何をテストするか**: 8項目のナビが `<div onClick>` ではなくフォーカス可能なリンクで実装されていること
  - **期待される動作**: Tab キーで各リンクにフォーカスでき、Enter で遷移できる
- **入力値**: `MemoryRouter` でラップした `<Sidebar />`
  - **入力データの意味**: サイドバーは主要ナビゲーション（NFR-202 キーボード操作）
- **期待される結果**: `getAllByRole('link')` が 8 件返し、各要素が `A` タグ（`NavLink` 由来）である
  - **期待結果の理由**: `Sidebar.tsx` が `NavLink`（内部 `<a>`）で実装済み
- **テストの目的**: ナビゲーションのキーボード操作可能性の確認
  - **確認ポイント**: 件数（8）と要素タイプ、`nav` の `aria-label` 存在
- 🔵 信頼性レベル: TASK-0034 TC-3 と `Sidebar.tsx` 実装より確実

### TC-04: フィルタUIの各コントロールにラベル関連付け／aria属性がある（TASK-0034 TC-5）

- **テスト名**: FilterBar の media_type/タグ/カテゴリ/ステータス select とお気に入り checkbox がラベルで取得できる
  - **何をテストするか**: 各フィルタコントロールにアクセシブルネームがあること
  - **期待される動作**: 支援技術・キーボードで各フィルタを識別・操作できる
- **入力値**: `<FilterBar filters={{}} onChange={vi.fn()} tagOptions={[...]} categoryOptions={[...]} />`（タグ/カテゴリを各1件以上）
  - **入力データの意味**: `ItemListFilters` の全操作対象（interfaces.ts）
- **期待される結果**: `getByLabelText('メディアタイプ')`, `'タグ'`, `'カテゴリ'`, `'ステータス'`, `'お気に入り'` がそれぞれ対応コントロールを返す
  - **期待結果の理由**: `FilterBar.tsx` は `label htmlFor`/`id` 関連付けとチェックボックス内包ラベルを実装済み
- **テストの目的**: フィルタUIのラベル関連付けの確認
  - **確認ポイント**: select 4種＋checkbox 1種すべて取得可能
- 🔵 信頼性レベル: TASK-0034 TC-5 と `FilterBar.tsx` 実装より確実

### TC-05: モバイル幅でハンバーガーメニューからサイドバーを開閉できる（追加・UI/UX要件）

- **テスト名**: モバイル用トリガーで `Sheet` サイドバーが開き、ナビリンクが表示される
  - **何をテストするか**: モバイル幅でのサイドバー折りたたみ表示（`Sheet`）とハンバーガー起動（新規実装）
  - **期待される動作**: `aria-label="メニューを開く"` のボタン押下で `Sheet` が開き、内部にナビリンク（link ロール）が現れる
- **入力値**: モバイルレイアウトをレンダリングし、`getByRole('button', { name: 'メニューを開く' })` を `user.click`
  - **入力データの意味**: TASK-0034 UI/UX要件「モバイル幅サイドバーは Sheet 形式」🟡
- **期待される結果**: クリック後 `Sheet` 内に `getAllByRole('link')` が 8 件現れる
  - **期待結果の理由**: モバイルではハンバーガー→ドロワーでナビを提供する一般的パターン
  - この機能は net-new 実装のため Red からの実装駆動対象。
- **テストの目的**: モバイルレスポンシブなサイドバーの新規実装を駆動・確認
  - **確認ポイント**: トリガーの aria-label、開閉状態遷移、リンク到達可能性
- 🟡 信頼性レベル: TASK-0034 UI/UX要件・要件定義 制約(モバイルサイドバー)より妥当な推測。具体的 DOM/API は shadcn/ui `Sheet` 標準に依存

---

# 2. 異常系テストケース（エラーハンドリング / 支援技術向け異常系）

### TC-06: バリデーションエラー時に入力へ aria-describedby でエラーが関連付く（TASK-0034 TC-6）

- **テスト名**: ItemFormPage で必須（タイトル）未入力送信時、エラーメッセージが `aria-describedby` で入力に関連付く
  - **エラーケースの概要**: 入力エラー発生時に支援技術がエラー内容をフィールド読み上げ時に取得できない状態を防ぐ
  - **エラー処理の重要性**: NFR-201（フィールド近傍エラー表示）/ NFR-202。スクリーンリーダー利用者がどのフィールドが不正か把握できる必要がある
- **入力値**: `<ItemFormPage mode="create" group="general" />` でタイトル空のまま「保存」を `user.click`
  - **不正な理由**: `createItemRequestSchema` で `title` が必須のため zod バリデーション失敗
  - **実際の発生シナリオ**: 必須項目を入力し忘れて送信するケース
- **期待される結果**: タイトル入力の `aria-describedby` が、表示されたエラーメッセージ要素（`FormMessage`）の `id` を指す。かつ `aria-invalid="true"` が付く
  - **エラーメッセージの内容**: zod のメッセージが `FormMessage` に描画される
  - **システムの安全性**: 送信は中断され、フォーム状態は保持される
- **テストの目的**: エラーの aria 関連付けの確認
  - **品質保証の観点**: 支援技術利用者のエラー把握可能性を担保
- 🟡 信頼性レベル: TASK-0034 TC-6 より。具体挙動は shadcn/ui `Form`（`FormControl`/`FormMessage`）の `aria-describedby`/`aria-invalid` 標準機能に依存

### TC-07: 削除確認モーダルが Esc キーで閉じる（TASK-0034 TC-4）

- **テスト名**: ConfirmDialog（open=true）で Esc 押下すると onCancel が呼ばれ閉じる
  - **エラーケースの概要**: モーダル表示中にユーザーがキーボードのみでキャンセル（離脱）する操作
  - **エラー処理の重要性**: NFR-202。フォーカストラップ中でも Esc で安全に離脱できる必要がある
- **入力値**: `<ConfirmDialog open title="削除しますか" onConfirm={vi.fn()} onCancel={onCancel} />` をレンダリングし `user.keyboard('{Escape}')`
  - **不正な理由**: 該当なし（正常なキャンセル操作。異常系＝離脱経路の検証として分類）
  - **実際の発生シナリオ**: 削除確認を出したが取りやめる
- **期待される結果**: `onCancel` が1回呼ばれる（Radix の `onOpenChange(false)` → `onCancel` 集約）。ダイアログ内容が非表示になる
  - **エラーメッセージの内容**: 該当なし
  - **システムの安全性**: `onConfirm` は呼ばれず、削除は実行されない
- **テストの目的**: モーダルの Esc クローズ／フォーカストラップ離脱の確認
  - **品質保証の観点**: キーボードのみ利用者の操作完結性
- 🟡 信頼性レベル: TASK-0034 TC-4 より。挙動は shadcn/ui `Dialog`（Radix）標準機能に依存

### TC-08: ネイティブ file 入力にラベルが無い欠落を検出する（エッジ・欠落検出）

- **テスト名**: SettingsPage インポートタブの CSV file 入力がラベル取得できる（未関連付けなら失敗）
  - **エラーケースの概要**: shadcn/ui `Form` を経由しないネイティブ `<input type="file">` のラベル欠落
  - **エラー処理の重要性**: NFR-202。file 入力はラベル欠落が起きやすい典型箇所
- **入力値**: `<SettingsPage />` でインポートタブを開き（`user.click` でタブ切替）、file 入力を対象
  - **不正な理由**: ラベル未関連付けだと支援技術で用途不明になる
  - **実際の発生シナリオ**: CSV インポート操作
- **期待される結果**: `getByLabelText('CSVファイル')` が file 入力を返す（現状 `Label htmlFor="booklog-file"`/`id` 済みのため通る想定）。もし将来欠落したら失敗して検出
  - **エラーメッセージの内容**: 該当なし
  - **システムの安全性**: 該当なし
- **テストの目的**: Form 非経由入力のラベル欠落検出
  - **品質保証の観点**: NFR-202 の重大欠落（ラベル未設定）検出を優先
- 🟡 信頼性レベル: TASK-0034 実装詳細1・要件定義 エッジケースより妥当な推測。現状実装は関連付け済み

---

# 3. 境界値テストケース（最小・最大・状態境界）

### TC-09: フォーカス順序が視覚的順序と一致する（境界: 順序の先頭〜末尾）

- **テスト名**: Sidebar のリンクを連続 Tab したとき DOM 上の順序で 1→8 とフォーカスが移動する
  - **境界値の意味**: フォーカス順序の先頭（最初のリンク）と末尾（最後のリンク）が視覚順と一致する境界を確認
  - **境界値での動作保証**: 途中に `tabIndex` の逆転や到達不能要素が無いこと
- **入力値**: `MemoryRouter` で `<Sidebar />` をレンダリングし `user.tab()` を 8 回
  - **境界値選択の根拠**: 先頭/末尾はフォーカス順序不整合が最も表れやすい
  - **実際の使用場面**: キーボードのみでナビゲーションを走査
- **期待される結果**: n 回目の `user.tab()` 後、`document.activeElement` が navItems[n-1] のリンク（対応ラベルテキスト）である
  - **境界での正確性**: 8 番目（設定）で末尾に到達
  - **一貫した動作**: 逆順（Shift+Tab）でも対称に戻る（追加アサート可）
- **テストの目的**: フォーカス順序＝視覚的順序の確認（TASK-0034 完了条件）
  - **堅牢性の確認**: 到達不能なインタラクティブ要素が無いこと
- 🟡 信頼性レベル: TASK-0034 完了条件「フォーカス順序が視覚的順序と一致」より妥当な推測。user-event の tab 挙動に依存

### TC-10: モバイル最小幅 375px でレイアウトが崩れず横スクロールが発生しない（境界: 最小サポート幅）

- **テスト名**: 一覧画面を 375px 相当でレンダリングしても意図しない横オーバーフローが無い
  - **境界値の意味**: 375px は要件で定めるモバイル最小サポート幅（TASK-0034 完了条件）
  - **境界値での動作保証**: カード一覧が `grid-cols-1` に落ち、要素幅がビューポートを超えない
- **入力値**: viewport 幅 375px 想定でレンダリング（jsdom では `matchMedia`/`window.innerWidth` を 375 にモック、または Tailwind クラス `grid-cols-1 sm:grid-cols-2 ...` の適用をDOM属性で検証）
  - **境界値選択の根拠**: 最小幅は横スクロール/オーバーフローが最も起きやすい
  - **実際の使用場面**: スマートフォン縦持ち閲覧
- **期待される結果**: グリッドコンテナがモバイル向けクラス（1カラム）を持つ／`scrollWidth <= clientWidth`（レイアウト検証が可能な範囲で）
  - **境界での正確性**: 375px でカードが縦積みになる
  - **一貫した動作**: `sm:`（640px+）で複数カラムに切替
  - 注: jsdom は実レイアウト計算をしないため、E2E（TASK-0035/Playwright）で `scrollWidth` を実測補完する前提。単体では Tailwind ブレークポイントクラスの存在をアサートする軽量検証とする。
- **テストの目的**: モバイル最小幅でのレイアウト非破壊の確認
  - **堅牢性の確認**: 最小幅でも操作領域が確保される
- 🟡 信頼性レベル: TASK-0034 完了条件「モバイル幅375px…横スクロール発生しない」より妥当な推測。単体での実測限界あり（E2E補完）

### TC-11: 削除確認ダイアログの状態境界（open=false→非レンダリング / true→フォーカストラップ）

- **テスト名**: ConfirmDialog は open=false で内容非表示、open=true でフォーカスがダイアログ内に入る
  - **境界値の意味**: 表示/非表示の状態境界（Radix のマウント境界）
  - **境界値での動作保証**: 閉じているときは背景要素にフォーカスが漏れない
- **入力値**: `open={false}` でレンダリング→内容不在を確認、`rerender` で `open={true}`→初期フォーカスを確認
  - **境界値選択の根拠**: モーダルは開閉境界でフォーカス管理の不具合が出やすい
  - **実際の使用場面**: 削除ボタン押下で開き、キャンセル/確定で閉じる
- **期待される結果**: open=false 時 `queryByTestId('confirm-dialog')` が null。open=true 時、フォーカスがダイアログ内要素（例: キャンセル/OKボタン等）にあり、外部にフォーカスが無い
  - **境界での正確性**: 開いた瞬間フォーカスがトラップ範囲内
  - **一貫した動作**: 閉じるとフォーカストラップが解除
- **テストの目的**: モーダルのフォーカストラップ境界の確認（TASK-0034 完了条件）
  - **堅牢性の確認**: 状態遷移で背後にフォーカスが漏れない
- 🟡 信頼性レベル: TASK-0034 完了条件「フォーカスがモーダル内にトラップ」より妥当な推測。Radix `Dialog` 標準機能に依存

---

# 4. テストケース実装時の日本語コメント指針（例）

各テストは以下の Given/When/Then コメント構造で実装する。例（TC-07 削除確認 Esc）:

```tsx
// 【テスト目的】: 削除確認モーダルが Esc キーで閉じ onCancel が呼ばれることを確認する
// 【テスト内容】: ConfirmDialog(open=true) 表示中に Escape を押下する
// 【期待される動作】: onCancel が1回呼ばれ、onConfirm は呼ばれない（削除は実行されない）
// 🟡 信頼性レベル: TASK-0034 TC-4（Radix Dialog 標準機能）
it('Escキーで削除確認モーダルが閉じる', async () => {
  // 【テストデータ準備】: 閉鎖/確定コールバックをスパイ化し呼び出しを観測できるようにする
  const onCancel = vi.fn();
  const onConfirm = vi.fn();
  const user = userEvent.setup();

  // 【初期条件設定】: open=true の削除確認ダイアログを表示状態でレンダリングする
  render(<ConfirmDialog open title="削除しますか" onConfirm={onConfirm} onCancel={onCancel} />);

  // 【実際の処理実行】: キーボードのみで Escape を押下しモーダル離脱を試みる
  await user.keyboard('{Escape}');

  // 【結果検証】: onCancel が呼ばれ、削除確定は行われないこと
  expect(onCancel).toHaveBeenCalledTimes(1); // 【検証項目】: Esc→onOpenChange(false)→onCancel 集約
  expect(onConfirm).not.toHaveBeenCalled();  // 【検証項目】: 誤って削除が実行されない安全性
});
```

---

# 5. 要件定義との対応関係

| TC | 概要 | 対象ファイル | 由来 | 信頼性 |
|---|---|---|---|---|
| TC-01 | 手動追加フォーム入力にラベル関連付け | `src/pages/ItemFormPage.tsx` | TASK-0034 TC-1 / NFR-202 | 🔵 |
| TC-02 | APIキー登録フォーム入力にラベル関連付け（欠陥修正駆動） | `src/pages/SettingsPage.tsx` | TASK-0034 TC-2 / REQ-403 | 🔵 |
| TC-03 | サイドバー各リンクが link ロールで Tab 可能 | `src/components/common/Sidebar.tsx` | TASK-0034 TC-3 / NFR-202 | 🔵 |
| TC-04 | フィルタUI各コントロールにラベル/aria | `src/components/common/FilterBar.tsx` | TASK-0034 TC-5 / NFR-202 | 🔵 |
| TC-05 | モバイル Sheet/ハンバーガーで開閉（新規実装） | `src/components/common/Sidebar.tsx` | TASK-0034 UI/UX要件 | 🟡 |
| TC-06 | エラーが aria-describedby で関連付く | `src/pages/ItemFormPage.tsx` | TASK-0034 TC-6 / NFR-201,202 | 🟡 |
| TC-07 | 削除確認モーダルが Esc で閉じる | `src/components/common/ConfirmDialog.tsx` | TASK-0034 TC-4 / NFR-202 | 🟡 |
| TC-08 | file 入力のラベル欠落検出 | `src/pages/SettingsPage.tsx` | TASK-0034 実装詳細1 | 🟡 |
| TC-09 | フォーカス順序＝視覚的順序 | `src/components/common/Sidebar.tsx` | TASK-0034 完了条件 | 🟡 |
| TC-10 | 375px でレイアウト非破壊（E2E補完前提） | 一覧 `pages/*.tsx` | TASK-0034 完了条件 | 🟡 |
| TC-11 | モーダル open 状態境界とフォーカストラップ | `src/components/common/ConfirmDialog.tsx` | TASK-0034 完了条件 | 🟡 |

- **参照した機能概要**: requirements.md 「1. 機能の概要」、TASK-0034 タスク概要
- **参照した入力・出力仕様**: requirements.md 「2. 入力・出力の仕様（確認可能属性）」
- **参照した制約条件**: requirements.md 「3. 制約条件」（div+onClick 禁止、モバイル Sheet、既存テスト非破壊）
- **参照した使用例**: requirements.md 「4. 想定される使用例」（基本/エッジ・エラーケース）

---

## テストケース網羅サマリ

- 正常系: TC-01〜TC-05（5件）
- 異常系/支援技術異常系: TC-06〜TC-08（3件）
- 境界値/状態境界: TC-09〜TC-11（3件）
- 合計: 11件（TASK-0034 の必須 TC-1〜6 を全網羅 + モバイルレスポンシブ/フォーカス順序/欠落検出の追加ケース）

## 品質判定

**判定: ✅ 高品質（一部🟡は外部ライブラリ標準機能依存のため妥当）**

- テストケース分類: 正常系・異常系・境界値を網羅（5/3/3）。
- 期待値定義: 各TCで取得クエリ・アサート対象（role/label/aria属性/コールバック）を明確化。
- 技術選択: TypeScript + Vitest + Testing Library + user-event に確定。
- 実装可能性: 既存テスト基盤で実現可能。TC-02（欠陥修正駆動）・TC-05（Sheet新規実装）は Red→Green で実装を駆動。
- 信頼性分布: 🔵4 / 🟡7 / 🔴0。🟡 は Radix/shadcn 標準機能挙動・モバイル実装パターン・375px 実測(単体限界→E2E補完)に起因し妥当。

**留意事項（tasknote 注意事項に反映推奨）**:
1. TC-02: `SettingsPage` `ProviderRow` は現状ラベル未関連付け。Green で `htmlFor`/`id` か `aria-label` を付与する実修正が必要。
2. TC-05: モバイル `Sheet`/ハンバーガーは net-new。shadcn/ui `Sheet` の導入（`npx shadcn add sheet`）が前提。
3. TC-10: jsdom は実レイアウト非計算。単体はブレークポイントクラス検証に留め、`scrollWidth` 実測は TASK-0035（Playwright）で補完。
