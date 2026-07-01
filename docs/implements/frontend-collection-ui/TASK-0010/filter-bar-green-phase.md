# FilterBar Greenフェーズ記録

**実装日時**: 2026-07-01  
**フェーズ**: Green（最小実装 → テスト全通過）

## テスト実行結果

```
Test Files  1 passed (1)
     Tests  14 passed (14)
  Duration  3.86s
```

**全14テスト通過** ✅

## 実装方針

- **controlledコンポーネント**: `filters` と `onChange` を外部から注入し、内部stateを持たない
- **ネイティブHTML select/checkbox**: jsdom環境でのテスト可能性を優先し、Radix UI Select/Switchは使用しない
- **空値→undefined変換**: select で空文字列を選択した場合は対応フィールドを `undefined` にしてonChangeを呼ぶ
- **isFavorite: checked ? true : undefined**: checkboxをOFFにした場合は `false` ではなく `undefined`

## 実装ファイル

`frontend/src/components/common/FilterBar.tsx`

## Refactorフェーズの候補

1. レスポンシブ対応（モバイルでは縦積みレイアウト）
2. Tailwind CSSクラスの整理・統一
3. お気に入りlabelの構造（現在はlabel内にinputを内包する実装）
4. shadcn/uiコンポーネントへの置き換え（E2Eテスト環境が整った場合）
