# groups/episodes APIフック・GroupSection テストケース定義書

**機能名**: groups-episodes-api-group-section
**タスクID**: TASK-0018
**要件名**: frontend-collection-ui
**作成日**: 2026-07-01

---

## 4. 開発言語・フレームワーク 🔵

- **プログラミング言語**: TypeScript 5.7+
- **テストフレームワーク**: Vitest + @testing-library/react（jsdom環境）
- **テストコマンド**: `yarn test`（`frontend/` ディレクトリ内）
- **テストファイル配置**: 実装ファイルと同ディレクトリ（`.test.ts` / `.test.tsx`）

---

## 1. 正常系テストケース

### TC-GRP-N-01: useItemGroupsQuery - GET /items/:id/groups でグループ一覧を取得する 🔵

- **何をテストするか**: `useItemGroupsQuery('item-001')` が正しいURLにfetchを行い、グループ一覧を返すこと
- **入力値**: `itemId = 'item-001'`、fetchが `{ success: true, data: [mockGroup] }` を返す
- **期待される結果**: `result.current.data.data` に `[mockGroup]` が含まれる；fetchURLが `/items/item-001/groups` を含む
- **テストの目的**: REQ-015のGETエンドポイント呼び出しを確認
- **参照要件**: REQ-015

```typescript
// 【テスト目的】: useItemGroupsQueryがGET /items/:id/groupsを正しく呼び出すことを確認
// 【期待される動作】: fetchが正しいURLで呼ばれ、グループ一覧データが返される
it('TC-GRP-N-01: GET /items/:id/groups でグループ一覧を取得する', async () => {
  // 【テストデータ準備】: 成功レスポンスのfetchモック設定
  const fetchMock = vi.fn().mockResolvedValue({
    ok: true,
    json: async () => ({ success: true, data: [mockGroup] }),
  } as Response);
  vi.stubGlobal('fetch', fetchMock);

  const { result } = renderHook(() => useItemGroupsQuery('item-001'), { wrapper });

  // 【実際の処理実行】: ローディング完了を待機
  await waitFor(() => expect(result.current.isLoading).toBe(false));

  // 【結果検証】: URLとデータの確認
  const calledUrl = fetchMock.mock.calls[0][0] as string;
  expect(calledUrl).toContain('/items/item-001/groups');
  expect(result.current.data?.data).toEqual([mockGroup]);
});
```

---

### TC-GRP-N-02: useCreateGroupMutation - POST /items/:id/groups でグループを作成し、一覧キャッシュを無効化する 🔵

- **何をテストするか**: `useCreateGroupMutation('item-001').mutate(body)` が正しくAPIコールし、`invalidateQueries` を呼ぶこと
- **入力値**: `CreateGroupRequest { groupType: 'season', groupName: 'Season 1', displayOrder: 1 }`
- **期待される結果**: fetchが `POST /items/item-001/groups` に呼ばれる；`invalidateQueries({ queryKey: ['items', 'groups', 'item-001'] })` が1回呼ばれる
- **テストの目的**: REQ-015のPOSTエンドポイントとキャッシュ無効化を確認
- **参照要件**: REQ-015

```typescript
// 【テスト目的】: グループ作成後にキャッシュが無効化されることを確認
it('TC-GRP-N-02: POST /items/:id/groups でグループを作成しキャッシュ無効化', async () => {
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
    ok: true,
    json: async () => ({ success: true, data: mockGroup }),
  } as Response));
  const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');

  const { result } = renderHook(() => useCreateGroupMutation('item-001'), { wrapper });
  result.current.mutate({ groupType: 'season', groupName: 'Season 1', displayOrder: 1 });

  await waitFor(() => expect(result.current.isSuccess).toBe(true));

  expect(invalidateSpy).toHaveBeenCalledWith(
    expect.objectContaining({ queryKey: ['items', 'groups', 'item-001'] })
  );
});
```

---

### TC-GRP-N-03: useGroupEpisodesQuery - GET /groups/:group_id/episodes で話数一覧を取得する 🔵

- **入力値**: `groupId = 'group-001'`、fetchが `{ success: true, data: [mockEpisode] }` を返す
- **期待される結果**: fetchURLが `/groups/group-001/episodes` を含む；`result.current.data.data` に `[mockEpisode]`
- **参照要件**: REQ-016

---

### TC-GRP-N-04: useCreateEpisodeMutation - POST /groups/:group_id/episodes で話数を作成しキャッシュ無効化 🔵

- **入力値**: `groupId = 'group-001'`、`CreateEpisodeRequest { episodeNumber: 1, title: 'Episode 1' }`
- **期待される結果**: `invalidateQueries({ queryKey: ['groups', 'episodes', 'group-001'] })` が1回呼ばれる
- **参照要件**: REQ-016

---

### TC-GS-N-01: GroupSection - anime の場合 SeasonGroupList が表示され話数登録ボタンが表示される 🔵

- **何をテストするか**: `mediaType='anime'` の Item を渡した時、SeasonGroupListが表示され話数登録ボタンが存在する
- **入力値**: `item.mediaType = 'anime'`
- **期待される結果**: `data-testid="season-group-list"` が存在；`data-testid="episode-register-button"` が存在
- **テストの目的**: REQ-101の分岐ロジックを確認
- **参照要件**: REQ-101

```typescript
// 【テスト目的】: anime mediaTypeのときSeasonGroupListと話数登録UIが表示されることを確認
it('TC-GS-N-01: anime のとき SeasonGroupList と話数登録ボタンが表示される', () => {
  vi.mocked(useItemGroupsQuery).mockReturnValue({
    data: { data: [mockSeasonGroup] },
    isLoading: false,
    isError: false,
  } as any);

  render(<QueryClientProvider client={queryClient}><GroupSection item={animeItem} /></QueryClientProvider>);

  expect(screen.getByTestId('season-group-list')).toBeInTheDocument();
  expect(screen.getByTestId('episode-register-button')).toBeInTheDocument();
});
```

---

### TC-GS-N-02: GroupSection - drama の場合も SeasonGroupList が表示される 🔵

- **入力値**: `item.mediaType = 'drama'`
- **期待される結果**: `data-testid="season-group-list"` が存在；`data-testid="episode-register-button"` が存在
- **参照要件**: REQ-101

---

### TC-GS-N-03: GroupSection - manga の場合 VolumeGroupList が表示され話数登録ボタンが表示されない 🔵

- **何をテストするか**: `mediaType='manga'` の時 VolumeGroupList が表示され、話数登録ボタンが存在しないこと（EDGE-004）
- **入力値**: `item.mediaType = 'manga'`
- **期待される結果**: `data-testid="volume-group-list"` が存在；`data-testid="episode-register-button"` が存在しない
- **テストの目的**: REQ-102 + EDGE-004の制御を確認
- **参照要件**: REQ-102, EDGE-004

```typescript
// 【テスト目的】: manga mediaTypeのとき話数登録ボタンが表示されないことを確認（EDGE-004）
it('TC-GS-N-03: manga のとき VolumeGroupList が表示され話数登録ボタンは非表示', () => {
  vi.mocked(useItemGroupsQuery).mockReturnValue({
    data: { data: [mockVolumeGroup] },
    isLoading: false,
    isError: false,
  } as any);

  render(<QueryClientProvider client={queryClient}><GroupSection item={mangaItem} /></QueryClientProvider>);

  expect(screen.getByTestId('volume-group-list')).toBeInTheDocument();
  expect(screen.queryByTestId('episode-register-button')).not.toBeInTheDocument();
});
```

---

### TC-GS-N-04: GroupSection - novel の場合も VolumeGroupList が表示され話数登録ボタンが非表示 🔵

- **入力値**: `item.mediaType = 'novel'`
- **期待される結果**: `data-testid="volume-group-list"` が存在；話数登録ボタンなし
- **参照要件**: REQ-102, EDGE-004

---

### TC-GS-N-05: GroupSection - movie の場合 ChapterGroupList が表示される 🟡

- **入力値**: `item.mediaType = 'movie'`
- **期待される結果**: `data-testid="chapter-group-list"` が存在
- **参照要件**: REQ-017/REQ-301（オプション）

---

### TC-GS-N-06: GroupSection - game の場合 null が返り何も表示されない 🟡

- **入力値**: `item.mediaType = 'game'`
- **期待される結果**: グループ関連のUI要素が一切表示されない
- **参照要件**: TASK-0018タスクノート（game等は対象外）

---

### TC-GRP-N-05: queryKey - useItemGroupsQuery のキーが ['items', 'groups', itemId] 形式である 🔵

- **入力値**: `itemId = 'item-001'`
- **期待される結果**: `queryClient.getQueriesData({ queryKey: ['items', 'groups', 'item-001'] }).length > 0`
- **テストの目的**: キャッシュが正しいキーで管理されることを確認

---

## 2. 異常系テストケース

### TC-GRP-E-01: useItemGroupsQuery - APIエラー時に isError=true を返す 🔵

- **入力値**: fetchが `{ ok: false, json: () => ({ success: false, error: { code: 'SERVER_ERROR', message: '...' } }) }` を返す
- **期待される結果**: `result.current.isError === true`；`result.current.error` が `ApiClientError` のインスタンス
- **テストの目的**: APIエラーが適切にハンドルされることを確認

```typescript
// 【テスト目的】: APIエラー時にisErrorがtrueになりApiClientErrorが返ることを確認
it('TC-GRP-E-01: APIエラー時に isError=true を返す', async () => {
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
    ok: false,
    json: async () => ({ success: false, error: { code: 'SERVER_ERROR', message: 'サーバーエラー' } }),
  } as Response));

  const { result } = renderHook(() => useItemGroupsQuery('item-001'), { wrapper });

  await waitFor(() => expect(result.current.isError).toBe(true));

  expect(result.current.error).toBeInstanceOf(ApiClientError);
  expect((result.current.error as ApiClientError).code).toBe('SERVER_ERROR');
});
```

---

### TC-GRP-E-02: useCreateGroupMutation - API失敗時に isError=true を返す 🔵

- **入力値**: fetchが `{ ok: false }` を返す
- **期待される結果**: `result.current.isError === true`；`invalidateQueries` は呼ばれない
- **テストの目的**: ミューテーション失敗時のキャッシュ汚染がないことを確認

---

### TC-GRP-E-03: useGroupEpisodesQuery - ネットワークエラー時に isError=true を返す 🔵

- **入力値**: fetchが `new Error('connection refused')` で reject
- **期待される結果**: `result.current.isError === true`
- **テストの目的**: ネットワーク障害時のエラーハンドリング確認

---

### TC-GS-E-01: EDGE-004 - VolumeGroupList にはどの状態でも話数登録ボタンが存在しない 🔵

- **何をテストするか**: `VolumeGroupList` コンポーネントを直接レンダリングしても話数登録ボタンが表示されない
- **入力値**: `<VolumeGroupList itemId="item-001" />`（グループデータあり）
- **期待される結果**: `queryByTestId('episode-register-button')` が `null`
- **テストの目的**: UI制御でバックエンドの `INVALID_GROUP_TYPE_FOR_EPISODES` エラーを誘発しないことを確認
- **参照要件**: EDGE-004

```typescript
// 【テスト目的】: EDGE-004 - VolumeGroupListには話数登録ボタンが存在しないことを確認
// 【期待される動作】: UIレベルでバックエンドエラーを未然に回避
it('TC-GS-E-01: EDGE-004 - VolumeGroupList には話数登録ボタンが存在しない', () => {
  vi.mocked(useItemGroupsQuery).mockReturnValue({
    data: { data: [mockVolumeGroup] },
    isLoading: false,
    isError: false,
  } as any);

  render(<QueryClientProvider client={queryClient}><VolumeGroupList itemId="item-001" /></QueryClientProvider>);

  // 【結果検証】: 話数登録ボタンが一切表示されないことを確認
  expect(screen.queryByTestId('episode-register-button')).not.toBeInTheDocument();
  expect(screen.queryByText(/話数を追加/)).not.toBeInTheDocument();
});
```

---

## 3. 境界値テストケース

### TC-GRP-B-01: useItemGroupsQuery - itemId が空文字のとき fetch しない 🔵

- **入力値**: `itemId = ''`
- **期待される結果**: fetchが呼ばれない（`enabled: !!itemId` の制御）
- **テストの目的**: 空IDで不要なAPIコールが発生しないことを確認

```typescript
// 【テスト目的】: itemIdが空文字の場合fetchが発生しないことを確認
it('TC-GRP-B-01: itemId が空文字のとき fetch しない', async () => {
  const fetchMock = vi.fn();
  vi.stubGlobal('fetch', fetchMock);

  renderHook(() => useItemGroupsQuery(''), { wrapper });

  await new Promise((r) => setTimeout(r, 100));

  // 【結果検証】: fetchが一度も呼ばれないことを確認
  expect(fetchMock).not.toHaveBeenCalled();
});
```

---

### TC-GRP-B-02: useGroupEpisodesQuery - groupId が空文字のとき fetch しない 🔵

- **入力値**: `groupId = ''`
- **期待される結果**: fetchが呼ばれない（`enabled: !!groupId`）
- **参照要件**: note.md 注意事項

---

### TC-GRP-B-03: useItemGroupsQuery - グループが0件のとき空配列を返す 🟡

- **入力値**: `itemId = 'item-001'`、fetchが `{ success: true, data: [] }` を返す
- **期待される結果**: `result.current.data.data` が `[]`；`isLoading: false`
- **テストの目的**: 空一覧の正常ハンドリングを確認

---

### TC-GS-B-01: GroupSection - academic_book のとき何も表示しない 🟡

- **入力値**: `item.mediaType = 'academic_book'`
- **期待される結果**: グループ関連UI要素なし
- **テストの目的**: game等と同様の非対応mediaTypeの処理確認

---

### TC-GS-B-02: GroupSection - paper のとき何も表示しない 🟡

- **入力値**: `item.mediaType = 'paper'`
- **期待される結果**: グループ関連UI要素なし

---

## テストセットアップ共通コード

```typescript
// 【テスト前準備】: QueryClientを毎テストで新規作成しキャッシュ汚染を防止
beforeEach(() => {
  queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  wrapper = createWrapper(queryClient);
});

// 【テスト後処理】: グローバルモックとQueryClientキャッシュをリセット
afterEach(() => {
  vi.unstubAllGlobals();
  queryClient.clear();
});
```

---

## テストデータ定義

```typescript
const mockGroup: ItemGroup = {
  id: 'group-001',
  itemId: 'item-001',
  groupType: 'season',
  groupName: 'Season 1',
  displayOrder: 1,
  createdAt: '2026-07-01T00:00:00Z',
  updatedAt: '2026-07-01T00:00:00Z',
};

const mockSeasonGroup: ItemGroup = { ...mockGroup, groupType: 'season' };
const mockVolumeGroup: ItemGroup = { ...mockGroup, groupType: 'volume', groupName: '第1巻' };

const mockEpisode: ItemEpisode = {
  id: 'ep-001',
  groupId: 'group-001',
  episodeNumber: 1,
  title: 'Episode 1',
  createdAt: '2026-07-01T00:00:00Z',
  updatedAt: '2026-07-01T00:00:00Z',
};

const baseItem: Item = {
  id: 'item-001',
  title: 'Test Item',
  status: 'not_started',
  isFavorite: false,
  source: 'manual',
  createdAt: '2026-07-01T00:00:00Z',
  updatedAt: '2026-07-01T00:00:00Z',
  details: { genreList: [] },
};

const animeItem: Item = { ...baseItem, mediaType: 'anime' };
const dramaItem: Item = { ...baseItem, mediaType: 'drama' };
const mangaItem: Item = { ...baseItem, mediaType: 'manga' };
const novelItem: Item = { ...baseItem, mediaType: 'novel' };
const movieItem: Item = { ...baseItem, mediaType: 'movie' };
const gameItem: Item = { ...baseItem, mediaType: 'game' };
```

---

## 6. 要件定義との対応関係

| テストケース | 参照要件 | 信頼性 |
|-------------|---------|--------|
| TC-GRP-N-01 | REQ-015 GET /items/:id/groups | 🔵 |
| TC-GRP-N-02 | REQ-015 POST /items/:id/groups | 🔵 |
| TC-GRP-N-03 | REQ-016 GET /groups/:group_id/episodes | 🔵 |
| TC-GRP-N-04 | REQ-016 POST /groups/:group_id/episodes | 🔵 |
| TC-GS-N-01/02 | REQ-101 anime/drama→SeasonGroupList | 🔵 |
| TC-GS-N-03/04 | REQ-102 manga/novel→VolumeGroupList | 🔵 |
| TC-GS-N-05 | REQ-017/REQ-301 movie→ChapterGroupList | 🟡 |
| TC-GS-N-06 | game等対象外 | 🟡 |
| TC-GRP-E-01/02/03 | エラーハンドリング | 🔵 |
| TC-GS-E-01 | EDGE-004 volume話数登録UI排除 | 🔵 |
| TC-GRP-B-01/02 | enabled制御 | 🔵 |
| TC-GRP-B-03 | 空配列ハンドリング | 🟡 |

---

## 信頼性レベルサマリー

- 🔵 青信号: 14件（74%）
- 🟡 黄信号: 5件（26%）
- 🔴 赤信号: 0件（0%）

**品質判定**: ✅ 高品質。正常系・異常系・境界値が網羅され、EDGE-004の重要制約も含む。movie/game等のオプション要件は🟡。
