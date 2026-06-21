# mediavault-backend 準備タスク（ユーザー作業）

> **仕様**: [requirements.md](requirements.md)
> **生成日**: 2026-06-21

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・ヒアリングで明確に必要と判明したタスク
- 🟡 **黄信号**: 要件定義書から妥当に推測されるタスク

## 必須（実装開始前に完了が必要）

- [ ] **PostgreSQL用Dockerコンテナの準備** 🔵 *tech-stack.md「開発環境」より*
  - `docker-compose.yml`に未作成のためdocker-compose.ymlとPostgresコンテナ定義を用意する（実装フェーズで作成予定だが、Docker自体のインストールはユーザー側作業）
  - 関連要件: REQ-405, NFR-201

- [ ] **ファイルサーバー用ディレクトリの準備** 🔵 *backend/docs/tech-stack.md「ファイルストレージ」より*
  - `/srv/files/pdf`（PDF用）・`/srv/media/photos`（画像用）のバインドマウント先ディレクトリを開発・本番環境に用意する
  - 関連要件: REQ-019, REQ-104, REQ-402

- [ ] **内部REST API用APIキーの発行** 🔵 *tech-stack.md「内部REST API認証」より*
  - 巡回バッチ・ファイルサーバー監視プロセスから使う固定APIキー文字列を決定し、`.env`に設定する
  - 関連要件: REQ-018, REQ-403, REQ-404

## 推奨（実装中に用意できればOK）

- [ ] **TMDb APIキーの取得** 🔵 *api-client-lib/tmdb.rsより必要と判明*
  - https://www.themoviedb.org/ でアカウント作成し、APIキー（v3 auth）を取得
  - 必要になるフェーズ: 外部API検索連携（映画・ドラマ）実装時
  - 関連要件: REQ-002, REQ-015

- [ ] **IGDB APIキー（Twitch Developer連携）の取得** 🔵 *api-client-lib/igdb.rsより必要と判明*
  - IGDBはTwitch Developer ConsoleでClient ID/Client Secretを発行する必要がある
  - 必要になるフェーズ: ゲーム外部API検索連携実装時
  - 関連要件: REQ-002, REQ-015

- [ ] **NDL（国立国会図書館サーチ）API利用確認** 🟡 *PRD外部API一覧より*
  - NDLサーチAPIはキー不要だが、利用規約・アクセス制限（レート等）を確認しておく
  - 必要になるフェーズ: 論文・学術書の外部API検索実装時
  - 関連要件: REQ-002, REQ-014

- [ ] **Steam Web APIキー・SteamIDの取得** 🔵 *ヒアリングQ3でSteamインポートを今回スコープに含めると確認*
  - https://steamcommunity.com/dev/apikey でAPIキーを取得し、対象アカウントのSteamID64を確認する（プロフィールを「公開」設定にする必要あり）
  - 関連要件: REQ-017

- [ ] **ブクログCSVのサンプルファイル準備** 🟡 *ヒアリングQ3より、実カラム形式の確認が必要*
  - ブクログの「データ管理」→「エクスポート」からCSVを取得し、実装時のカラム形式確認用に共有する
  - 関連要件: REQ-016, EDGE-002

- [ ] **Calibre-Webの構築・連携設定** 🟡 *PRDのPDF管理機能より妥当な推測*
  - Calibre-Webを別途セルフホストし、`/srv/files/pdf`を共有マウントする構成を準備する
  - 必要になるフェーズ: REQ-020/REQ-103実装時
  - 関連要件: REQ-020, REQ-103

## 確認事項（判断が必要）

- [ ] **外部APIキーの暗号化要否** 🟡 *acceptance-criteria.md TC-015-01付近の前提*
  - 現要件ではDBへの平文保存を前提としているが、セルフホスト環境のセキュリティ方針として暗号化（例: AES-GCM＋マスターキー）が必要か判断してほしい
  - 関連要件: REQ-015, NFR-202

- [ ] **Obsidian/Notionエクスポート機能の実施時期** 🔵 *ヒアリングQ3で今回スコープ外と確認済み*
  - 次フェーズでの実施予定。具体的な着手時期が決まれば共有してほしい
  - 関連要件: REQ-301

---

## サマリー

| 優先度 | 件数 | 🔵 | 🟡 | 🔴 |
|--------|------|-----|-----|-----|
| 必須 | 3 | 3 | 0 | 0 |
| 推奨 | 6 | 4 | 2 | 0 |
| 確認事項 | 2 | 1 | 1 | 0 |

## 関連文書

- **要件定義書**: [requirements.md](requirements.md)
- **ヒアリング記録**: [interview-record.md](interview-record.md)
