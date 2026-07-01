# items APIフック TDD開発完了記録

## 確認すべきドキュメント

- `docs/tasks/frontend-collection-ui/TASK-0009.md`
- `docs/implements/frontend-collection-ui/TASK-0009/items-api-hooks-requirements.md`
- `docs/implements/frontend-collection-ui/TASK-0009/items-api-hooks-testcases.md`

## 🎯 最終結果 (2026-07-01)
- **実装率**: 100% (17/17テストケース)
- **品質判定**: ✅ 合格
- **全体テスト**: 93/93 (11ファイル全通過)
- **TODO更新**: ✅ 完了マーク追加

## 💡 重要な技術学習

### 実装パターン
- `filtersToSearchParams` ユーティリティ: `ItemListFilters` のキャメルケース→スネークケース変換と undefined スキップを一箇所に集約。TASK-0015（search.ts）でも同様ロジックが必要になる場合は共通化を検討すること
- TanStack Query v5 でのキャッシュキー設計: `['items', filters]`（一覧）、`['items', 'detail', id]`（詳細）の二層構造。`invalidateQueries({ queryKey: ['items'] })` で一覧と詳細の両方を無効化できる
- `enabled: !!id` で id 未指定時のクエリスキップを実現

### テスト設計
- `createQueryClient({ defaultOptions: { queries: { retry: false } } })` でテスト時の再試行を無効化
- `vi.spyOn(queryClient, 'invalidateQueries')` で onSuccess の副作用を検証
- `vi.stubGlobal('fetch', vi.fn())` でfetchをモックし、URLのクエリパラメータを `fetchMock.mock.calls[0][0]` で検証

### 品質保証
- lint で `as any` 使用禁止 → テストファイルでも `as UpdateItemStatusRequest['status']` のように型安全に書く
- `tsc --noEmit` でコンパイルエラーがないことを確認してからコミット
