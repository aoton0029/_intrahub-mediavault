# backend/models 共通項目・ドメイン項目 刷新案

前提: [refactor-analysis.md](./refactor-analysis.md) の分析結果に基づく。DB層(`items`テーブル + JSONB)はすでに一本化されているため、本刷新案はRustアプリケーション層(`Item`, `MediaCore`, `*Details`, 外部APIクライアント)のみを対象とする。

## 1. 共通コアの一本化

`Item`(DB永続層)と`MediaCore`(外部API正規化層)は役割が異なる(前者は永続化された行、後者は外部APIレスポンスの正規化ビュー)ため、**構造体自体を統合するのではなく、フィールド名/型を揃えたうえで変換を1箇所に集約する**方針を推奨する。

- `MediaCore` のフィールド名を `Item` に合わせる:
  - `image_url` → `cover_image_url`
  - `url` → `homepage_url`
- 型は用途が異なるため無理に揃えない(`MediaCore.release_date`はプロバイダ間の精度差を吸収するため文字列のまま維持。`Item`側でパースする)。ただし変換関数を`ImportItemRequest`の場当たり的な実装から、`impl From<&MediaCore> for CreateItemRequest`相当の一箇所に集約し、`parse_release_date`等のヘルパーもそこに集める。
- `external_id`: `MediaCore`は必須のまま(外部API起源のデータは常にIDを持つ)、`Item`はManual作成を許容するためOptionのまま維持。変換時に`Some(core.external_id)`とする。

## 2. `HasMediaCore` トレイト導入

`MediaDetails::core()`の手書きmatchを解消するため、共通トレイトを導入する。

```rust
trait HasMediaCore {
    fn core(&self) -> &MediaCore;
}
```

`AnimeDetails`/`MangaDetails`/`MovieDetails`/`DramaDetails`/`GameDetails`/`NovelDetails`それぞれに実装(`self.core`を返すだけ)。`MediaDetails::core()`はmatchで各バリアントの`.core()`を呼ぶだけになり、新規メディア種別追加時も1行追加で済む。

## 3. 重複フィールドの共通化

- `status`(放送中/連載中/配信中等): `AnimeDetails`/`MangaDetails`/`DramaDetails`で意味が重複しているが、値のバリエーション(anime: "Currently Airing"等、manga: "Publishing"等)がプロバイダ依存の自由文字列であるため、無理に共通enumへ昇格させるとプロバイダ追加のたびにマッピング表が必要になる。**`MediaCore`に`publication_status: Option<String>`として引き上げ、フィールド定義の重複のみ解消する**(値の意味論の統一は将来課題として明記するに留める)。
- `authors`: `MangaDetails`/`NovelDetails`で完全に同じ意味・型のため、`MediaCore`に`Vec<String>`として引き上げる。Movie/Drama/Game/Animeでは常に空配列となるが、`#[serde(default, skip_serializing_if = "Vec::is_empty")]`でAPIレスポンスの肥大化を防ぐ。

## 4. 外部APIクライアント層の整理

### 型付きモデルと生JSON再パースの二重構造

TMDb/IGDB/Jikanは型付きレスポンス(`MovieModel`等)を経由後、`ExternalSearchService`が生JSON文字列に戻してdomain mapperが再パースしている。2つの選択肢:

- (a) 型付きモデルを実際に使う形にdomain mapperを書き換える(型安全性が上がるが、プロバイダのレスポンス形状が多様/ネストが深く、`serde_json::Value`ベースの柔軟な抽出の方が現状のフィールド網羅性を保ちやすい)
- (b) 型付きモデル(`MovieModel`, `TvModel`, `GameModel`等)を廃止し、api-client-libは「HTTPリクエスト実行 + 生JSON/XML取得」のみを担当、パースは全てdomain mapper側の`serde_json::Value`処理に一本化する

NDL(XML→型付き構造体→mapper)とIGDB(型付き`Value`配列を直接使用)は既に(a)寄りの一貫した経路になっているため、**(b)を推奨**: TMDb/Jikan/AniList/OpenLibrary/Steamの「型定義→デシリアライズ→捨てて生JSON再パース」という無駄な往復をなくし、api-client-libの責務を「HTTP実行とレスポンス生データの受け渡し」に絞る。型付きDTO定義(`models.rs`内の`MovieModel`等)は削除し、レスポンスは`RawData::Json(String)`のまま返す形に単純化する。

### 未使用ドメインマッパーの削除

以下は`ExternalSearchService`のディスパッチ経路から到達不能で、単体テストのfixtureのみで参照されるデッドコードのため削除する:

- `AnimeDetails::from_anilist_media` (`models/domain/anime.rs`)
- `MangaDetails::from_anilist_media` (`models/domain/manga.rs`)
- `NovelDetails::from_openlibrary_edition` / `from_openlibrary_search_doc` (`models/domain/novel.rs`)
- `GameDetails::from_steam_app` (`models/domain/game.rs`)

付随して削除:
- `backend/api-client-lib/src/clients/anilist/` クライアント一式(`mod.rs`, `models.rs`)
- `backend/api-client-lib/src/clients/openlibrary/` クライアント一式
- `docs/api-samples/anilist/`, `docs/api-samples/openlibrary/` のfixture
- 上記マッパーに対応する`#[cfg(test)]`ユニットテスト
- `ApiProvider::AniList` / `ApiProvider::OpenLibrary` enumバリアント(他に参照がなければ)

Steamクライアント本体(`clients/steam/`)は`import/steam_import.rs`の所有ゲームインポート機能で使用中のため存続。削除対象は`GameDetails::from_steam_app`とそれに紐づくテスト/fixtureのみ。

## 5. 影響範囲と移行手順

### 変更対象ファイル(代表例)

- `backend/mediavault-api/src/models/domain/core.rs` — `MediaCore`フィールド名変更・`publication_status`/`authors`追加・`HasMediaCore`トレイト定義
- `backend/mediavault-api/src/models/domain/{anime,manga,movie,drama,game,novel}.rs` — トレイト実装、重複フィールド削除、フィールド名変更に伴う`from_*`修正
- `backend/mediavault-api/src/models/domain/media_details.rs` — `core()`をトレイトディスパッチに置換
- `backend/mediavault-api/src/models/item_import.rs` — 変換ロジックの一本化
- `backend/api-client-lib/src/clients/{tmdb,jikan}/models.rs` — 型付きDTO削除(方針(b)採用時)
- `backend/api-client-lib/src/clients/{anilist,openlibrary}/` — ディレクトリごと削除
- `backend/mediavault-api/src/models/domain/{anime,manga,novel,game}.rs` — 未使用マッパー削除
- `docs/api-samples/{anilist,openlibrary}/` — fixture削除

### 推奨する移行順序

1. **`HasMediaCore`トレイト導入**(振る舞いに影響しない純粋なリファクタ、最もリスクが低い)
2. **重複フィールドの共通化**(`status`→`publication_status`, `authors`引き上げ) — 各`from_*`関数の修正が必要だが機械的
3. **`Item`⇔`MediaCore`のフィールド名統一 + 変換一本化**(`image_url`→`cover_image_url`等のリネームはAPIレスポンスのJSON互換性に影響するため、フロントエンド側の対応要否を要確認)
4. **外部APIクライアント層の整理**(型付きDTO削除・デッドコード削除) — 実行時の挙動に影響しないコード削除が中心のため最後に実施

ステップを分けるのは、1〜2が振る舞い不変の内部整理である一方、3はAPIレスポンス形状(JSON互換性)に影響し得るため、影響範囲を切り分けて段階的にレビュー・検証できるようにするため。

## 検証方法(実装時)

- `cargo build -p mediavault-api` / `cargo test --workspace`
- `cargo test --workspace --all-targets -- --include-ignored`(DB依存統合テスト含む)
- `cargo clippy --all-targets --all-features -- -D warnings`
- フィールド名変更(3.)を実施する場合は、フロントエンド(`frontend/src`)の対応するAPIクライアント/型定義への影響を確認
