import { describe, expect, it } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom'
import { Sidebar } from './Sidebar'

// 【テストヘルパー】: 現在のパスを画面に表示し、クリック後の遷移先を検証するための補助コンポーネント
function LocationDisplay() {
  const location = useLocation()
  return <div data-testid="location-display">{location.pathname}</div>
}

function renderWithRouter(initialEntries: string[] = ['/']) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <Sidebar />
      <Routes>
        <Route path="*" element={<LocationDisplay />} />
      </Routes>
    </MemoryRouter>
  )
}

describe('Sidebar', () => {
  // ===== 正常系 =====

  it('TC-01: 全8項目のナビゲーションリンクが表示される', () => {
    // 【テスト目的】: Sidebarに全体一覧・3メディアグループ・マイリスト・タグ/カテゴリ・スタッフ・設定の8リンクが表示されることを確認する
    // 【テスト内容】: MemoryRouter内でSidebarをレンダリングし、各リンクの存在をgetByRoleで検証する
    // 【期待される動作】: 8つのリンクすべてがroleとaccessible nameで取得できる
    // 🔵 信頼性レベル: TASK-0007.md「完了条件」「ナビゲーション構成」より確定

    // 【テストデータ準備】: 初期ルート'/'での標準的なレンダリングケース
    // 【初期条件設定】: propsなしでSidebarをレンダリング
    renderWithRouter(['/'])

    // 【結果検証】: 8つのナビゲーション項目がすべて存在することを確認する
    // 【期待値確認】: ラベル文言の一致、リンク数が過不足ないこと
    expect(screen.getByRole('link', { name: '全体一覧' })).toBeInTheDocument() // 【確認内容】: 全体一覧リンクが表示されることを確認
    expect(screen.getByRole('link', { name: '一般メディア' })).toBeInTheDocument() // 【確認内容】: 一般メディアリンクが表示されることを確認
    expect(screen.getByRole('link', { name: '学術書・専門書' })).toBeInTheDocument() // 【確認内容】: 学術書・専門書リンクが表示されることを確認
    expect(screen.getByRole('link', { name: '論文・文献' })).toBeInTheDocument() // 【確認内容】: 論文・文献リンクが表示されることを確認
    expect(screen.getByRole('link', { name: 'マイリスト' })).toBeInTheDocument() // 【確認内容】: マイリストリンクが表示されることを確認
    expect(screen.getByRole('link', { name: 'タグ/カテゴリ' })).toBeInTheDocument() // 【確認内容】: タグ/カテゴリリンクが表示されることを確認
    expect(screen.getByRole('link', { name: 'スタッフ' })).toBeInTheDocument() // 【確認内容】: スタッフリンクが表示されることを確認
    expect(screen.getByRole('link', { name: '設定' })).toBeInTheDocument() // 【確認内容】: 設定リンクが表示されることを確認
    expect(screen.getAllByRole('link')).toHaveLength(8) // 【確認内容】: リンク総数が過不足なく8件であることを確認
  })

  it('TC-02: 各リンクのhrefが正しいパスを指す', () => {
    // 【テスト目的】: 各ナビゲーションリンクのhref属性が要件定義の対応表と一致することを確認する
    // 【テスト内容】: 8項目それぞれのto値を検証する
    // 【期待される動作】: 各リンクのhrefが仕様通りのパスである
    // 🔵 信頼性レベル: requirements.md「入力・出力の仕様」の対応表より確定

    // 【テストデータ準備】: TASK-0007.md記載のnavItems全件を網羅
    // 【初期条件設定】: 初期ルート'/'でレンダリング
    renderWithRouter(['/'])

    // 【結果検証】: 各リンクのhrefが対応表通りであることを確認する
    // 【期待値確認】: タイポや末尾スラッシュ等の差異がないこと
    expect(screen.getByRole('link', { name: '全体一覧' })).toHaveAttribute('href', '/') // 【確認内容】: 全体一覧のhrefが'/'であることを確認
    expect(screen.getByRole('link', { name: '一般メディア' })).toHaveAttribute('href', '/collections/general') // 【確認内容】: 一般メディアのhrefを確認
    expect(screen.getByRole('link', { name: '学術書・専門書' })).toHaveAttribute('href', '/collections/academic') // 【確認内容】: 学術書・専門書のhrefを確認
    expect(screen.getByRole('link', { name: '論文・文献' })).toHaveAttribute('href', '/collections/paper') // 【確認内容】: 論文・文献のhrefを確認
    expect(screen.getByRole('link', { name: 'マイリスト' })).toHaveAttribute('href', '/mylists') // 【確認内容】: マイリストのhrefを確認
    expect(screen.getByRole('link', { name: 'タグ/カテゴリ' })).toHaveAttribute('href', '/tags-categories') // 【確認内容】: タグ/カテゴリのhrefを確認
    expect(screen.getByRole('link', { name: 'スタッフ' })).toHaveAttribute('href', '/staff') // 【確認内容】: スタッフのhrefを確認
    expect(screen.getByRole('link', { name: '設定' })).toHaveAttribute('href', '/settings') // 【確認内容】: 設定のhrefを確認
  })

  it('TC-03: 「マイリスト」リンクをクリックすると/mylistsへ遷移する', () => {
    // 【テスト目的】: ナビリンクのクリックでReact Routerによる画面遷移が発生することを確認する
    // 【テスト内容】: fireEvent.clickでリンクをクリックし、現在のパス表示が変化することを検証する
    // 【期待される動作】: クリック後、現在パスが/mylistsになる
    // 🔵 信頼性レベル: TASK-0007.md「単体テスト要件 テストケース2」より確定

    // 【テストデータ準備】: 実際のクリック操作によるSPA内遷移を再現する
    // 【初期条件設定】: 初期ルート'/'でレンダリング
    renderWithRouter(['/'])

    // 【実際の処理実行】: 「マイリスト」リンクをクリックする
    // 【処理内容】: fireEvent.clickでクリックイベントを発火する
    fireEvent.click(screen.getByRole('link', { name: 'マイリスト' }))

    // 【結果検証】: 現在のパス表示が/mylistsに変化したことを確認する
    // 【期待値確認】: NavLinkのデフォルト遷移挙動が阻害されていないこと
    expect(screen.getByTestId('location-display')).toHaveTextContent('/mylists') // 【確認内容】: クリック後に/mylistsへ遷移したことを確認
  })

  it('TC-04: 現在ルートが/staffのとき「スタッフ」リンクのみactiveクラスが付与される', () => {
    // 【テスト目的】: NavLinkのisActiveに基づくアクティブ状態のクラス切り替えを確認する
    // 【テスト内容】: initialEntries=['/staff']でレンダリングし、「スタッフ」リンクのみactiveクラスを持つことを検証する
    // 【期待される動作】: 「スタッフ」リンクのclassに'active'が含まれ、他7リンクには含まれない
    // 🔵 信頼性レベル: TASK-0007.md「単体テスト要件 テストケース3」より（クラス名の具体仕様は🟡推測）

    // 【テストデータ準備】: 他ルートとは異なる単一ルートを明示的に指定し区別を確認する
    // 【初期条件設定】: 初期ルート'/staff'でレンダリング
    renderWithRouter(['/staff'])

    // 【結果検証】: 「スタッフ」リンクのみactiveクラスを持つことを確認する
    // 【期待値確認】: アクティブなのは常に1つのみであること
    expect(screen.getByRole('link', { name: 'スタッフ' }).className).toContain('active') // 【確認内容】: スタッフリンクがactive状態であることを確認
    expect(screen.getByRole('link', { name: '全体一覧' }).className).not.toContain('active') // 【確認内容】: 全体一覧が非アクティブであることを確認
    expect(screen.getByRole('link', { name: 'マイリスト' }).className).not.toContain('active') // 【確認内容】: マイリストが非アクティブであることを確認
    expect(screen.getByRole('link', { name: '設定' }).className).not.toContain('active') // 【確認内容】: 設定が非アクティブであることを確認
  })

  it('TC-05: ルートパス("/")表示時は「全体一覧」のみがアクティブになる', () => {
    // 【テスト目的】: NavLinkのto="/"が前方一致で誤って他パスにマッチしないことを確認する（endProp等の正しい実装確認）
    // 【テスト内容】: initialEntries=['/']でレンダリングし、「全体一覧」のみactiveであることを検証する
    // 【期待される動作】: 「全体一覧」リンクのみactiveクラスを持つ
    // 🟡 信頼性レベル: React Router v7の一般的なNavLink仕様からの妥当な推測（設計文書に明記なし）

    // 【テストデータ準備】: ルートパスの前方一致バグを検出する代表的な境界ケース
    // 【初期条件設定】: 初期ルート'/'でレンダリング
    renderWithRouter(['/'])

    // 【結果検証】: 「全体一覧」のみがactiveであり、他の/collections/...等が誤ってアクティブにならないことを確認する
    // 【期待値確認】: end指定の有無による誤判定が起きていないこと
    expect(screen.getByRole('link', { name: '全体一覧' }).className).toContain('active') // 【確認内容】: 全体一覧がactiveであることを確認
    expect(screen.getByRole('link', { name: '一般メディア' }).className).not.toContain('active') // 【確認内容】: 一般メディアが誤ってactiveにならないことを確認
    expect(screen.getByRole('link', { name: 'マイリスト' }).className).not.toContain('active') // 【確認内容】: マイリストが誤ってactiveにならないことを確認
  })

  // ===== 異常系 =====

  it('TC-06: 未知のルート(/items/123)表示時にどのリンクもactiveにならない', () => {
    // 【エラーケースの概要】: ナビゲーション項目に存在しない動的パラメータ付きパスを表示している場合を想定する
    // 【エラー処理の重要性】: 該当しないリンクが誤ってアクティブ表示されるとユーザーが現在位置を誤認するため
    // 【テスト内容】: initialEntries=['/items/123']でレンダリングし、8リンクすべてが非アクティブであることを検証する
    // 🟡 信頼性レベル: TASK-0007.md「想定される使用例」エッジケース記載（🟡）から導出

    // 【不正な理由】: navItemsの8パスいずれにも該当しない動的パラメータ付きルートである
    // 【実際の発生シナリオ】: アイテム詳細画面（TASK-0017以降で実装）をサイドバーと共に表示する場合
    renderWithRouter(['/items/123'])

    // 【結果検証】: 8つのリンクすべてにactiveクラスが付与されないことを確認する
    // 【システムの安全性】: アクティブ状態が「不明」として扱われても画面遷移自体は問題なく機能する
    const links = screen.getAllByRole('link')
    links.forEach((link) => {
      expect(link.className).not.toContain('active') // 【確認内容】: 未対応ルートでどのリンクも誤ってactiveにならないことを確認
    })
  })

  it('TC-07: Sidebarがpropsなしでもレンダリングエラーを起こさない', () => {
    // 【エラーケースの概要】: Sidebarは外部からpropsを受け取らない静的コンポーネントであるため、props未指定時に例外が発生しないことを確認する
    // 【エラー処理の重要性】: 将来的な呼び出しミスに対する堅牢性を最低限担保するため
    // 【実際の発生シナリオ】: RootLayoutからの通常呼び出し
    // 🔵 信頼性レベル: requirements.md「入力・出力の仕様」（propsを受け取らない静的なナビゲーション構成）より確定

    // 【結果検証】: レンダリング時に例外がスローされないことを確認する
    // 【品質保証の観点】: 最低限のレンダリング安定性を担保する
    expect(() => {
      render(
        <MemoryRouter>
          <Sidebar />
        </MemoryRouter>
      )
    }).not.toThrow() // 【確認内容】: propsなしでもクラッシュしないことを確認
  })

  // ===== 境界値 =====

  it('TC-08: /collections/general表示時に「一般メディア」のみactiveになる（隣接パスとの混同なし）', () => {
    // 【境界値の意味】: /collections/配下の3パスは接頭辞が共通するため、NavLinkのマッチングロジックが完全一致でない場合に誤判定が起きやすい境界
    // 【境界値での動作保証】: 接頭辞が共通する隣接パス間でも正確に区別できることを保証する
    // 【境界値選択の根拠】: 3つの/collections/*パスのうち最初の項目を選び、残り2項目との混同有無を確認する代表ケース
    // 🟡 信頼性レベル: React Routerの`NavLink`一般仕様＋requirements.mdのパス一覧からの妥当な推測

    // 【実際の使用場面】: ユーザーが一般メディア一覧画面を閲覧中の状態
    renderWithRouter(['/collections/general'])

    // 【結果検証】: 「一般メディア」のみactiveであり、「学術書・専門書」「論文・文献」は非アクティブであることを確認する
    // 【境界での正確性】: パス文字列の前方一致による誤マッチがないこと
    expect(screen.getByRole('link', { name: '一般メディア' }).className).toContain('active') // 【確認内容】: 一般メディアがactiveであることを確認
    expect(screen.getByRole('link', { name: '学術書・専門書' }).className).not.toContain('active') // 【確認内容】: 学術書・専門書が誤ってactiveにならないことを確認
    expect(screen.getByRole('link', { name: '論文・文献' }).className).not.toContain('active') // 【確認内容】: 論文・文献が誤ってactiveにならないことを確認
  })
})
