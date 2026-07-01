# Greenフェーズ記録: groups/episodes APIフック・GroupSection

**作成日**: 2026-07-01
**TASK-ID**: TASK-0018

## テスト実行結果

- `src/api/groups.test.ts`: 11件 ✅ 全て通過
- `src/features/groups/GroupSection.test.tsx`: 7件 ✅ 全て通過
- **合計: 18件 全て成功**

## 実装ファイル

| ファイル | 行数 | 役割 |
|---------|------|------|
| `frontend/src/api/groups.ts` | ~110行 | APIフック（4フック + 4fetch関数） |
| `frontend/src/features/groups/GroupSection.tsx` | ~40行 | mediaType分岐コンポーネント |
| `frontend/src/features/groups/SeasonGroupList.tsx` | ~65行 | season用（話数登録UI付き） |
| `frontend/src/features/groups/VolumeGroupList.tsx` | ~45行 | volume用（話数登録UIなし、EDGE-004） |
| `frontend/src/features/groups/ChapterGroupList.tsx` | ~35行 | chapter用（movie、オプション） |

## 実装方針

1. `groups.ts`: `items.ts`と同じパターン（apiClient呼び出し + useQuery/useMutation + invalidateQueries）
2. `GroupSection.tsx`: switch文でmediaTypeを分岐
3. `SeasonGroupList.tsx`: `useItemGroupsQuery` + `useCreateEpisodeMutation` 使用、話数登録ボタン付き
4. `VolumeGroupList.tsx`: `useItemGroupsQuery` のみ使用、話数登録ボタン一切なし（EDGE-004）
5. テスト修正: `useCreateEpisodeMutation` のモック設定をbeforeEachに追加

## 課題・改善点（Refactorフェーズ対応）

- `EpisodeRegisterButton` の実際のフォームUI（番号・タイトル入力ダイアログ）未実装
- グループ追加ボタン・フォームUI未実装
- ローディング・エラー表示の改善
- `EpisodeList` コンポーネント（話数一覧）の実装
- movie/TC-GS-N-05テストが未追加（`data-testid="chapter-group-list"`の動作確認）
