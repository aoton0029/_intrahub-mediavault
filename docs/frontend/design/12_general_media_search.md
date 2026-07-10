# 12. 検索して追加（一般メディア）

対応モック: `docs/frontend/ui/12_general_media_search.html`（`13_academic_book_search.html` はほぼ同一構造、差分は本文末尾）

## 1. 画面概要 / ルート

外部API（Jikan/TMDB/IGDB等）をタイトルで検索し、結果一覧から作品をコレクションに取り込む画面。ルート: `/media/search`。`02_general_media.md` のタイトルバー「＋ 作品を追加」から遷移。

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
          title originalTitle year source
          action={imported ? <Button disabled>取り込み済み</Button> : <Button variant="accent">取り込む</Button>} />
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

```ts
interface SearchResultItem {
  externalId: string;
  mediaType: string;
  title: string;
  originalTitle?: string;
  year?: number;
  source: 'Jikan' | 'TMDB' | 'IGDB';   // 種別ごとの外部API名
  alreadyImported: boolean;             // true時は「取り込み済み」を disabled 表示
}
```

## 4. 画面固有コンポーネント

- `EmptyState`（APIキー未設定時）は `00_common.md` の共通コンポーネントを利用

## 5. インタラクション仕様

- 種別セレクト変更時に再検索（またはユーザーが明示的に「検索」ボタンを押すまで再検索しない、の2案。モックはボタン押下起点のためボタン起点で実装）
- 取り込み済みカードの「取り込み済み」ボタンは常に disabled
- 検索結果一覧とAPIキー未設定の空状態は排他表示（両方同時に出さない）

## 6. API連携

- 検索: `GET /external-search?media_type=...&title=...`【要確認→ `items.md` 記載の `GET /items/search`（外部API横断検索）が該当する可能性。エンドポイント名・パラメータ名を実装時に突き合わせて確定する】
- 取り込み: `POST /items`（外部APIレスポンスをそのまま新規作成）。取り込み済み判定は取り込み時のレスポンス `409 ITEM_ALREADY_IMPORTED` として実装済みの想定（モックHTMLコメントより高確度）【要確認→ `items.md` 記載の `POST /items/import`（外部検索結果からアイテムをインポート）が該当する可能性】
- APIキー未設定エラー: `422 API_KEY_NOT_CONFIGURED`（モックHTMLコメントより高確度）

参照: [items.md](../../backend/mediavault-api/items.md)

## 7. Tailwindスタイリング上の注意

- 取り込み済みボタンは `.btn[disabled]` で `opacity: 0.5; pointer-events: none;`
- `EmptyState` と検索結果一覧は同一画面内で排他的に表示（両方レンダーしない）

---

## 差分: 13_academic_book_search（学術書・専門書 検索して追加）

- ルート `/academic-books/search`、遷移元は `03_academic_books.md`
- 種別が `academic_book` 単一のため `FilterSelect（種別）` は無く、`SearchBox` のみ（プレースホルダは「タイトル・著者名で検索…」）
- `source` は `NDL`（国立国会図書館）固定
- 取り込み済み例では和書のみのため `originalTitle` 行を省略する
- APIキー未設定時の説明文は「学術書・専門書の検索には国立国会図書館(NDL)のAPIキーが必要です」
