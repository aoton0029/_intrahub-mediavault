# FilterBarコンポーネント テストケース定義書

**機能名**: FilterBar コンポーネント  
**タスクID**: TASK-0010  
**要件名**: frontend-collection-ui  
**出力ファイル**: `docs/implements/frontend-collection-ui/TASK-0010/filter-bar-testcases.md`

---

## 4. 開発言語・フレームワーク

- **プログラミング言語**: TypeScript / TSX
  - **言語選択の理由**: プロジェクト標準（React 18.3+ / TypeScript 5.7+）
- **テストフレームワーク**: Vitest + @testing-library/react + @testing-library/user-event
  - **フレームワーク選択の理由**: プロジェクト標準（vitest.config.ts）
  - **テスト実行環境**: jsdom（vitest.config.tsで設定済み）
  - **テストファイルパス**: `frontend/src/components/common/FilterBar.test.tsx`
- 🔵 プロジェクトのvitest.config.ts・CLAUDE.mdより確実

---

## 1. 正常系テストケース

### TC-FB-N-01: media_type変更でonChangeが正しい値で呼ばれる

- **テスト名**: media_typeセレクトで`anime`を選択するとonChangeが`{mediaType:'anime'}`で呼ばれる
  - **何をテストするか**: media_typeセレクトの操作によりonChangeコールバックが正しい引数で呼ばれること
  - **期待される動作**: selectの選択変更 → `onChange({ mediaType: 'anime' })` が1回呼ばれる
- **入力値**: `filters={}`, `onChange=vi.fn()`, `mediaTypeOptions`省略, ユーザーがselectで`anime`を選択
  - **入力データの意味**: 初期フィルタ空の状態でmedia_typeを設定する最も基本的なケース
- **期待される結果**: `onChange` が `{ mediaType: 'anime' }` を引数として1回呼ばれる
  - **期待結果の理由**: REQ-002に基づくmedia_typeフィルタの動作。controlledコンポーネントなのでonChange呼び出しのみが責務
- **テストの目的**: REQ-002 media_type絞り込みの動作確認
  - **確認ポイント**: `onChange`の呼び出し回数と引数
- 🔵 TASK-0010テストケース1・REQ-002より確実

### TC-FB-N-02: お気に入りトグルONでonChangeが呼ばれる

- **テスト名**: お気に入りチェックボックスをONにするとonChangeが`{isFavorite:true}`で呼ばれる
  - **何をテストするか**: favoriteチェックボックスのON操作でonChangeが正しい引数で呼ばれること
  - **期待される動作**: checkbox click → `onChange({ isFavorite: true })` が呼ばれる
- **入力値**: `filters={}`, `onChange=vi.fn()`, checkboxをクリック
  - **入力データの意味**: お気に入りフィルタが未設定の状態でONにする最も基本的なケース
- **期待される結果**: `onChange` が `{ isFavorite: true }` を引数として1回呼ばれる
  - **期待結果の理由**: REQ-002 isFavoriteフィルタの動作。isFavorite=trueでAPIへ渡すことでお気に入りのみ取得
- **テストの目的**: REQ-002 お気に入りフィルタの動作確認
  - **確認ポイント**: checkboxのON操作でonChangeが`isFavorite:true`で呼ばれること
- 🔵 TASK-0010テストケース2・REQ-002より確実

### TC-FB-N-03: statusセレクト操作でonChangeが呼ばれる

- **テスト名**: statusセレクトで`in_progress`を選択するとonChangeが`{status:'in_progress'}`で呼ばれる
  - **何をテストするか**: statusセレクトの操作でonChangeが正しい引数で呼ばれること
  - **期待される動作**: selectで`in_progress`選択 → `onChange({ status: 'in_progress' })` が呼ばれる
- **入力値**: `filters={}`, `onChange=vi.fn()`, statusセレクトで`in_progress`を選択
  - **入力データの意味**: status未設定から`in_progress`を選択する基本ケース
- **期待される結果**: `onChange` が `{ status: 'in_progress' }` を引数として1回呼ばれる
  - **期待結果の理由**: REQ-002 statusフィルタの動作。ItemStatus型の3値（not_started/in_progress/completed）
- **テストの目的**: REQ-002 statusフィルタの動作確認
  - **確認ポイント**: statusセレクトの値変更がonChangeに正しく伝播すること
- 🔵 TASK-0010テストケース3・REQ-002より確実

### TC-FB-N-04: タグ選択でonChangeが呼ばれる

- **テスト名**: タグセレクトでタグIDを選択するとonChangeが`{tagId:'tag-1'}`で呼ばれる
  - **何をテストするか**: タグセレクトの操作でonChangeが正しい引数で呼ばれること
  - **期待される動作**: tagセレクトでタグ選択 → `onChange({ tagId: 'tag-1' })` が呼ばれる
- **入力値**: `filters={}`, `tagOptions=[{id:'tag-1',name:'SF'}]`, タグセレクトで`tag-1`を選択
  - **入力データの意味**: 1件のタグオプションがある状態でタグフィルタを設定するケース
- **期待される結果**: `onChange` が `{ tagId: 'tag-1' }` を引数として1回呼ばれる
  - **期待結果の理由**: REQ-002 タグ絞り込みの動作
- **テストの目的**: REQ-002 タグフィルタの動作確認
  - **確認ポイント**: tagOptions由来のIDが正しくonChangeに渡されること
- 🔵 REQ-002・filterBarProps設計より確実

### TC-FB-N-05: カテゴリ選択でonChangeが呼ばれる

- **テスト名**: カテゴリセレクトでカテゴリIDを選択するとonChangeが`{categoryId:'cat-1'}`で呼ばれる
  - **何をテストするか**: カテゴリセレクトの操作でonChangeが正しい引数で呼ばれること
  - **期待される動作**: categoryセレクトで選択 → `onChange({ categoryId: 'cat-1' })` が呼ばれる
- **入力値**: `filters={}`, `categoryOptions=[{id:'cat-1',name:'趣味'}]`, カテゴリセレクトで`cat-1`を選択
  - **入力データの意味**: カテゴリオプションがある状態でカテゴリフィルタを設定するケース
- **期待される結果**: `onChange` が `{ categoryId: 'cat-1' }` を引数として1回呼ばれる
  - **期待結果の理由**: REQ-002 カテゴリ絞り込みの動作
- **テストの目的**: REQ-002 カテゴリフィルタの動作確認
- 🔵 REQ-002・filterBarProps設計より確実

### TC-FB-N-06: mediaTypeOptionsで選択肢が制限される

- **テスト名**: `mediaTypeOptions=['academic_book']`を渡すとmedia_typeセレクトの選択肢が`academic_book`のみになる
  - **何をテストするか**: `mediaTypeOptions`propsによるmedia_typeセレクトの選択肢制限
  - **期待される動作**: selectのoption要素が`academic_book`のみ（「すべて」空オプション＋1件）レンダリングされる
- **入力値**: `filters={}`, `onChange=vi.fn()`, `mediaTypeOptions=['academic_book']`, tagOptions=[], categoryOptions=[]
  - **入力データの意味**: AcademicListPageが固定のmedia_typeサブセットを表示する用途（REQ-004連携）
- **期待される結果**: media_typeセレクトに`academic_book`のオプションが存在し、他のmedia_type（anime等）は存在しない
  - **期待結果の理由**: REQ-004 グループ別一覧ページでの用途・TASK-0010完了条件
- **テストの目的**: `mediaTypeOptions`による選択肢制限の動作確認
  - **確認ポイント**: 指定したmedia_typeのみがselectのoptionに存在すること
- 🔵 TASK-0010テストケース5・REQ-004より確実

### TC-FB-N-07: 既存filtersの値がUIに反映される

- **テスト名**: `filters={mediaType:'anime'}`を渡すとmedia_typeセレクトがanimeに初期選択される
  - **何をテストするか**: controlledコンポーネントとして`filters`propsがUI初期値に反映されること
  - **期待される動作**: media_typeセレクトの`value`が`anime`に設定されている
- **入力値**: `filters={mediaType:'anime'}`, `onChange=vi.fn()`, tagOptions=[], categoryOptions=[]
  - **入力データの意味**: URLクエリパラメータからfiltersを復元した場合のUI表示確認
- **期待される結果**: media_typeのselectの値が`anime`になっている（`select.value === 'anime'`）
  - **期待結果の理由**: REQ-003 URLクエリパラメータからの絞り込み状態復元の要件
- **テストの目的**: controlledコンポーネントとしてのUI同期確認
  - **確認ポイント**: `filters.mediaType`がセレクトのvalue属性に正しく反映されること
- 🔵 REQ-003・controlledコンポーネント設計より確実

---

## 2. 異常系テストケース

### TC-FB-E-01: お気に入りトグルOFFでisFavoriteがundefinedになる

- **テスト名**: お気に入りチェックボックスをOFF（unchecked）にするとonChangeが`isFavorite`なし（undefined）で呼ばれる
  - **エラーケースの概要**: isFavorite=falseではなくundefined（パラメータ削除）であることの確認
  - **エラー処理の重要性**: `isFavorite: false`でAPIを呼ぶとフィルタとして機能してしまう恐れがあるため
- **入力値**: `filters={isFavorite:true}`, `onChange=vi.fn()`, checkboxをクリックしてOFFにする
  - **不正な理由**: この「異常系」はisFavorite=falseをURLパラメータに含めないことの確認
  - **実際の発生シナリオ**: ユーザーがお気に入りフィルタをONにした後にOFFにする操作
- **期待される結果**: `onChange` が `{}` または `{ isFavorite: undefined }` で呼ばれる（フィールドが除去される）
  - **エラーメッセージの内容**: UIレベルのエラーは発生しない
  - **システムの安全性**: URLから`favorite`パラメータが除去される
- **テストの目的**: isFavoriteのOFF操作がundefined（パラメータ除去）として扱われることを確認
  - **品質保証の観点**: API呼び出し時の不要なクエリパラメータを排除
- 🟡 要件定義書「isFavoriteのOff」セクションより妥当な推測

### TC-FB-E-02: selectで「すべて」を選ぶと対応フィールドがundefinedになる

- **テスト名**: media_typeセレクトで空値（「すべて」）を選択するとonChangeがmediaTypeなしで呼ばれる
  - **エラーケースの概要**: 「すべて」オプション（空値）選択でフィルタが除去されることの確認
  - **エラー処理の重要性**: 空値をAPIに渡すとエラーになる可能性があるため、undefinedとして処理する必要がある
- **入力値**: `filters={mediaType:'anime'}`, `onChange=vi.fn()`, media_typeセレクトで空値を選択
  - **不正な理由**: 空文字列をAPIに渡すことは不正
  - **実際の発生シナリオ**: ユーザーがmedia_typeフィルタを解除したい場合
- **期待される結果**: `onChange` が `{}` または `{ mediaType: undefined }` で呼ばれる
  - **エラーメッセージの内容**: UIエラーは発生しない
  - **システムの安全性**: URLから`media_type`パラメータが除去される
- **テストの目的**: select空値選択でフィルタが除去されることを確認
  - **品質保証の観点**: 不正なAPIクエリパラメータの防止
- 🟡 要件定義書「空値の扱い」より妥当な推測

### TC-FB-E-03: tagOptionsが空配列の場合もエラーなくレンダリングされる

- **テスト名**: `tagOptions=[]`を渡してもFilterBarが例外なくレンダリングされ、タグセレクトは選択肢なし
  - **エラーケースの概要**: タグが1件も存在しない場合のUI表示
  - **エラー処理の重要性**: タグ未登録の初期状態でも画面が正常表示される必要がある
- **入力値**: `filters={}`, `tagOptions=[]`, `categoryOptions=[]`, `onChange=vi.fn()`
  - **不正な理由**: 空配列は有効な入力だが「何も選べない」状態
  - **実際の発生シナリオ**: 新規インストール直後でタグが未登録の状態
- **期待される結果**: エラーなくコンポーネントがレンダリングされ、タグセレクトに「すべて」のみ表示
  - **エラーメッセージの内容**: エラーは発生しない
  - **システムの安全性**: 空配列で例外が発生しない
- **テストの目的**: tagOptions空配列でのレジリエンス確認
  - **品質保証の観点**: 初期状態・空状態での安定動作
- 🟡 境界値ケースとして妥当な推測

---

## 3. 境界値テストケース

### TC-FB-B-01: クリアボタンで全フィルタがリセットされる

- **テスト名**: 複数フィルタ設定後にクリアボタンを押すとonChangeが空オブジェクトで呼ばれる
  - **境界値の意味**: 「すべてのフィルタが設定されている」から「すべてリセット」への最大変化
  - **境界値での動作保証**: クリア操作が確実に全フィルタを除去すること
- **入力値**: `filters={mediaType:'anime',tagId:'t1',categoryId:'c1',isFavorite:true,status:'in_progress'}`, 「クリア」ボタンをクリック
  - **境界値選択の根拠**: 全5フィルタが設定された最大フィルタ状態
  - **実際の使用場面**: ユーザーが絞り込みを全解除したい場合
- **期待される結果**: `onChange` が `{}` （空オブジェクト）で呼ばれる
  - **境界での正確性**: 個別フィールドを`undefined`にするのではなく、空オブジェクトとして渡す
  - **一貫した動作**: page/limitも含めて全フィールドがリセットされる
- **テストの目的**: TASK-0010完了条件「クリアボタンで全項目リセット」の確認
  - **堅牢性の確認**: クリアボタンの押下で確実に全フィルタが除去されること
- 🟡 TASK-0010テストケース4・クリアボタン仕様は推測含む

### TC-FB-B-02: mediaTypeOptions未指定時は全8種のmedia_typeが選択肢に表示される

- **テスト名**: `mediaTypeOptions`を省略するとmedia_typeセレクトに全8種の選択肢が表示される
  - **境界値の意味**: `mediaTypeOptions`省略（undefined）時のデフォルト動作
  - **境界値での動作保証**: オプション省略時に全種表示というデフォルト動作の確認
- **入力値**: `filters={}`, `onChange=vi.fn()`, `mediaTypeOptions`プロパティ未指定, tagOptions=[], categoryOptions=[]
  - **境界値選択の根拠**: propsが省略された場合のデフォルト動作が境界
  - **実際の使用場面**: HomePage等が全media_typeを表示するケース
- **期待される結果**: media_typeセレクトに8種すべてのoption要素が存在する（anime, movie, drama, manga, novel, game, academic_book, paper）
  - **境界での正確性**: 8種すべてが表示されることと、過不足がないこと
  - **一貫した動作**: MediaType型定義の全値と一致すること
- **テストの目的**: `mediaTypeOptions`省略時のデフォルト動作確認
  - **堅牢性の確認**: propsの省略でエラーなく全種表示されること
- 🔵 TASK-0010完了条件・MediaType型定義より確実

### TC-FB-B-03: disabled=trueでフィルタUIがすべてdisabledになる

- **テスト名**: `disabled=true`を渡すと全フィルタUIがdisabled状態になる
  - **境界値の意味**: ローディング中など操作不能状態のUI
  - **境界値での動作保証**: disabled時に誤操作が発生しないこと
- **入力値**: `filters={}`, `onChange=vi.fn()`, `disabled=true`, tagOptions=[], categoryOptions=[]
  - **境界値選択の根拠**: disabled=true/falseの境界
  - **実際の使用場面**: タグ・カテゴリ取得中のローディング状態
- **期待される結果**: media_typeセレクト・statusセレクト・チェックボックス等がdisabled属性を持つ
  - **境界での正確性**: disabledの場合はonChangeが呼ばれないこと
  - **一貫した動作**: すべてのフィルタUIが一括でdisabledになること
- **テストの目的**: disabled propsによるフィルタ全体の操作不能化確認
  - **堅牢性の確認**: ローディング中の誤操作防止
- 🟡 要件定義書「disabled props」より妥当な推測

### TC-FB-B-04: filtersが空オブジェクトでも各selectの値が空（未選択）になる

- **テスト名**: `filters={}`を渡すと全フィルタUIが未選択（空値）状態で表示される
  - **境界値の意味**: フィルタ未設定（初期状態）のUI表示
  - **境界値での動作保証**: 空オブジェクトで初期状態が正しく表示されること
- **入力値**: `filters={}`, `onChange=vi.fn()`, tagOptions=[], categoryOptions=[]
  - **境界値選択の根拠**: 最小値（フィルタ未設定）の境界
  - **実際の使用場面**: ページ初期表示またはクリアボタン後の再レンダリング
- **期待される結果**: media_typeセレクト・statusセレクト・タグセレクト・カテゴリセレクトがすべて空値（「すべて」を選択）、checkboxがunchecked
  - **境界での正確性**: undefinedフィールドが空のselectとuncheckedのcheckboxに対応
  - **一貫した動作**: クリア後の状態と初期表示状態が一致
- **テストの目的**: filters空オブジェクトでの初期UI状態確認
  - **堅牢性の確認**: 空フィルタで全UIが正しい初期状態を持つこと
- 🔵 REQ-003・controlledコンポーネント設計より確実

---

## 5. テストケース実装時の日本語コメント指針

### TC-FB-N-01 実装例

```tsx
it('TC-FB-N-01: media_typeセレクトでanimeを選択するとonChangeが正しい値で呼ばれる', async () => {
  // 【テスト目的】: media_typeセレクトの操作でonChangeコールバックが正しい引数で呼ばれることを確認する
  // 【テスト内容】: FilterBarをレンダリングし、media_typeセレクトでanimeを選択してonChangeの引数を検証する
  // 【期待される動作】: onChange({ mediaType: 'anime' }) が1回呼ばれる
  // 🔵 REQ-002・TASK-0010テストケース1より確実

  const user = userEvent.setup()

  // 【テストデータ準備】: 初期フィルタ空の状態でonChangeモックを用意する
  // 【初期条件設定】: filters={}でFilterBarをレンダリングする
  const onChange = vi.fn()
  render(
    <FilterBar
      filters={{}}
      onChange={onChange}
      tagOptions={[]}
      categoryOptions={[]}
    />
  )

  // 【実際の処理実行】: media_typeセレクトでanimeを選択する
  // 【処理内容】: ユーザーがmedia_typeドロップダウンから「アニメ」を選択する操作を再現する
  const mediaTypeSelect = screen.getByRole('combobox', { name: /メディアタイプ/i })
  await user.selectOptions(mediaTypeSelect, 'anime')

  // 【結果検証】: onChangeが正しい引数で呼ばれたことを確認する
  // 【期待値確認】: { mediaType: 'anime' } でonChangeが1回呼ばれること
  expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
  expect(onChange).toHaveBeenCalledWith({ mediaType: 'anime' }) // 【確認内容】: mediaType:'anime'が正しく渡されることを確認
})
```

### TC-FB-B-01 実装例

```tsx
it('TC-FB-B-01: クリアボタンで全フィルタがリセットされる', async () => {
  // 【テスト目的】: クリアボタン押下で全フィルタがリセットされることを確認する
  // 【テスト内容】: 複数フィルタ設定済み状態でクリアボタンをクリックしonChange引数を検証する
  // 【期待される動作】: onChange({}) が呼ばれる
  // 🟡 TASK-0010テストケース4・クリアボタン仕様は推測含む

  const user = userEvent.setup()

  // 【テストデータ準備】: 全5フィルタが設定された最大フィルタ状態を用意する
  // 【初期条件設定】: mediaType/tagId/categoryId/isFavorite/statusが全て設定されたfiltersを渡す
  const onChange = vi.fn()
  render(
    <FilterBar
      filters={{ mediaType: 'anime', tagId: 't1', categoryId: 'c1', isFavorite: true, status: 'in_progress' }}
      onChange={onChange}
      tagOptions={[{ id: 't1', name: 'SF' }]}
      categoryOptions={[{ id: 'c1', name: '趣味' }]}
    />
  )

  // 【実際の処理実行】: クリアボタンをクリックする
  // 【処理内容】: ユーザーが「クリア」ボタンを押して全フィルタをリセットする操作を再現する
  const clearButton = screen.getByRole('button', { name: /クリア/i })
  await user.click(clearButton)

  // 【結果検証】: onChangeが空オブジェクトで呼ばれたことを確認する
  // 【期待値確認】: {} でonChangeが1回呼ばれること
  expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
  expect(onChange).toHaveBeenCalledWith({}) // 【確認内容】: 空オブジェクトが渡されて全フィルタがリセットされることを確認
})
```

---

## 6. 要件定義との対応関係

- **参照した機能概要**: `filter-bar-requirements.md` セクション1「機能の概要」
- **参照した入力・出力仕様**: `filter-bar-requirements.md` セクション2「入力・出力の仕様」（FilterBarProps・各フィルタUIと操作仕様）
- **参照した制約条件**: `filter-bar-requirements.md` セクション3「制約条件」（jsdomでネイティブselect使用）
- **参照した使用例**: `filter-bar-requirements.md` セクション4「想定される使用例」
- **参照したタスク要件**: `docs/tasks/frontend-collection-ui/TASK-0010.md`「単体テスト要件」テストケース1〜5

---

## テストケース一覧サマリー

| ID | 分類 | 内容 | 信頼性 |
|---|---|---|---|
| TC-FB-N-01 | 正常系 | media_type変更でonChange呼び出し確認 | 🔵 |
| TC-FB-N-02 | 正常系 | お気に入りトグルONでonChange呼び出し確認 | 🔵 |
| TC-FB-N-03 | 正常系 | statusセレクト操作でonChange呼び出し確認 | 🔵 |
| TC-FB-N-04 | 正常系 | タグ選択でonChange呼び出し確認 | 🔵 |
| TC-FB-N-05 | 正常系 | カテゴリ選択でonChange呼び出し確認 | 🔵 |
| TC-FB-N-06 | 正常系 | mediaTypeOptionsで選択肢が制限される | 🔵 |
| TC-FB-N-07 | 正常系 | 既存filtersの値がUIに反映される | 🔵 |
| TC-FB-E-01 | 異常系 | お気に入りトグルOFFでisFavoriteがundefined | 🟡 |
| TC-FB-E-02 | 異常系 | selectで「すべて」選択でフィールドがundefined | 🟡 |
| TC-FB-E-03 | 異常系 | tagOptions空配列でエラーなくレンダリング | 🟡 |
| TC-FB-B-01 | 境界値 | クリアボタンで全フィルタリセット | 🟡 |
| TC-FB-B-02 | 境界値 | mediaTypeOptions未指定で全8種表示 | 🔵 |
| TC-FB-B-03 | 境界値 | disabled=trueで全UI操作不能 | 🟡 |
| TC-FB-B-04 | 境界値 | filters={}で全UI未選択状態 | 🔵 |

**品質評価**: ✅ 高品質（正常系7・異常系3・境界値4、🔵:8 🟡:6 🔴:0）
