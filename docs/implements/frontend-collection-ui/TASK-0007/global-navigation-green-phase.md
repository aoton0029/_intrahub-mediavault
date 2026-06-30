# TASK-0007: グローバルナビゲーション実装 Greenフェーズ記録

## 実装方針

- `frontend/src/components/common/Sidebar.tsx` を新規作成。`navItems`配列でlabel/to対応表を定義し、`NavLink`でマップする最小実装。
- `to="/"`のみ`end`propを付与し、前方一致によるアクティブ誤判定（TC-05）を防止。
- `className`関数で`isActive`時に`'active'`を含むクラス名を返却（`cn()`ユーティリティ使用、既存コンポーネントと同じスタイル合成パターン）。
- `RootLayout.tsx`にSidebarを統合し、`<Outlet />`をflexレイアウトのmain領域に配置（TASK-0007「レイアウトとの統合」要件）。

## 実装コード

`frontend/src/components/common/Sidebar.tsx`:

```tsx
import { NavLink } from 'react-router-dom'
import { cn } from '@/lib/utils'

const navItems = [
  { to: '/', label: '全体一覧', end: true },
  { to: '/collections/general', label: '一般メディア' },
  { to: '/collections/academic', label: '学術書・専門書' },
  { to: '/collections/paper', label: '論文・文献' },
  { to: '/mylists', label: 'マイリスト' },
  { to: '/tags-categories', label: 'タグ/カテゴリ' },
  { to: '/staff', label: 'スタッフ' },
  { to: '/settings', label: '設定' },
] as const

export function Sidebar() {
  return (
    <nav>
      {navItems.map((item) => (
        <NavLink
          key={item.to}
          to={item.to}
          end={'end' in item ? item.end : undefined}
          className={({ isActive }) => cn('nav-link', isActive && 'active')}
        >
          {item.label}
        </NavLink>
      ))}
    </nav>
  )
}
```

`frontend/src/pages/RootLayout.tsx`（差分）:

```tsx
import { Outlet } from 'react-router-dom';
import { Sidebar } from '@/components/common/Sidebar';

export default function RootLayout() {
  return (
    <div className="flex min-h-screen">
      <Sidebar />
      <main className="flex-1">
        <Outlet />
      </main>
    </div>
  );
}
```

## テスト実行結果

```
$ yarn test src/components/common/Sidebar.test.tsx
 Test Files  1 passed (1)
      Tests  8 passed (8)
```

全8件（TC-01〜TC-08）成功。

## 追加確認

- `yarn lint`: エラーなし
- `yarn build`（tsc -b含む）: 成功、型エラーなし

## 課題・改善点（Refactorフェーズで対応）

- `nav-link`/`active`クラスは現状CSSクラス名のみでスタイル未定義（Tailwindユーティリティクラスへの置換、または既存shadcn/uiトークンとの統合を検討）。
- `navItems`の型を`'end' in item`という実行時チェックに頼っており、TypeScriptの型としてより明示的にできる余地がある。
- アクセシビリティ（`aria-current`等）は未対応、TASK-0034（アクセシビリティ対応）で扱う可能性あり。
