# Greenフェーズ記録: items APIフック

**実装日**: 2026-07-01
**実装ファイル**: `frontend/src/api/items.ts`

## 実装方針

- `filtersToSearchParams` ユーティリティ関数でキャメルケース→スネークケース変換を集約
- undefinedフィールドは `params.set` を呼ばないことで自然にスキップ
- `fetchItems`/`fetchItem`/`deleteItem`/`updateItemStatus` の4つのfetch関数を実装
- `useItemsQuery`/`useItemQuery`/`useDeleteItemMutation`/`useUpdateItemStatusMutation` の4フックを実装
- `enabled: !!id` で空文字時のクエリスキップを実現
- `onSuccess` コールバックで `queryClient.invalidateQueries` を呼び出してキャッシュを無効化

## テスト結果

```
Test Files  1 passed (1)
     Tests  17 passed (17)
  Duration  6.44s
```

**全17テストケース成功** ✅

## Refactorフェーズの候補

- `filtersToSearchParams` 関数は TASK-0015（search.ts）でも再利用可能。共通ユーティリティへの抽出を検討
- 現時点ではシンプルな実装で十分
