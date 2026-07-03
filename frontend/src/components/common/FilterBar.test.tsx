import { describe, expect, it, vi } from 'vitest'
import { render, screen, fireEvent, within } from '@testing-library/react'
import { FilterBar } from './FilterBar'
import type { ItemListFilters, Tag, Category, MediaType } from '@/types'

// ===== テストデータ =====
const mockTags: Tag[] = [
  { id: 'tag-1', name: 'SF' },
  { id: 'tag-2', name: 'アクション' },
]

const mockCategories: Category[] = [
  { id: 'cat-1', name: '趣味' },
  { id: 'cat-2', name: '学習' },
]

const emptyFilters: ItemListFilters = {}

// ===== 正常系テストケース =====

describe('FilterBar - 正常系', () => {
  it('TC-FB-N-01: media_typeセレクトでanimeを選択するとonChangeが正しい値で呼ばれる', () => {
    // 【テスト目的】: media_typeセレクトの操作でonChangeコールバックが正しい引数で呼ばれることを確認する
    // 【テスト内容】: FilterBarをレンダリングし、media_typeセレクトでanimeを選択してonChangeの引数を検証する
    // 【期待される動作】: onChange({ mediaType: 'anime' }) が1回呼ばれる
    // 🔵 REQ-002・TASK-0010テストケース1より確実

    // 【テストデータ準備】: 初期フィルタ空の状態でonChangeモックを用意する
    // 【初期条件設定】: filters={}でFilterBarをレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={mockTags}
        categoryOptions={mockCategories}
      />
    )

    // 【実際の処理実行】: media_typeセレクトでanimeを選択する
    // 【処理内容】: ユーザーがmedia_typeドロップダウンから「アニメ」を選択する操作を再現する
    const mediaTypeSelect = screen.getByRole('combobox', { name: /メディアタイプ/i })
    fireEvent.change(mediaTypeSelect, { target: { value: 'anime' } })

    // 【結果検証】: onChangeが正しい引数で呼ばれたことを確認する
    // 【期待値確認】: { mediaType: 'anime' } でonChangeが1回呼ばれること
    expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
    expect(onChange).toHaveBeenCalledWith({ mediaType: 'anime' }) // 【確認内容】: mediaType:'anime'が正しく渡されることを確認
  })

  it('TC-FB-N-02: お気に入りチェックボックスをONにするとonChangeが{isFavorite:true}で呼ばれる', () => {
    // 【テスト目的】: favoriteチェックボックスのON操作でonChangeが正しい引数で呼ばれることを確認する
    // 【テスト内容】: FilterBarをレンダリングし、お気に入りチェックボックスをクリックしてonChangeの引数を検証する
    // 【期待される動作】: onChange({ isFavorite: true }) が呼ばれる
    // 🔵 REQ-002・TASK-0010テストケース2より確実

    // 【テストデータ準備】: 初期フィルタ空の状態を用意する
    // 【初期条件設定】: isFavoriteが未設定のfiltersでFilterBarをレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
      />
    )

    // 【実際の処理実行】: お気に入りチェックボックスをクリックする
    // 【処理内容】: ユーザーがお気に入りスイッチをONにする操作を再現する
    const favoriteCheckbox = screen.getByRole('checkbox', { name: /お気に入り/i })
    fireEvent.click(favoriteCheckbox)

    // 【結果検証】: onChangeが正しい引数で呼ばれたことを確認する
    // 【期待値確認】: { isFavorite: true } でonChangeが1回呼ばれること
    expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
    expect(onChange).toHaveBeenCalledWith({ isFavorite: true }) // 【確認内容】: isFavorite:trueが正しく渡されることを確認
  })

  it('TC-FB-N-03: ステータスchipで進行中をクリックするとonChangeが{status:"in_progress"}で呼ばれる', () => {
    // 【テスト目的】: statusのchipボタン操作でonChangeが正しい引数で呼ばれることを確認する
    // 【テスト内容】: FilterBarをレンダリングし、ステータスchip群から「進行中」をクリックしてonChangeの引数を検証する
    // 【期待される動作】: onChange({ status: 'in_progress' }) が呼ばれる
    // 🔵 REQ-002・REQ-005・TASK-0006（chip化）より確実

    // 【テストデータ準備】: status未設定のfiltersを用意する
    // 【初期条件設定】: statusが未設定のfiltersでFilterBarをレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
      />
    )

    // 【実際の処理実行】: ステータスchip群から「進行中」をクリックする
    // 【処理内容】: ユーザーがステータスchipグループから「進行中」ボタンを押す操作を再現する
    const statusGroup = screen.getByRole('group', { name: /ステータス/i })
    const inProgressChip = within(statusGroup).getByRole('button', { name: '進行中' })
    fireEvent.click(inProgressChip)

    // 【結果検証】: onChangeが正しい引数で呼ばれたことを確認する
    // 【期待値確認】: { status: 'in_progress' } でonChangeが1回呼ばれること
    expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
    expect(onChange).toHaveBeenCalledWith({ status: 'in_progress' }) // 【確認内容】: status:'in_progress'が正しく渡されることを確認
  })

  it('TC-FB-N-04: タグセレクトでタグを選択するとonChangeが{tagId:"tag-1"}で呼ばれる', () => {
    // 【テスト目的】: タグセレクトの操作でonChangeが正しい引数で呼ばれることを確認する
    // 【テスト内容】: FilterBarをレンダリングし、タグセレクトでタグを選択してonChangeの引数を検証する
    // 【期待される動作】: onChange({ tagId: 'tag-1' }) が呼ばれる
    // 🔵 REQ-002・filterBarProps設計より確実

    // 【テストデータ準備】: 1件のタグオプションを用意する
    // 【初期条件設定】: tagOptions=[{id:'tag-1',name:'SF'}]でFilterBarをレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={mockTags}
        categoryOptions={[]}
      />
    )

    // 【実際の処理実行】: タグセレクトでtag-1を選択する
    // 【処理内容】: ユーザーがタグドロップダウンから「SF」を選択する操作を再現する
    const tagSelect = screen.getByRole('combobox', { name: /タグ/i })
    fireEvent.change(tagSelect, { target: { value: 'tag-1' } })

    // 【結果検証】: onChangeが正しい引数で呼ばれたことを確認する
    expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
    expect(onChange).toHaveBeenCalledWith({ tagId: 'tag-1' }) // 【確認内容】: tagId:'tag-1'が正しく渡されることを確認
  })

  it('TC-FB-N-05: カテゴリセレクトでカテゴリを選択するとonChangeが{categoryId:"cat-1"}で呼ばれる', () => {
    // 【テスト目的】: カテゴリセレクトの操作でonChangeが正しい引数で呼ばれることを確認する
    // 【テスト内容】: FilterBarをレンダリングし、カテゴリセレクトでカテゴリを選択してonChangeの引数を検証する
    // 【期待される動作】: onChange({ categoryId: 'cat-1' }) が呼ばれる
    // 🔵 REQ-002・filterBarProps設計より確実

    // 【テストデータ準備】: カテゴリオプションを用意する
    // 【初期条件設定】: categoryOptions=[{id:'cat-1',name:'趣味'}]でFilterBarをレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={mockCategories}
      />
    )

    // 【実際の処理実行】: カテゴリセレクトでcat-1を選択する
    // 【処理内容】: ユーザーがカテゴリドロップダウンから「趣味」を選択する操作を再現する
    const categorySelect = screen.getByRole('combobox', { name: /カテゴリ/i })
    fireEvent.change(categorySelect, { target: { value: 'cat-1' } })

    // 【結果検証】: onChangeが正しい引数で呼ばれたことを確認する
    expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
    expect(onChange).toHaveBeenCalledWith({ categoryId: 'cat-1' }) // 【確認内容】: categoryId:'cat-1'が正しく渡されることを確認
  })

  it('TC-FB-N-06: mediaTypeOptions=["academic_book"]を渡すとmedia_typeセレクトの選択肢がacademic_bookのみになる', () => {
    // 【テスト目的】: mediaTypeOptionsプロパティによるmedia_typeセレクトの選択肢制限を確認する
    // 【テスト内容】: mediaTypeOptions=['academic_book']を渡してFilterBarをレンダリングし、selectのoptionを検証する
    // 【期待される動作】: media_typeセレクトにacademic_bookのoptionのみが存在し、animeは存在しない
    // 🔵 TASK-0010テストケース5・REQ-004より確実

    // 【テストデータ準備】: AcademicListPage用のmediaTypeOptionsを用意する
    // 【初期条件設定】: mediaTypeOptions=['academic_book']でFilterBarをレンダリングする
    const onChange = vi.fn()
    const mediaTypeOptions: MediaType[] = ['academic_book']
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
        mediaTypeOptions={mediaTypeOptions}
      />
    )

    // 【実際の処理実行】: media_typeセレクトのoption要素を確認する
    // 【処理内容】: selectのoption要素の数と内容を検証する
    const mediaTypeSelect = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const optionValues = Array.from(mediaTypeSelect.querySelectorAll('option')).map(
      opt => (opt as HTMLOptionElement).value
    )

    // 【結果検証】: academic_bookのみが選択肢として存在することを確認する
    expect(optionValues).toContain('academic_book') // 【確認内容】: academic_bookオプションが存在することを確認
    expect(optionValues).not.toContain('anime') // 【確認内容】: animeオプションが存在しないことを確認
    expect(optionValues).not.toContain('movie') // 【確認内容】: movieオプションが存在しないことを確認
  })

  it('TC-FB-N-07: filters={mediaType:"anime"}を渡すとmedia_typeセレクトがanimeに初期選択される', () => {
    // 【テスト目的】: controlledコンポーネントとしてfiltersプロパティがUI初期値に反映されることを確認する
    // 【テスト内容】: filters={mediaType:'anime'}でFilterBarをレンダリングし、selectのvalueを検証する
    // 【期待される動作】: media_typeセレクトのvalueがanimeになっている
    // 🔵 REQ-003・controlledコンポーネント設計より確実

    // 【テストデータ準備】: URLクエリパラメータから復元されたfiltersを想定する
    // 【初期条件設定】: mediaType='anime'が設定されたfiltersでFilterBarをレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={{ mediaType: 'anime' }}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
      />
    )

    // 【実際の処理実行】: media_typeセレクトのvalue属性を確認する
    // 【処理内容】: controlledコンポーネントとしてfiltersの値がselectに反映されているか検証する
    const mediaTypeSelect = screen.getByRole('combobox', { name: /メディアタイプ/i }) as HTMLSelectElement

    // 【結果検証】: selectのvalueがanimeになっていることを確認する
    // 【期待値確認】: filters.mediaType='anime'がselectのvalueに正しく反映されること
    expect(mediaTypeSelect.value).toBe('anime') // 【確認内容】: media_typeセレクトの初期値がfiltersと同期していることを確認
  })

  it('TC-FB-N-08: filters={status:"completed"}を渡すと該当のステータスchipがactive状態（aria-pressed=true）で表示される', () => {
    // 【テスト目的】: controlledコンポーネントとしてfiltersプロパティがステータスchipのactive状態に反映されることを確認する
    // 【テスト内容】: filters={status:'completed'}でFilterBarをレンダリングし、対応するchipのaria-pressedを検証する
    // 【期待される動作】: 「完了」chipのaria-pressedがtrue、他のchipはfalseになる
    // 🟡 _shared.css .chip.active定義・タスク指示「chipのactive状態の色分け」から妥当な推測

    const onChange = vi.fn()
    render(
      <FilterBar
        filters={{ status: 'completed' }}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
      />
    )

    const statusGroup = screen.getByRole('group', { name: /ステータス/i })
    const completedChip = within(statusGroup).getByRole('button', { name: '完了' })
    const notStartedChip = within(statusGroup).getByRole('button', { name: '未開始' })

    expect(completedChip).toHaveAttribute('aria-pressed', 'true') // 【確認内容】: 選択中のchipがactive状態であることを確認
    expect(completedChip.className).toContain('active') // 【確認内容】: active用のCSSクラスが付与されていることを確認
    expect(notStartedChip).toHaveAttribute('aria-pressed', 'false') // 【確認内容】: 未選択のchipはactiveでないことを確認
  })
})

// ===== 異常系テストケース =====

describe('FilterBar - 異常系', () => {
  it('TC-FB-E-01: お気に入りチェックボックスをOFFにするとisFavoriteがundefinedとして渡される', () => {
    // 【テスト目的】: isFavoriteのOFF操作がundefined（パラメータ除去）として扱われることを確認する
    // 【テスト内容】: isFavorite=trueのfiltersでCheckboxをクリックしてOFFにし、onChangeの引数を検証する
    // 【期待される動作】: onChangeがisFavoriteフィールドを含まない（またはundefinedの）オブジェクトで呼ばれる
    // 🟡 要件定義書「isFavoriteのOff: isFavorite:undefined（除去）」より妥当な推測

    // 【テストデータ準備】: isFavorite=trueが設定されたfiltersを用意する
    // 【初期条件設定】: お気に入りがONの状態のFilterBarをレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={{ isFavorite: true }}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
      />
    )

    // 【実際の処理実行】: お気に入りチェックボックスをクリックしてOFFにする
    // 【処理内容】: ユーザーがお気に入りスイッチをOFFにする操作を再現する
    const favoriteCheckbox = screen.getByRole('checkbox', { name: /お気に入り/i })
    fireEvent.click(favoriteCheckbox)

    // 【結果検証】: onChangeがisFavoriteを除去したオブジェクトで呼ばれたことを確認する
    // 【期待値確認】: onChangeの引数にisFavoriteが含まれないか、undefinedであること
    expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
    const calledWith = onChange.mock.calls[0][0] as ItemListFilters
    expect(calledWith.isFavorite).toBeUndefined() // 【確認内容】: isFavoriteがundefinedであることを確認（falseではなくundefined）
  })

  it('TC-FB-E-02: media_typeセレクトで空値を選択するとmediaTypeがundefinedとして渡される', () => {
    // 【テスト目的】: 「すべて」オプション（空値）選択でmedia_typeフィルタが除去されることを確認する
    // 【テスト内容】: mediaType='anime'のfiltersから空値を選択し、onChangeの引数を検証する
    // 【期待される動作】: onChangeがmediaTypeフィールドを含まない（またはundefinedの）オブジェクトで呼ばれる
    // 🟡 要件定義書「空値の扱い: undefinedとして処理」より妥当な推測

    // 【テストデータ準備】: mediaType='anime'が設定されたfiltersを用意する
    // 【初期条件設定】: media_typeフィルタが設定されている状態のFilterBarをレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={{ mediaType: 'anime' }}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
      />
    )

    // 【実際の処理実行】: media_typeセレクトで空値（すべて）を選択する
    // 【処理内容】: ユーザーがmedia_typeドロップダウンで「すべて」を選択してフィルタを解除する操作を再現する
    const mediaTypeSelect = screen.getByRole('combobox', { name: /メディアタイプ/i })
    fireEvent.change(mediaTypeSelect, { target: { value: '' } })

    // 【結果検証】: onChangeがmediaTypeを除去したオブジェクトで呼ばれたことを確認する
    expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
    const calledWith = onChange.mock.calls[0][0] as ItemListFilters
    expect(calledWith.mediaType).toBeUndefined() // 【確認内容】: mediaTypeがundefinedであることを確認（空文字列ではなくundefined）
  })

  it('TC-FB-E-03: tagOptionsが空配列の場合もエラーなくレンダリングされる', () => {
    // 【テスト目的】: タグが1件も存在しない場合でも画面が正常表示されることを確認する
    // 【テスト内容】: tagOptions=[]でFilterBarをレンダリングし、例外が発生しないことを検証する
    // 【期待される動作】: エラーなくコンポーネントがレンダリングされる
    // 🟡 境界値ケースとして妥当な推測

    // 【テストデータ準備】: 空のtagOptionsを用意する
    // 【初期条件設定】: タグが未登録の状態を想定してtagOptions=[]でレンダリングする
    const onChange = vi.fn()
    expect(() => {
      render(
        <FilterBar
          filters={emptyFilters}
          onChange={onChange}
          tagOptions={[]}
          categoryOptions={[]}
        />
      )
    }).not.toThrow() // 【確認内容】: 空配列でも例外が発生しないことを確認

    // 【結果検証】: FilterBarコンテナが存在することを確認する
    expect(screen.getByTestId('filter-bar')).toBeInTheDocument() // 【確認内容】: コンポーネント自体が正常に描画されることを確認
  })

  it('TC-FB-E-04: キーワード検索欄に入力するとonChangeが{keyword:入力値}で呼ばれる', () => {
    // 【テスト目的】: キーワード検索inputの操作でonChangeが正しい引数で呼ばれることを確認する
    // 【テスト内容】: FilterBarをレンダリングし、検索inputに文字列を入力してonChangeの引数を検証する
    // 【期待される動作】: onChange({ keyword: 'test' }) が呼ばれる
    // 🟡 REQ-005「検索ボックス」・タスク指示「検索ボックスのフォーカス状態」から妥当な推測

    const onChange = vi.fn()
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
      />
    )

    const keywordInput = screen.getByPlaceholderText('キーワード検索')
    fireEvent.change(keywordInput, { target: { value: 'test' } })

    expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
    expect(onChange).toHaveBeenCalledWith({ keyword: 'test' }) // 【確認内容】: keyword:'test'が正しく渡されることを確認
  })
})

// ===== 境界値テストケース =====

describe('FilterBar - 境界値', () => {
  it('TC-FB-B-01: 複数フィルタ設定後にクリアボタンを押すとonChangeが{}で呼ばれる', () => {
    // 【テスト目的】: クリアボタン押下で全フィルタがリセットされることを確認する
    // 【テスト内容】: 複数フィルタ設定済み状態でクリアボタンをクリックしonChange引数を検証する
    // 【期待される動作】: onChange({}) が呼ばれる
    // 🟡 TASK-0010テストケース4・クリアボタン仕様は推測含む

    // 【テストデータ準備】: 全5フィルタが設定された最大フィルタ状態を用意する
    // 【初期条件設定】: mediaType/tagId/categoryId/isFavorite/statusが全て設定されたfiltersを渡す
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={{
          mediaType: 'anime',
          tagId: 'tag-1',
          categoryId: 'cat-1',
          isFavorite: true,
          status: 'in_progress',
        }}
        onChange={onChange}
        tagOptions={mockTags}
        categoryOptions={mockCategories}
      />
    )

    // 【実際の処理実行】: クリアボタンをクリックする
    // 【処理内容】: ユーザーが「クリア」ボタンを押して全フィルタをリセットする操作を再現する
    const clearButton = screen.getByRole('button', { name: /クリア/i })
    fireEvent.click(clearButton)

    // 【結果検証】: onChangeが空オブジェクトで呼ばれたことを確認する
    // 【期待値確認】: {} でonChangeが1回呼ばれること
    expect(onChange).toHaveBeenCalledTimes(1) // 【確認内容】: onChangeが1回だけ呼ばれることを確認
    expect(onChange).toHaveBeenCalledWith({}) // 【確認内容】: 空オブジェクトが渡されて全フィルタがリセットされることを確認
  })

  it('TC-FB-B-02: mediaTypeOptions未指定時はmedia_typeセレクトに全8種の選択肢が表示される', () => {
    // 【テスト目的】: mediaTypeOptions省略時のデフォルト動作として全8種が表示されることを確認する
    // 【テスト内容】: mediaTypeOptionsプロパティ未指定でFilterBarをレンダリングし、selectのoptionを検証する
    // 【期待される動作】: media_typeセレクトに全8種のoptionが存在する
    // 🔵 TASK-0010完了条件・MediaType型定義より確実

    // 【テストデータ準備】: mediaTypeOptionsを省略したFilterBarのpropsを用意する
    // 【初期条件設定】: mediaTypeOptionsプロパティを渡さずにFilterBarをレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
      />
    )

    // 【実際の処理実行】: media_typeセレクトのoption要素を取得する
    const mediaTypeSelect = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const optionValues = Array.from(mediaTypeSelect.querySelectorAll('option')).map(
      opt => (opt as HTMLOptionElement).value
    )

    // 【結果検証】: 全8種のmedia_typeが選択肢として存在することを確認する
    const allMediaTypes: MediaType[] = ['anime', 'movie', 'drama', 'manga', 'novel', 'game', 'academic_book', 'paper']
    for (const mediaType of allMediaTypes) {
      expect(optionValues).toContain(mediaType) // 【確認内容】: 各media_typeのoptionが存在することを確認
    }
  })

  it('TC-FB-B-03: disabled=trueで各フィルタUIがdisabledになる', () => {
    // 【テスト目的】: disabled=trueでフィルタUIが操作不能状態になることを確認する
    // 【テスト内容】: disabled=trueでFilterBarをレンダリングし、UIがdisabled属性を持つことを検証する
    // 【期待される動作】: media_typeセレクト・statusセレクト等がdisabled属性を持つ
    // 🟡 要件定義書「disabled props」より妥当な推測

    // 【テストデータ準備】: disabled=trueのpropsを用意する
    // 【初期条件設定】: ローディング中の状態を想定してdisabled=trueでレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={[]}
        categoryOptions={[]}
        disabled
      />
    )

    // 【実際の処理実行】: セレクト要素・chipボタンのdisabled属性を確認する
    const mediaTypeSelect = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const statusGroup = screen.getByRole('group', { name: /ステータス/i })
    const statusChip = within(statusGroup).getByRole('button', { name: '進行中' })
    const favoriteCheckbox = screen.getByRole('checkbox', { name: /お気に入り/i })

    // 【結果検証】: 各フィルタUIがdisabledになっていることを確認する
    expect(mediaTypeSelect).toBeDisabled() // 【確認内容】: media_typeセレクトがdisabledになることを確認
    expect(statusChip).toBeDisabled() // 【確認内容】: statusのchipボタンがdisabledになることを確認
    expect(favoriteCheckbox).toBeDisabled() // 【確認内容】: お気に入りcheckboxがdisabledになることを確認
  })

  it('TC-FB-B-04: filters={}を渡すと全フィルタUIが未選択状態で表示される', () => {
    // 【テスト目的】: フィルタ未設定の初期状態のUI表示が正しいことを確認する
    // 【テスト内容】: filters={}でFilterBarをレンダリングし、全フィルタUIの初期状態を検証する
    // 【期待される動作】: media_typeセレクト・statusセレクトが空値、checkboxがunchecked
    // 🔵 REQ-003・controlledコンポーネント設計より確実

    // 【テストデータ準備】: 空のfiltersオブジェクトを用意する
    // 【初期条件設定】: フィルタ未設定の初期状態を想定してfilters={}でレンダリングする
    const onChange = vi.fn()
    render(
      <FilterBar
        filters={emptyFilters}
        onChange={onChange}
        tagOptions={mockTags}
        categoryOptions={mockCategories}
      />
    )

    // 【実際の処理実行】: 各フィルタUIの状態を確認する
    const mediaTypeSelect = screen.getByRole('combobox', { name: /メディアタイプ/i }) as HTMLSelectElement
    const statusGroup = screen.getByRole('group', { name: /ステータス/i })
    const statusChips = within(statusGroup).getAllByRole('button')
    const favoriteCheckbox = screen.getByRole('checkbox', { name: /お気に入り/i }) as HTMLInputElement

    // 【結果検証】: 全フィルタUIが未選択状態であることを確認する
    expect(mediaTypeSelect.value).toBe('') // 【確認内容】: media_typeセレクトが未選択（空値）であることを確認
    for (const chip of statusChips) {
      expect(chip).toHaveAttribute('aria-pressed', 'false') // 【確認内容】: 全ステータスchipが未選択状態（aria-pressed=false）であることを確認
    }
    expect(favoriteCheckbox.checked).toBe(false) // 【確認内容】: お気に入りcheckboxがuncheckedであることを確認
  })
})
