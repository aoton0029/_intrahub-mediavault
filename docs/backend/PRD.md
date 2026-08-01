# MediaVault Backend

## 概要
MediaVaultのバックエンド部分のPRD。映画・アニメ・漫画・小説・ドラマ・ゲーム・論文/文献・書籍などのメタデータを一元管理するセルフホスト型アプリケーションのうち、API提供・外部API連携・データベース管理を担う。
全体構想は[ルートPRD](../PRD.md)を参照。フロントエンド側は[docs/frontend/PRD.md](../frontend/PRD.md)を参照。

## 技術スタック
| 要素 | 技術 |
|------|------|
| バックエンド | Rust (Axum) |
| データベース | DBサーバーコンテナ（Postgres） |
| APIクライアント | Rust (api-client-lib) |
| デプロイ | Docker |

## 機能要件

### 共通機能（API）
| 機能 | 概要 |
|---|---|
| 作品検索・追加（API） | 外部APIをタイトル等で検索し、結果からコレクションに追加する |
| 作品手動追加 | API検索に依存せず、フォーム入力でアイテムを新規作成する |
| 作品編集・削除 | 既存アイテムの全項目を編集・削除する |
| 一覧・絞り込み | media_type・タグ・カテゴリ・お気に入り・status等でコレクションを一覧・絞り込みする |
| タグ/カテゴリ管理 | タグ・カテゴリの作成・付与・削除を行う |
| マイリスト管理 | 任意名称のリストを作成し、アイテムを追加・削除する |
| 関連付け | 他メディアのアイテムを引用・関連として紐付ける（item_relations） |
| リンク/ファイル/トレーラー管理 | item_links・item_files・item_trailersの追加・編集・削除 |
| 視聴・読了記録 | statusとconsumed_dateを更新する |
| スタッフ管理 | スタッフの追加・役割付与・作品への紐付け |

### メディア別機能（API）
| メディア | 機能 |
|---|---|
| アニメ・ドラマ | シーズン（item_groups）単位での話数（item_episodes）の登録・編集・一覧表示 |
| 映画 | 章単位のグループ管理（任意） |
| 漫画・小説 | 巻単位のグループ管理（item_groups, group_type=volume） |
| ゲーム | DLC/拡張パックを本体作品に紐付け（item_relations, relation_type=dlc） |
| 学術書・専門書 | 学術書特有の属性（著者・出版社・ISBN）でのCRUD |
| 論文・文献 | DOI・掲載誌・巻号・ページ等の学術メタデータでのCRUD |

### 外部連携・インポート/エクスポート
| 機能 | 概要 |
|---|---|
| 外部API検索連携 | メディア種別に応じたAPI（Jikan/TMDb/NDL等）を呼び分けて検索結果を取得する |
| APIキー管理 | 各外部APIのキーを登録・更新するAPIを提供する |
| インポート | ブクログ(csv)・Steamライブラリからアイテムを一括取込する |
| エクスポート | コレクションをObsidian/Notion向け形式で出力する |
| 内部REST API | 外部ツール（巡回バッチ・ファイルサーバー監視）からのアイテム登録・更新・検索・ファイル紐付け |

## 外部から登録・更新・検索をするためのAPI
他のスクリプト・ツール（巡回取得バッチ、ファイルサーバー監視プロセスなど）からMediaVaultのデータを操作するための内部向けREST API。単一ユーザー前提のためAPIキー1本での簡易認証とする。

**メタデータ登録・更新・検索**
- アイテムの新規登録（手動 or 外部API取得結果を渡してインポート）
- 既存アイテムのメタデータ更新（部分更新）
- アイテム検索（タイトル・media_type・タグ・外部IDなどで条件検索）
- グループ/エピソード（シーズン・巻・話数など）の登録・更新

**ファイルアップロード**
- `item_files`へのファイル登録（ファイルサーバー上のパスを紐付け、file_type・labelを指定）
- ファイル本体のアップロード自体は本APIでは扱わず、ファイルサーバー側に配置済みのパスを登録する想定（直接バイナリを送ってサーバー側で配置するエンドポイントも検討）
- PDFの場合はCalibre-Web側の取り込み完了後、`calibre_book_id`を紐付け更新できるようにする

## やらなくていいこと
- ユーザー管理機能（単一ユーザー前提で運用し、認証・ログイン機能も持たない）

---

# api-client-lib
Cargoのワークスペース機能を使うことで、MediaVaultとapi-client-libを同一リポジトリ内で管理し、両者の開発を効率化。
## 概要
APIクライアント・データモデル・ユーティリティ関数などを提供するライブラリ。

## 技術スタック
| 要素 | 技術 |
|------|------|
| APIクライアント | Rust |
| データモデル | Rust構造体 |

## 外部API

| メディア種別 | API |
|------------|-----|
| アニメ | Jikan |
| 映画 | TMDb |
| ドラマ | TMDb |
| 漫画 | Jikan |
| 小説 | OpenLibrary / Google Books |
| ゲーム | Steam / IGDB |
| 論文 | NDL |
| 学術書・専門書 | NDL / Google Books |

---

# データベース
- Dockerで立てたPostgresに保存する

## データモデル（共通項目 / メディア独自項目）
正規化構成：共通`items`テーブル + メディア種別ごとの詳細テーブル（`anime_details`など）をJOINする。
グループ単位管理（シーズン/巻/章/DLC等）は共通の抽象モデル`item_groups`として全メディアで共有する。

### 共通項目（`items`テーブル）
| フィールド | 型 | 説明 |
|---|---|---|
| id | UUID | 一意識別子 |
| media_type | enum | anime / movie / drama / manga / novel / game / academic_book / paper |
| title | string | タイトル |
| original_title | string? | 原題 |
| description | text? | 概要・あらすじ |
| cover_image_url | string? | カバー画像 |
| release_date | date? | 公開・発売・出版日 |
| homepage_url | ホームページのURL |
| status | enum | 視聴中/読了/未着手などユーザー管理ステータス |
| consumed_date | date? | 読了/観賞日 |
| rating | float? | ユーザーが設定する評価(APIから取得した評価ではない) |
| is_favorite | bool | お気に入り |
| source | enum | api / manual（外部API取得 or 手動追加） |
| external_id | string? | 外部API側のID（Jikan/TMDb/NDLなど） |
| created_at / updated_at | datetime | 作成・更新日時 |

共通の関連エンティティ：

| エンティティ | 用途 |
|---|---|
| `tags` / `item_tags` | タグ付け（多対多） |
| `categories` / `item_categories` | カテゴリ分類 |
| `mylists` / `mylist_items` | 任意名称のマイリスト |
| `item_relations` | 他メディアアイテムへの引用・関連付け（item_id ⇔ related_item_id, relation_type）。relation_typeで関係の種類を区別する（例：reference=単純な引用・関連、dlc=本体作品へのDLC/拡張パック紐付け） |
| `item_links` | 配信サイト等へのURLリンクリスト（item_id, url, label）。labelは配信サイト名（例：Amazon, Netflix, Disney+, Youtube, DMM TV等）を想定 |
| `item_files` | ファイルリスト（item_id, path, label, file_type）。アップロードは`/srv/mediavault`配下へ保存し、アニメ・実写・マンガの既存ファイルは絶対パスでリンクする。 |
| `item_trailers` | トレーラーURLリスト（item_id, url, label） |
| `item_groups` | 作品内の入れ子構造を持つグループ単位管理（シーズン/巻/章の汎用モデル。DLC等の作品間関係は`item_relations`で表現するため対象外）：group_name, group_type(season/volume/chapter), order（表示順）, number（巻数・章数など対外的な番号）, parent_item_id |
| `item_episodes` | エピソード（話数）単位の情報。season/chapter配下のみで使用し、volume（巻）には使用しない：group_id(item_groupsへの外部キー), episode_number, title, original_title?, air_date?, description? |
| `staff` / `item_staff` | スタッフ管理（多対多）：`staff`はexternal_id（外部APIのスタッフID）, name, image_url等を保持。`item_staff`はitem_id, staff_id, role（監督/声優/著者/イラストレーター等）, character_name?（声優の場合の役名）を保持 |

### メディア独自項目（詳細テーブル）
各詳細テーブルは`item_id`で`items`に1:1で紐づく。

**`anime_details`**: episode_count, season_count, studio, genre_list, source_type(原作種別), jikan_id

**`movie_details`**: runtime_minutes, director, genre_list, tmdb_id

**`drama_details`**: episode_count, season_count, network, genre_list, tmdb_id

**`manga_details`**: volume_count, chapter_count, author, illustrator, magazine, jikan_id

**`novel_details`**: volume_count, author, publisher, isbn, openlibrary_id / google_books_id

**`game_details`**: platform_list, developer, publisher, steam_appid, igdb_id（DLC/拡張パックの本体作品への紐付けは`item_relations`の relation_type=dlc で表現する）

**`academic_book_details`**: author, publisher, isbn, ndl_id / google_books_id

**`paper_details`**（学術資料特有フィールド）: doi, journal_name（掲載誌名）, volume_issue（巻/号）, page_range（掲載ページ）, author_list, ndl_id
