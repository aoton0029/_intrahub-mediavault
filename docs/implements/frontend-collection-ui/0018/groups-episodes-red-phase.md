# Redフェーズ記録: groups/episodes APIフック・GroupSection

**作成日**: 2026-07-01
**TASK-ID**: TASK-0018

## 作成したテストケース一覧

### `frontend/src/api/groups.test.ts`（APIフックテスト）

| テストID | 説明 | 信頼性 |
|---------|------|--------|
| TC-GRP-N-01 | GET /items/:id/groups でグループ一覧を取得する | 🔵 |
| TC-GRP-N-05 | queryKeyが['items','groups',itemId]形式である | 🔵 |
| TC-GRP-B-01 | itemId が空文字のとき fetch しない | 🔵 |
| TC-GRP-B-03 | グループが0件のとき空配列を返す | 🟡 |
| TC-GRP-E-01 | APIエラー時に isError=true を返す | 🔵 |
| TC-GRP-N-02 | POST /items/:id/groups でグループを作成しキャッシュ無効化 | 🔵 |
| TC-GRP-E-02 | グループ作成API失敗時に isError=true でキャッシュ無効化しない | 🔵 |
| TC-GRP-N-03 | GET /groups/:group_id/episodes で話数一覧を取得する | 🔵 |
| TC-GRP-B-02 | groupId が空文字のとき fetch しない | 🔵 |
| TC-GRP-E-03 | ネットワークエラー時に isError=true を返す | 🔵 |
| TC-GRP-N-04 | POST /groups/:group_id/episodes で話数を作成しキャッシュ無効化 | 🔵 |

### `frontend/src/features/groups/GroupSection.test.tsx`（コンポーネントテスト）

| テストID | 説明 | 信頼性 |
|---------|------|--------|
| TC-GS-N-01 | anime → SeasonGroupList と話数登録ボタンが表示される | 🔵 |
| TC-GS-N-02 | drama → SeasonGroupList が表示される | 🔵 |
| TC-GS-N-03 | manga → VolumeGroupList が表示され話数登録ボタンは非表示 | 🔵 |
| TC-GS-N-04 | novel → VolumeGroupList が表示され話数登録ボタンは非表示 | 🔵 |
| TC-GS-N-06 | game → 何も表示されない | 🟡 |
| TC-GS-E-01 | EDGE-004: VolumeGroupList には話数登録ボタンが存在しない | 🔵 |
| TC-GS-N-07 | SeasonGroupList には話数登録ボタンが存在する | 🔵 |

## 期待される失敗内容

```
groups.test.ts:
Error: Failed to resolve import "./groups" from "src/api/groups.test.ts". Does the file exist?
→ frontend/src/api/groups.ts が未実装

GroupSection.test.tsx:
Error: Failed to resolve import "./GroupSection" from "src/features/groups/GroupSection.test.tsx"
→ frontend/src/features/groups/GroupSection.tsx が未実装
```

## Greenフェーズで実装すべき内容

### 1. `frontend/src/api/groups.ts`

```typescript
// 実装が必要な関数/フック:
export async function fetchItemGroups(itemId: string): Promise<{ data: ItemGroup[] }>
export async function createGroup(itemId: string, body: CreateGroupRequest): Promise<{ data: ItemGroup }>
export async function fetchGroupEpisodes(groupId: string): Promise<{ data: ItemEpisode[] }>
export async function createEpisode(groupId: string, body: CreateEpisodeRequest): Promise<{ data: ItemEpisode }>

export function useItemGroupsQuery(itemId: string)
export function useCreateGroupMutation(itemId: string)
export function useGroupEpisodesQuery(groupId: string)
export function useCreateEpisodeMutation(groupId: string)
```

### 2. `frontend/src/features/groups/GroupSection.tsx`

```typescript
// mediaType による分岐:
// anime/drama → SeasonGroupList
// manga/novel → VolumeGroupList
// movie       → ChapterGroupList
// その他      → null
```

### 3. `frontend/src/features/groups/SeasonGroupList.tsx`

- `data-testid="season-group-list"` のルート要素
- `data-testid="episode-register-button"` の話数登録ボタン
- `useItemGroupsQuery` / `useCreateEpisodeMutation` を使用

### 4. `frontend/src/features/groups/VolumeGroupList.tsx`

- `data-testid="volume-group-list"` のルート要素
- 話数登録ボタン・テキスト一切なし（EDGE-004）

### 5. `frontend/src/features/groups/ChapterGroupList.tsx`（オプション）

- `data-testid="chapter-group-list"` のルート要素
