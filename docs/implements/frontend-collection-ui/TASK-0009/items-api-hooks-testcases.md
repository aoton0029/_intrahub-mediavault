# items APIフック テストケース定義書

**機能名**: items APIフック実装
**タスクID**: TASK-0009
**要件名**: frontend-collection-ui
**作成日**: 2026-07-01
**テストファイル**: `frontend/src/api/items.test.ts`

---

## 4. 開発言語・フレームワーク

- **プログラミング言語**: TypeScript 5.7+（🔵 プロジェクト標準）
- **テストフレームワーク**: Vitest + @testing-library/react（🔵 `frontend/vitest.config.ts` より）
- **テスト環境**: jsdom（🔵 vitest.config.ts `environment: 'jsdom'`）
- **モック戦略**: `vi.stubGlobal('fetch', vi.fn())` によるfetchモック（🔵 `client.test.ts` パターンに倣う）
- **TanStack Query ラッパー**: `QueryClient` + `QueryClientProvider`（🔵 TanStack Query v5 テスト標準）

---

## 1. 正常系テストケース

### TC-IQ-N-01: useItemsQueryがフィルタなしでGET /itemsを呼び出す
🔵 TASK-0009.md 完了条件・requirements.md 2-1より

- **入力値**: `filters = {}`
- **期待される結果**: `GET /items`（クエリパラメータなし）が発行され、`data` に `Item[]` が返る
- **テストの目的**: フィルタ未指定時に余分なクエリパラメータが付かないことを確認

### TC-IQ-N-02: useItemsQueryがmediaType/page/limitフィルタ付きでGET /itemsを呼び出す
🔵 TASK-0009.md テストケース1・requirements.md 2-1より

- **入力値**: `filters = { mediaType: 'anime', page: 1, limit: 20 }`
- **期待される結果**: `GET /items?media_type=anime&page=1&limit=20` が発行される
- **テストの目的**: フィルタオブジェクトのプロパティ名がスネークケースのクエリパラメータに正しく変換されることを確認

### TC-IQ-N-03: useItemsQueryがis_favorite/status/tag_id/category_idフィルタ付きで呼び出す
🔵 requirements.md 2-1「URLクエリパラメータ変換ルール」より

- **入力値**: `filters = { isFavorite: true, status: 'completed', tagId: 't1', categoryId: 'c1' }`
- **期待される結果**: `GET /items?is_favorite=true&status=completed&tag_id=t1&category_id=c1` が発行される
- **テストの目的**: キャメルケース→スネークケース変換の全フィールドカバレッジ確認

### TC-IQ-N-04: フィルタ変更でqueryKeyが変化し再取得される
🔵 TASK-0009.md テストケース2より

- **入力値**: 初回 `filters = {}`、次回 `filters = { isFavorite: true }`
- **期待される結果**: queryKeyが `['items', {}]` → `['items', { isFavorite: true }]` に変化し、別エントリとしてキャッシュされる
- **テストの目的**: フィルタ変更でTanStack Queryが自動再取得することを確認

### TC-IQ-N-05: useItemQueryが正常系でアイテム詳細を返す
🔵 TASK-0009.md テストケース3・完了条件より

- **入力値**: `id = 'item-001'`
- **期待される結果**: `GET /items/item-001` が呼び出され、`data.data` に `Item` が返る
- **テストの目的**: 詳細取得APIの正常動作確認

### TC-IQ-N-06: useItemsQueryのqueryKeyが['items', filters]形式である
🔵 TASK-0009.md 完了条件より

- **入力値**: `filters = { mediaType: 'manga' }`
- **期待される結果**: queryKeyが `['items', { mediaType: 'manga' }]` である
- **テストの目的**: キャッシュキーの形式確認

### TC-IQ-N-07: useItemQueryのqueryKeyが['items', 'detail', id]形式である
🔵 TASK-0009.md 完了条件より

- **入力値**: `id = 'item-001'`
- **期待される結果**: queryKeyが `['items', 'detail', 'item-001']` である
- **テストの目的**: 詳細クエリのキャッシュキー形式確認

### TC-IQ-N-08: useDeleteItemMutation成功時にinvalidateQueriesが['items']で呼ばれる
🔵 TASK-0009.md テストケース4・完了条件より

- **入力値**: `mutate('item-001')`、DELETE /items/item-001 が 204 を返す
- **期待される結果**: `queryClient.invalidateQueries({ queryKey: ['items'] })` が呼ばれる
- **テストの目的**: 削除後の一覧キャッシュ無効化を確認

### TC-IQ-N-09: useUpdateItemStatusMutation成功時に一覧と詳細のキャッシュが無効化される
🔵 TASK-0009.md 完了条件より

- **入力値**: `mutate({ id: 'item-001', body: { status: 'completed' } })`
- **期待される結果**: `invalidateQueries({ queryKey: ['items'] })` と `invalidateQueries({ queryKey: ['items', 'detail', 'item-001'] })` の両方が呼ばれる
- **テストの目的**: ステータス更新後の一覧・詳細両方のキャッシュ無効化確認

### TC-IQ-N-10: useUpdateItemStatusMutationが成功時に更新後のItemを返す
🔵 requirements.md 2-4「出力」より

- **入力値**: `mutate({ id: 'item-001', body: { status: 'in_progress' } })`
- **期待される結果**: `mutation.data.data` に更新後の `Item` が返る
- **テストの目的**: PATCH /items/:id/status のレスポンスが正しく取得できることを確認

### TC-IQ-N-11: undefinedフィールドはクエリパラメータに含まれない
🔵 TASK-0009.md 注意事項・requirements.md 2-1より

- **入力値**: `filters = { mediaType: 'anime', tagId: undefined, page: 1 }`
- **期待される結果**: URLに `tag_id` パラメータが含まれない（`?media_type=anime&page=1` のみ）
- **テストの目的**: undefinedフィールドのスキップ動作確認

---

## 2. 異常系テストケース

### TC-IQ-E-01: useItemsQueryがAPIエラー時にApiClientErrorを返す
🔵 TASK-0009.md 完了条件「APIエラー時、各フックのerrorにApiClientErrorがそのまま伝播する」より

- **入力値**: `filters = {}`、APIが `{ success: false, error: { code: 'SERVER_ERROR', message: 'サーバーエラー' } }` を返す
- **期待される結果**: `query.error` が `ApiClientError` インスタンスで、`error.code === 'SERVER_ERROR'`
- **テストの目的**: 一覧取得時のエラー伝播確認

### TC-IQ-E-02: useItemQueryがAPIエラー時にApiClientErrorを返す
🔵 TASK-0009.md 完了条件より

- **入力値**: `id = 'not-found'`、APIが `{ success: false, error: { code: 'NOT_FOUND', message: '...' } }` を返す
- **期待される結果**: `query.error` が `ApiClientError` インスタンスで `error.code === 'NOT_FOUND'`
- **テストの目的**: 詳細取得時のエラー伝播確認

### TC-IQ-E-03: useUpdateItemStatusMutationがバリデーションエラー時にApiClientErrorを伝播する
🟡 TASK-0009.md テストケース5より（バックエンドエラーコードは推測）

- **入力値**: `mutate({ id: 'item-001', body: { status: 'invalid_status' as ItemStatus } })`、APIが `VALIDATION_ERROR` を返す
- **期待される結果**: `mutation.error` が `ApiClientError` で `error.code === 'VALIDATION_ERROR'`
- **テストの目的**: ミューテーション時のエラー伝播確認

### TC-IQ-E-04: useDeleteItemMutationがAPIエラー時にApiClientErrorを返す
🔵 requirements.md 制約条件「エラー伝播」より

- **入力値**: `mutate('not-exist')`、APIが `NOT_FOUND` エラーを返す
- **期待される結果**: `mutation.error` が `ApiClientError` インスタンス
- **テストの目的**: 削除ミューテーションのエラー伝播確認

### TC-IQ-E-05: ネットワークエラー時にNETWORK_ERRORコードのApiClientErrorが伝播する
🔵 `client.ts`・`client.test.ts` パターンより

- **入力値**: fetchが `Error('connection refused')` をthrow
- **期待される結果**: `query.error.code === 'NETWORK_ERROR'`
- **テストの目的**: ネットワーク障害時のエラーコード確認

---

## 3. 境界値テストケース

### TC-IQ-B-01: useItemQueryはidが空文字のときfetchしない（enabled=false）
🔵 TASK-0009.md 実装詳細「enabled: !!id」より

- **入力値**: `id = ''`
- **期待される結果**: fetchが呼ばれない、`query.data` は `undefined`
- **テストの目的**: id未指定時のクエリスキップ動作確認

### TC-IQ-B-02: フィルタが空オブジェクトの場合クエリパラメータなしでGET /itemsを呼び出す
🔵 requirements.md 4-1より

- **入力値**: `filters = {}`
- **期待される結果**: `GET /items`（`?` なし、もしくはパラメータ空）でリクエスト
- **テストの目的**: 空フィルタの処理確認

### TC-IQ-B-03: isFavorite=falseはクエリパラメータに含まれない（またはis_favorite=false）
🟡 requirements.md注意事項「undefinedフィールドはスキップ」の境界

- **入力値**: `filters = { isFavorite: false }`
- **期待される結果**: `is_favorite=false` がURLに含まれる（falseは明示的な値なので含める）もしくはundefined扱いでスキップ。実装で決定する。
- **テストの目的**: boolean=falseのクエリパラメータ変換動作確認
- **備考**: 実装時に挙動を確定し、テストコードと一致させること

### TC-IQ-B-04: useDeleteItemMutationの削除成功後にinvalidateQueriesが1回のみ呼ばれる
🔵 TASK-0009.md 完了条件より

- **入力値**: `mutate('item-001')` を1回呼ぶ
- **期待される結果**: `queryClient.invalidateQueries({ queryKey: ['items'] })` が厳密に1回呼ばれる
- **テストの目的**: onSuccess内の副作用が二重実行されないことを確認

---

## 5. テストケース実装コメント指針

```ts
// 【テスト目的】: useItemsQueryがフィルタ条件をクエリパラメータに変換してGET /itemsを呼び出すことを確認
// 【テスト内容】: filters={mediaType:'anime', page:1, limit:20}で useItemsQuery を実行し、fetchの呼び出し引数を検証
// 【期待される動作】: GET /items?media_type=anime&page=1&limit=20 相当のリクエストが発行される
// 🔵 TASK-0009.md テストケース1より

// 【テストデータ準備】: QueryClientのキャッシュを無効化するためretry:falseで生成
// 【初期条件設定】: fetchをモックし正常レスポンスを返すよう設定
// 【前提条件確認】: TanStack Query v5 の QueryClientProvider でラップ必須

// 【実際の処理実行】: useItemsQuery({ mediaType: 'anime', page: 1, limit: 20 }) を renderHook で実行
// 【処理内容】: queryKey=['items', filters]でクエリを発行し、fetchを通じてAPIを呼び出す
// 【実行タイミング】: waitFor でデータ取得完了を待機

// 【結果検証】: fetchの第1引数URLにクエリパラメータが含まれることを確認
// 【期待値確認】: 'media_type=anime&page=1&limit=20' がURLに含まれる
// 【品質保証】: フィルタ変換ロジックの正確性を保証
```

---

## 6. 要件定義との対応関係

- **参照した機能概要**: `items-api-hooks-requirements.md` セクション1
- **参照した入力・出力仕様**: `items-api-hooks-requirements.md` セクション2（2-1〜2-4）
- **参照した制約条件**: `items-api-hooks-requirements.md` セクション3（エラー伝播、TanStack Query v5制約）
- **参照した使用例**: `items-api-hooks-requirements.md` セクション4（4-1〜4-6）
- **参照したタスク完了条件**: `docs/tasks/frontend-collection-ui/TASK-0009.md` 完了条件5項目

---

## 品質判定

✅ **高品質**
- テストケース分類: 正常系11件・異常系5件・境界値4件（計20件）で網羅
- 期待値定義: 全ケースでURL形式・queryKey・invalidateQueries呼び出しを明確に定義
- 技術選択: Vitest + Testing Library（プロジェクト標準）
- 実装可能性: 既存 `client.test.ts` のパターンに準拠、実現可能
- 信頼性レベル: 🔵18件・🟡2件（TC-IQ-E-03・TC-IQ-B-03）
