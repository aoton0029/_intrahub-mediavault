# 要件定義書: GeneralListPage実装

**機能名**: GeneralListPage（一般メディア一覧画面）
**タスクID**: TASK-0012
**要件名**: frontend-collection-ui
**出力ファイル**: `docs/implements/frontend-collection-ui/0012/general-list-page-requirements.md`

---

## 1. 機能の概要

🔵 **一般メディア専用一覧画面**
- `anime` / `movie` / `drama` / `manga` / `novel` / `game` の6種のメディアタイプに絞り込んだコレクション一覧画面を提供する
- `academic_book` / `paper` の学術系2種は表示対象に含めない
- ルート `/collections/general` でアクセス可能

🔵 **解決する問題**
- ユーザーが一般エンターテインメント系メディアのみを一覧したい場合に、学術書・論文が混在することなく目的のコンテンツを見つけられる

🔵 **想定ユーザー**
- MediaVaultを利用するコレクター（アニメ・映画・ドラマ・漫画・小説・ゲームを管理するユーザー）

🔵 **システム内での位置づけ**
- `pages/` 層の画面コンポーネント
- 共有コンポーネント（FilterBar, MediaCard, EmptyState）とフック（useItemsQuery, useSearchParamsFilter）を呼び出す
- HomePage との差分は `mediaTypeOptions` の6種固定のみ

- **参照したEARS要件**: REQ-004
- **参照した設計文書**: `docs/design/frontend-collection-ui/architecture.md` 画面構成とルーティング表

---

## 2. 入力・出力の仕様

### 入力

🔵 **URLクエリパラメータ（`useSearchParamsFilter` 経由）**

| URLパラメータ | 型 | 説明 |
|---|---|---|
| `media_type` | `string \| undefined` | `anime`/`movie`/`drama`/`manga`/`novel`/`game` のいずれか。未指定時は6種全件対象 |
| `tag_id` | `string \| undefined` | タグIDによる絞り込み |
| `category_id` | `string \| undefined` | カテゴリIDによる絞り込み |
| `favorite` | `"true" \| "false" \| undefined` | お気に入りフィルタ |
| `status` | `string \| undefined` | ステータスフィルタ |
| `page` | `string \| undefined` | ページ番号（デフォルト: 1） |
| `limit` | `string \| undefined` | 1ページあたり件数 |

🔵 **FilterBar の Props**

```ts
mediaTypeOptions: ['anime', 'movie', 'drama', 'manga', 'novel', 'game']
```

🟡 **APIリクエスト（`useItemsQuery` 経由）**
- `GET /items` にフィルタパラメータを送信
- `mediaType` 未指定時: `media_type` パラメータを送信しない（全件取得）
- `mediaType` 指定時: `media_type={value}` を送信
- **注意**: バックエンドAPIが単一 `media_type` 値のみ受け付ける場合、6種まとめての絞り込みはクライアント側では行わない（UIの選択肢を6種に固定することで実質的に制御）

### 出力

🔵 **画面表示**
- MediaCard グリッド（`grid-cols-2 md:grid-cols-4 lg:grid-cols-6`）
- FilterBar（mediaTypeOptions = 6種固定）
- EmptyState（0件時）
- ページネーションUI（前後ページボタン）
- ローディング時: スケルトン shimmer グリッド
- エラー時: エラーメッセージ + リトライボタン

🔵 **URLクエリパラメータへの同期**
- フィルタ変更時に URL を更新（`useSearchParamsFilter` の `setFilters` 経由）
- ページ変更時に `page` クエリパラメータを更新

- **参照したEARS要件**: REQ-002, REQ-003, REQ-004
- **参照した設計文書**: `frontend/src/types/index.ts` の `ItemListFilters`, `frontend/src/hooks/useSearchParamsFilter.ts`

---

## 3. 制約条件

🔵 **mediaTypeOptions の制約**
- FilterBar に渡す選択肢は `['anime', 'movie', 'drama', 'manga', 'novel', 'game']` の6種のみ
- `academic_book` と `paper` は選択肢に含めない（TASK要件）

🔵 **ルーティング**
- `/collections/general` ルートは `routes.tsx` に既定義済み（追加変更不要）
- `GeneralListPage` コンポーネントをルートに紐付けるのみ

🟡 **FilterBar コンポーネント制約**
- native HTML `<select>` を使用（Radix Select は jsdom で動作不安定）
- controlled component パターン（内部 state を持たない）

🔵 **共有ロジック制約**
- FilterBar, MediaCard, EmptyState, useItemsQuery, useSearchParamsFilter はHomePageと同一のものを再利用
- 本画面固有の追加コンポーネント・フックは原則作成しない

🔵 **アーキテクチャ制約**
- `pages/` 層のコンポーネントが `components/common/`, `hooks/`, `api/` を呼び出す構造を維持

- **参照したEARS要件**: REQ-004
- **参照した設計文書**: `docs/design/frontend-collection-ui/architecture.md`

---

## 4. 想定される使用例

### 正常系

🔵 **基本フロー: 初期表示**
1. ユーザーが `/collections/general` にアクセス
2. `useSearchParamsFilter` が URLパラメータから `filters` を取得（初期: 空）
3. `useItemsQuery(filters)` が `GET /items` を呼び出す（`media_type` パラメータなし）
4. レスポンスのアイテムを MediaCard グリッドで表示
5. FilterBar には `anime`/`movie`/`drama`/`manga`/`novel`/`game` の6種のみ表示

🔵 **フィルタ操作: media_type 選択**
1. ユーザーが FilterBar の `media_type` セレクトで `manga` を選択
2. `setFilters({ mediaType: 'manga' })` が呼ばれ URL が `?media_type=manga` に更新
3. `useItemsQuery({ mediaType: 'manga' })` が再実行され `GET /items?media_type=manga` を送信
4. 漫画のみの一覧が表示される

🔵 **空状態表示**
1. フィルタ条件に合うアイテムが0件
2. EmptyState コンポーネントが表示される（「コレクションがありません」等のメッセージ）

### 異常系・エッジケース

🟡 **ローディング状態**
- API取得中はスケルトン shimmer グリッドを表示

🟡 **エラー状態**
- API呼び出し失敗時はエラーメッセージとリトライボタンを表示

🟡 **URLに academic_book/paper が直接入力された場合**
- `?media_type=academic_book` のようなURLで直接アクセスしても、FilterBar の選択肢には表示されない
- APIは実行されるが、その結果がUIに表示されることになる（フィルタ選択肢への非表示のみで制御）

- **参照したEARS要件**: REQ-002, REQ-003, REQ-004
- **参照した設計文書**: `docs/design/frontend-collection-ui/dataflow.md`

---

## 5. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-002（メディアタイプ等による絞り込み）, REQ-003（URL同期）, REQ-004（グループ別専用一覧）
- **参照した設計文書**:
  - **アーキテクチャ**: `docs/design/frontend-collection-ui/architecture.md` 画面構成・ルーティング表・共有コンポーネント方針
  - **データフロー**: `docs/design/frontend-collection-ui/dataflow.md`
  - **型定義**: `frontend/src/types/index.ts` の `ItemListFilters`, `Item`
  - **API仕様**: `frontend/src/api/items.ts` の `useItemsQuery`, `filtersToSearchParams`
- **依存タスク**: TASK-0011（HomePage実装・共有ロジック抽出元）

---

## 品質評価

| 項目 | 評価 | 備考 |
|---|---|---|
| 要件の曖昧さ | ✅ なし | 6種固定という明確な要件 |
| 入出力定義 | ✅ 完全 | FilterBarオプション・URLパラメータ・API仕様を明確化 |
| 制約条件 | ✅ 明確 | ルート定義済み・共有コンポーネント再利用 |
| 実装可能性 | ✅ 確実 | HomePageのパターンをほぼそのまま流用 |
| 信頼性レベル分布 | 🔵×12 🟡×5 🔴×0 | 高品質 |

**判定: ✅ 高品質**
