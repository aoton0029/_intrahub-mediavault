# API設計 - 楽天ブックス

## 基本方針
- RESTful API（GETのみ）
- ベースURL: `https://openapi.rakuten.co.jp/services/api`（`https://app.rakuten.co.jp/services/api` は新方式のUUID形式`applicationId`を受け付けず `400 wrong_parameter` になるため使用不可。IP制限は`openapi.rakuten.co.jp`側でチェックされる）
- 認証: `applicationId`（楽天Web ServiceのアプリID、UUID形式）と `accessKey`（アクセスキー）の2つが必須。両方ともクエリパラメータまたはHTTPヘッダで送信可能
  - Rakuten Developersのアプリ設定で「Allowed IP Addresses」を許可しているとその制限も適用される（未許可のIPからは `403 CLIENT_IP_NOT_ALLOWED` が返る）
- レスポンス形式: JSON（`format=json` を明示、未指定時はXML）
- ページネーション: `page`（デフォルト1） / `hits`（1ページあたりの件数、デフォルト30・最大30）。レスポンスに `count`（総件数） / `page`（現在ページ） / `first` / `last`（当該ページの先頭・末尾の通し番号） / `pageCount`（総ページ数）を含む
- 検索条件（`title` / `author` / `publisherName` / `isbn` / `booksGenreId` など）は最低1つ以上の指定が必須

## エンドポイント一覧
| Method | Path | 説明 | 認証 |
|--------|------|------|------|
| GET | /BooksBook/Search/20170404 | 書籍を検索・取得する | 必要 |

---

## GET /BooksBook/Search/20170404

### メソッド
GET

### URL
https://openapi.rakuten.co.jp/services/api/BooksBook/Search/20170404

### 説明
楽天ブックスに登録されている書籍情報を、タイトル・著者名・出版社名・ISBN・ジャンルIDなどの条件で検索する。

### パラメータ
- `applicationId` (string, 必須): 楽天Web ServiceのアプリID（UUID形式）
- `accessKey` (string, 必須): アクセスキー
- `format` (string): レスポンス形式（`json` を指定。未指定時はXML）
- `title` (string): 書籍タイトルで絞り込み（部分一致）
- `author` (string): 著者名で絞り込み
- `publisherName` (string): 出版社名で絞り込み
- `isbn` (string): ISBNコード（13桁）で絞り込み
- `booksGenreId` (string): 楽天ブックスジャンルIDで絞り込み
- `size` (string): 書籍サイズで絞り込み
- `sort` (string): ソート順（例: `+itemPrice` / `-itemPrice` / `sales`（売上順） / `standard`（標準）など）
- `page` (integer): ページ番号（デフォルト1）
- `hits` (integer): 1ページあたりの件数（デフォルト30、最大30）
- `availability` (string): 在庫状況で絞り込み
- `outOfStockFlag` (integer): 品切れ商品を含めるか
- `carrier` (integer): モバイル端末向けフラグ（0: PC/スマホ）

### レスポンスフィールド
- `count`: 検索結果の総件数
- `page`: 現在のページ番号
- `first` / `last`: このページに含まれる結果の先頭・末尾の通し番号
- `hits`: 1ページあたりの件数
- `carrier`: 端末種別
- `pageCount`: 総ページ数
- `Items[].Item`: 書籍情報本体
  - `title` / `titleKana`: タイトル / 読み仮名
  - `subTitle` / `subTitleKana`: サブタイトル / 読み仮名
  - `seriesName` / `seriesNameKana`: シリーズ名 / 読み仮名
  - `author` / `authorKana`: 著者名 / 読み仮名
  - `publisherName`: 出版社名
  - `isbn`: ISBNコード
  - `itemCaption`: 内容紹介
  - `salesDate`: 発売日（日本語表記の文字列）
  - `itemPrice` / `listPrice`: 販売価格 / 定価
  - `discountRate` / `discountPrice`: 割引率 / 割引額
  - `itemUrl`: 楽天ブックス商品ページURL
  - `affiliateUrl`: アフィリエイトURL（`affiliateId` 指定時）
  - `smallImageUrl` / `mediumImageUrl` / `largeImageUrl`: 商品画像URL（サイズ違い）
  - `chirayomiUrl`: 立ち読みページURL
  - `availability`: 在庫状況コード
  - `postageFlag`: 送料フラグ
  - `limitedFlag`: 限定販売フラグ
  - `reviewCount` / `reviewAverage`: レビュー件数 / 平均評価
  - `booksGenreId`: 楽天ブックスジャンルID
- `GenreInformation[]`: 該当ジャンルの階層情報（該当なしの場合は空配列）

### リクエスト例
```bash
curl "https://openapi.rakuten.co.jp/services/api/BooksBook/Search/20170404?applicationId=YOUR_APP_ID&accessKey=YOUR_ACCESS_KEY&isbn=9784088725093&format=json"
```

### レスポンス例
```json
{
  "count": 1,
  "page": 1,
  "first": 1,
  "last": 1,
  "hits": 1,
  "carrier": 0,
  "pageCount": 1,
  "Items": [
    {
      "Item": {
        "title": "ONE PIECE 1",
        "titleKana": "ワン ピース",
        "seriesName": "ジャンプコミックス",
        "author": "尾田 栄一郎",
        "authorKana": "オダエイイチロウ",
        "publisherName": "集英社",
        "isbn": "9784088725093",
        "itemCaption": "時は大海賊時代。いまや伝説の海賊王G・ロジャーの遺した『ひとつなぎの大秘宝』を巡って、幾人もの海賊達が戦っていた。そんな海賊に憧れる少年ルフィは、海賊王目指して大いなる旅に出る!!",
        "salesDate": "1997年12月24日",
        "itemPrice": 484,
        "listPrice": 0,
        "itemUrl": "https://books.rakuten.co.jp/rb/941204/",
        "smallImageUrl": "https://thumbnail.image.rakuten.co.jp/@0_mall/book/cabinet/5093/9784088725093_1_2.jpg?_ex=64x64",
        "availability": "1",
        "reviewCount": 755,
        "reviewAverage": "4.67",
        "booksGenreId": "001001001008"
      }
    }
  ],
  "GenreInformation": []
}
```

サンプルレスポンス全文は以下を参照:
- タイトル検索（`title=ワンピース`）: [`docs/api-samples/rakuten/search_by_title.json`](../api-samples/rakuten/search_by_title.json)
- ISBN検索（`isbn=9784088725093`）: [`docs/api-samples/rakuten/search_by_isbn.json`](../api-samples/rakuten/search_by_isbn.json)
- 著者名検索（`author=尾田栄一郎`）: [`docs/api-samples/rakuten/search_by_author.json`](../api-samples/rakuten/search_by_author.json)

---

## 使用上の注意・Tips
- `applicationId` と `accessKey` は両方必須。片方でも欠けると `400 wrong_parameter`（`specify valid applicationId` 等）が返る。
- Rakuten Developersのアプリ設定で「Allowed IP Addresses」を設定している場合、許可されていないIPからのアクセスは `403 CLIENT_IP_NOT_ALLOWED` になる。ローカル開発時は自マシンのグローバルIPを許可リストに登録する必要がある（プライベートIPは無効）。
- `format=json` を明示しないとXMLが返るため、JSONで受け取る場合は必ず指定する。
- レート制限あり。短時間に連続してリクエストすると `429 Rate limit is exceeded` が返るため、リトライ／バックオフを実装する。
- `hits` は最大30件までのため、全件取得には `page` によるページネーションが必要（`pageCount` で総ページ数を確認できる）。
- 検索条件（`title` / `author` / `publisherName` / `isbn` / `size` / `booksGenreId` など）はいずれか1つ以上を指定しないとエラーになる。
- 認証情報（`applicationId` / `accessKey`）は `.env` などのシークレット管理を利用し、リポジトリに直接含めない。

## 参考リンク
- [楽天ウェブサービス - Books書籍検索API](https://webservice.rakuten.co.jp/documentation/books-book-search)
- [楽天ウェブサービス - API Explorer](https://webservice.rakuten.co.jp/explorer/api/BooksBook/Search)
