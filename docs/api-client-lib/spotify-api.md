# APIクライアント設計 - Spotify Web API

## 1. 目的とスコープ

MediaVault の音楽メタデータ検索・取り込みで Spotify Web API を利用する Rust クライアントを設計する。初期スコープは読み取り専用とする。

- アルバム・トラック・アーティストの横断検索
- アルバム詳細と全トラックの取得
- トラック詳細の取得
- アーティスト詳細とリリース一覧の取得
- Spotify レスポンスから MediaVault 取り込み用モデルへの正規化

ユーザーライブラリ、プレイリスト、再生制御、音源ダウンロードは対象外とする。Spotify コンテンツを機械学習・AI モデルの学習へ利用しない。

## 2. 基本仕様

| 項目 | 設計値 |
|---|---|
| API Base URL | `https://api.spotify.com/v1` |
| Token URL | `https://accounts.spotify.com/api/token` |
| 認証 | OAuth 2.0 Client Credentials Flow |
| API認証ヘッダー | `Authorization: Bearer <access_token>` |
| 形式 | JSON / UTF-8 |
| 既定マーケット | `JP`（設定で変更可能） |
| ページネーション | `limit` / `offset`、レスポンスの `next` |
| タイムアウト | connect 5秒、request 15秒 |
| User-Agent | `intrahub-mediavault/<version>` |

Client Credentials Flow はユーザー情報へアクセスできないが、本設計のカタログ読み取りには十分である。資格情報はバックエンドのみで保持する。

```text
SPOTIFY_CLIENT_ID=<required>
SPOTIFY_CLIENT_SECRET=<required>
SPOTIFY_MARKET=JP
SPOTIFY_API_BASE_URL=https://api.spotify.com/v1
SPOTIFY_TOKEN_URL=https://accounts.spotify.com/api/token
```

Base URL の上書きはテスト用モックサーバー向けであり、本番では Spotify の公式HTTPSホストだけを許可する。

## 3. 対象エンドポイント

| Method | Path | 用途 | ページサイズ |
|---|---|---|---|
| POST | `https://accounts.spotify.com/api/token` | アプリ用トークン取得 | - |
| GET | `/search` | album / track / artist 検索 | Development Mode: 最大10 |
| GET | `/albums/{id}` | アルバム詳細 | - |
| GET | `/albums/{id}/tracks` | アルバム収録曲 | 既定20、最大50 |
| GET | `/tracks/{id}` | トラック詳細 | - |
| GET | `/artists/{id}` | アーティスト詳細 | - |
| GET | `/artists/{id}/albums` | アーティストのリリース | Development Mode: 既定5、最大10 |

Spotify ID は不透明な文字列として扱い、固定長に依存しない。空文字を拒否し、パスへはURLエンコードして埋め込む。

## 4. 公開インターフェース

Spotify の生レスポンス型を呼び出し側へ漏らさず、安定したドメイン型を返す。

```rust
#[async_trait::async_trait]
pub trait SpotifyCatalogClient: Send + Sync {
    async fn search(&self, request: SearchRequest) -> Result<SearchPage, SpotifyError>;
    async fn album(&self, id: &str) -> Result<Album, SpotifyError>;
    async fn album_tracks(&self, id: &str) -> Result<Vec<TrackSummary>, SpotifyError>;
    async fn track(&self, id: &str) -> Result<Track, SpotifyError>;
    async fn artist(&self, id: &str) -> Result<Artist, SpotifyError>;
    async fn artist_albums(
        &self,
        id: &str,
        request: ArtistAlbumsRequest,
    ) -> Result<Page<AlbumSummary>, SpotifyError>;
}

pub struct SearchRequest {
    pub query: String,
    pub types: Vec<SearchType>,
    pub market: Option<String>,
    pub limit: Option<u8>,
    pub offset: Option<u16>,
}

pub enum SearchType { Album, Artist, Track }
pub enum AlbumGroup { Album, Single, AppearsOn, Compilation }
```

### バリデーション

- `query`: trim 後1文字以上。高度検索フィルタ（`album:`, `artist:`, `track:`, `year:`, `upc:`, `isrc:`, `genre:`）も許可する。
- `types`: 1件以上。重複を除去し、初期スコープ外の型は送らない。
- `market`: ISO 3166-1 alpha-2 の大文字2文字。未指定時は設定値 `JP`。
- `limit`: 1〜10。Extended Quota Mode でも互換性を優先し10を上限とする。
- `offset`: 0〜1000。
- artist albums の `include_groups`: 既定は `album,single,compilation`。`appears_on` は明示時のみ。

検索はタイプごとに別のページオブジェクトを返す。複数タイプ指定時は同じ `offset` が各タイプへ適用されるため、UIの「さらに表示」は単一タイプで再検索する。

## 5. データモデル

外部DTOは `serde` でSpotify JSONを寛容に受け、ドメインモデルへ変換する。廃止・欠落し得るフィールドは `Option<T>` または `#[serde(default)]` とし、未知フィールドは拒否しない。

```rust
pub struct Page<T> {
    pub items: Vec<T>,
    pub limit: u32,
    pub offset: u32,
    pub total: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
}

pub struct Image { pub url: String, pub width: Option<u32>, pub height: Option<u32> }
pub struct ArtistSummary { pub id: String, pub name: String, pub spotify_url: Option<String> }

pub struct Artist {
    pub id: String,
    pub name: String,
    pub images: Vec<Image>,
    pub genres: Vec<String>,
    pub followers: Option<u64>,
    pub popularity: Option<u8>,
    pub spotify_url: Option<String>,
}

pub struct AlbumSummary {
    pub id: String,
    pub name: String,
    pub album_type: AlbumType,
    pub artists: Vec<ArtistSummary>,
    pub images: Vec<Image>,
    pub release_date: PartialDate,
    pub total_tracks: u32,
    pub spotify_url: Option<String>,
}

pub struct Album {
    pub summary: AlbumSummary,
    pub tracks: Vec<TrackSummary>,
    pub external_ids: ExternalIds,
    pub label: Option<String>,
    pub popularity: Option<u8>,
}

pub struct TrackSummary {
    pub id: String,
    pub name: String,
    pub artists: Vec<ArtistSummary>,
    pub disc_number: u32,
    pub track_number: u32,
    pub duration_ms: u64,
    pub explicit: bool,
    pub is_local: bool,
    pub is_playable: Option<bool>,
    pub spotify_url: Option<String>,
}

pub struct Track {
    pub summary: TrackSummary,
    pub album: AlbumSummary,
    pub external_ids: ExternalIds,
    pub popularity: Option<u8>,
}

pub struct ExternalIds { pub isrc: Option<String>, pub ean: Option<String>, pub upc: Option<String> }
pub struct PartialDate { pub value: String, pub precision: DatePrecision }
```

`release_date` は `YYYY`、`YYYY-MM`、`YYYY-MM-DD` があり得るため日付型へ直接デシリアライズしない。画像配列の順序にも依存せず、用途に最も近いサイズを選ぶ。

## 6. MediaVault への正規化

| Spotify | MediaVault取り込み値 | 備考 |
|---|---|---|
| `album.id` | 外部ID `spotify:album` | 冪等キー |
| `album.name` | item title | 空文字は不正データ |
| `album.album_type` | subtype | album / single / compilation |
| `album.release_date` | released_on / released_year | precisionも保持 |
| `album.artists[]` | staff (`artist`) | artist IDも保存 |
| `album.images[]` | item_images | 代表画像を選択 |
| `album.external_urls.spotify` | item_links | 帰属リンク |
| `album.external_ids.upc/ean` | external identifiers | 存在時のみ |
| `track.id` | 外部ID `spotify:track` | 冪等キー |
| `track.disc_number` | group number | ディスク単位 |
| `track.track_number` | episode number | 収録順 |
| `track.duration_ms` | duration_ms | ミリ秒のまま |
| `track.external_ids.isrc` | external identifier | 存在時のみ |

同名・別版アルバムをタイトルだけで統合しない。`spotify:album:{id}` をプロバイダ内の一意キーとする。アルバム詳細の埋め込み `tracks.next` が非nullなら `/albums/{id}/tracks` を最後まで取得する。

## 7. 認証とトークン管理

```bash
curl -X POST "https://accounts.spotify.com/api/token" \
  -H "Authorization: Basic BASE64_CLIENT_ID_COLON_CLIENT_SECRET" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials"
```

- トークンはプロセス内で共有し、`acquired_at + expires_in - 60秒` まで再利用する。
- 更新は single-flight とし、同時リクエストによるトークン取得の集中を防ぐ。
- 401時はキャッシュを破棄し、一度だけトークン再取得・再送する。
- Client Credentials に refresh token はないため、期限切れ時は再発行する。
- secret、Basic認証値、アクセストークンはログと `Debug` から必ずマスクする。

## 8. 再試行とキャッシュ

| 条件 | 動作 |
|---|---|
| 401 | トークン再取得後に1回だけ再送 |
| 429 | `Retry-After` 秒 + jitter、最大3回 |
| 500 / 502 / 503 / 504 | 250ms起点の指数バックオフ + jitter、最大3回 |
| timeout / connection reset | GETのみ最大2回 |
| その他の4xx | 再試行しない |

Spotifyのレート制限はローリング30秒窓で、固定上限値は公開されていない。429時はクライアント全体で送信を抑制する。

- album / track / artist: 24時間
- search: 15分（正規化済み query + types + market + limit + offset がキー）
- album tracks / artist albums: 6時間
- 404: 5分のnegative cache
- 401 / 403 / 429 / 5xx: キャッシュしない

## 9. エラー型

```rust
pub enum SpotifyError {
    Configuration { field: &'static str },
    InvalidRequest { field: &'static str, reason: String },
    Authentication,
    Forbidden,
    NotFound { resource: &'static str, id: String },
    RateLimited { retry_after: Option<std::time::Duration> },
    Upstream { status: u16, message: Option<String> },
    Transport { retryable: bool, source: reqwest::Error },
    Decode { source: serde_json::Error },
}
```

標準APIエラーとトークンAPIエラーの両形式を受理する。上流本文全体はログへ出さず、status、message、request IDのみ記録する。

## 10. リクエスト例とサンプル

```bash
curl --get "https://api.spotify.com/v1/search" \
  -H "Authorization: Bearer ACCESS_TOKEN" \
  --data-urlencode "q=album:Discovery artist:Daft Punk" \
  --data-urlencode "type=album,track" \
  --data-urlencode "market=JP" \
  --data-urlencode "limit=10" \
  --data-urlencode "offset=0"
```

- [`../api-samples/spotify/search_album_track.json`](../api-samples/spotify/search_album_track.json)
- [`../api-samples/spotify/album.json`](../api-samples/spotify/album.json)
- [`../api-samples/spotify/album_tracks.json`](../api-samples/spotify/album_tracks.json)
- [`../api-samples/spotify/artist.json`](../api-samples/spotify/artist.json)
- [`../api-samples/spotify/track.json`](../api-samples/spotify/track.json)

サンプルJSONは公式スキーマに沿ったテストfixture用の縮約例で、実APIから採取した完全レスポンスではない。

## 11. Development Mode（2026年変更）

新規アプリの既定である Development Mode を基準にする。

- アプリ所有者に有効なSpotify Premiumが必要。
- 新規アプリは開発者あたりClient ID 1件、アプリあたりユーザー5人。
- Searchの `limit` は最大10。
- `/tracks?ids=...`、`/albums?ids=...`、`/artists?ids=...` 等の一括取得は使わない。
- artist top tracks、related artists、new releases 等の削除対象に依存しない。
- Trackの `available_markets`、`linked_from`、`popularity`、Albumの `album_group`、`available_markets`、`label`、`popularity`、Artistの `followers`、`popularity` は欠落を許容する。
- `external_ids` は2026年3月に削除が撤回されたが任意フィールドとして扱う。
- Extended Quota Mode固有機能は capability flag で分離する。

## 12. テスト方針

- `docs/api-samples/spotify/*.json` をDTO fixtureに使う。
- `popularity`、`followers`、`label`、`external_ids`、画像サイズの欠落を受理する。
- year / month / day の各日付precisionを検証する。
- `next` を追跡し、nullで停止する。最大ページ数の安全弁も設ける。
- 同時呼び出しでもトークン取得1回、期限60秒前更新、401再送1回を検証する。
- `Retry-After`、全体抑制、最大試行回数を仮想時計で検証する。
- queryはURL builderでエンコードし、secretが含まれないことを検証する。

## 13. 利用上の制約

- 音源ダウンロードやstream rippingを実装しない。
- アートワークを切り抜く、文字・ロゴを重ねる等の加工をしない。
- メタデータや画像の表示には対象の `external_urls.spotify` とSpotifyの帰属表示を併記する。
- Spotify Platform / Spotify ContentをAI・機械学習モデルの学習へ利用しない。

## 14. 参考資料

- [Spotify Web API](https://developer.spotify.com/documentation/web-api)
- [OpenAPI schema](https://developer.spotify.com/reference/web-api/open-api-schema.yaml)
- [Search](https://developer.spotify.com/documentation/web-api/reference/search)
- [Get Album](https://developer.spotify.com/documentation/web-api/reference/get-an-album)
- [Get Album Tracks](https://developer.spotify.com/documentation/web-api/reference/get-an-albums-tracks)
- [Get Track](https://developer.spotify.com/documentation/web-api/reference/get-track)
- [Get Artist](https://developer.spotify.com/documentation/web-api/reference/get-an-artist)
- [Get Artist's Albums](https://developer.spotify.com/documentation/web-api/reference/get-an-artists-albums)
- [Client Credentials Flow](https://developer.spotify.com/documentation/web-api/tutorials/client-credentials-flow)
- [Rate Limits](https://developer.spotify.com/documentation/web-api/concepts/rate-limits)
- [February 2026 Dev Mode Changes](https://developer.spotify.com/documentation/web-api/tutorials/february-2026-migration-guide)
