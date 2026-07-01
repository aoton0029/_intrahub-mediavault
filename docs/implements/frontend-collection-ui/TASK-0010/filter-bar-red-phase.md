# FilterBar Redフェーズ記録

**作成日**: 2026-07-01  
**フェーズ**: Red（失敗するテスト作成）

## 作成したテストケース一覧

| ID | 分類 | 内容 | 結果 |
|---|---|---|---|
| TC-FB-N-01 | 正常系 | media_type変更でonChange呼び出し | ❌ 失敗 |
| TC-FB-N-02 | 正常系 | お気に入りトグルONでonChange呼び出し | ❌ 失敗 |
| TC-FB-N-03 | 正常系 | statusセレクト操作でonChange呼び出し | ❌ 失敗 |
| TC-FB-N-04 | 正常系 | タグ選択でonChange呼び出し | ❌ 失敗 |
| TC-FB-N-05 | 正常系 | カテゴリ選択でonChange呼び出し | ❌ 失敗 |
| TC-FB-N-06 | 正常系 | mediaTypeOptionsで選択肢制限 | ❌ 失敗 |
| TC-FB-N-07 | 正常系 | 既存filtersの値がUIに反映 | ❌ 失敗 |
| TC-FB-E-01 | 異常系 | お気に入りOFFでisFavoriteがundefined | ❌ 失敗 |
| TC-FB-E-02 | 異常系 | select空値選択でmediaTypeがundefined | ❌ 失敗 |
| TC-FB-E-03 | 異常系 | tagOptions空配列でエラーなくレンダリング | ✅ 成功 |
| TC-FB-B-01 | 境界値 | クリアボタンで全フィルタリセット | ❌ 失敗 |
| TC-FB-B-02 | 境界値 | mediaTypeOptions未指定で全8種表示 | ❌ 失敗 |
| TC-FB-B-03 | 境界値 | disabled=trueで全UI操作不能 | ❌ 失敗 |
| TC-FB-B-04 | 境界値 | filters={}で全UI未選択状態 | ❌ 失敗 |

**結果**: 13テスト失敗（1成功）- Redフェーズ確認完了

## 期待された失敗内容

現在の FilterBar 実装（最小コンテナ）には以下の要素が存在しないため失敗：
- media_typeセレクト（`role="combobox"` + `name=/メディアタイプ/i`）
- statusセレクト（`role="combobox"` + `name=/ステータス/i`）
- タグセレクト（`role="combobox"` + `name=/タグ/i`）
- カテゴリセレクト（`role="combobox"` + `name=/カテゴリ/i`）
- お気に入りチェックボックス（`role="checkbox"` + `name=/お気に入り/i`）
- クリアボタン（`role="button"` + `name=/クリア/i`）

## Greenフェーズで実装すべき内容

### FilterBar コンポーネントの完全実装

```tsx
// frontend/src/components/common/FilterBar.tsx
interface FilterBarProps {
  filters: ItemListFilters
  onChange: (filters: ItemListFilters) => void
  tagOptions: Tag[]
  categoryOptions: Category[]
  mediaTypeOptions?: MediaType[]
  disabled?: boolean
}
```

### 必要なUI要素

1. **メディアタイプセレクト** - `<label htmlFor="media-type">メディアタイプ</label>` + `<select id="media-type">`
2. **タグセレクト** - `<label htmlFor="tag">タグ</label>` + `<select id="tag">`
3. **カテゴリセレクト** - `<label htmlFor="category">カテゴリ</label>` + `<select id="category">`
4. **お気に入りチェックボックス** - `<label htmlFor="favorite">お気に入り</label>` + `<input type="checkbox" id="favorite">`
5. **statusセレクト** - `<label htmlFor="status">ステータス</label>` + `<select id="status">`
6. **クリアボタン** - `<button>クリア</button>`

### onChange の呼び出しロジック

- 各セレクト変更: `onChange({ ...filters, [field]: value || undefined })`
- checkbox クリック: `onChange({ ...filters, isFavorite: checked ? true : undefined })`
- クリアボタン: `onChange({})`

### filterの既存値のUI反映
- `<select value={filters.mediaType ?? ''}>` のような controlled パターン
- `<input type="checkbox" checked={filters.isFavorite ?? false}>` のような controlled パターン

### テストファイル
- `frontend/src/components/common/FilterBar.test.tsx`

### 実装ファイル
- `frontend/src/components/common/FilterBar.tsx`
