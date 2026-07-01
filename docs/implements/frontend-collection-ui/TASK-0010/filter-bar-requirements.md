# FilterBarコンポーネント詳細実装 要件定義書

**機能名**: FilterBar コンポーネント  
**タスクID**: TASK-0010  
**要件名**: frontend-collection-ui  
**出力ファイル**: `docs/implements/frontend-collection-ui/TASK-0010/filter-bar-requirements.md`

---

## 1. 機能の概要

- 🔵 **何をする機能か**: 一覧画面（HomePage/GeneralListPage/AcademicListPage/PaperListPage）に配置し、`media_type`・タグ・カテゴリ・お気に入り・statusの5種の絞り込みUIを提供するcontrolledコンポーネント
- 🔵 **問題解決**: コレクション全体から目的のアイテムを絞り込んで閲覧できるようにする（REQ-002）
- 🔵 **想定ユーザー**: 単一ユーザー（認証なしのセルフホスト）
- 🔵 **システム内での位置づけ**: `src/components/common/FilterBar.tsx` として複数の一覧ページで再利用される共通コンポーネント。各ページが `useSearchParamsFilter` を保持してFilterBarに渡す設計（controlledコンポーネント）
- 🔵 **参照したEARS要件**: REQ-002, REQ-003, REQ-004
- 🔵 **参照した設計文書**: `docs/design/frontend-collection-ui/architecture.md`「コンポーネント粒度 common/FilterBar」

---

## 2. 入力・出力の仕様

### FilterBar Props

```ts
interface FilterBarProps {
  filters: ItemListFilters          // 🔵 現在の絞り込み状態
  onChange: (filters: ItemListFilters) => void  // 🔵 絞り込み変更時のコールバック
  tagOptions: Tag[]                 // 🔵 タグ選択肢（呼び出し元から受け取る）
  categoryOptions: Category[]       // 🔵 カテゴリ選択肢（呼び出し元から受け取る）
  mediaTypeOptions?: MediaType[]    // 🔵 表示するmedia_typeを制限（省略時は全8種）
  disabled?: boolean                // 🟡 ローディング中はフィルタ全体をdisabledに
}
```

### ItemListFilters（入出力の型）

```ts
interface ItemListFilters {
  mediaType?: MediaType        // 'anime' | 'movie' | 'drama' | 'manga' | 'novel' | 'game' | 'academic_book' | 'paper'
  tagId?: string
  categoryId?: string
  isFavorite?: boolean
  status?: ItemStatus          // 'not_started' | 'in_progress' | 'completed'
  page?: number
  limit?: number
}
```

- 🔵 **入出力の関係**: ユーザーがUIを操作 → `onChange({ ...filters, <変更フィールド>: 新値 })` が呼ばれる
- 🔵 **クリア操作**: `onChange({})` で全フィルタをリセット（`page`/`limit`も含めてクリア）
- 🔵 **参照したEARS要件**: REQ-002, REQ-003
- 🔵 **参照した設計文書**: `docs/design/frontend-collection-ui/interfaces.ts`の`ItemListFilters`、`Tag`、`Category`

### 各フィルタUIと操作仕様

| フィルタ | UI種別 | 操作 | onChange呼び出し |
|---|---|---|---|
| media_type | `<select>` | 値選択 | `{ ...filters, mediaType: 選択値 }` |
| タグ | `<select>` | 値選択 | `{ ...filters, tagId: 選択値 }` |
| カテゴリ | `<select>` | 値選択 | `{ ...filters, categoryId: 選択値 }` |
| お気に入り | `<input type="checkbox">` | チェックOn/Off | `{ ...filters, isFavorite: true/undefined }` |
| status | `<select>` | 値選択 | `{ ...filters, status: 選択値 }` |
| クリア | `<button>` | クリック | `{}` |

- 🟡 **「空」値の扱い**: selectで「すべて」（空オプション）を選んだ場合は対応フィールドを`undefined`（削除）にする
- 🟡 **isFavoriteのOff**: Checkboxをオフにした場合は`isFavorite: undefined`（`false`ではなく除去）

---

## 3. 制約条件

### アーキテクチャ制約
- 🔵 **controlledコンポーネント**: FilterBar自身は内部stateを持たず、`filters`と`onChange`で完全に制御される（呼び出し元がフックを保持する設計）
- 🔵 **API不呼び出し**: FilterBar自身はタグ/カテゴリ取得APIを呼ばない。選択肢は必ずpropsで受け取る（関心の分離）
- 🔵 **URL管理は呼び出し元**: `useSearchParamsFilter`のsetFiltersはFilterBarの外（呼び出し元のPage）が管理する

### UI/テスト制約
- 🔴 **shadcn/ui Selectの代替**: jsdom環境でRadix UI SelectコンポーネントのPortalが正常動作しないため、ネイティブHTML `<select>` をTailwind CSSでスタイリングして使用する
- 🟡 **レスポンシブ**: フィルタ項目はデスクトップで横並び（`flex flex-wrap`）、モバイルで縦積み（`flex-col`）対応（NFR-202から推測）
- 🟡 **アクセシビリティ**: 各フィルタに`<label>`と`htmlFor`を設定し、キーボード操作可能にする（NFR-202）

### フィルタ選択肢の制限
- 🔵 **`mediaTypeOptions`省略時**: 全8種の`MediaType`を選択肢として表示
- 🔵 **`mediaTypeOptions`指定時**: 指定された`MediaType`配列のみを選択肢として表示（GeneralListPage等での用途）

---

## 4. 想定される使用例

### 基本パターン（HomePage）
```tsx
const { filters, setFilters } = useSearchParamsFilter()
// ...タグ・カテゴリ取得クエリ...
<FilterBar
  filters={filters}
  onChange={setFilters}
  tagOptions={tags}
  categoryOptions={categories}
/>
```

### media_type制限パターン（GeneralListPage）
```tsx
<FilterBar
  filters={filters}
  onChange={setFilters}
  tagOptions={tags}
  categoryOptions={categories}
  mediaTypeOptions={['anime', 'movie', 'drama', 'manga', 'novel', 'game']}
/>
```

### フィルタ変更フロー（dataflow.md 機能1より）
1. ユーザーがmedia_typeセレクトで「アニメ」を選択
2. `onChange({ ...filters, mediaType: 'anime' })` が呼ばれる
3. 呼び出し元ページが `setFilters` でURLクエリパラメータを更新
4. TanStack Queryが`queryKey`変化を検知してAPIを再取得

### エッジケース
- 🟡 **selectで「すべて」を選択**: `onChange({ ...filters, mediaType: undefined })` → URLからパラメータ削除
- 🟡 **クリアボタン押下時にfiltersが空**: `onChange({})` が呼ばれるが副作用なし
- 🟡 **tagOptions/categoryOptionsが空配列**: selectは「すべて」オプションのみ表示
- 🟡 **disabled=true時**: 全フィルタUIをdisabled属性でグレーアウト

---

## 5. EARS要件・設計文書との対応関係

- **参照した機能要件**:
  - REQ-002: media_type・タグ・カテゴリ・お気に入り・statusによる絞り込み提供
  - REQ-003: 絞り込み状態をURLクエリパラメータに反映（controlledコンポーネントとして呼び出し元が管理）
  - REQ-004: 一般メディア/学術書/論文の3グループ別一覧画面との連携（`mediaTypeOptions`で対応）
- **参照した非機能要件**:
  - NFR-202: WCAG準拠アクセシビリティ（labelとキーボード操作）
- **参照した設計文書**:
  - **アーキテクチャ**: `docs/design/frontend-collection-ui/architecture.md`「コンポーネント粒度」
  - **データフロー**: `docs/design/frontend-collection-ui/dataflow.md`「機能1: フィルタUI変更時はsetSearchParamsでURLを更新」
  - **型定義**: `docs/design/frontend-collection-ui/interfaces.ts`「ItemListFilters, Tag, Category, MediaType, ItemStatus」
  - **API仕様**: 本コンポーネント自身はAPIを呼ばないため直接の対象なし

---

## 品質評価

| 項目 | 評価 | 備考 |
|---|---|---|
| 要件の曖昧さ | ✅ なし | controlledコンポーネント設計が明確 |
| 入出力定義 | ✅ 完全 | Props型とonChange動作が定義済み |
| 制約条件 | ✅ 明確 | jsdom制約でnative select採用が明確 |
| 実装可能性 | ✅ 確実 | 依存タスク（TASK-0006, TASK-0008）完了済み |
| 信頼性分布 | 🔵:13 🟡:7 🔴:1 | shadcn/ui代替のみ🔴 |

**総合評価**: ✅ 高品質
