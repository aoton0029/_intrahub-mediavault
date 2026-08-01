# MediaVault 基本設計 — 概要

← [../PRD.md](../PRD.md)

## この文書群について

本 `basic-design/` は、MediaVault のアプリケーションアーキテクチャ（コンポーネント構成・データモデル・API設計・ジョブ/エージェント連携）を横断的に整理した基本設計文書群である。

対象範囲は **アプリケーション設計のみ**。コンテナ構成・ネットワーク分離・デプロイ方式・環境変数などインフラ寄りの詳細は、インフラ設計側の以下のドキュメントを正とし、本文書群では参照リンクに留める。

- `インフラ設計/デバイス/ミニPC/サービス/MediaVault/README.md`
- `インフラ設計/デバイス/ミニPC/サービス/MediaVault/設計.md`
- `インフラ設計/デバイス/ミニPC/サービス/MediaVault/Jellyfin/README.md`
- `インフラ設計/デバイス/ミニPC/サービス/MediaVault/Calibre-Web/README.md`

## 目的・全体像

MediaVault は、メディア・書誌・ファイル・ナレッジを1つのアプリで一元管理する自己完結型セルフホストアプリである。データ・ファイルの登録・管理・閲覧を担い、動画/写真の視聴は Jellyfin、書籍/PDF/漫画の閲覧は Calibre-Web に委譲する。

コンポーネント構成:

| コンポーネント | 対応docs | 役割 |
|---|---|---|
| MediaVault-web | [../frontend/](../frontend/PRD.md) | 一覧/検索/詳細/登録/編集を担うSPA。ビューア機能は持たず、Jellyfin/Calibre-Webへリンクアウトする |
| MediaVault-api | [../backend/](../backend/PRD.md) | メタデータ・検索・ファイル・ジョブ登録・ナレッジを扱う単一 `/api`。全データ変更の唯一の経路 |
| MediaVault-worker | [../backend/mediavault-worker/PRD.md](../backend/mediavault-worker/PRD.md) | `jobs` テーブルをポーリングし、パイプラインジョブ/エージェント駆動ジョブを実行 |
| MediaVault-mcp | [../backend/mediavault-mcp/PRD.md](../backend/mediavault-mcp/PRD.md) | AIエージェント向けの薄いMCPアダプタ。`MediaVault-api` の `/api` のみを呼び出す |
| Jellyfin / Calibre-Web | （インフラ設計側docsを参照） | バンドルされたOSSビューア。読み取り専用で `/data` を参照 |

詳細なコンポーネント責務・依存関係は [01_architecture.md](01_architecture.md) を参照。

## 設計原則

インフラ設計側 `設計.md` に記載の7原則を、アプリケーション設計の観点で再整理する。

1. **単一の真実source**: PostgreSQL を `MediaVault-api` 経由でのみ更新する。Jellyfin/Calibre-Webはメタデータを所有しない読み取り専用の下流表示に過ぎない。
2. **`item` を正準キーとする**: 1つの `item` がファイル・外部リンク・関連ナレッジを束ねる（[02_data-model.md](02_data-model.md)）。
3. **視聴機能を作り直さない**: 動画はJellyfin、書籍/PDFはCalibre-Webへブラウザ遷移で委譲する。`MediaVault-web` はビューアを持たない。
4. **共有面を限定**: DBと読み取り専用の`/srv/anime`・`/srv/live-action`・`/srv/manga`だけを共有する。
5. **部分的縮退運転**: workerが停止してもメタデータ層（一覧/検索/詳細）は動作し続ける。
6. **書き込み経路の一本化・生成ロジックはエージェントの責務**: webもmcpも書き込みは必ず `MediaVault-api` を経由する。要約/wiki/embeddingの「どう生成するか」はアプリ内実装ではなく、KnowledgeHub側エージェントにmcp経由で委譲する（[04_jobs-and-agent-integration.md](04_jobs-and-agent-integration.md)）。
7. **公開面の一本化**: 個々のコンテナはポートを公開しない。外部公開はインフラ設計側のリバースプロキシ（Caddy）が担う — 詳細はインフラ設計側ドキュメントを参照。

## 関連ドキュメント

- [PRD.md（ルート）](../PRD.md)
- [backend/PRD.md](../backend/PRD.md) / [backend/mediavault-api/index.md](../backend/mediavault-api/index.md)（API詳細リファレンス）
- [frontend/PRD.md](../frontend/PRD.md)
- [mcp/PRD.md](../backend/mediavault-mcp/PRD.md)
- [worker/PRD.md](../backend/mediavault-worker/PRD.md)
