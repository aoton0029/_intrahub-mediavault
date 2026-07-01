# TASK-0018 コンテキストノート: groups/episodes APIフック・GroupSection実装

## 1. 技術スタック

- React 18.3+ / TypeScript 5.7+ / Vite 6
- TanStack Query v5（useQuery / useMutation）
- Tailwind CSS 4 + shadcn/ui
- Vitest + @testing-library/react（jsdom環境）
- テスト環境: `globals: true`, setupFiles: `src/test/setup.ts`

参照元: `docs/spec/frontend-collection-ui/note.md`, `frontend/vitest.config.ts`

## 2. 開発ルール

- APIフックは `frontend/src/api/` に配置（`groups.ts`）
- コンポーネントは `frontend/src/features/groups/` に配置
- fetchラッパーは `apiClient<T>(path)` を使用（`frontend/src/api/client.ts`）
- APIエラーは `ApiClientError`（`code`, `message`）として例外化される
- fetchモックは `vi.stubGlobal('fetch', ...)` で行い、`afterEach` で `vi.unstubAllGlobals()` を呼ぶ
- `QueryClient` は `retry: false` でテスト用に初期化

参照元: `frontend/src/api/items.ts`, `frontend/src/api/client.ts`

## 3. 関連実装

### APIフック構造パターン（items.tsより）

```typescript
// fetch関数
export async function fetchItemGroups(itemId: string) {
  return apiClient<ItemGroup[]>(`/items/${itemId}/groups`);
}

// Queryフック
export function useItemGroupsQuery(itemId: string) {
  return useQuery({
    queryKey: ['items', 'groups', itemId],
    queryFn: () => fetchItemGroups(itemId),
    enabled: !!itemId,
  });
}

// Mutationフック
export function useCreateGroupMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateGroupRequest) => createGroup(itemId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', 'groups', itemId] });
    },
  });
}
```

参照元: `frontend/src/api/items.ts`, `frontend/src/api/items.test.ts`

### コンポーネントモックパターン（HomePage.test.tsxより）

```typescript
vi.mock('@/api/groups', () => ({
  useItemGroupsQuery: vi.fn(),
}));
vi.mocked(useItemGroupsQuery).mockReturnValue({ data: {...}, isLoading: false } as any);
```

参照元: `frontend/src/pages/HomePage.test.tsx`

## 4. 設計文書

### 対象APIエンドポイント

- `GET  /items/:id/groups` → グループ一覧取得
- `POST /items/:id/groups` → グループ作成
- `GET  /groups/:group_id/episodes` → 話数一覧取得
- `POST /groups/:group_id/episodes` → 話数作成

### mediaType別GroupSection分岐

| mediaType | groupType | 話数登録UI |
|-----------|-----------|-----------|
| anime / drama | season | 表示する |
| manga / novel | volume | 表示しない（EDGE-004） |
| movie | chapter | オプション（最小実装） |
| game / academic_book / paper | なし | null返却 |

参照元: `docs/tasks/frontend-collection-ui/TASK-0018.md`, `docs/design/frontend-collection-ui/dataflow.md`

### 型定義

```typescript
// frontend/src/types/index.ts より
interface ItemGroup { id, itemId, groupType: GroupType, groupName, displayOrder, createdAt, updatedAt }
interface ItemEpisode { id, groupId, episodeNumber, title?, airDate?, createdAt, updatedAt }
interface CreateGroupRequest { groupType, groupName, number?, displayOrder? }
interface CreateEpisodeRequest { episodeNumber, title?, originalTitle?, airDate?, description? }
type GroupType = 'season' | 'volume' | 'chapter'
type MediaType = 'anime' | 'movie' | 'drama' | 'manga' | 'novel' | 'game' | 'academic_book' | 'paper'
```

参照元: `frontend/src/types/index.ts`, `docs/design/frontend-collection-ui/interfaces.ts`

## 5. テスト関連情報

### テストフレームワーク・設定

- Vitest（`frontend/vitest.config.ts`）: `environment: 'jsdom'`, `globals: true`
- セットアップ: `frontend/src/test/setup.ts` → `@testing-library/jest-dom/vitest`
- テストコマンド: `yarn test`（frontend ディレクトリ内）

### 既存テストの命名パターン

- APIフックテスト: `frontend/src/api/items.test.ts` → `TC-IQ-N-01` 形式
- コンポーネントテスト: `frontend/src/pages/HomePage.test.tsx`
- テストファイル配置: 実装ファイルと同ディレクトリ（`.test.ts` / `.test.tsx`）

### fetchモックパターン

```typescript
// 成功
vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
  ok: true,
  json: async () => ({ success: true, data: [...] })
} as Response));

// APIエラー
vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
  ok: false,
  json: async () => ({ success: false, error: { code: 'NOT_FOUND', message: '...' } })
} as Response));

// 非同期完了待機
await waitFor(() => expect(result.current.isLoading).toBe(false));

// mutation副作用検証
const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
```

### QueryClientラッパー

```typescript
function createWrapper(queryClient: QueryClient) {
  return ({ children }: { children: React.ReactNode }) =>
    createElement(QueryClientProvider, { client: queryClient }, children);
}
```

## 6. 注意事項

- **EDGE-004**: `group_type=volume` のグループには話数登録ボタンを表示しない（UI制御でバックエンドエラー回避）
- `ItemDetailPage.tsx` は現在スタブ状態のため、GroupSectionの統合テストは`GroupSection`単体でItemを渡す形で実施
- `frontend/src/features/` ディレクトリは未作成のため新規作成が必要
- APIベースURL: `http://localhost:8080/api/v1`（`apiClient`内で自動付与）
- `enabled: !!itemId` / `enabled: !!groupId` で空IDの場合はfetchしない

参照元: `docs/tasks/frontend-collection-ui/TASK-0018.md`, `docs/spec/frontend-collection-ui/note.md`
