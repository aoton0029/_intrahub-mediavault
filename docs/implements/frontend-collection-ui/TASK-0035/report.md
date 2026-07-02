# TASK-0035 E2E主要フローテスト整備 実装記録

## 概要

- **タスクID**: TASK-0035
- **状態**: ✅ 完了
- **実行日時**: 2026-07-02

## 実装内容

### 共通モック基盤

- `frontend/tests/e2e/helpers/mockApi.ts`: `page.route()`ベースの共通モックヘルパー群（items一覧/詳細/作成/更新、外部検索成功/エラー、インポート成功/エラー、booklogインポート）
- `frontend/tests/e2e/fixtures/booklog-sample.csv`: インポートテスト用固定CSVファイル

### E2Eテストファイル（4フロー、計7ケース+補助1件）

- `list-filter.spec.ts`: TC1 全体一覧の絞り込みとURL同期（フィルタ変更→`?media_type=`反映→戻る操作で復元）
- `search-add.spec.ts`: TC2（正常系）・TC3（API_KEY_NOT_CONFIGURED 422）・TC4（EXTERNAL_API_TIMEOUT 502）
- `manual-add-edit.spec.ts`: TC5（手動追加正常系）・バリデーションNGケース・TC6（編集正常系）
- `import.spec.ts`: TC7（インポート成功・失敗混在結果表示）

## 動作確認結果

```
yarn test:e2e
9 passed (15.5s)
```

既存の`smoke.spec.ts`を含む全9件のE2Eテストが成功。全テストはバックエンド実APIに依存せず、`page.route()`によるモックのみで完結する。

## 遭遇した問題と解決

1. **`インポート実行`ボタンの多重マッチ**: `SettingsPage`のブクログ/Steam両セクションが同時にDOMへレンダリングされているため、ボタン名だけでは一意に特定できなかった。`section`要素をテキストでスコープして解決。
2. **エラー系テストのタイムアウト**: TanStack Queryのデフォルト`retry`設定によりエラー表示までの時間が伸びるため、該当アサーションのタイムアウトを15秒に延長して対応。

## 完了条件チェック

- [x] フロー①〜④それぞれのE2Eテストファイルを`tests/e2e/`配下に作成
- [x] フロー②に`API_KEY_NOT_CONFIGURED`（422）・`EXTERNAL_API_TIMEOUT`（502）ケースを含む
- [x] 全E2Eテストがモックレスポンスのみで完結
- [x] `npm run test:e2e`で全テストが成功
- [x] `playwright.config.ts`のbaseURL・webServer設定が整備済み（既存設定を確認、変更なし）

## 次のステップ

- TASK-0034の残作業（モバイルレイアウト・コントラスト比）が未完了のまま残っているため、別途対応すること
- TASK-0017/0019/0020/0021（ItemDetailPageへのGroupSection/RelationsSection/LinksFilesSection統合）も未完了であり、別途対応が必要
