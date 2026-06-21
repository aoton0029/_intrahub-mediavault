# API設計

## 基本方針
- OpenSearch（HTTP GET、RSS/XML）
- ベースURL: `https://ndlsearch.ndl.go.jp/api`
- 認証: なし（公開 OpenSearch）
- レスポンス形式: XML（RSS / OpenSearch）

## エンドポイント一覧
| Method | Path | 説明 | 認証 |
|--------|------|------|------|
| GET | /opensearch | 図書・雑誌記事・デジタルコレクション等の検索（OpenSearch, RSS/XML） | 不要 |
| GET | /opensearch?isbn={isbn}&dpid={dpid} | ISBNによる書誌データ取得 | 不要 |
| GET | /thumbnail/{isbn13}.jpg | 書影（表紙画像）取得（ISBN-13） | 不要 |

---

## GET /opensearch

### リクエスト
```http
GET https://ndlsearch.ndl.go.jp/api/opensearch?cnt={cnt}&title={title}&dpid=[{dpid}]
```

### パラメーター / バリデーション
- `title`: 任意（書籍タイトル）
- `isbn`: 任意（10桁または13桁）
- `cnt`: 取得件数（デフォルト20、最大100）
- `dpid`: 複数指定可（データプロバイダーID。例: `jpro`, `jpro-book`, `ndl-dl`, `ma-db`）
- `any`: 任意（任意キーワード）
- `creator`: 任意（著者名）
- `publisher`: 任意（出版社名）

### レスポンス（成功 200）
レスポンスは RSS/XML（OpenSearch 準拠）で返却されます。例：
```xml
<rss xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:openSearch="http://a9.com/-/spec/opensearchrss/1.0/" xmlns:dcndl="http://ndl.go.jp/dcndl/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" version="2.0">
<channel>
<title>SF - 国立国会図書館サーチ OpenSearch</title>
<link>https://ios-v2-prod-eks-alb.ndlsearch.ndl.go.jp/api/opensearch?title=SF&cnt=20</link>
<description>Search results for title=SF cnt=20</description>
<language>ja</language>
<openSearch:totalResults>34710</openSearch:totalResults>
<openSearch:startIndex>1</openSearch:startIndex>
<openSearch:itemsPerPage>20</openSearch:itemsPerPage>
<item>
<title>アークエンジェル・プロトコル</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000002-I000008308017</link>
<description>
<![CDATA[ <p>早川書房,2006,4-15-011581-8<p><ul><li>タイトル：アークエンジェル・プロトコル</li><li>タイトル（読み）：アーク エンジェル プロトコル</li><li>責任表示：ライダ・モアハウス 著,金子司 訳</li><li>シリーズ名：ハヤカワ文庫 ; SF</li><li>シリーズ名（読み）：ハヤカワ ブンコ</li><li>NDC(9)：933.7</li></ul> ]]>
</description>
<author>Morehouse, Lyda,金子, 司, 1968-,ライダ・モアハウス 著,金子司 訳</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000002-I000008308017</guid>
<pubDate>Fri, 13 Oct 2006 19:26:56 +0900</pubDate>
<dc:title>アークエンジェル・プロトコル</dc:title>
<dcndl:titleTranscription>アーク エンジェル プロトコル</dcndl:titleTranscription>
<dc:creator>Morehouse, Lyda</dc:creator>
<dc:creator>金子, 司, 1968-</dc:creator>
<dcndl:creatorTranscription>カネコ, ツカサ, 1968-</dcndl:creatorTranscription>
<dcndl:seriesTitle>ハヤカワ文庫 ; SF</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>ハヤカワ ブンコ</dcndl:seriesTitleTranscription>
<dc:publisher>早川書房</dc:publisher>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:date xsi:type="dcterms:W3CDTF">2006</dc:date>
<dcterms:issued>2006.9</dcterms:issued>
<dcndl:price>940円</dcndl:price>
<dc:extent>591p</dc:extent>
<dc:identifier xsi:type="dcndl:ISBN">4-15-011581-8</dc:identifier>
<dc:identifier xsi:type="dcndl:ISBN13">4-15-011581-8</dc:identifier>
<dc:identifier xsi:type="dcndl:NDLBibID">000008308017</dc:identifier>
<dc:identifier xsi:type="dcndl:JPNO">21108248</dc:identifier>
<dc:subject xsi:type="dcndl:NDLC">KS164</dc:subject>
<dc:subject xsi:type="dcndl:NDC9">933.7</dc:subject>
<dc:description>原タイトル: Archangel protocol</dc:description>
<rdfs:seeAlso rdf:resource="https://www.library.city.sapporo.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1006600358283"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.iwate.jp/opac/advanced-search"/>
<rdfs:seeAlso rdf:resource="https://www.lib.city.saitama.jp/bookdetail?num=2105389&ctg=1"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.chiba.jp/licsxp-opac/WOpacTifSchCmpdDispAction.do"/>
<rdfs:seeAlso rdf:resource="https://catalog.library.metro.tokyo.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1107000493"/>
<rdfs:seeAlso rdf:resource="https://opac.lib.city.yokohama.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1106066824"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.kawasaki.jp/bookdetail?num=529898&ctg=1"/>
<rdfs:seeAlso rdf:resource="https://www.toshokan.city.shizuoka.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1001100572813"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.nagoya.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1009916043755"/>
<rdfs:seeAlso rdf:resource="https://www.shiga-pref-library.jp/wo/opc_srh/srh_detail/1972536/"/>
<rdfs:seeAlso rdf:resource="https://www.oml.city.osaka.lg.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000011283642"/>
<rdfs:seeAlso rdf:resource="https://www.lib-sakai.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000600191445"/>
<rdfs:seeAlso rdf:resource="https://alislibopac.bunmori.tokushima.jp/winj/opac/switch-detail-iccap.do?bibid=1102383744"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.kagawa.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1100068256"/>
<rdfs:seeAlso rdf:resource="https://opac.miraionlibrary.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1009810945611"/>
<rdfs:seeAlso rdf:resource="https://library.pref.oita.jp/winj/opac/switch-detail-iccap.do?bibid=1100167680"/>
<rdfs:seeAlso rdf:resource="https://ndlsearch.ndl.go.jp/books/R100000002-I000008308017"/>
<rdfs:seeAlso rdf:resource="https://ci.nii.ac.jp/ncid/BC02295344"/>
<rdfs:seeAlso rdf:resource="https://www.books.or.jp/book-details/9784150115814"/>
<dc:description> 2006</dc:description>
</item>
<item>
<title>アーク遮断時を模擬したレーザ生成プラズマによる高温SF_6ガスの絶縁破壊電圧特性</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000136-I1572824500133021952</link>
<description>
<![CDATA[ <p>2000-09-07,<p><ul><li>タイトル：アーク遮断時を模擬したレーザ生成プラズマによる高温SF_6ガスの絶縁破壊電圧特性</li></ul> ]]>
</description>
<author>大塚 信也,長澤 暁,趙 孟佑,三浦 浩司,中村 道昭,匹田 政幸</author>
<category>記事</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000136-I1572824500133021952</guid>
<pubDate>Fri, 9 Jan 2026 11:13:45 +0900</pubDate>
<dc:title>アーク遮断時を模擬したレーザ生成プラズマによる高温SF_6ガスの絶縁破壊電圧特性</dc:title>
<dc:creator>大塚 信也</dc:creator>
<dc:creator>長澤 暁</dc:creator>
<dc:creator>趙 孟佑</dc:creator>
<dc:creator>三浦 浩司</dc:creator>
<dc:creator>中村 道昭</dc:creator>
<dc:creator>匹田 政幸</dc:creator>
<dc:date xsi:type="dcterms:W3CDTF">2000-09-07</dc:date>
<dcterms:issued>2000-09-07</dcterms:issued>
<dc:subject>SF_6 gas</dc:subject>
<rdfs:seeAlso rdf:resource="https://cir.nii.ac.jp/crid/1572824500133021952"/>
<dc:description>掲載誌：電気学会基礎・材料・共通部門大会講演論文集 = Proceeding of Annual Conference of Fundamentals and Materials Society, IEE Japan 2000-09-07 p.339-</dc:description>
</item>
<item>
<title>アークに対するSF6ガスの物理化学的特性</title>
<link>https://ndlsearch.ndl.go.jp/books/R000000004-I8456220</link>
<description>
<![CDATA[ <p><p><ul><li>タイトル：アークに対するSF6ガスの物理化学的特性</li><li>タイトル（読み）：アーク ニ タイスル SF6 ガス ノ モノ リカガクテキ トクセイ</li><li>責任表示：宮本 紀男 他</li><li>シリーズ名：SF6ガスシャ断器(特集)</li><li>シリーズ名（読み）：SF6 ガスシャダンキ トクシュウ</li></ul> ]]>
</description>
<author>宮本 紀男,宮本 紀男 他</author>
<category>記事</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R000000004-I8456220</guid>
<pubDate>Wed, 12 Apr 2023 00:20:10 +0900</pubDate>
<dc:title>アークに対するSF6ガスの物理化学的特性</dc:title>
<dcndl:titleTranscription>アーク ニ タイスル SF6 ガス ノ モノ リカガクテキ トクセイ</dcndl:titleTranscription>
<dc:creator>宮本 紀男</dc:creator>
<dcndl:seriesTitle>SF6ガスシャ断器(特集)</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>SF6 ガスシャダンキ トクシュウ</dcndl:seriesTitleTranscription>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:identifier xsi:type="dcndl:NDLBibID">8456220</dc:identifier>
<dc:subject xsi:type="dcndl:NDLC">ZN31</dc:subject>
<rdfs:seeAlso rdf:resource="https://ndlsearch.ndl.go.jp/books/R000000004-I8456220"/>
<rdfs:seeAlso rdf:resource="https://cir.nii.ac.jp/crid/1521980705328046848"/>
<dc:description>掲載誌：三菱電機技報 8 p.????</dc:description>
</item>
<item>
<title>アーサー王宮廷のヤンキー</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000001-I31111100115223</link>
<description>
<![CDATA[ <p>早川書房,1966,<p><ul><li>タイトル：アーサー王宮廷のヤンキー</li><li>タイトル（読み）：アーサーオウ　キユウテイノ　ヤンキー</li><li>責任表示：マーク・トウェイン 著 ; 小倉多加志 訳</li><li>シリーズ名：ハヤカワ・ＳＦ・シリーズ</li><li>シリーズ名（読み）：ﾊﾔｶﾜ ｴｽｴﾌ ｼﾘｰｽﾞ / ﾊﾔｶﾜ SF ｼﾘｰｽﾞ</li></ul> ]]>
</description>
<author>マーク・トウェイン,小倉多加志,マーク・トウェイン 著 ; 小倉多加志 訳</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000001-I31111100115223</guid>
<pubDate>Mon, 9 Nov 1992 00:00:00 +0900</pubDate>
<dc:title>アーサー王宮廷のヤンキー</dc:title>
<dcndl:titleTranscription>アーサーオウ　キユウテイノ　ヤンキー</dcndl:titleTranscription>
<dc:creator>マーク・トウェイン</dc:creator>
<dc:creator>小倉多加志</dc:creator>
<dcndl:creatorTranscription>トウエイン，マーク</dcndl:creatorTranscription>
<dcndl:creatorTranscription>オグラ，タカシ</dcndl:creatorTranscription>
<dcndl:seriesTitle>ハヤカワ・ＳＦ・シリーズ</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>ﾊﾔｶﾜ ｴｽｴﾌ ｼﾘｰｽﾞ / ﾊﾔｶﾜ SF ｼﾘｰｽﾞ</dcndl:seriesTitleTranscription>
<dc:publisher>早川書房</dc:publisher>
<dc:date xsi:type="dcterms:W3CDTF">1966</dc:date>
<dcterms:issued>196607</dcterms:issued>
<dcndl:price>３３０円</dcndl:price>
<dc:extent>２９０ｐ ; １９㎝</dc:extent>
<dc:subject xsi:type="dcndl:NDC8">933</dc:subject>
<rdfs:seeAlso rdf:resource="http://www.library.pref.tottori.jp/winj/opac/switch-detail-iccap.do?bibid=1100115223"/>
<dc:description> 1966</dc:description>
</item>
<item>
<title>アーサー王宮廷のヤンキー</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000002-I000001040440</link>
<description>
<![CDATA[ <p>早川書房,1963,<p><ul><li>タイトル：アーサー王宮廷のヤンキー</li><li>タイトル（読み）：アーサー オウ キュウテイ ノ ヤンキー</li><li>責任表示：マーク・トゥウェイン 著,小倉多加志 訳</li><li>シリーズ名：ハヤカワ・SF・シリーズ</li><li>シリーズ名（読み）：ハヤカワ SF シリーズ</li></ul> ]]>
</description>
<author>Twain, Mark, 1835-1910,小倉, 多加志, 1911-1991,マーク・トゥウェイン 著,小倉多加志 訳</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000002-I000001040440</guid>
<pubDate>Wed, 29 Mar 1995 09:00:00 +0900</pubDate>
<dc:title>アーサー王宮廷のヤンキー</dc:title>
<dcndl:titleTranscription>アーサー オウ キュウテイ ノ ヤンキー</dcndl:titleTranscription>
<dc:creator>Twain, Mark, 1835-1910</dc:creator>
<dc:creator>小倉, 多加志, 1911-1991</dc:creator>
<dcndl:creatorTranscription>オグラ, タカシ, 1911-1991</dcndl:creatorTranscription>
<dcndl:seriesTitle>ハヤカワ・SF・シリーズ</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>ハヤカワ SF シリーズ</dcndl:seriesTitleTranscription>
<dc:publisher>早川書房</dc:publisher>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:date xsi:type="dcterms:W3CDTF">1963</dc:date>
<dcterms:issued>1963</dcterms:issued>
<dc:extent>290p</dc:extent>
<dc:identifier xsi:type="dcndl:NDLBibID">000001040440</dc:identifier>
<dc:identifier xsi:type="dcndl:JPNO">63002439</dc:identifier>
<dc:subject xsi:type="dcndl:NDC">933</dc:subject>
<rdfs:seeAlso rdf:resource="https://opac.library.fcs.ed.jp/winj/opac/switch-detail-iccap.do?bibid=1100353320"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.chiba.lg.jp/licsxp-iopac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000000600125"/>
<rdfs:seeAlso rdf:resource="https://catalog.library.metro.tokyo.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1100656945"/>
<rdfs:seeAlso rdf:resource="http://www.library.pref.gifu.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1101333366"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.nagoya.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1009810023564"/>
<rdfs:seeAlso rdf:resource="https://www.oml.city.osaka.lg.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000080341440"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.kumamoto.jp/winj/opac/switch-detail-iccap.do?bibid=1106163874"/>
<rdfs:seeAlso rdf:resource="https://ndlsearch.ndl.go.jp/books/R100000002-I000001040440"/>
<rdfs:seeAlso rdf:resource="https://dl.ndl.go.jp/pid/1697499"/>
<rdfs:seeAlso rdf:resource="https://ci.nii.ac.jp/ncid/BA88692401"/>
<dc:description> 1963</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000001-I07111100270941</link>
<description>
<![CDATA[ <p>岩崎書店,1975,<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：ア－サ－　オウ　ト　アツタ　オトコ</li><li>責任表示：マーク・トウェーン 著 ; 亀山龍樹 訳</li><li>シリーズ名：ＳＦ少年文庫　７</li><li>シリーズ名（読み）：エスエフ　シヨウネン　ブンコ　７</li></ul> ]]>
</description>
<author>マーク・トウェーン,亀山, 龍樹,マーク・トウェーン 著 ; 亀山龍樹 訳</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000001-I07111100270941</guid>
<pubDate>Thu, 13 Feb 2020 15:00:13 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>ア－サ－　オウ　ト　アツタ　オトコ</dcndl:titleTranscription>
<dc:creator>マーク・トウェーン</dc:creator>
<dc:creator>亀山, 龍樹</dc:creator>
<dcndl:creatorTranscription>トウエイン, マ－ク</dcndl:creatorTranscription>
<dcndl:creatorTranscription>カメヤマ, タツキ</dcndl:creatorTranscription>
<dcndl:seriesTitle>ＳＦ少年文庫　７</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>エスエフ　シヨウネン　ブンコ　７</dcndl:seriesTitleTranscription>
<dc:publisher>岩崎書店</dc:publisher>
<dc:date xsi:type="dcterms:W3CDTF">1975</dc:date>
<dcterms:issued>１９７５</dcterms:issued>
<dc:extent>２５４Ｐ ; １８ｃｍ</dc:extent>
<dc:identifier xsi:type="dcndl:OPLMARCNO">40003220</dc:identifier>
<rdfs:seeAlso rdf:resource="https://opac.library.fcs.ed.jp/winj/opac/switch-detail-iccap.do?bibid=1100270941"/>
<dc:description> 1975</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000001-I10111100313647</link>
<description>
<![CDATA[ <p>岩崎書店,1978,<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：アーサーオウ　ト　アッタ　オトコ</li><li>責任表示：マーク・トウェイン 著 ; 亀山竜樹 訳</li><li>シリーズ名：SF少年文庫 ; 7</li><li>シリーズ名（読み）：エスエフ　ショウネン　ブンコ ; ７</li><li>NDC(9)：933.6</li></ul> ]]>
</description>
<author>マーク・トウェイン,亀山, 竜樹,マーク・トウェイン 著 ; 亀山竜樹 訳</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000001-I10111100313647</guid>
<pubDate>Mon, 1 Jan 1900 20:00:00 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>アーサーオウ　ト　アッタ　オトコ</dcndl:titleTranscription>
<dc:creator>マーク・トウェイン</dc:creator>
<dc:creator>亀山, 竜樹</dc:creator>
<dcndl:creatorTranscription>トウェーン, マーク</dcndl:creatorTranscription>
<dcndl:creatorTranscription>カメヤマ, タツキ</dcndl:creatorTranscription>
<dcndl:seriesTitle>SF少年文庫 ; 7</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>エスエフ　ショウネン　ブンコ ; ７</dcndl:seriesTitleTranscription>
<dc:publisher>岩崎書店</dc:publisher>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:date xsi:type="dcterms:W3CDTF">1978</dc:date>
<dcterms:issued>1978</dcterms:issued>
<dcndl:price>\880</dcndl:price>
<dc:extent>254p ; 19cm</dc:extent>
<dc:identifier xsi:type="dcndl:TRCMARCNO">76-01110</dc:identifier>
<dc:subject xsi:type="dcndl:NDC9">933.6</dc:subject>
<dc:subject xsi:type="dcndl:NDC8">933</dc:subject>
<rdfs:seeAlso rdf:resource="http://www.library.pref.hokkaido.jp/wo/opc/srh/"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.sapporo.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1001000457090"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.iwate.jp/opac/advanced-search"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.miyagi.jp/wo/opc_srh/srh_detail/5010031574/"/>
<rdfs:seeAlso rdf:resource="https://www1.library.pref.gunma.jp/winj/opac/switch-detail-iccap.do?bibid=1100313647"/>
<rdfs:seeAlso rdf:resource="https://www.lib.city.saitama.jp/bookdetail?num=1579847&ctg=1"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.chiba.jp/licsxp-opac/WOpacTifSchCmpdDispAction.do"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.kawasaki.jp/bookdetail?num=714343&ctg=1"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.ishikawa.lg.jp/wo/opc_srh/srh_detail/1009910126941/"/>
<rdfs:seeAlso rdf:resource="https://www.toshokan.city.shizuoka.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1001100724219"/>
<rdfs:seeAlso rdf:resource="http://opac1.library.pref.mie.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1102877302"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.osaka.jp/bib/?B10415823"/>
<rdfs:seeAlso rdf:resource="https://www.oml.city.osaka.lg.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000070030656"/>
<rdfs:seeAlso rdf:resource="https://www.lib-sakai.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000000507632"/>
<rdfs:seeAlso rdf:resource="http://www.lib.wakayama-c.ed.jp/winj/opac/switch-detail-iccap.do?bibid=1100087842"/>
<rdfs:seeAlso rdf:resource="https://www2.library.pref.shimane.lg.jp/opac/switch-detail-iccap.do?bibid=1130830147"/>
<rdfs:seeAlso rdf:resource="https://www2.hplibra.pref.hiroshima.jp/bib/?B11220583"/>
<rdfs:seeAlso rdf:resource="http://www.library.city.hiroshima.jp/winj/opac/switch-detail-iccap.do?bibid=1100632846"/>
<rdfs:seeAlso rdf:resource="https://opac.library.kochi.jp/winj/opac/switch-detail-iccap.do?bibid=1108147369"/>
<rdfs:seeAlso rdf:resource="https://opac.toshokan.city.fukuoka.lg.jp/licsxp-opac/WOpacTifSchCmpdDispAction.do"/>
<rdfs:seeAlso rdf:resource="https://www2.tosyo-saga.jp/kentosyo2/opac/switch-detail-iccap.do?bibid=1100849850"/>
<rdfs:seeAlso rdf:resource="https://opac.miraionlibrary.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000001031562"/>
<rdfs:seeAlso rdf:resource="https://www2.library.pref.kagoshima.jp/kento/opac/switch-detail-iccap.do?bibid=1132192327"/>
<dc:description> 1978</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000001-I18111103072303</link>
<description>
<![CDATA[ <p>岩崎書店,1982,<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：アーサー オウ ト アッタ オトコ</li><li>責任表示：マーク・トウェーン 作,亀山竜樹 訳</li><li>シリーズ名：SF少年文庫 ; 7</li><li>シリーズ名（読み）：エスエフ ショウネン ブンコ ; 7</li><li>NDC(9)：933</li></ul> ]]>
</description>
<author>亀山 竜樹,カメヤマ,タツキ,マーク・トウェーン 作,亀山竜樹 訳</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000001-I18111103072303</guid>
<pubDate>Sun, 3 Nov 2002 09:00:00 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>アーサー オウ ト アッタ オトコ</dcndl:titleTranscription>
<dc:creator>亀山 竜樹</dc:creator>
<dc:creator>カメヤマ,タツキ</dc:creator>
<dcndl:creatorTranscription>トウェーン,マーク</dcndl:creatorTranscription>
<dcndl:creatorTranscription>カメヤマ,タツキ</dcndl:creatorTranscription>
<dcndl:seriesTitle>SF少年文庫 ; 7</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>エスエフ ショウネン ブンコ ; 7</dcndl:seriesTitleTranscription>
<dc:publisher>岩崎書店</dc:publisher>
<dc:date xsi:type="dcterms:W3CDTF">1982</dc:date>
<dcndl:price>880円</dcndl:price>
<dc:extent>254p ; 19cm</dc:extent>
<dc:identifier xsi:type="dcndl:NSMARCNO">825276900</dc:identifier>
<dc:subject xsi:type="dcndl:NDC9">933</dc:subject>
<dc:subject xsi:type="dcndl:NDC8">933</dc:subject>
<dc:description>解説：亀山竜樹</dc:description>
<rdfs:seeAlso rdf:resource="https://www.lib.pref.saitama.jp/winj/opac/switch-detail-iccap.do?bibid=1115348918"/>
<rdfs:seeAlso rdf:resource="https://www.lib.sagamihara.kanagawa.jp/TOSHOW/asp/WwShousaiKen.aspx?FCode=613182"/>
<rdfs:seeAlso rdf:resource="https://www.library-archives.pref.fukui.lg.jp/wo/opc_srh/srh_detail/1103072303/"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.nagoya.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1009210132333"/>
<dc:description> 1982</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000001-I2611B10192930</link>
<description>
<![CDATA[ <p>岩崎書店,1973,<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：アーサーオウ ト アツタ オトコ</li><li>シリーズ名：SF少年文庫 7</li><li>シリーズ名（読み）：エスエフ ショウネン ブンコ 7</li></ul> ]]>
</description>
<author>マーク・トウェーン／作</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000001-I2611B10192930</guid>
<pubDate>Fri, 1 Jan 2010 00:00:00 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>アーサーオウ ト アツタ オトコ</dcndl:titleTranscription>
<dc:creator>マーク・トウェーン／作</dc:creator>
<dcndl:creatorTranscription>Mark Twain</dcndl:creatorTranscription>
<dcndl:seriesTitle>SF少年文庫 7</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>エスエフ ショウネン ブンコ 7</dcndl:seriesTitleTranscription>
<dc:publisher>岩崎書店</dc:publisher>
<dc:date xsi:type="dcterms:W3CDTF">1973</dc:date>
<dcterms:issued>1973</dcterms:issued>
<dc:extent>254p</dc:extent>
<rdfs:seeAlso rdf:resource="https://www.library.pref.kyoto.jp/bib/?B10192930"/>
<dc:description> 1973</dc:description>
</item>
<item>
<title>アーサー王とあった男　初版</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000001-I43111104098790</link>
<description>
<![CDATA[ <p>岩崎書店,1971,<p><ul><li>タイトル：アーサー王とあった男　初版</li><li>タイトル（読み）：ア－サ－オウトアツタオトコ</li><li>責任表示：マーク＊トウェーン 著 ; 亀山龍樹 訳 ; Ｄ．Ｎ＊ベアード さし絵</li><li>シリーズ名：ＳＦ少年文庫・７</li><li>シリーズ名（読み）：エスエフシヨウネンブンコ　０００７</li></ul> ]]>
</description>
<author>マーク＊トウェーン,亀山, 龍樹,Ｄ．Ｎ＊ベアード,マーク＊トウェーン 著 ; 亀山龍樹 訳 ; Ｄ．Ｎ＊ベアード さし絵</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000001-I43111104098790</guid>
<pubDate>Thu, 28 Apr 2022 12:00:00 +0900</pubDate>
<dc:title>アーサー王とあった男　初版</dc:title>
<dcndl:titleTranscription>ア－サ－オウトアツタオトコ</dcndl:titleTranscription>
<dc:creator>マーク＊トウェーン</dc:creator>
<dc:creator>亀山, 龍樹</dc:creator>
<dc:creator>Ｄ．Ｎ＊ベアード</dc:creator>
<dcndl:creatorTranscription>マ－ク，トウエ－ン</dcndl:creatorTranscription>
<dcndl:creatorTranscription>カメヤマ，タツキ</dcndl:creatorTranscription>
<dcndl:creatorTranscription>Ｄ．Ｎ，ベア－ド</dcndl:creatorTranscription>
<dcndl:seriesTitle>ＳＦ少年文庫・７</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>エスエフシヨウネンブンコ　０００７</dcndl:seriesTitleTranscription>
<dc:publisher>岩崎書店</dc:publisher>
<dc:date xsi:type="dcterms:W3CDTF">1971</dc:date>
<dcterms:issued>１９７１年</dcterms:issued>
<dc:extent>２５４ ; １９</dc:extent>
<rdfs:seeAlso rdf:resource="https://www.library.pref.kumamoto.jp/winj/opac/switch-detail-iccap.do?bibid=1104098790"/>
<dc:description> 1971</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000134-I000305998</link>
<description>
<![CDATA[ <p>岩波書店,1971-01-25,<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：ｱｰｻｰｵｳ ﾄ ｱｯﾀ ｵﾄｺ</li><li>責任表示：亀山竜樹</li><li>シリーズ名：ＳＦ少年文庫 ; ７</li><li>シリーズ名（読み）：ｴｽｴﾌ ｼｮｳﾈﾝ ﾌﾞﾝｺ ; 7</li></ul> ]]>
</description>
<author>亀山竜樹</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000134-I000305998</guid>
<pubDate>Thu, 24 Aug 2023 09:00:00 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>ｱｰｻｰｵｳ ﾄ ｱｯﾀ ｵﾄｺ</dcndl:titleTranscription>
<dcndl:seriesTitle>ＳＦ少年文庫 ; ７</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>ｴｽｴﾌ ｼｮｳﾈﾝ ﾌﾞﾝｺ ; 7</dcndl:seriesTitleTranscription>
<dc:publisher>岩波書店</dc:publisher>
<dc:date xsi:type="dcterms:W3CDTF">1971-01-25</dc:date>
<dcterms:issued>1971.1.25</dcterms:issued>
<rdfs:seeAlso rdf:resource="https://www.kanabun.or.jp/CARIN/CARINOPACLINK.HTM?ID=B00118120"/>
<dc:description> 1971-01-25</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000038-I821700</link>
<description>
<![CDATA[ <p>岩崎書店,1986,<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：アーサーオウ　ト　アッタ　オトコ</li><li>責任表示：マーク・トウェーン 作,亀山竜樹 訳</li><li>シリーズ名：ＳＦロマン文庫</li><li>シリーズ名（読み）：エスエフ　ロマン　ブンコ</li></ul> ]]>
</description>
<author>Ｔｗａｉｎ，Ｍａｒｋ,亀山　竜樹,マーク・トウェーン 作,亀山竜樹 訳</author>
<category>図書</category>
<category>記録メディア</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000038-I821700</guid>
<pubDate>Wed, 12 Mar 2014 09:00:00 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>アーサーオウ　ト　アッタ　オトコ</dcndl:titleTranscription>
<dc:creator>Ｔｗａｉｎ，Ｍａｒｋ</dc:creator>
<dc:creator>亀山　竜樹</dc:creator>
<dcndl:creatorTranscription>トウェーン，マーク</dcndl:creatorTranscription>
<dcndl:creatorTranscription>カメヤマ，タツキ</dcndl:creatorTranscription>
<dcndl:seriesTitle>ＳＦロマン文庫</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>エスエフ　ロマン　ブンコ</dcndl:seriesTitleTranscription>
<dc:publisher>岩崎書店</dc:publisher>
<dc:date xsi:type="dcterms:W3CDTF">1986</dc:date>
<dcterms:issued>1986年</dcterms:issued>
<dc:extent>4巻 (5時間0分)</dc:extent>
<dc:subject xsi:type="dcndl:NDC">900</dc:subject>
<rdfs:seeAlso rdf:resource="https://library.sapie.or.jp/cgi-bin/CN1MN1?S00101=J00DTL04&S00222=821700"/>
<dc:description> 1986</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000038-I959914</link>
<description>
<![CDATA[ <p>岩崎書店,1986,<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：アーサー　オウ　ト　アッタ　オトコ</li><li>責任表示：マーク・トウェーン 作,亀山龍樹 訳</li><li>シリーズ名：ＳＦロマン文庫　７ :</li><li>シリーズ名（読み）：エスエフ　ロマン　ブンコ : ＳＦ　ロマン　ブンコ</li></ul> ]]>
</description>
<author>Ｔｗａｉｎ，Ｍａｒｋ,亀山　龍樹,マーク・トウェーン 作,亀山龍樹 訳</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000038-I959914</guid>
<pubDate>Sat, 17 Dec 2005 09:00:00 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>アーサー　オウ　ト　アッタ　オトコ</dcndl:titleTranscription>
<dc:creator>Ｔｗａｉｎ，Ｍａｒｋ</dc:creator>
<dc:creator>亀山　龍樹</dc:creator>
<dcndl:creatorTranscription>トウェーン，マーク</dcndl:creatorTranscription>
<dcndl:creatorTranscription>カメヤマ，タツキ</dcndl:creatorTranscription>
<dcndl:seriesTitle>ＳＦロマン文庫　７ :</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>エスエフ　ロマン　ブンコ : ＳＦ　ロマン　ブンコ</dcndl:seriesTitleTranscription>
<dc:publisher>岩崎書店</dc:publisher>
<dc:date xsi:type="dcterms:W3CDTF">1986</dc:date>
<dcterms:issued>1986年</dcterms:issued>
<dc:extent>3巻</dc:extent>
<dc:subject xsi:type="dcndl:NDC">933</dc:subject>
<rdfs:seeAlso rdf:resource="https://library.sapie.or.jp/cgi-bin/CN1MN1?S00101=J00DTL04&S00222=959914"/>
<dc:description> 1986</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000002-I000000797986</link>
<description>
<![CDATA[ <p>岩崎書店,1971,<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：アーサー オウ ト アッタ オトコ</li><li>責任表示：マーク・トウェーン 作,亀山竜樹 訳,ベアード 絵</li><li>シリーズ名：SF少年文庫 ; 7</li><li>シリーズ名（読み）：SF ショウネン ブンコ</li></ul> ]]>
</description>
<author>Twain, Mark, 1835-1910,Beard, Daniel Carter, 1850-1941,亀山, 竜樹, 1922-1980,マーク・トウェーン 作,亀山竜樹 訳,ベアード 絵</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000002-I000000797986</guid>
<pubDate>Mon, 15 Aug 2022 21:10:08 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>アーサー オウ ト アッタ オトコ</dcndl:titleTranscription>
<dc:creator>Twain, Mark, 1835-1910</dc:creator>
<dc:creator>Beard, Daniel Carter, 1850-1941</dc:creator>
<dc:creator>亀山, 竜樹, 1922-1980</dc:creator>
<dcndl:creatorTranscription>カメヤマ, タツキ, 1922-1980</dcndl:creatorTranscription>
<dcndl:seriesTitle>SF少年文庫 ; 7</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>SF ショウネン ブンコ</dcndl:seriesTitleTranscription>
<dc:publisher>岩崎書店</dc:publisher>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:date xsi:type="dcterms:W3CDTF">1971</dc:date>
<dcterms:issued>1971</dcterms:issued>
<dc:extent>254p</dc:extent>
<dc:identifier xsi:type="dcndl:NDLBibID">000000797986</dc:identifier>
<dc:identifier xsi:type="dcndl:JPNO">45005453</dc:identifier>
<dcndl:genre>児童図書</dcndl:genre>
<dcndl:genreTranscription>ジドウ トショ</dcndl:genreTranscription>
<rdfs:seeAlso rdf:resource="https://catalog.library.metro.tokyo.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1100507406"/>
<rdfs:seeAlso rdf:resource="https://opacsvr01.library.pref.nara.jp/opac/volume/178091"/>
<rdfs:seeAlso rdf:resource="https://ndlsearch.ndl.go.jp/books/R100000002-I000000797986"/>
<rdfs:seeAlso rdf:resource="https://dl.ndl.go.jp/pid/12931216"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.osaka.jp/bibj/?B16294580"/>
<rdfs:seeAlso rdf:resource="https://ci.nii.ac.jp/ncid/BA32786511"/>
<dc:description> 1971</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000002-I000007927337</link>
<description>
<![CDATA[ <p>岩崎書店,2005,4-265-04651-7<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：アーサー オウ ト アッタ オトコ</li><li>責任表示：マーク・トウェーン 原作,亀山龍樹 訳</li><li>シリーズ名：SF名作コレクション ; 1</li><li>シリーズ名（読み）：SF メイサク コレクション</li><li>NDC(9)：933.6</li></ul> ]]>
</description>
<author>Twain, Mark, 1835-1910,亀山, 竜樹, 1922-1980,マーク・トウェーン 原作,亀山龍樹 訳</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000002-I000007927337</guid>
<pubDate>Tue, 19 Jul 2022 20:28:11 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>アーサー オウ ト アッタ オトコ</dcndl:titleTranscription>
<dc:creator>Twain, Mark, 1835-1910</dc:creator>
<dc:creator>亀山, 竜樹, 1922-1980</dc:creator>
<dcndl:creatorTranscription>カメヤマ, タツキ, 1922-1980</dcndl:creatorTranscription>
<dcndl:seriesTitle>SF名作コレクション ; 1</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>SF メイサク コレクション</dcndl:seriesTitleTranscription>
<dc:publisher>岩崎書店</dc:publisher>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:date xsi:type="dcterms:W3CDTF">2005</dc:date>
<dcterms:issued>2005.10</dcterms:issued>
<dcndl:price>1500円</dcndl:price>
<dc:extent>239p</dc:extent>
<dc:identifier xsi:type="dcndl:ISBN">4-265-04651-7</dc:identifier>
<dc:identifier xsi:type="dcndl:ISBN13">4-265-04651-7</dc:identifier>
<dc:identifier xsi:type="dcndl:NDLBibID">000007927337</dc:identifier>
<dc:identifier xsi:type="dcndl:JPNO">20899885</dc:identifier>
<dc:subject>SF--小説集</dc:subject>
<dcndl:genre>児童図書</dcndl:genre>
<dcndl:genreTranscription>ジドウ トショ</dcndl:genreTranscription>
<dc:subject xsi:type="dcndl:NDLC">Y9</dc:subject>
<dc:subject xsi:type="dcndl:NDC9">933.6</dc:subject>
<dc:description>絵: D.N.ベアード</dc:description>
<dc:description>原タイトル: A connecticut yankee in King Arthur's court</dc:description>
<rdfs:seeAlso rdf:resource="https://www.library.city.sapporo.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1006500263226"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.iwate.jp/opac/advanced-search"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.miyagi.jp/wo/opc_srh/srh_detail/1000477041/"/>
<rdfs:seeAlso rdf:resource="https://www.apl.pref.akita.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000000000719"/>
<rdfs:seeAlso rdf:resource="https://opac.library.fcs.ed.jp/winj/opac/switch-detail-iccap.do?bibid=1102216670"/>
<rdfs:seeAlso rdf:resource="https://www.lib.pref.ibaraki.jp/licsxp-kopac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1001001259072"/>
<rdfs:seeAlso rdf:resource="https://www.lib.pref.saitama.jp//winj/opac/switch-detail-iccap.do?bibid=1197492960"/>
<rdfs:seeAlso rdf:resource="https://www.lib.city.saitama.jp/bookdetail?num=2052485&ctg=1"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.chiba.lg.jp//licsxp-iopac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000000690880"/>
<rdfs:seeAlso rdf:resource="https://catalog.library.metro.tokyo.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1106647584"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.kawasaki.jp/bookdetail?num=432300&ctg=1"/>
<rdfs:seeAlso rdf:resource="https://opac.pref-lib.niigata.niigata.jp/winj/opac/switch-detail-iccap.do?bibid=1106459730"/>
<rdfs:seeAlso rdf:resource="https://lib2.lib.pref.toyama.jp/opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1009810580143"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.ishikawa.lg.jp/wo/opc_srh/srh_detail/1009910667865/"/>
<rdfs:seeAlso rdf:resource="https://www.library-archives.pref.fukui.lg.jp/wo/opc_srh/srh_detail/1104541105/"/>
<rdfs:seeAlso rdf:resource="https://www.lib.pref.yamanashi.jp/licsxp-opac/WOpacMnuTopInitAction.do"/>
<rdfs:seeAlso rdf:resource="http://www.library.pref.gifu.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1100111058"/>
<rdfs:seeAlso rdf:resource="https://www.tosyokan.pref.shizuoka.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000510182072"/>
<rdfs:seeAlso rdf:resource="https://www.toshokan.city.shizuoka.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1001100484714"/>
<rdfs:seeAlso rdf:resource="https://www.shiga-pref-library.jp/wo/opc_srh/srh_detail/1877061/"/>
<rdfs:seeAlso rdf:resource="https://www.oml.city.osaka.lg.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000011065270"/>
<rdfs:seeAlso rdf:resource="https://www.lib-sakai.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000500095768"/>
<rdfs:seeAlso rdf:resource="http://www.lib.wakayama-c.ed.jp/winj/opac/switch-detail-iccap.do?bibid=1100487553"/>
<rdfs:seeAlso rdf:resource="http://www.library.pref.tottori.jp/winj/opac/switch-detail-iccap.do?bibid=1101268454"/>
<rdfs:seeAlso rdf:resource="https://www2.library.pref.shimane.lg.jp/opac/switch-detail-iccap.do?bibid=1100972234"/>
<rdfs:seeAlso rdf:resource="https://opac.libnet.pref.okayama.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1009810789386"/>
<rdfs:seeAlso rdf:resource="http://www.library.city.hiroshima.jp/winj/opac/switch-detail-iccap.do?bibid=1102723289"/>
<rdfs:seeAlso rdf:resource="https://library.pref.yamaguchi.lg.jp//wo/opc_srh/srh_detail/1000262094/"/>
<rdfs:seeAlso rdf:resource="https://alislibopac.bunmori.tokushima.jp/winj/opac/switch-detail-iccap.do?bibid=1102146523"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.kagawa.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1108642680"/>
<rdfs:seeAlso rdf:resource="http://www.lib.pref.fukuoka.jp/winj/opac/switch-detail-iccap.do?bibid=1106230045"/>
<rdfs:seeAlso rdf:resource="https://opac.toshokan.city.fukuoka.lg.jp/licsxp-opac/WOpacTifSchCmpdDispAction.do"/>
<rdfs:seeAlso rdf:resource="https://opac.miraionlibrary.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1009810840982"/>
<rdfs:seeAlso rdf:resource="https://library.pref.oita.jp/winj/opac/switch-detail-iccap.do?bibid=1100070705"/>
<rdfs:seeAlso rdf:resource="https://ndlsearch.ndl.go.jp/books/R100000002-I000007927337"/>
<rdfs:seeAlso rdf:resource="https://library.sapie.or.jp/cgi-bin/CN1MN1?S00101=J00DTL04&S00222=4744319"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.osaka.jp/bibj/?B16450086"/>
<rdfs:seeAlso rdf:resource="https://ci.nii.ac.jp/ncid/BB12156019"/>
<rdfs:seeAlso rdf:resource="https://www.books.or.jp/book-details/9784265046515"/>
<dc:description> 2005</dc:description>
</item>
<item>
<title>アーサー王とあった男</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000002-I000001785041</link>
<description>
<![CDATA[ <p>岩崎書店,1986,4-265-01507-7<p><ul><li>タイトル：アーサー王とあった男</li><li>タイトル（読み）：アーサーオウ ト アッタ オトコ</li><li>責任表示：マーク・トウェーン 作,亀山竜樹 訳</li><li>シリーズ名：SFロマン文庫</li><li>NDC(9)：933</li></ul> ]]>
</description>
<author>Twain, Mark, 1835-1910,亀山, 竜樹, 1922-1980,マーク・トウェーン 作,亀山竜樹 訳</author>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000002-I000001785041</guid>
<pubDate>Tue, 11 Oct 2022 21:34:41 +0900</pubDate>
<dc:title>アーサー王とあった男</dc:title>
<dcndl:titleTranscription>アーサーオウ ト アッタ オトコ</dcndl:titleTranscription>
<dc:creator>Twain, Mark, 1835-1910</dc:creator>
<dc:creator>亀山, 竜樹, 1922-1980</dc:creator>
<dcndl:creatorTranscription>カメヤマ, タツキ, 1922-1980</dcndl:creatorTranscription>
<dcndl:seriesTitle>SFロマン文庫</dcndl:seriesTitle>
<dc:publisher>岩崎書店</dc:publisher>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:date xsi:type="dcterms:W3CDTF">1986</dc:date>
<dcterms:issued>1986.1</dcterms:issued>
<dcndl:price>680円</dcndl:price>
<dc:extent>254p</dc:extent>
<dc:identifier xsi:type="dcndl:ISBN">4-265-01507-7</dc:identifier>
<dc:identifier xsi:type="dcndl:ISBN13">4-265-01507-7</dc:identifier>
<dc:identifier xsi:type="dcndl:NDLBibID">000001785041</dc:identifier>
<dc:identifier xsi:type="dcndl:JPNO">86023880</dc:identifier>
<dcndl:genre>児童図書</dcndl:genre>
<dcndl:genreTranscription>ジドウ トショ</dcndl:genreTranscription>
<dc:subject xsi:type="dcndl:NDC9">933</dc:subject>
<rdfs:seeAlso rdf:resource="https://www.library.city.sapporo.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1001000119521"/>
<rdfs:seeAlso rdf:resource="https://www.lib.city.saitama.jp/bookdetail?num=634819&ctg=1"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.chiba.jp/licsxp-opac/WOpacTifSchCmpdDispAction.do"/>
<rdfs:seeAlso rdf:resource="https://www.lib.sagamihara.kanagawa.jp/TOSHOW/asp/WwShousaiKen.aspx?FCode=134856"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.ishikawa.lg.jp/wo/opc_srh/srh_detail/1005010163276/"/>
<rdfs:seeAlso rdf:resource="https://www.library-archives.pref.fukui.lg.jp/wo/opc_srh/srh_detail/1103177074/"/>
<rdfs:seeAlso rdf:resource="https://www.toshokan.city.shizuoka.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1001100844551"/>
<rdfs:seeAlso rdf:resource="https://www.library.city.nagoya.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1009410040943"/>
<rdfs:seeAlso rdf:resource="https://www.shiga-pref-library.jp/wo/opc_srh/srh_detail/0231363/"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.osaka.jp/bib/?B10437079"/>
<rdfs:seeAlso rdf:resource="https://www.oml.city.osaka.lg.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000070044554"/>
<rdfs:seeAlso rdf:resource="https://www.lib-sakai.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1000000819598"/>
<rdfs:seeAlso rdf:resource="https://opac.libnet.pref.okayama.jp/licsxp-opac/WOpacMsgNewListToTifTilDetailAction.do?tilcod=1009810650056"/>
<rdfs:seeAlso rdf:resource="https://www2.hplibra.pref.hiroshima.jp/bib/?B11128190"/>
<rdfs:seeAlso rdf:resource="https://alislibopac.bunmori.tokushima.jp/winj/opac/switch-detail-iccap.do?bibid=1100309378"/>
<rdfs:seeAlso rdf:resource="https://www.ehimetosyokan.jp/winj/opac/switch-detail-iccap.do?bibid=1100381922"/>
<rdfs:seeAlso rdf:resource="https://opac.toshokan.city.fukuoka.lg.jp/licsxp-opac/WOpacTifSchCmpdDispAction.do"/>
<rdfs:seeAlso rdf:resource="https://www.lib.pref.miyazaki.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1100908351"/>
<rdfs:seeAlso rdf:resource="https://www2.library.pref.kagoshima.jp/kento/opac/switch-detail-iccap.do?bibid=1132264513"/>
<rdfs:seeAlso rdf:resource="https://ndlsearch.ndl.go.jp/books/R100000002-I000001785041"/>
<rdfs:seeAlso rdf:resource="https://dl.ndl.go.jp/pid/13800627"/>
<rdfs:seeAlso rdf:resource="https://www.library.pref.osaka.jp/bibj/?B16341830"/>
<rdfs:seeAlso rdf:resource="https://www.books.or.jp/book-details/9784265015078"/>
<dc:description> 1986</dc:description>
</item>
<item>
<title>アーサー・C.クラーク--神なき人間への愛</title>
<link>https://ndlsearch.ndl.go.jp/books/R000000004-I2406981</link>
<description>
<![CDATA[ <p><p><ul><li>タイトル：アーサー・C.クラーク--神なき人間への愛</li><li>タイトル（読み）：アーサー C クラーク カミ ナキ ニンゲン エ ノ アイ</li><li>責任表示：富山 太佳夫</li><li>シリーズ名：現代文学・SFの衝撃<特集> ; SF・その現代の古典</li><li>シリーズ名（読み）：ゲンダイ ブンガク SF ノ ショウゲキ トクシュウ ; SF ソノ ゲンダイ ノ コテン</li></ul> ]]>
</description>
<author>富山 太佳夫,富山 太佳夫</author>
<category>記事</category>
<category>デジタル</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R000000004-I2406981</guid>
<pubDate>Thu, 4 Feb 1999 13:30:33 +0900</pubDate>
<dc:title>アーサー・C.クラーク--神なき人間への愛</dc:title>
<dcndl:titleTranscription>アーサー C クラーク カミ ナキ ニンゲン エ ノ アイ</dcndl:titleTranscription>
<dc:creator>富山 太佳夫</dc:creator>
<dcndl:seriesTitle>現代文学・SFの衝撃<特集> ; SF・その現代の古典</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>ゲンダイ ブンガク SF ノ ショウゲキ トクシュウ ; SF ソノ ゲンダイ ノ コテン</dcndl:seriesTitleTranscription>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:identifier xsi:type="dcndl:NDLBibID">2406981</dc:identifier>
<dc:subject xsi:type="dcndl:NDLC">ZK22</dc:subject>
<rdfs:seeAlso rdf:resource="https://ndlsearch.ndl.go.jp/books/R000000004-I2406981"/>
<rdfs:seeAlso rdf:resource="https://cir.nii.ac.jp/crid/1523951029598748032"/>
<dc:description>掲載誌：國文學 : 解釈と教材の研究 / 學燈社 [編] 11 p.p96～98</dc:description>
</item>
<item>
<title>アーサー・Ｃ・クラーク特集</title>
<link>https://ndlsearch.ndl.go.jp/books/R100000001-I45111100479587</link>
<description>
<![CDATA[ <p>早川書房,2001,<p><ul><li>タイトル：アーサー・Ｃ・クラーク特集</li><li>タイトル（読み）：アーサー　Ｃ　クラーク　トクシュウ　ＳＦ　マガジン　エスエフ　マガジン</li><li>NDC(9)：905</li></ul> ]]>
</description>
<author/>
<category>図書</category>
<category>紙</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R100000001-I45111100479587</guid>
<pubDate>Thu, 16 Jul 2015 09:00:00 +0900</pubDate>
<dc:title>アーサー・Ｃ・クラーク特集</dc:title>
<dcndl:titleTranscription>アーサー　Ｃ　クラーク　トクシュウ　ＳＦ　マガジン　エスエフ　マガジン</dcndl:titleTranscription>
<dc:publisher>早川書房</dc:publisher>
<dc:date xsi:type="dcterms:W3CDTF">2001</dc:date>
<dcterms:issued>2001.5</dcterms:issued>
<dcndl:price>８４８円</dcndl:price>
<dc:extent>２５６ｐ ; ２１ｃｍ</dc:extent>
<dc:subject xsi:type="dcndl:NDC9">905</dc:subject>
<dc:subject xsi:type="dcndl:NDC8">905</dc:subject>
<dc:description>『Ｓ－Ｆ　マガジン』２００１年５月１日（第４２巻　第５号）</dc:description>
<rdfs:seeAlso rdf:resource="https://www.lib.pref.miyazaki.lg.jp/winj/opac/switch-detail-iccap.do?bibid=1100479587"/>
<dc:description> 2001</dc:description>
</item>
<item>
<title>アーサー・C・クラークのSFを読んで「知らぬが仏」という言葉を思い起した(マス・カルチャー遊泳)</title>
<link>https://ndlsearch.ndl.go.jp/books/R000000004-I2694715</link>
<description>
<![CDATA[ <p><p><ul><li>タイトル：アーサー・C・クラークのSFを読んで「知らぬが仏」という言葉を思い起した(マス・カルチャー遊泳)</li><li>タイトル（読み）：アーサー C クラーク ノ SF オ ヨンデ シラヌ ガ ホトケ ト イウ コ</li><li>責任表示：青木 保</li></ul> ]]>
</description>
<author>青木 保,青木 保</author>
<category>記事</category>
<category>デジタル</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R000000004-I2694715</guid>
<pubDate>Fri, 5 Feb 1999 05:32:29 +0900</pubDate>
<dc:title>アーサー・C・クラークのSFを読んで「知らぬが仏」という言葉を思い起した(マス・カルチャー遊泳)</dc:title>
<dcndl:titleTranscription>アーサー C クラーク ノ SF オ ヨンデ シラヌ ガ ホトケ ト イウ コ</dcndl:titleTranscription>
<dc:creator>青木 保</dc:creator>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:identifier xsi:type="dcndl:NDLBibID">2694715</dc:identifier>
<dc:subject xsi:type="dcndl:NDLC">ZW1</dc:subject>
<rdfs:seeAlso rdf:resource="https://ndlsearch.ndl.go.jp/books/R000000004-I2694715"/>
<rdfs:seeAlso rdf:resource="https://cir.nii.ac.jp/crid/1521417755118158464"/>
<dc:description>掲載誌：中央公論 11 p.p266～273</dc:description>
</item>
<item>
<title>アーシュラ・K・ル=グイン--女性文明としての中世</title>
<link>https://ndlsearch.ndl.go.jp/books/R000000004-I2406979</link>
<description>
<![CDATA[ <p><p><ul><li>タイトル：アーシュラ・K・ル=グイン--女性文明としての中世</li><li>タイトル（読み）：アーシュラ K ル グイン ジョセイ ブンメイ トシテノ チュウセイ</li><li>責任表示：山野 浩一</li><li>シリーズ名：現代文学・SFの衝撃<特集> ; SF・その現代の古典</li><li>シリーズ名（読み）：ゲンダイ ブンガク SF ノ ショウゲキ トクシュウ ; SF ソノ ゲンダイ ノ コテン</li></ul> ]]>
</description>
<author>山野 浩一,山野 浩一</author>
<category>記事</category>
<category>デジタル</category>
<guid isPermaLink="true">https://ndlsearch.ndl.go.jp/books/R000000004-I2406979</guid>
<pubDate>Thu, 4 Feb 1999 13:30:33 +0900</pubDate>
<dc:title>アーシュラ・K・ル=グイン--女性文明としての中世</dc:title>
<dcndl:titleTranscription>アーシュラ K ル グイン ジョセイ ブンメイ トシテノ チュウセイ</dcndl:titleTranscription>
<dc:creator>山野 浩一</dc:creator>
<dcndl:seriesTitle>現代文学・SFの衝撃<特集> ; SF・その現代の古典</dcndl:seriesTitle>
<dcndl:seriesTitleTranscription>ゲンダイ ブンガク SF ノ ショウゲキ トクシュウ ; SF ソノ ゲンダイ ノ コテン</dcndl:seriesTitleTranscription>
<dcndl:publicationPlace>JP</dcndl:publicationPlace>
<dc:identifier xsi:type="dcndl:NDLBibID">2406979</dc:identifier>
<dc:subject xsi:type="dcndl:NDLC">ZK22</dc:subject>
<rdfs:seeAlso rdf:resource="https://ndlsearch.ndl.go.jp/books/R000000004-I2406979"/>
<rdfs:seeAlso rdf:resource="https://cir.nii.ac.jp/crid/1520010380738860032"/>
<dc:description>掲載誌：國文學 : 解釈と教材の研究 / 學燈社 [編] 11 p.p90～92</dc:description>
</item>
</channel>
</rss>
```

### エラーレスポンス
| ステータス | コード | メッセージ |
|------------|--------|------------|
| 400 | BAD_REQUEST | パラメーター不正 |
| 404 | NOT_FOUND | 該当データなし |
| 500 | SERVER_ERROR | サーバーエラー |

---

## GET /opensearch?isbn={isbn}&dpid={dpid}

### リクエスト
```http
GET https://ndlsearch.ndl.go.jp/api/opensearch?isbn=4152083336&dpid=ndl-dl
```

### 説明
ISBN で書誌データを取得するエンドポイント。レスポンスは RSS/XML。

---

## GET /thumbnail/{isbn13}.jpg

### リクエスト
```
https://ndlsearch.ndl.go.jp/thumbnail/{isbn13}.jpg
```

### パラメーター
- `isbn`: ISBN-13（13桁）

### レスポンス例
```
https://ndlsearch.ndl.go.jp/thumbnail/9784422311074.jpg
```

---

## 使用上の注意・Tips
- OpenSearch の返却は RSS/XML 形式が基本。XML ネームスペースを正しく扱ってパースすること。
- `dpid` パラメーターで対象データプロバイダーを指定可能（例: `ndl-dl` はデジタルコレクション）。
- 複数の識別子（ISBN, NSMARCNO 等）が返る場合があるため、優先する識別子を決めて処理する。
- 書影 URL は必ず存在するとは限らない。HTTP 404 を想定したフォールバックを用意する。
- Rate Limit に注意。大量アクセス時は間隔を設けること。

## 参考リンク
- https://ndlsearch.ndl.go.jp/help/api
- https://ndlsearch.ndl.go.jp/file/help/api/specifications/ndlsearch_api_20250326.pdf
- https://ndlsearch.ndl.go.jp/help/api/provider
- https://ndlsearch.ndl.go.jp/help/api/thumbnail

