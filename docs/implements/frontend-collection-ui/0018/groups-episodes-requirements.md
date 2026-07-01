# groups/episodes APIフック・GroupSection 要件定義書

**機能名**: groups-episodes-api-group-section
**タスクID**: TASK-0018
**要件名**: frontend-collection-ui
**作成日**: 2026-07-01

---

## 1. 機能の概要

🔵 *requirements.md REQ-015/016/101/102、dataflow.md 機能4 より*

- **何をする機能か**: メディア種別（anime/drama/manga/novel/movie）に応じてグループ（season/volume/chapter）と話数（episode）をCRUD操作するAPIフックと、そのUIを表示するGroupSectionコンポーネントを実装する。
- **解決する問題**: アニメ・ドラマのシーズン話数管理、漫画・小説の巻管理、映画の章管理をメディア種別に応じて分岐し、誤操作（volumeへの話数登録）をUI側で排除する。
- **想定ユーザー**: MediaVaultを利用するコレクション管理者（単一ユーザー）
- **システム内位置づけ**: ItemDetailPage（TASK-0017）に組み込まれる詳細画面の構成要素。`api/groups.ts` がデータ取得/操作を担当し、`features/groups/` コンポーネントが表示を担当する。

**参照したEARS要件**: REQ-015, REQ-016, REQ-101, REQ-102
**参照した設計文書**: dataflow.md 機能4、architecture.md「リソース別フック」方針

---

## 2. 入力・出力の仕様

🔵 *interfaces.ts、api-endpoints.md より*

### APIフック

#### `useItemGroupsQuery(itemId: string)`

- **入力**: `itemId: string`（空文字の場合 `enabled: false` でfetchしない）
- **出力**: `{ data: { data: ItemGroup[] }, isLoading, isError, error }`
- **APIコール**: `GET /items/:id/groups`
- **queryKey**: `['items', 'groups', itemId]`

#### `useCreateGroupMutation(itemId: string)`

- **入力**: `CreateGroupRequest { groupType: GroupType, groupName: string, number?: number, displayOrder?: number }`
- **出力**: `{ data: { data: ItemGroup }, mutate, isLoading, isError }`
- **APIコール**: `POST /items/:id/groups`
- **副作用**: 成功時に `['items', 'groups', itemId]` を `invalidateQueries`

#### `useGroupEpisodesQuery(groupId: string)`

- **入力**: `groupId: string`（空文字の場合 `enabled: false`）
- **出力**: `{ data: { data: ItemEpisode[] }, isLoading, isError }`
- **APIコール**: `GET /groups/:group_id/episodes`
- **queryKey**: `['groups', 'episodes', groupId]`

#### `useCreateEpisodeMutation(groupId: string)`

- **入力**: `CreateEpisodeRequest { episodeNumber: number, title?: string, originalTitle?: string, airDate?: string, description?: string }`
- **出力**: `{ data: { data: ItemEpisode }, mutate, isLoading, isError }`
- **APIコール**: `POST /groups/:group_id/episodes`
- **副作用**: 成功時に `['groups', 'episodes', groupId]` を `invalidateQueries`

### コンポーネントProps

#### `GroupSection`

```typescript
interface GroupSectionProps {
  item: Item; // mediaType で分岐判断
}
```

#### `SeasonGroupList`

```typescript
interface SeasonGroupListProps {
  itemId: string;
}
```

#### `VolumeGroupList`

```typescript
interface VolumeGroupListProps {
  itemId: string;
}
```

#### `ChapterGroupList`（オプション）

```typescript
interface ChapterGroupListProps {
  itemId: string;
}
```

#### `EpisodeList`

```typescript
interface EpisodeListProps {
  groupId: string;
}
```

🔵 **型定義**: `frontend/src/types/index.ts` の `ItemGroup`, `ItemEpisode`, `CreateGroupRequest`, `CreateEpisodeRequest`, `GroupType`, `MediaType`

---

## 3. 制約条件

### UI制御制約（EDGE-004）🔵

- `group_type=volume` のグループには話数登録ボタンを**絶対に表示してはならない**。
- バックエンドの `INVALID_GROUP_TYPE_FOR_EPISODES` エラーはUI制御で未然に回避する（エラーハンドリングではなくUI設計で対応）。

### GroupSection分岐ロジック 🔵

```
anime / drama → SeasonGroupList（話数登録UI付き）
manga / novel → VolumeGroupList（話数登録UIなし）
movie         → ChapterGroupList（最小実装、表示のみ）
game / academic_book / paper → null（グループ管理対象外）
```

### アーキテクチャ制約 🔵

- APIフックは `frontend/src/api/groups.ts` に配置する
- コンポーネントは `frontend/src/features/groups/` に配置する
- `apiClient<T>(path, options?)` を経由してAPIコールを行う
- `ApiClientError` を例外クラスとして使用する

### パフォーマンス 🟡

- TanStack Query のデフォルトキャッシュ設定に従う（staleTime未指定でデフォルト0）
- `enabled: !!itemId` / `enabled: !!groupId` で不要なfetchを防ぐ

---

## 4. 想定される使用例

### 正常系: anime詳細画面でシーズン管理 🔵

1. ItemDetailPageがanimeのItemを受け取る
2. `<GroupSection item={animeItem} />` をレンダリング
3. `mediaType='anime'` → `SeasonGroupList` を表示
4. `useItemGroupsQuery(itemId)` でシーズン一覧を取得
5. 話数登録ボタンが表示される
6. ボタン押下 → `useCreateEpisodeMutation(groupId)` を呼び出し
7. 成功後、`['groups', 'episodes', groupId]` キャッシュが無効化され一覧が更新

### 正常系: manga詳細画面で巻管理 🔵

1. `<GroupSection item={mangaItem} />` をレンダリング
2. `mediaType='manga'` → `VolumeGroupList` を表示
3. 話数登録ボタンが**表示されない**（EDGE-004）
4. 巻一覧と「巻として管理する旨」のメッセージを表示

### エッジケース: itemIdが空の場合 🔵

- `useItemGroupsQuery('')` → `enabled: false` でfetchしない
- グループ一覧は空のまま

### エッジケース: gameのItemが渡された場合 🟡

- `GroupSection` は `null` を返す（グループ管理対象外）
- 画面に何も表示されない

### エラーケース: グループ取得API失敗 🟡

- `isError: true` の状態でエラーメッセージまたはエラートーストを表示
- 再取得可能な状態を保つ

---

## 5. EARS要件・設計文書との対応関係

| 要件 | 内容 | 信頼性 |
|------|------|--------|
| REQ-015 | `POST/GET /items/:id/groups` によるグループCRUDフック実装 | 🔵 |
| REQ-016 | `POST/GET /groups/:group_id/episodes` による話数CRUDフック実装 | 🔵 |
| REQ-101 | anime/drama → SeasonGroupList + 話数登録UI | 🔵 |
| REQ-102 | manga/novel → VolumeGroupList + 話数登録UI非表示 | 🔵 |
| EDGE-004 | volume groupには話数登録ボタンを表示しない（UI制御） | 🔵 |
| REQ-017/REQ-301 | movie → ChapterGroupList（オプション・最小実装） | 🟡 |
| game等対象外 | game/academic_book/paperはグループ管理対象外でnull返却 | 🟡 |

**参照した設計文書**:
- **アーキテクチャ**: `docs/design/frontend-collection-ui/architecture.md`（リソース別フック方針）
- **データフロー**: `docs/design/frontend-collection-ui/dataflow.md`（機能4フロー図）
- **型定義**: `docs/design/frontend-collection-ui/interfaces.ts`（ItemGroup, ItemEpisode, CreateGroupRequest, CreateEpisodeRequest）
- **API仕様**: `docs/design/mediavault-backend/api-endpoints.md`（groups/episodesエンドポイント）

---

## 信頼性レベルサマリー

- 🔵 青信号: 12件（80%）
- 🟡 黄信号: 3件（20%）
- 🔴 赤信号: 0件（0%）

**品質評価**: ✅ 高品質。REQ-015/016/101/102・EDGE-004との対応が明確。movie章グループ作成UIの詳細とgame等の対象外判断は推測含む（🟡）。
