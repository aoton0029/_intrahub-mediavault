# apiClient相対パス化 TDD開発完了記録

## 確認すべきドキュメント

- `docs/tasks/backend-frontend-integration/TASK-0003.md`
- `docs/implements/backend-frontend-integration/TASK-0003/api-client-relative-baseurl-requirements.md`
- `docs/implements/backend-frontend-integration/TASK-0003/api-client-relative-baseurl-testcases.md`

## 🎯 最終結果 (2026-07-02)
- **実装率**: 100% (2/2 新規テストケース + 既存回帰確認)
- **品質判定**: 合格
- **TODO更新**: ✅完了マーク追加

## 💡 重要な技術学習

### 実装パターン
- `frontend/src/api/client.ts:4` の `BASE_URL` を絶対URLから相対パス `/api/v1` に変更。`VITE_API_BASE_URL` によるフォールバックは維持。

### テスト設計
- `BASE_URL` はモジュールトップレベルの `const` で評価されるため、環境変数を変えて挙動を検証するテストでは `vi.resetModules()` + 動的 `import()` が必要（`vi.stubEnv` だけでは反映されない）。

### 品質保証
- `yarn vitest run` 全体（21ファイル182テスト）が全通過。既存の `groups.test.ts` / `relations.test.ts` / `search.test.ts` 等はURLを直接アサーションしていなかったため回帰なし。

## 全体テスト状況

- 全テストケース総数: 182個
- 成功: 182個 / 失敗: 0個
- 全体テスト成功率: 100%
