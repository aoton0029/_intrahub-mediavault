# backend/models 共通項目・ドメイン項目 分析

対象: `backend/mediavault-api/src/models/`, `backend/api-client-lib/src/clients/`, `backend/mediavault-api/migrations/`, `docs/api-samples/`

## 1. DBスキーマ

出典: `backend/mediavault-api/migrations/20260623000001_init_schema.up.sql`

全メディア種別を単一の `items` テーブルで表現するpolymorphicモデル。`media_type` 列が種別を判別し、種別固有情報は `details JSONB` に格納する(別テーブル方式ではない)。

### Enum型

- `media_type`: anime, movie, drama, manga, novel, game, academic_book, paper
- `item_status`: not_started, in_progress, completed
- `item_source`: api, manual
- `group_type`: season, volume, chapter
- `relation_type`: reference, dlc
- `file_type`: pdf, image, other
- `api_provider`: tmdb, igdb, ndl, steam, openlibrary, anilist

### `items` (コアテーブル)

| 列 | 型 | 備考 |
|---|---|---|
| id | UUID PK | `gen_random_uuid()` |
| media_type | media_type NOT NULL | 判別列 |
| title | VARCHAR(500) NOT NULL | |
| original_title | VARCHAR(500) | |
| description | TEXT | |
| cover_image_url | VARCHAR(1000) | |
| release_date | DATE | |
| homepage_url | VARCHAR(1000) | |
| status | item_status NOT NULL DEFAULT 'not_started' | 視聴/読了ステータス |
| consumed_date | DATE | |
| rating | REAL | |
| is_favorite | BOOLEAN NOT NULL DEFAULT FALSE | |
| source | item_source NOT NULL | |
| external_id | VARCHAR(255) | |
| details | JSONB | 種別固有データ(キー集合はmedia_typeに依存) |
| created_at / updated_at | TIMESTAMP | updated_atはトリガーで自動更新 |

CHECK制約: `source='manual' OR (source='api' AND external_id IS NOT NULL)`、`title <> ''`。インデックス: media_type, status, is_favorite, external_id。

### 関連テーブル(すべて`items`参照、種別非依存で共通)

- `tags` / `item_tags` (多対多)
- `categories` / `item_categories` (多対多)
- `mylists` / `mylist_items` (多対多)
- `item_relations`: item_id, related_item_id, relation_type(reference/dlc)
- `item_links`: item_id, url, label
- `item_files`: item_id, path, label, file_type, calibre_book_id
- `item_trailers`: item_id, url, label
- `item_groups`: item_id, parent_item_id, group_type(season/volume/chapter), group_name, number, display_order — シーズン/巻/章の階層構造
- `item_episodes`: group_id, episode_number, title, original_title, air_date, description — `group_type='volume'`には紐付け不可(トリガーで制約)
- `staff` / `item_staff`: role, character_name
- `api_credentials`: provider(PK), api_key

**結論**: DB層はすでに「共通項目1テーブル + JSONBによる種別固有拡張」という設計で一本化されている。問題はRustアプリケーション層(モデル定義・外部APIマッピング)側にある。

## 2. Rustモデル構造

出典: `backend/mediavault-api/src/models/item.rs`, `models/domain/*.rs`

共通項目を表現する仕組みが**2系統並存**している。

### System 1: `Item` (DB行モデル、`item.rs:46-64`)

```
id, media_type, title, original_title, description, cover_image_url,
release_date: Option<NaiveDate>, homepage_url, status, consumed_date,
rating: Option<f32>, is_favorite, source, external_id: Option<String>,
created_at, updated_at
```
`#[derive(sqlx::FromRow)]` — DBテーブルと1:1対応。種別固有フィールドは一切持たず、`details: serde_json::Value` に逃がす(`ItemDetail.detail`として別途露出)。

### System 2: `MediaCore` + `*Details` (外部API正規化モデル、`domain/core.rs:21-45`)

```
media_type, provider: Option<ApiProvider>, external_id: String, title,
original_title, alternative_titles: Vec<String>, description,
release_date: Option<String>, image_url, genres: Vec<String>,
rating: Option<f64>, url
```

`#[serde(flatten)] core: MediaCore` を6つのDetails構造体が持つ:

| 構造体 | 固有フィールド |
|---|---|
| `AnimeDetails` | episodes, status, season, year, studios, source, duration, trailer_url |
| `MangaDetails` | chapters, volumes, status, authors, serializations |
| `MovieDetails` | runtime_minutes, original_language, vote_count, collection, production_companies |
| `DramaDetails` | number_of_seasons, number_of_episodes, networks, status, original_language, first_air_date, last_air_date |
| `GameDetails` | platforms, developers, publishers, screenshots, metacritic, steam_appid, storyline |
| `NovelDetails` (AcademicBook/Paper共用) | authors, publisher, isbn, page_count, physical_format |

`MediaDetails` enumが8種別(AcademicBook/PaperはNovelDetailsを再利用)を束ね、`core()`アクセサで共通部分に手動ディスパッチする(match文、traitなし)。

### 共通項目マトリクス(MediaCore基準)

| フィールド | Core共通 | Anime | Manga | Movie | Drama | Game | Novel |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| media_type/provider/external_id/title/original_title/alternative_titles/description/release_date/image_url/genres/rating/url | ✓ | | | | | | |
| episodes | | ✓ | | | | | |
| status | | ✓ | ✓ | | ✓ | | |
| season/year/studios/source/duration/trailer_url | | ✓ | | | | | |
| chapters/volumes/serializations | | | ✓ | | | | |
| authors | | | ✓ | | | | ✓ |
| runtime_minutes/original_language/vote_count/collection/production_companies | | | | ✓ | (original_language共通) | | |
| number_of_seasons/number_of_episodes/networks/first_air_date/last_air_date | | | | | ✓ | | |
| platforms/developers/publishers/screenshots/metacritic/steam_appid/storyline | | | | | | ✓ | |
| publisher/isbn/page_count/physical_format | | | | | | | ✓ |

### `Item` ⇔ `MediaCore` のフィールド不一致

| 意味 | Item | MediaCore |
|---|---|---|
| 画像URL | `cover_image_url` | `image_url` (フィールド名不一致) |
| 公式サイト | `homepage_url` | `url` (フィールド名不一致) |
| 公開日 | `release_date: NaiveDate` | `release_date: String` (型不一致、プロバイダごとの精度差を吸収するため文字列) |
| 評価 | `rating: Option<f32>` | `rating: Option<f64>` (精度不一致) |
| 外部ID | `external_id: Option<String>` | `external_id: String` (Optionality不一致) |

橋渡しは `models/item_import.rs` の `ImportItemRequest` が手動変換(`parse_release_date`等のアドホック関数)で行っている。

### その他の重複・欠落

- `status` が `AnimeDetails`/`MangaDetails`/`DramaDetails` にそれぞれ独立重複定義(意味は放送中/連載中/配信中で近いが型は`Option<String>`のまま統一されていない)
- `authors` が `MangaDetails`/`NovelDetails` に重複定義
- `MediaDetails::core()` はtrait抽象なしの手書きmatch — 新規メディア種別追加のたびに修正箇所が増える

## 3. 外部APIごとの取得可能情報

出典: `docs/api-samples/**/*.json|*.xml`, `backend/api-client-lib/src/clients/*/models.rs`, `backend/mediavault-api/src/models/domain/*.rs`

### アーキテクチャ上の重複

`backend/api-client-lib` は各プロバイダごとに型付きレスポンスDTO(`MovieModel`, `GameModel`等)を持つが、`ExternalSearchService`(`services/external_search.rs`)はこれを**生JSON文字列に戻し**、domain mapper(`from_tmdb`等)が**生JSONを再度パース**する。型付きモデルは実質的に「一度デシリアライズして捨てられるだけ」になっている(TMDb/IGDB/Jikanで顕著。IGDBの`dispatch_igdb`のみ例外的に型付き`Value`配列を直接使用)。

### プロバイダ別サマリ

| プロバイダ | 対応メディア | 本番ディスパッチ接続 | 備考 |
|---|---|:---:|---|
| TMDb | Movie, Drama | ✓ | Movie/TVのみマッピング。TVSeason(話数・キャスト情報含む)はクライアントに実装済みだがdomain mapperなし=未使用 |
| IGDB | Game | ✓ | 唯一、型付き`Value`配列を直接消費する経路 |
| Jikan (MAL) | Anime, Manga | ✓ | APIキー不要。`trailer.url`, `source`, `duration`, `season`等豊富だが manga fixtureは未整備 |
| AniList | Anime, Manga | ✗ 未接続 | `from_anilist_media`はテストのみで呼ばれるデッドコード |
| OpenLibrary | Novel | ✗ 未接続 | `from_openlibrary_edition`/`from_openlibrary_search_doc`はテストのみ |
| NDL | Novel/AcademicBook/Paper | ✓ | XMLパース(quick_xml手書き)。唯一、型付きモデル(`NdlItemModel`)を直接使う経路 |
| Steam | Game(検索は対象外、設計判断B) / 所有ゲームインポート | 部分的 | `from_steam_app`はテストのみのデッドコード。`import/steam_import.rs`は型付き`SteamGameEntry`を直接`CreateItemRequest`に変換する別経路 |

### 未マッピングの実データ例

- TMDb Movie: `belongs_to_collection`, `production_companies`はマッピング済みだが`vote_count`はMovieのみ(Dramaにはない)。TVSeasonの`episodes[].crew`/`guest_stars`は取得可能だが未使用。
- IGDB: `alternative_names`, `storyline`はマッピング済み。
- Jikan: `trailer.url`, `source`, `duration`, `broadcast`, `relations`, `theme`, `external`リンク等、`/anime/{id}/full`の情報の一部は未活用。
- Steam AppDetails: 実データには`categories`, `price_overview`, `achievements`, `ratings`(年齢制限), `recommendations.total`, `dlc[]`等があるが、`from_steam_app`(未接続)ですら6フィールドしか使っていない。

## 4. 既知の不整合点まとめ

1. `Item`(DB永続層) と `MediaCore`(外部API正規化層) のフィールド名/型不一致(上記表)
2. `status` が3つのDetails構造体に重複定義
3. `authors` が2つのDetails構造体に重複定義
4. AniList/OpenLibrary/Steamの`from_*`マッパーが本番導線から到達不能(テストのみ)
5. TMDb/IGDB/Jikanクライアントが型付きモデルをデシリアライズ後に捨て、生JSON再パースする二重構造
6. `MediaDetails`にtrait抽象がなく、`core()`は手書きmatchディスパッチ
