---
name: mediavault-api-reference
description: MediaVaultのバックエンドAPI仕様書(docs/backend/mediavault-api/)を、API呼び出し・追加・変更に関わる設計や実装作業の際に必ず参照する。フロントエンドのデータ取得処理・APIクライアント関数・フォーム送信処理、バックエンドのルートハンドラ・DTO/レスポンス型など、item・tag・category・mylist・staff・item配下のfiles/episodes/groups/links/relations/trailers・import・settings・health・internal APIのいずれかに関わる作業であれば、ユーザーが具体的なエンドポイント名を出さず「保存ボタンをつけて」「一覧を読み込んで」「このフォームをAPIに繋いで」「実装して」のように言った場合でも発火する。URLパス・リクエスト/レスポンス形式・エラーコードは必ずこのドキュメントを正とし、記憶や無関係なコードからの推測で実装しない。
---

# MediaVault API 仕様書リファレンス

MediaVaultのバックエンドAPIは `docs/backend/mediavault-api/` 配下に、共通仕様をまとめた
`index.md` と、リソースカテゴリごとのAPI仕様書mdファイルという構成でドキュメント化されている。
これらはRust/Axumバックエンドの実装元となる正のドキュメントであり、記憶や別箇所のコードからの
類推で実装すると、実行時にしか気づけない形式のズレが生じる。設計・実装の際は必ずこれらを参照する。

## 作業手順

1. **まず `docs/backend/mediavault-api/index.md` を読む。**
   ここには全エンドポイント共通の規約が定義されている:
   - レスポンス形式（`ApiOk<T>` / `PaginatedOk<T>` / `ApiError` の各エンベロープ）
   - エラーコード一覧（コードごとのHTTPステータス）
   - 認証ルール（`/api/v1/*` は認証なし、`/internal/*` は `api_key_auth` ミドルウェアで
     `INTERNAL_API_KEY` が必須）
   - ベースURL

   これを飛ばすと、レスポンスの取り出し方やエラーハンドリングを誤りやすい。

2. **タスクに関係するカテゴリ別ファイルを特定して読む。** 同ディレクトリ内の以下のファイルから
   タスク内容に合うものを選ぶ:

   | ファイル | 内容 |
   |---|---|
   | `items.md` | アイテムのCRUD・検索・インポート（`/items`, `/items/search`, `/items/import`, `/items/{id}`） |
   | `item-episodes.md` | アイテム配下のエピソード |
   | `item-files.md` | アイテムの添付ファイル |
   | `item-groups.md` | アイテムのグルーピング |
   | `item-links.md` | アイテムに紐づく外部リンク |
   | `item-relations.md` | アイテム間の関連 |
   | `item-trailers.md` | アイテム配下のトレーラー |
   | `tags.md` | タグのCRUD |
   | `categories.md` | カテゴリのCRUD |
   | `mylists.md` | マイリスト機能 |
   | `staff.md` | スタッフ/キャスト |
   | `import.md` | 一括・外部インポート処理 |
   | `settings.md` | アプリ設定 |
   | `health.md` | ヘルスチェック |
   | `internal-api.md` | `/internal/*`（内部APIキーが必須） |

   1つのタスクが複数ファイルにまたがることも多い（例: アイテム詳細画面なら `items.md` に加えて
   `item-files.md` や `item-trailers.md` も必要）。関係するものは全て読むこと。

3. **読んだ内容をそのまま契約として扱う。** フロントエンドのAPIクライアント・フォーム処理や、
   バックエンドのルート/ハンドラ実装を書く際は:
   - クエリパラメータ・リクエストボディの項目名と必須/任意を仕様書通りに一致させる
   - レスポンスは `index.md` のエンベロープ形式（`data`、ページネーションありなら `pagination`）
     通りに取り出す。推測した形式で書かない
   - エラーハンドリングは仕様書に記載された `code` の値を使う。存在しないコードを作らない
   - 必要なエンドポイントやフィールドが仕様書に無い場合は、勝手に作らず、ユーザーに確認するか
     「バックエンド側の追加実装とドキュメント更新が必要」と明示する

4. **エンドポイントを追加・変更した場合**、対応するカテゴリ別ファイル（共通のエラーコードや
   規約が増える場合は `index.md` も）を更新し、ドキュメントを正の情報源として保つ。
