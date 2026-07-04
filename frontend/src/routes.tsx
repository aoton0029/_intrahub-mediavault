import { createBrowserRouter, Link, useRouteError } from 'react-router-dom'
import HomePage from './pages/HomePage'
import ItemDetailPage from './pages/ItemDetailPage'

/**
 * 【プレースホルダ画面】: 手動追加・編集画面（04_item_form.html相当）は別要件のため未実装。
 * ItemDetailPageの「編集」ボタンはこのルートへnavigateするため、ルート自体が存在しないことによる
 * 未定義ルートエラー（React Routerのマッチなしエラー）を避ける目的の暫定コンポーネント。
 * 🟡 信頼性レベル: TASK-0022要件（編集ボタン押下時に例外が発生しないこと）からの妥当な推測
 */
function ItemFormPlaceholder() {
  return (
    <main className="main">
      <p>編集画面は準備中です。</p>
      <Link to="/">一覧へ戻る</Link>
    </main>
  )
}

/**
 * 【ルーティング全体のエラーバウンダリ】: 未定義ルートへのアクセスや描画時例外でアプリ全体が
 * 白画面クラッシュしないよう、最低限の404/エラー表示にフォールバックする。
 * 🟡 信頼性レベル: TASK-0022「アプリケーションがクラッシュしないこと」からの妥当な推測
 */
function RouteErrorBoundary() {
  const error = useRouteError()
  console.error(error)
  return (
    <main className="main">
      <p>ページが見つかりません</p>
      <Link to="/">一覧へ戻る</Link>
    </main>
  )
}

export const router = createBrowserRouter([
  {
    path: '/',
    element: <HomePage />,
    errorElement: <RouteErrorBoundary />,
  },
  {
    path: '/items/:id',
    element: <ItemDetailPage />,
    errorElement: <RouteErrorBoundary />,
  },
  {
    // 【暫定プレースホルダルート】: 04_item_form.html相当の編集画面は別要件で未実装のため、
    // 404にならないようプレースホルダを用意する（REQ-007、TASK-0022要件）🟡
    path: '/items/:id/edit',
    element: <ItemFormPlaceholder />,
    errorElement: <RouteErrorBoundary />,
  },
  {
    path: '*',
    element: <RouteErrorBoundary />,
  },
])
