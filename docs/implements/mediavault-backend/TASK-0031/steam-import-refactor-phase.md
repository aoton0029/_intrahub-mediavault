# TASK-0031 Steamライブラリインポート機能 TDD Refactorフェーズ

**機能名**: Steamライブラリインポート機能（`POST /import/steam`）
**タスクID**: TASK-0031
**要件名**: mediavault-backend
**作成日**: 2026-06-27

---

## 1. リファクタリング方針

Greenフェーズの実装者コメント（「`import_steam_library`関数の責務分割の余地がある」）に従い、
**挙動を一切変えない**前提で `backend/mediavault-api/src/import/steam_import.rs` の
`import_steam_library` を責務ごとに3つの関数へ分割した。TASK-0030（`booklog_csv.rs`）・
TASK-0023（`external_search.rs`）のコード規約（小さなヘルパー関数への分割、日本語コメント
【機能概要】【実装方針】【テスト対応】【信頼性レベル】の付与）との一貫性を確認し、同様の
コメント構造を適用した。

### 分割後の構成

| 関数名 | 責務 | 信頼性レベル |
|---|---|---|
| `build_steam_client` | APIキー文字列＋任意のベースURLからSteamClientを構築する | 🟡（TASK-0023踏襲） |
| `fetch_owned_games` | APIキー解決→クライアント構築→`get_owned_games`呼び出し | 🔵🟡 |
| `register_single_game` | 1件分の重複チェック・変換・DB登録・`ImportSummary`反映 | 🟡 |
| `import_steam_library` | 上記3関数を順序立てて呼び出すオーケストレーションのみ | 🔵 |

### 改善前後の比較

- **改善前**: `import_steam_library`が「steam_id検証」「APIキー取得」「クライアント構築」
  「Steam API呼び出し」「ループ内重複チェック・変換・DB登録・集計」をすべて1関数（約75行）に
  集約していた。
- **改善後**: 「Steam API呼び出しまで」を`fetch_owned_games`、「1件分の登録処理」を
  `register_single_game`、「クライアント構築」を`build_steam_client`へ分離。
  `import_steam_library`本体は「steam_id検証 → ゲーム一覧取得 → ループで1件ずつ登録」という
  4行の骨格のみになり、各責務を独立して読み・テストできる構成へ整理した。

**機能的な変更は一切行っていない**: 分岐条件・エラーコード・カウント方法・重複判定・
フォールバックタイトル生成ロジックはGreenフェーズと完全に同一。関数境界のみを変更した
（純粋なExtract Methodリファクタリング）。

---

## 2. セキュリティレビュー

- **APIキー漏洩防止**: `fetch_owned_games`内で取得した`api_key`はログ出力されない
  （`tracing::error!`は`err:?}`のみ出力し、APIキー自体を含まない）。Greenフェーズから変更なし。
- **入力検証**: `validate_steam_id`は引き続き外部API呼び出し前に実行される
  （関数分割後も呼び出し順序は不変）。
- **エラーメッセージ**: `STEAM_API_KEY_INVALID`・`EXTERNAL_API_TIMEOUT`のメッセージは
  内部実装詳細（DB接続文字列・スタックトレース等）を含まない。
- **SQLインジェクション**: `find_existing_import`・`create_item_with_source`はsqlxの
  パラメータバインディングを使用（既存実装を変更していないため対象外）。
- **重大な脆弱性は発見されなかった。**

---

## 3. パフォーマンスレビュー

- **計算量**: 変更なし。`O(n)`（nは所持ゲーム数）。1件ごとに独立した非同期DB呼び出し
  （重複チェック1回＋登録1回）を行う構成はGreenフェーズと同一。
- **メモリ使用量**: `fetch_owned_games`が`Vec<SteamGameEntry>`を返す点はGreenフェーズの
  `response.model.games`を直接イテレートしていた構成と同等（所有権の移動のみで追加コピーなし）。
- **ボトルネック**: 関数分割によるオーバーヘッドは無視できる（Rustの関数呼び出しはinline化
  対象になり得るうえ、ボトルネックは外部API呼び出し・DB I/Oであり関数境界ではない）。
- **重大な性能課題は発見されなかった。**

---

## 4. テスト実行結果

### ビルド

```
cargo build -p mediavault-api
```
→ **成功**（既存warning7件のみ、新規warningなし、エラーなし）

### Steam関連テスト

```
cargo test -p mediavault-api steam
```

結果（リファクタ前後で完全一致）:
- **15 passed**: 全DB非依存ユニットテストが成功
- **6 ignored**: 実DB必要な統合テスト（Docker未起動のため未実行、想定通り）
- **0 failed**

### 全体テスト

```
cargo test -p mediavault-api
```
→ **198 passed / 0 failed / 192 ignored**（Greenフェーズ時点の既知の不安定テスト
`services::file_storage`もこの実行では成功。本リファクタリングと無関係の環境依存事象）

---

## 5. 品質判定

```
✅ 高品質:
- テスト結果: Steam関連15件・全体198件が継続成功（リファクタ前後で結果完全一致）
- セキュリティ: 重大な脆弱性なし
- パフォーマンス: 重大な性能課題なし（計算量・メモリ使用量に変化なし）
- リファクタ目標: import_steam_library の責務分割を達成
  （build_steam_client / fetch_owned_games / register_single_game に分離）
- コード品質: 単一責任原則に基づく分割、TASK-0030/0023の日本語コメント規約に準拠
- ファイルサイズ: steam_import.rs 807行（うちテストコードが約520行、本体実装は約290行で
  500行制限内）
- 日本語コメント: 【ヘルパー関数】【単一責任】【再利用性】【改善内容】等のテンプレートに
  従って各関数に付与済み
- ドキュメント: 本ファイル・メモファイルともに更新完了
```

---

## 6. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-verify-complete` で完全性検証を実行します。
