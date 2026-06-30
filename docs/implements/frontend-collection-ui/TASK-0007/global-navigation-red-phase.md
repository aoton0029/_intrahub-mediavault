# TASK-0007: グローバルナビゲーション実装 Redフェーズ記録

## 作成したテストケース一覧

テストファイル: `frontend/src/components/common/Sidebar.test.tsx`

| ID | 分類 | 内容 | 信頼性 |
|---|---|---|---|
| TC-01 | 正常系 | 全8項目のナビゲーションリンクが表示される | 🔵 |
| TC-02 | 正常系 | 各リンクのhrefが正しいパスを指す | 🔵 |
| TC-03 | 正常系 | 「マイリスト」リンクをクリックすると/mylistsへ遷移する | 🔵 |
| TC-04 | 正常系 | 現在ルートが/staffのとき「スタッフ」リンクのみactiveクラスが付与される | 🔵 |
| TC-05 | 正常系 | ルートパス("/")表示時は「全体一覧」のみがアクティブになる | 🟡 |
| TC-06 | 異常系 | 未知のルート(/items/123)表示時にどのリンクもactiveにならない | 🟡 |
| TC-07 | 異常系 | Sidebarがpropsなしでもレンダリングエラーを起こさない | 🔵 |
| TC-08 | 境界値 | /collections/general表示時に「一般メディア」のみactiveになる（隣接パスとの混同なし） | 🟡 |

計8件（テストケース定義書の全8件を実装、目標10件未満だが利用可能な全テストケースを実装済み）。

## テストコード

`frontend/src/components/common/Sidebar.test.tsx` に保存済み（全文はファイル参照）。
`MemoryRouter` + 補助コンポーネント `LocationDisplay`（`useLocation`でパスを表示）を用いて、レンダリング・href・クリック遷移・アクティブクラスを検証。

## 実行結果（失敗確認）

```
$ yarn test src/components/common/Sidebar.test.tsx

FAIL src/components/common/Sidebar.test.tsx
Error: Failed to resolve import "./Sidebar" from "src/components/common/Sidebar.test.tsx". Does the file exist?
```

`frontend/src/components/common/Sidebar.tsx` が未実装のため、importエラーで全テストが失敗することを確認した（期待通りのRed状態）。

## Greenフェーズで実装すべき内容

- `frontend/src/components/common/Sidebar.tsx` を新規作成
- named export `Sidebar` コンポーネント（propsなし）
- `<nav>`内に8つの`NavLink`（react-router-dom v7）
  - `to="/"` には `end` propを付与し、前方一致による誤マッチを防止
  - `className`関数で`isActive`に応じて`'active'`を含むクラス名を返す
- navItems: 全体一覧(`/`) / 一般メディア(`/collections/general`) / 学術書・専門書(`/collections/academic`) / 論文・文献(`/collections/paper`) / マイリスト(`/mylists`) / タグ/カテゴリ(`/tags-categories`) / スタッフ(`/staff`) / 設定(`/settings`)
