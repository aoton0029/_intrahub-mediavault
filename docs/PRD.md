# MediaVault
## 概要
映画・アニメ・漫画・小説・ドラマ・ゲーム・論文/文献・書籍などのメタデータを一元管理するセルフホスト型アプリケーション。
ユーザーがコレクションを整理・カスタマイズできるようにする。
api-client-libを利用して外部APIからデータを取得する。

| ドメイン | 対象 | 責務 |
|----------|------|------|
| **アニメ管理** | アニメ | メタデータ収集・API連携・CRUD・整理・カスタマイズ |
| **映画管理** | 映画 | メタデータ収集・API連携・CRUD・整理・カスタマイズ |
| **ドラマ管理** | ドラマ | メタデータ収集・API連携・CRUD・整理・カスタマイズ |
| **漫画管理** | 漫画 | メタデータ収集・API連携・CRUD・整理・カスタマイズ |
| **小説管理** | 小説 | メタデータ収集・API連携・CRUD・整理・カスタマイズ |
| **ゲーム管理** | ゲーム | メタデータ収集・API連携・CRUD・整理・カスタマイズ |
| **学術専門書管理** | 学術書・専門書 | メタデータ収集・API連携・CRUD・整理・カスタマイズ |
| **文献管理** | 論文・文献 | 学術資料の管理。メタデータ収集・API連携・CRUD・整理・カスタマイズ |

## 技術スタック
| 要素 | 技術 |
|------|------|
| フロントエンド | React + TypeScript |
| バックエンド | Rust (Actix-web) |
| データベース | DBサーバーコンテナ |
| APIクライアント | Rust (api-client-lib) |
| デプロイ | Docker |


## やりたいこと

**共通機能**
- 外部APIで作品を検索してコレクションに追加できる
- APIを使わず手動で作品を追加できる
- タグ・カテゴリ・お気に入りでコレクションを整理できる
- 作品を任意の名前のマイリストに追加して管理できる
- 他メディアのアイテムを引用・関連付けできる
- URLのリンクリストを管理できる
- ファイルのパスリストを管理できる
- トレーラーのURLリストを管理できる
- 読了/観賞日を記録できる
- スタッフ（監督・声優・著者など）のリストを管理できる（外部APIのID・役割・名前など）

**アニメ管理**
- シーズンのナンバリングをグループ単位でまとめて管理できる
- シーズン内のエピソード（話数）ごとの情報を管理できる


**映画管理**
- 章などのナンバリングをグループ単位でまとめて管理できる


**ドラマ管理**
- シーズンのナンバリングをグループ単位でまとめて管理できる
- シーズン内のエピソード（話数）ごとの情報を管理できる


**漫画管理**
- 巻のナンバリングをグループ単位でまとめて管理できる


**小説管理**
- 巻のナンバリングをグループ単位でまとめて管理できる


**ゲーム管理**
- DLC・拡張パックを本体作品に関連付けて管理できる（item_relations: relation_type=dlc）

**学術専門書管理**
- 学術書・専門書メタデータを一元管理する
- 外部APIで作品を検索してコレクションに追加できる
- APIを使わず手動で作品を追加できる

**文献管理**
- 学術資料特有のフィールド（DOI・掲載誌名・巻/号・掲載ページなど）をサポートする
- 外部APIで作品を検索してコレクションに追加できる
- APIを使わず手動で作品を追加できる

**設定**
- エクスポート
    - Obsidian
    - Notion
- インポート
    - ブクログ(csv)
    - Steam(ゲームライブラリ)
- APIキー管理


## やらなくていいこと
- ユーザー管理機能（単一ユーザー前提で運用し、認証・ログイン機能も持たない）

## 詳細
- バックエンドの詳細草案（機能要件・内部REST API・データベース・api-client-lib）は[backend/docs/PRD.md](../backend/docs/PRD.md)を参照。
- フロントエンドの詳細草案（UI機能要件・画面構成）は[frontend/docs/PRD.md](../frontend/docs/PRD.md)を参照。

- 映画・アニメ・漫画・小説・ドラマ・ゲーム・論文/文献・書籍等のメタデータを一元管理するセルフホスト型アプリケーション。
- コンテナは`selfhosted-net`・`db-net`([PostgreSQL](../PostgreSQL/README.md)利用)・`ai-net`([RAG-Service](../RAG-Service/README.md)の`POST /ingest`呼び出し用)に参加する。Caddy経由(`app.home.lan`)で公開するため`proxy-net`にも参加し、ホスト直接ポート公開は行わない。
- アップロードファイルはコンテナ内ではなくファイルサーバー用HDDへ直接保存する（バインドマウント）。
  - PDF: `/srv/files/pdf`（[Calibre-Web](../Calibre-Web/README.md)のライブラリパスと共用）。アップロードされたPDFはCalibre-Webからも自動でライブラリ認識され、MediaVault側の作品詳細にCalibre-Webの閲覧URL(`calibre.home.lan`)へのリンクを保持して直接遷移できるようにする。
  - 画像: `/srv/media/photos`（Jellyfin/Sambaの`photos`共有と同一パスを共用）。
  - OCRテキスト: `/srv/files/ocr`（[RAG-Service](../RAG-Service/README.md)が`/ingest`処理時に書き込み、レスポンスで受け取った`ocr_text_path`をPostgreSQLに保存し、作品詳細から参照する）。

```yaml
# selfhosted/docker-compose.yml (例)
services:
  mediavault:
    image: <未定>
    networks:
      - selfhosted-net
      - db-net      # PostgreSQL利用
      - ai-net      # RAG-Service呼び出し(PDF全文ベクトル化トリガー)
      - proxy-net   # Caddy経由で公開
    volumes:
      - /srv/files/pdf:/data/pdf      # PDFアップロード保存先（Calibre-Webと共用）
      - /srv/media/photos:/data/photos  # 画像アップロード保存先（Samba/Jellyfin photosと共用）
      - /srv/files/ocr:/data/ocr:ro     # OCRテキスト参照用（RAG-Serviceが書き込み、本サービスは読み取り専用）
    # ports: ホスト直接公開はしない方針。Caddy経由(app.home.lan)とする
```

