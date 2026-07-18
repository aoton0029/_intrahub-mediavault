# 12. 検索して追加（メディア）

対応モック: `docs/frontend/ui/12_general_media_search.html`（`13_academic_book_search.html` はほぼ同一構造、差分は本文末尾）

## 1. 画面概要 / ルート

外部API（Annict/TMDB/Steam/楽天ブックス）をタイトルで検索し、結果一覧から作品をコレクションに取り込む画面。ルート: `/media/search`。`02_general_media.md` のタイトルバー「＋ 作品を追加」から遷移。

## 2. レイアウト構成

```
<AppShell>
  <Titlebar title="検索して追加" action={<Link to="/media/new">手動で入力する</Link>} />
  <Content>
    <FilterBar>
      <FilterSelect label="種別" options={[anime, movie, drama, manga, novel, game]} />
      <SearchBox flex placeholder="作品名で検索…" />
      <Button variant="accent">検索</Button>
    </FilterBar>

    {hasApiKey ? (
      <MediaGrid density="compact">
        <MediaCard variant="search-result" compact
          title provider thumbnailUrl
          action={imported ? <Button disabled>取り込み済み</Button> : <Button variant="accent" onClick={importItem}>取り込む</Button>} />
      </MediaGrid>
    ) : (
      <EmptyState title="APIキーが設定されていません"
        description="この種別の検索には {provider} のAPIキーが必要です。設定画面から登録してください。"
        action={<Link to="/settings?tab=api">設定を開く</Link>} />
    )}
  </Content>
</AppShell>
```

## 3. 表示データ / Props型

`GET /items/search` のレスポンス（`SearchResultItem[]`、`items.md` 参照）をそのまま画面表示に使う。バックエンドは検索段階で `year` や `originalTitle`、取り込み済み判定を返さない（軽量な候補一覧のみ）ため、モックHTMLの表示項目のうち一部はAPI非対応。詳細は7節参照。

```ts
// items.md GET /items/search のレスポンス型そのもの
interface SearchResultItem {
  id: string;                 // 外部プロバイダ固有ID。POST /items/import の external_id にそのまま渡す
  media_type: string;         // リクエストと同じ media_type
  provider: string | null;    // 'annict' / 'rakuten' / 'tmdb' / 'steam' / 'ndl'
  title: string;
  thumbnail_url: string | null;
}

// フロント側で「取り込み済み」表示を管理するためのローカル状態（APIには存在しない）
type ImportedIdSet = Set<string>; // POST /items/import が 409 を返した id、または取り込み成功した id を保持
```

## 4. 画面固有コンポーネント

- `EmptyState`（APIキー未設定時）は `00_common.md` の共通コンポーネントを利用

## 5. インタラクション仕様

- 種別セレクト変更時に再検索（またはユーザーが明示的に「検索」ボタンを押すまで再検索しない、の2案。モックはボタン押下起点のためボタン起点で実装）
- 「取り込む」ボタン押下時: `POST /items/import` を呼ぶ。成功(201)時はそのカードを「取り込み済み」disabled表示に切り替える。`409 ITEM_ALREADY_IMPORTED` が返った場合も同様に「取り込み済み」disabled表示に切り替える(検索結果には取り込み済みかどうかの情報が含まれないため、実際に押下されるまで判定できない)
- 検索結果一覧とAPIキー未設定の空状態は排他表示（両方同時に出さない）

## 6. API連携

- 検索: `GET /items/search?media_type=...&q=...`（`items.md` 記載どおり。クエリパラメータ名は `title` ではなく `q`）
- 取り込み: `POST /items/import`（`media_type` / `provider`（`GET /items/search` の `provider` をそのまま渡せる） / `external_id`（`GET /items/search` の `id`））。取り込み済み判定はサーバー側で行われ、レスポンス `409 ITEM_ALREADY_IMPORTED` として返る
- APIキー未設定エラー: `422 API_KEY_NOT_CONFIGURED`
- プロバイダ振り分け（`items.md` より）: `anime`→Annict, `manga`/`novel`/`academic_book`→楽天ブックス, `movie`/`drama`→TMDb, `game`→Steam（キー不要）, `paper`→NDL

参照: [items.md](../../backend/mediavault-api/items.md)

## 7. Tailwindスタイリング上の注意

- 取り込み済みボタンは `.btn[disabled]` で `opacity: 0.5; pointer-events: none;`
- `EmptyState` と検索結果一覧は同一画面内で排他的に表示（両方レンダーしない）

---

## 差分: 13_academic_book_search（学術書・専門書 検索して追加）

- ルート `/academic-books/search`、遷移元は `03_academic_books.md`
- 種別が `academic_book` 単一のため `FilterSelect（種別）` は無く、`SearchBox` のみ（プレースホルダは「タイトル・著者名で検索…」）
- `provider` は `items.md` のプロバイダ振り分けにより `rakuten`（楽天ブックス）固定（NDLは `paper`（論文・文献）用であり、この画面には適用されない）
- APIキー未設定時の説明文は「学術書・専門書の検索には楽天ブックスのAPIキーが必要です」
