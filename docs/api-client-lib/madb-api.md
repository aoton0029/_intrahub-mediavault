# API設計 - MADB Lab

メディア芸術データベース（Media Arts Database / MADB）の LOD（Linked Open Data）版。
文化庁が公開しており、**マンガ・アニメ・ゲーム・メディアアート**の書誌／作品データを SPARQL で取得できる。

## 基本方針
- 提供形態: **SPARQL 1.1 エンドポイント**（REST/JSON API ではない）
- エンドポイント: `https://mediaarts-db.artmuseums.go.jp/sparql`
  - 人間向けのクエリエディタ（YASGUI）は `https://mediag.bunka.go.jp/madb_lab/lod/sparql/`。上記が実体のエンドポイント。
- 認証: なし（APIキー不要）
- メソッド: `GET`（`?query=`）／ `POST`（`application/x-www-form-urlencoded` の `query=`）
- レスポンス形式: SPARQL Results JSON（デフォルト）／ XML は `?format=xml`
- CORS ヘッダーは返らない → **ブラウザから直叩き不可。必ずサーバー側（BFF / バッチ）から呼ぶ**
- 文字コードは UTF-8。クエリに日本語リテラルを含める場合は UTF-8 でパーセントエンコードすること（後述）

## エンドポイント一覧
| Method | Path | 説明 | 認証 |
|--------|------|------|------|
| GET | /sparql?query={sparql} | SPARQLクエリ実行（短いクエリ向け） | 不要 |
| POST | /sparql（body: `query={sparql}`） | SPARQLクエリ実行（推奨） | 不要 |
| GET | /sparql?query={sparql}&format=xml | SPARQL Results XML で取得 | 不要 |

---

## POST /sparql

### リクエスト
```http
POST https://mediaarts-db.artmuseums.go.jp/sparql
Content-Type: application/x-www-form-urlencoded
Accept: application/sparql-results+json

query={URLエンコードしたSPARQLクエリ}
```

### パラメーター / バリデーション
- `query`: 必須（SPARQL 1.1 クエリ文字列。`SELECT` / `ASK` / `DESCRIBE` / `CONSTRUCT`）
- `format`: 任意。`xml` を指定すると SPARQL Results XML。省略時は JSON
  - `Accept` ヘッダーによるネゴシエーションは**効かない**（`text/csv` 等を指定しても JSON が返る）。形式を変えたいときは `format` パラメーターを使う
- 結果件数の上限はクエリ側の `LIMIT` で制御する（サーバー側のデフォルト上限はない）

### レスポンス（成功 200）
SPARQL Results JSON。`head.vars` に SELECT した変数、`results.bindings` に行が入る。
`type` は `uri` / `literal` の 2 種で、`literal` には `datatype` が付くことがある。

```json
{
  "head" : { "vars" : [ "book", "label", "volumeNumber", "datePublished", "publisher", "creator", "series" ] },
  "results" : {
    "bindings" : [ {
      "book" : { "type" : "uri", "value" : "https://mediaarts-db.artmuseums.go.jp/id/M522145" },
      "label" : { "type" : "literal", "value" : "ONE PIECE 巻91" },
      "volumeNumber" : { "type" : "literal", "value" : "巻91" },
      "datePublished" : { "type" : "literal", "value" : "2018-12" },
      "publisher" : { "type" : "literal", "value" : "集英社　∥　シュウエイシャ" },
      "creator" : { "type" : "literal", "value" : "[著]尾田栄一郎" },
      "series" : { "type" : "uri", "value" : "https://mediaarts-db.artmuseums.go.jp/id/C268196" }
    } ]
  }
}
```

### エラーレスポンス
| Status | 内容 | body |
|--------|------|------|
| 400 | クエリの構文エラー | `{"detailedMessage":"Malformed query: ...","code":"MalformedQueryException","requestId":"..."}` |
| 5xx / エラーメッセージ | 推定実行時間が60秒を超えるクエリ | タイムアウト相当。`LIMIT` や絞り込み条件を追加する |
| 接続不可 | 短時間の連続呼び出しによる一時遮断 | 同一IPからの接続が一時的に遮断される（後述） |

```json
{
  "detailedMessage" : "Malformed query: Encountered \"<EOF>\" at line 1, column 17. ...",
  "code" : "MalformedQueryException",
  "requestId" : "b2d006c9-a686-47f0-8bb7-1ae03bcba2fe"
}
```

---

## データモデル

### 名前空間
| 接頭辞 | URI | 用途 |
|--------|-----|------|
| `class:` | `https://mediaarts-db.artmuseums.go.jp/data/class#` | MADB独自クラス |
| `ma:` | `https://mediaarts-db.artmuseums.go.jp/data/property#` | MADB独自プロパティ |
| `schema:` | `https://schema.org/` | 主要な記述項目（**末尾スラッシュ付きの https**。`http://schema.org/` ではないので注意） |
| `rdfs:` | `http://www.w3.org/2000/01/rdf-schema#` | `rdfs:label` |
| `dcterms:` | `http://purl.org/dc/terms/` | `dcterms:creator`（Agentリソースへの参照） |
| `neptune-fts:` | `http://aws.amazon.com/neptune/vocab/v01/services/fts#` | 全文検索（Amazon Neptune拡張） |

### リソースURIの体系
- Item / Collection / Curation: `https://mediaarts-db.artmuseums.go.jp/id/{M|C}{連番}`
  - `M…` = 個別のモノ（単行本1冊、雑誌1号など）
  - `C…` = まとまり（シリーズ、責任主体など）
- Supplement（出典情報）: `https://mediaarts-db.artmuseums.go.jp/ref/S{連番}`
- URI をブラウザで開くと HTML の詳細ページが返る（RDF のコンテンツネゴシエーションはなし）

### 主なクラス（`?s a ?class`）
全一覧: [`docs/api-samples/madb/classes.json`](../api-samples/madb/classes.json)

| クラス | 内容 |
|--------|------|
| `class:MangaBookSeries` / `class:MangaBook` | マンガ単行本シリーズ / 単行本1冊 |
| `class:MangaMagazine` / `class:MangaMagazineIssue` | マンガ雑誌 / 雑誌の単号 |
| `class:MangaMagazinePublication` | 雑誌掲載（内容細目＝目次の1件） |
| `class:MangaOther` / `class:Supplement` | マンガその他 / 付録・出典 |
| `class:AnimationTVRegularSeries` / `class:AnimationTVProgram` | アニメTVレギュラーシリーズ / TV番組 |
| `class:AnimationMovieSeries` / `class:AnimationMovie` | アニメ映画シリーズ / 映画 |
| `class:AnimationVideoPackageSeries` / `class:AnimationVideoPackage` | アニメビデオパッケージ |
| `class:AnimationRelatedItem` | アニメ関連資料 |
| `class:GameWork` / `class:GamePackage` / `class:GameVariation` | ゲーム作品 / パッケージ / バリエーション |
| `class:GameRelatedItem` | ゲーム関連資料 |
| `class:MediaArtWork` / `class:MediaArtEvent` / `class:MediaArtExhibitionOrPerformance` | メディアアート作品 / 催事 / 展示・実演 |
| `class:Agent` | 責任主体（著者・出版社・制作会社など） |
| `class:Event` | 催事 |

### `schema:genre`（分類文字列）
クラスとほぼ対応する日本語の分類が `schema:genre` に入っており、**クエリの絞り込みで最もよく使う**。
全一覧: [`docs/api-samples/madb/genres.json`](../api-samples/madb/genres.json)

主な値: `マンガ単行本シリーズ` / `マンガ単行本` / `マンガ雑誌` / `マンガ雑誌単号` / `マンガ作品` /
`アニメテレビレギュラーシリーズ` / `アニメテレビ番組` / `アニメ映画` / `アニメ映画シリーズ` / `アニメビデオパッケージ` /
`ゲーム作品` / `ゲームパッケージ` / `ゲームバリエーション` / `メディアアート作品` / `責任主体`

雑誌の目次（`schema:hasPart` の空白ノード）側には `表紙` / `目次(パス)` / `広告` / `付録` / `特集記事` /
`小説・読物` / `写真・グラビア` など、記事種別の値が入る。

### 主なプロパティ
| プロパティ | 内容 | 例 |
|-----------|------|-----|
| `rdfs:label` | 表示名（巻数まで含む） | `ONE PIECE 巻91` |
| `schema:name` | 名称（複数値。読みガナも同じプロパティに入る） | `ONE PIECE`, `ワンピース` |
| `schema:identifier` | MADB ID | `M522145` |
| `schema:genre` | 分類 | `マンガ単行本` |
| `schema:isbn` | ISBN（ハイフンなし。**10桁と13桁が混在**） | `9784088816449` / `4088725093` |
| `schema:isPartOf` | 上位リソース（単行本→シリーズ、TV番組→シリーズ） | `.../id/C268196` |
| `schema:volumeNumber` / `schema:position` | 巻表示 / ソート用の数値 | `巻91` / `91.0` |
| `schema:datePublished` | 刊行・公開日（`YYYY-MM` など粒度が揺れる） | `2018-12` |
| `schema:publisher` | 出版社・放送局（読みガナが `∥` 区切りで連結される） | `集英社　∥　シュウエイシャ` |
| `schema:creator` / `schema:contributor` | 役割付きの人名文字列（`／` 区切り、役割は `[…]`） | `[著]尾田栄一郎` |
| `dcterms:creator` | 責任主体（`class:Agent`）への **URI参照** | `.../id/C59771` |
| `schema:brand` | レーベル | `ジャンプコミックス` |
| `schema:numberOfItems` | 構成点数（シリーズの巻数、TVの話数） | `110` / `47` |
| `schema:numberOfPages` / `schema:size` | ページ数 / 判型（単位付き文字列） | `223p` / `18cm` |
| `schema:alternateName` / `schema:description` | サブタイトル・内容紹介 | `侍の国の冒険` |
| `schema:actor` / `schema:productionCompany` | 声の出演 / 制作（`／` 区切りの文字列） | `【太陽】塩屋翼 ／ …` |
| `schema:startDate` / `schema:endDate` | 放送開始 / 終了日 | `1984-10-06` / `1985-09-20` |
| `schema:hasPart` | 雑誌単号の内容細目（空白ノード） | — |
| `schema:pageStart` / `schema:pageEnd` / `schema:alternativeHeadline` | 内容細目の開始／終了ページ・タイトル | — |
| `ma:ndc` / `ma:jpno` / `ma:ndla` | NDC分類 / 全国書誌番号 / NDL典拠URI | `726.1` / `23147545` |
| `ma:note` / `ma:source` / `ma:providerName` | 注記 / 出典 / データ提供元 | `国立国会図書館` |
| `ma:programDuration` / `ma:numberOfPrograms` / `ma:periodDisplayed` | 放送分数 / 話数 / 放送期間の表示用文字列 | `30` / `47` |
| `schema:provider` | 出典 Supplement（`/ref/S…`）への URI参照 | `.../ref/S1375984` |

> 人名・出版社・声の出演などは**正規化されていない1本の文字列**として入っている場所が多い。
> 構造化された著者を取りたい場合は `dcterms:creator` → `class:Agent` を辿る。

---

## クエリパターン

### 1. ISBNから単行本を引く
MediaVault の書籍取り込みで最初に使う経路。ISBN はハイフンなしで格納されているが、
**古い巻は10桁 ISBN で入っている**ため、13桁で当たらない場合は10桁に変換して再試行する
（または `VALUES ?isbn { "9784088725093" "4088725093" }` で両方を一度に問い合わせる）。

```sparql
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX schema: <https://schema.org/>
SELECT ?book ?label ?volumeNumber ?datePublished ?publisher ?creator ?series
WHERE {
  ?book schema:isbn "9784088816449" ;
        rdfs:label ?label ;
        schema:genre "マンガ単行本" .
  OPTIONAL { ?book schema:volumeNumber ?volumeNumber }
  OPTIONAL { ?book schema:datePublished ?datePublished }
  OPTIONAL { ?book schema:publisher ?publisher }
  OPTIONAL { ?book schema:creator ?creator }
  OPTIONAL { ?book schema:isPartOf ?series }
}
```
レスポンス例: [`docs/api-samples/madb/search_isbn.json`](../api-samples/madb/search_isbn.json)

### 2. タイトルの全文検索（Amazon Neptune 全文検索拡張）
`FILTER(CONTAINS(...))` は全件走査になり実用にならないため、**タイトル検索は全文検索を使う**。

```sparql
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX schema: <https://schema.org/>
PREFIX neptune-fts: <http://aws.amazon.com/neptune/vocab/v01/services/fts#>
SELECT ?resource ?genre ?label
WHERE {
  SERVICE neptune-fts:search {
    neptune-fts:config neptune-fts:endpoint "https://vpc-mediaarts-db-qaymrmtqbprlhmqq33a2ncf4ke.ap-northeast-1.es.amazonaws.com" .
    neptune-fts:config neptune-fts:field rdfs:label .
    neptune-fts:config neptune-fts:queryType "query_string" .
    neptune-fts:config neptune-fts:query '"呪術廻戦"' .
    neptune-fts:config neptune-fts:return ?resource .
  }
  ?resource schema:genre ?genre ; rdfs:label ?label .
  FILTER(?genre = "マンガ単行本シリーズ")
}
LIMIT 10
```
レスポンス例: [`docs/api-samples/madb/search_fulltext.json`](../api-samples/madb/search_fulltext.json)

- `neptune-fts:queryType`: `simple_query_string` / `match` / `prefix` / `fuzzy` / `term` / `query_string`
- `neptune-fts:query` の値は SPARQL のシングルクォート文字列にすると、中で `"…"` や `AND` / `OR` を使える
  - 例: `'"魔法" AND "戦"'`
- **主語が空白ノードのリソース（雑誌の内容細目など）は全文検索の対象外**
- `neptune-fts:endpoint` の値は MADB Lab 側の環境に依存するため、変更されうる前提で設定値として持つ

### 3. シリーズに属する巻を並べる
`schema:isPartOf` は「巻 → シリーズ」の向き。ソートは文字列の `volumeNumber` ではなく数値の `schema:position` を使う。

```sparql
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX schema: <https://schema.org/>
PREFIX xsd:    <http://www.w3.org/2001/XMLSchema#>
SELECT ?volume ?label ?volumeNumber ?isbn ?datePublished
WHERE {
  ?volume schema:isPartOf <https://mediaarts-db.artmuseums.go.jp/id/C268196> ;
          rdfs:label ?label .
  OPTIONAL { ?volume schema:volumeNumber ?volumeNumber }
  OPTIONAL { ?volume schema:isbn ?isbn }
  OPTIONAL { ?volume schema:datePublished ?datePublished }
  OPTIONAL { ?volume schema:position ?position }
}
ORDER BY xsd:float(?position)
LIMIT 100 
```
レスポンス例: [`docs/api-samples/madb/series_volumes.json`](../api-samples/madb/series_volumes.json)

### 4. 1リソースの全項目を取得（詳細画面用）
プロパティが可変なので、詳細取得は「述語・目的語を総なめ」するのが実用的。

```sparql
SELECT ?p ?o
WHERE { <https://mediaarts-db.artmuseums.go.jp/id/M522145> ?p ?o }
```
レスポンス例: [`docs/api-samples/madb/resource_detail.json`](../api-samples/madb/resource_detail.json)（マンガ単行本）/
[`docs/api-samples/madb/anime_series_detail.json`](../api-samples/madb/anime_series_detail.json)（アニメTVシリーズ）

`DESCRIBE <uri>` も使えるが、返るのは RDF ではなく `subject` / `predicate` / `object` / `context` を持つ
SPARQL Results JSON なので、扱いは上のクエリと同じ。

### 5. マンガ雑誌単号の目次を取得
内容細目は空白ノードとして `schema:hasPart` にぶら下がる。

```sparql
PREFIX xsd:    <http://www.w3.org/2001/XMLSchema#>
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX schema: <https://schema.org/>
PREFIX class:  <https://mediaarts-db.artmuseums.go.jp/data/class#>
SELECT ?issueLabel ?pageStart ?pageEnd ?articleGenre ?title
WHERE {
  <https://mediaarts-db.artmuseums.go.jp/id/M535428> a class:MangaMagazineIssue ;
      schema:genre "マンガ雑誌単号" ;
      rdfs:label ?issueLabel ;
      schema:hasPart [
          schema:genre ?articleGenre ;
          schema:pageStart ?pageStart ;
          schema:pageEnd ?pageEnd ;
          schema:alternativeHeadline ?title ;
      ].
}
ORDER BY xsd:float(?pageStart)
```

### 6. 責任主体（著者）を辿る
```sparql
PREFIX rdfs:    <http://www.w3.org/2000/01/rdf-schema#>
PREFIX schema:  <https://schema.org/>
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX ma:      <https://mediaarts-db.artmuseums.go.jp/data/property#>
SELECT ?agent ?name ?kind ?ndla
WHERE {
  <https://mediaarts-db.artmuseums.go.jp/id/C268196> dcterms:creator ?agent .
  ?agent rdfs:label ?name .
  OPTIONAL { ?agent ma:additionalGenre ?kind }   # 個人 / 団体 など
  OPTIONAL { ?agent ma:ndla ?ndla }              # NDL典拠のURI
}
```

### 7. Wikidata との連結（フェデレーテッドクエリ）
`SERVICE` で外部エンドポイントを併用できる。Wikidata 側は `wdt:P7886`（メディア芸術データベースID）で紐づく。

```sparql
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX schema: <https://schema.org/>
PREFIX wdt:    <http://www.wikidata.org/prop/direct/>
SELECT ?wikidataEntity ?title (LANG(?title) AS ?lang)
WHERE {
  # 例: アニメ映画シリーズ「君の名は。 your name.」
  <https://mediaarts-db.artmuseums.go.jp/id/C413599> schema:identifier ?madbId .
  SERVICE <https://query.wikidata.org/sparql> {
    ?wikidataEntity wdt:P7886 ?madbId ;
                    rdfs:label ?title .
  }
}
```

---

## 使用上の注意・Tips

### レート制限・タイムアウト
- 推定実行時間が **60秒** を超えると判断されたクエリはエラーになる
- **同一IPからの短時間の連続呼び出しを検知すると、そのIPからの接続が一時的に遮断される**
  → MediaVault からは逐次（直列）＋インターバルを入れて呼ぶ。バルク取り込みはバッチで夜間に流す
- 取得結果はローカルにキャッシュする前提で設計する（データ自体の更新頻度は低い。`ma:dateModified` 相当の
  `.../property-data/dateModified` で更新時刻が分かる）

### 書き込みは絶対に送らない
このエンドポイントは SPARQL **Update**（`update=` パラメーター）も受け付けてしまう。
公開データを壊すことになるため、クライアント実装では
`query=` のみを送り、`update=` / `INSERT` / `DELETE` を組み立てる経路を作らないこと。

### 日本語リテラルのエンコーディング
クエリ中に日本語リテラル（`"マンガ単行本"` など）を含める場合、**UTF-8 で正しくエンコードしないと
マッチ0件が静かに返る**（400 にならないので気づきにくい）。
- HTTPクライアントからは `Content-Type: application/x-www-form-urlencoded; charset=UTF-8` で送る
- 手元検証で `curl` を使う場合は、クエリを UTF-8 のファイルに書いて
  `--data-urlencode "query@query.rq"` で渡すのが確実（シェルの文字コードに引きずられない）

### データの癖
- `schema:name` に**タイトルと読みガナが混在**する（複数値）。表示名は `rdfs:label` を使う
- `schema:publisher` は `集英社　∥　シュウエイシャ` のように**全角空白＋`∥`＋読み**が連結されている。
  `∥` で split して先頭を取る
- `schema:creator` / `schema:actor` / `schema:contributor` は `／` 区切り、役割が `[…]` / `【…】` 付きの
  1本の文字列。パースは正規表現で `[\[【](?<role>[^\]】]+)[\]】](?<name>.+)` 相当
- `schema:datePublished` の粒度は `1997-12` / `1984-10-06` など不定。日付型ではなく文字列として保持する
- 数値項目も `223p` / `18cm` / `巻91` のように単位・接頭辞付きの文字列。ソートには `schema:position` を使う
- 同名シリーズが複数存在する（例: `ONE PIECE` 本編・クイズブック・愛蔵版）。
  ISBN が取れる場合は ISBN 起点で引き、取れない場合は `schema:publisher` / `schema:datePublished` /
  `schema:numberOfItems` で判別する
- ベータ版データセットのため、`ma:dataPublisher` が `メディア芸術データベースベータ版データセット` の
  レコードが多い。表記揺れ・重複はある前提で名寄せする

### MediaVault での使い分け
- **マンガの巻単位メタデータ・雑誌掲載履歴**は MADB が最も強い（NDL 由来の書誌＋巻数構造）
- ISBN 起点の一般書誌は [NDLサーチ](ndl-api.md) / [楽天ブックス](rakuten-books-api.md) を優先し、
  MADB は「シリーズ構造」「巻数」「雑誌掲載」の補完に使う
- アニメは [Annict](annict-api.md) / [Jikan](jikan-api.md)、ゲームは [IGDB](igdb-api.md) の方が
  現行タイトルの網羅性・画像が強い。MADB は**旧作・国内作品の網羅性**で補完する
- 画像（表紙・サムネイル）は提供されない

### ライセンス・データ配布
- RDF ダンプおよびメタデータスキーマ仕様書は GitHub で公開されている
  → https://github.com/mediaarts-db/dataset
- 大量取得が必要な場合は SPARQL を叩き続けるのではなく**ダンプを取り込む**

---

## 参考リンク
- MADB Lab LOD の使い方（本ドキュメントの一次情報）: https://mediag.bunka.go.jp/madb_lab/lod/howto/
- SPARQLクエリサービス（YASGUI）: https://mediag.bunka.go.jp/madb_lab/lod/sparql/
- SPARQLエンドポイント: https://mediaarts-db.artmuseums.go.jp/sparql
- メディア芸術データベース（本体）: https://mediaarts-db.artmuseums.go.jp/
- データセット・スキーマ仕様書（GitHub）: https://github.com/mediaarts-db/dataset
- Amazon Neptune 全文検索（`neptune-fts:`）: https://docs.aws.amazon.com/neptune/latest/userguide/full-text-search.html
