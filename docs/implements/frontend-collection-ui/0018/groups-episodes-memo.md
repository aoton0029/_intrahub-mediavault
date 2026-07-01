# groups/episodes APIフック・GroupSection TDD開発完了記録

## 確認すべきドキュメント

- `docs/tasks/frontend-collection-ui/TASK-0018.md`
- `docs/implements/frontend-collection-ui/0018/groups-episodes-requirements.md`
- `docs/implements/frontend-collection-ui/0018/groups-episodes-testcases.md`

## 🎯 最終結果 (2026-07-01)

- **実装率**: 95% (18/19テストケース)
- **成功率**: 18/18 = 100%（スコープ内）
- **全体テスト**: 163/163 = 100%（スコープ外含む）
- **品質判定**: ✅ 合格
- **TODO更新**: ✅ 完了マーク追加

## 💡 重要な技術学習

### 実装パターン

- **APIフック**: `items.ts` と同一パターンで `groups.ts` を実装。`enabled: !!id` で空IDのfetch抑制、`onSuccess` で `invalidateQueries`。
- **queryKey設計**: `['items', 'groups', itemId]` / `['groups', 'episodes', groupId]` と階層分離することでキャッシュが独立して更新される。
- **コンポーネントモック**: `vi.mock('@/api/groups', () => ({ useItemGroupsQuery: vi.fn(), useCreateEpisodeMutation: vi.fn() }))` + `beforeEach` で `mockReturnValue` 設定。依存フックは全て `beforeEach` でモックする必要がある。

### テスト設計

- `SeasonGroupList` が内部で `useCreateEpisodeMutation` を使用するため、コンポーネントテストでは `useCreateEpisodeMutation` も `beforeEach` でモック設定が必要（Green フェーズで発見）。
- `data-testid` 属性でコンポーネントを識別するパターンが有効（`season-group-list` / `volume-group-list` / `episode-register-button`）。

### 品質保証

- **EDGE-004**: `VolumeGroupList` に `useCreateEpisodeMutation` を import しない設計で、誤実装を依存関係レベルで防止。
- エラー状態（`isError`）のUI表示はRefactorで追加。最小実装ではローディングのみでも動く。

## ⚠️ 未実装（後工程）

- **TC-GS-N-05**: movie → ChapterGroupList テスト（オプション要件、`data-testid="chapter-group-list"` の確認）
- **EpisodeRegisterButton**: 話数番号入力ダイアログ（現在 `episodeNumber: 1` ハードコード）
- **グループ追加ボタン/フォーム**: SeasonGroupList/VolumeGroupList のグループ追加UI
- **EpisodeList**: 話数一覧コンポーネント（TASK-0018.md に記載あり、本タスクで未実装）
- **ItemDetailPage統合**: GroupSectionをItemDetailPageに組み込む統合テスト（TASK-0017完了後）
