# ItemDetailPage基本実装 - 要件定義書

**機能名**: ItemDetailPage基本実装  
**タスクID**: TASK-0017  
**要件名**: frontend-collection-ui  
**作成日**: 2026-07-01

---

## 1. 機能の概要

🔵 `/items/:id` URLで単一アイテムの詳細情報を表示する画面を実装する。

### 目的・解決する問題

- 🔵 ユーザーが登録済みアイテムの全情報（タイトル・カバー画像・メディア別詳細・ステータス）を一画面で確認できる
- 🔵 status / consumed_date の更新操作（REQ-013）を提供する
- 🔵 アイテムの編集フォーム（ItemFormPage mode=edit）への遷移導線と削除操作（REQ-007）を提供する

### 想定ユーザー

- 🔵 単一ユーザー（認証なし、REQ-401相当）

### システム内での位置づけ

- 🔵 Phase 3の基盤画面。TASK-0018（GroupSection）・TASK-0019（RelationsSection）・TASK-0020（LinksFilesSection）のサブセクションがここに組み込まれる
- 🔵 `MediaCard` クリック → `/items/:id` ルートで表示される

**参照したEARS要件**: REQ-007, REQ-013  
**参照した設計文書**: `docs/tasks/frontend-collection-ui/TASK-0017.md` コンポーネント構成

---

## 2. 入力・出力の仕様

### 入力

| 入力 | 型 | 取得元 | 備考 |
|---|---|---|---|
| `id` | `string` | `useParams<{ id: string }>()` | URLパスパラメータ `/items/:id` 🔵 |
| `status` | `ItemStatus` | `StatusUpdateControl` のUI操作 | `'not_started'\|'in_progress'\|'completed'` 🔵 |
| `consumedDate` | `string \| undefined` | `StatusUpdateControl` のUI操作 | ISO日付文字列 YYYY-MM-DD 🔵 |

### 出力（画面表示）

| 出力 | 内容 | 信頼性 |
|---|---|---|
| アイテム詳細情報 | `Item`の全フィールド（title, coverImageUrl, status, details等） | 🔵 |
| mediaType別詳細 | `switch(item.mediaType)` で8種類の `details` フィールドを表示 | 🔵 |
| 編集ボタン | `/items/:id/edit` へのナビゲーション（ItemFormPage mode=edit）| 🔵 |
| 削除ボタン | ConfirmDialog経由でDELETE /items/:id実行、成功後に一覧へ遷移 | 🔵 |
| status更新UI | select/ボタンでステータスを変更 | 🔵 |
| consumed_date UI | 日付ピッカーで完了日を設定 | 🟡 |
| ローディング状態 | `isPending` 中はスケルトンまたはスピナー | 🔵 |
| エラー状態 | `ITEM_NOT_FOUND` 時はトースト＋一覧リダイレクト | 🔵 |
| Phase3プレースホルダ | GroupSection/RelationsSection/LinkFilesSection の空枠 | 🔵 |

### データフロー

```
URL /items/:id
  → useParams({ id })
  → useItemQuery(id)  [GET /items/:id]
    → isPending → スケルトン表示
    → isError (ITEM_NOT_FOUND) → トースト + navigate('/')
    → data.data → Item表示
        → switch(item.mediaType) → details表示分岐
        → StatusUpdateControl → useUpdateItemStatusMutation [PATCH /items/:id/status]
        → 削除ボタン → ConfirmDialog → useDeleteItemMutation [DELETE /items/:id] → navigate('/')
        → 編集ボタン → navigate(`/items/${id}/edit`)
```

**参照したEARS要件**: REQ-007, REQ-013  
**参照した設計文書**: `frontend/src/types/index.ts` Item型, `frontend/src/api/items.ts`

---

## 3. 制約条件

### アーキテクチャ制約

- 🔵 TanStack Query v5 の `useItemQuery` / `useDeleteItemMutation` / `useUpdateItemStatusMutation` を使用（既実装）
- 🔵 shadcn/ui + Tailwind CSS v4 でUIを構築
- 🔵 `ConfirmDialog` コンポーネント（既実装）を削除確認に使用
- 🔵 `useConfirmDialog` フック（既実装）で open/close 状態を管理可能

### API制約

- 🔵 `GET /items/:id` → `{ data: Item }` を返す
- 🔵 `DELETE /items/:id` → 204 No Content（成功後に `['items']` クエリを invalidate）
- 🔵 `PATCH /items/:id/status` → `UpdateItemStatusRequest` を受け取り `{ data: Item }` を返す

### エラー処理制約

- 🔵 `ITEM_NOT_FOUND`（404）受信時: エラートースト表示 + 一覧画面（履歴があれば直前、なければ `/`）へリダイレクト
- 🟡 `ApiClientError.code === 'ITEM_NOT_FOUND'` で判別（`ApiClientError` の `code` プロパティを使用）

### レスポンシブ制約

- 🟡 Tailwind CSSのレスポンシブユーティリティ（`md:`, `lg:`プレフィックス）でモバイル対応

### スコープ制約

- 🔵 本タスクではTASK-0018〜0020のサブセクションは空のプレースホルダ枠のみ実装
- 🔵 実際の編集フォームはItemFormPage(mode=edit)が担当（本タスクは遷移導線のみ）

**参照したEARS要件**: REQ-007, REQ-013  
**参照した設計文書**: `docs/tasks/frontend-collection-ui/TASK-0017.md` UI/UX要件

---

## 4. 想定される使用例

### 基本使用パターン

1. 🔵 **詳細表示**: ユーザーが一覧からアイテムをクリック → `/items/:id` に遷移 → 全詳細情報が表示される
2. 🔵 **status更新**: ステータスを `not_started` → `in_progress` に変更 → `PATCH /items/:id/status` 実行 → UIが即時反映される
3. 🔵 **削除**: 削除ボタン押下 → `ConfirmDialog` 表示 → 確認 → `DELETE /items/:id` → 一覧へ遷移
4. 🔵 **編集遷移**: 編集ボタン押下 → `/items/:id/edit` へナビゲーション

### mediaType別表示分岐（8パターン）

| mediaType | 表示する主なdetailsフィールド | 信頼性 |
|---|---|---|
| anime | episodeCount, seasonCount, studio, genreList | 🔵 |
| movie | runtimeMinutes, director, genreList | 🔵 |
| drama | episodeCount, seasonCount, network, genreList | 🔵 |
| manga | volumeCount, chapterCount, author, illustrator | 🔵 |
| novel | volumeCount, author, publisher, isbn | 🔵 |
| game | platformList, developer, publisher | 🔵 |
| academic_book | author, publisher, isbn, ndlId | 🔵 |
| paper | doi, journalName, authorList, pageRange | 🔵 |

### エラーケース

| エラー | トリガー | 対処 | 信頼性 |
|---|---|---|---|
| ITEM_NOT_FOUND | APIが404を返す | トースト表示 + 一覧へリダイレクト | 🔵 |
| isPending | データ取得中 | スケルトン/スピナー表示 | 🔵 |
| 削除失敗 | DELETEがエラーを返す | 🟡 エラートースト表示（詳細は推測） | 🟡 |

**参照したEARS要件**: REQ-007, REQ-013  
**参照した設計文書**: `docs/tasks/frontend-collection-ui/TASK-0017.md` テスト要件

---

## 5. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-007（編集・削除導線）, REQ-013（status/consumed_date更新）
- **参照したEdgeケース**: ITEM_NOT_FOUND 404エラー時のリダイレクト
- **参照した受け入れ基準**:
  - `/items/:id` で全mediaTypeのアイテム詳細が正しく表示される
  - status・consumed_date更新が `PATCH /items/:id/status` を通じて反映される
  - 削除導線が `ConfirmDialog` を経由して動作する
  - `ITEM_NOT_FOUND` 時に一覧へリダイレクトされる
  - モバイル幅でレイアウト崩れがない
  - 単体テストが全てパスする
- **参照した設計文書**:
  - **型定義**: `frontend/src/types/index.ts` の `Item`, `UpdateItemStatusRequest`, `ApiClientError`
  - **APIフック**: `frontend/src/api/items.ts` の `useItemQuery`, `useDeleteItemMutation`, `useUpdateItemStatusMutation`
  - **共通コンポーネント**: `frontend/src/components/common/ConfirmDialog.tsx`
  - **タスク定義**: `docs/tasks/frontend-collection-ui/TASK-0017.md`

---

## 品質評価

- **要件の曖昧さ**: ほぼなし（consumed_date UI の具体的なコンポーネント選択は🟡）
- **入出力定義**: 完全（8種mediaType別詳細フィールド含む）
- **制約条件**: 明確（APIエンドポイント・エラーコード・スコープ全て定義済み）
- **実装可能性**: 確実（既存APIフック・共通コンポーネント全て揃っている）
- **信頼性レベル分布**: 🔵 多数（90%+）、🟡 少数（削除失敗時UI・日付ピッカー詳細）、🔴 なし

**判定**: ✅ 高品質
