# グローバルナビゲーション（Sidebar） TDD開発完了記録

## 確認すべきドキュメント

- `docs/tasks/frontend-collection-ui/TASK-0007.md`
- `docs/implements/frontend-collection-ui/TASK-0007/global-navigation-requirements.md`
- `docs/implements/frontend-collection-ui/TASK-0007/global-navigation-testcases.md`
- `docs/implements/frontend-collection-ui/TASK-0007/note.md`
- `docs/implements/frontend-collection-ui/TASK-0007/global-navigation-red-phase.md`
- `docs/implements/frontend-collection-ui/TASK-0007/global-navigation-green-phase.md`
- `docs/implements/frontend-collection-ui/TASK-0007/global-navigation-refactor-phase.md`

## 🎯 最終結果 (2026-06-30)

- **実装率**: 100% (8/8テストケース)
- **品質判定**: 合格（高品質）
- **TODO更新**: ✅完了マーク追加

実装ファイル: `frontend/src/components/common/Sidebar.tsx`（53行）、`frontend/src/pages/RootLayout.tsx`（統合）
テストファイル: `frontend/src/components/common/Sidebar.test.tsx`（8件: 正常系5・異常系2・境界値1）

全体テスト状況: 8ファイル60件すべて成功（スコープ外失敗なし）。`yarn lint`/`yarn build`もエラーなし。

## 💡 重要な技術学習

### 実装パターン

- react-router-dom v7の`NavLink`で`to="/"`のみ`end`propを付与することで、ルートパスの前方一致による誤アクティブ化（"/"が常にアクティブと判定される問題）を防止できる。`/collections/general`等の隣接パスはendなしで完全一致判定により正しく区別される。
- `NavLink`の`children`をrender-prop（`{({isActive}) => ...}`）形式にすると、`className`の`isActive`判定とは別に内部要素（`aria-current`等）にも同じ状態を反映できる。

### テスト設計

- `MemoryRouter`内に検証用の`<Routes><Route path="*" element={<LocationDisplay />} /></Routes>`を併設し、`useLocation`でクリック後の遷移先パスを直接アサーションする手法が有効（実際のページ遷移をモックなしで検証できる）。
- 隣接する類似パス（`/collections/general` vs `/collections/academic`等）間でのアクティブ判定混同は、境界値テストとして明示的に検証する価値がある。

### 品質保証

- 静的・props無しのナビゲーションコンポーネントでは、セキュリティ（外部入力なし）・パフォーマンス（固定8件map）共にリスクが低く、レビューの主眼はアクセシビリティ（aria-current, aria-label）と型安全性に置くのが効果的だった。

## ⚠️ 注意点・修正が必要な項目

- `nav-link`/`active`クラスは構造のみでTailwindの実スタイル定義は未実装（🟡設計文書に視覚仕様の明記なし）。Phase 2以降でRootLayoutへの本格的なスタイリングが入る際に併せて対応する想定。
- アクセシビリティの本格対応（キーボード操作・フォーカス管理等）はTASK-0034（アクセシビリティ・レスポンシブ対応）で扱う。
