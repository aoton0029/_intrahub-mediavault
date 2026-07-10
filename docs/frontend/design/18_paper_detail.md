# 18. 論文・文献 詳細

対応モック: `docs/frontend/ui/18_paper_detail.html`

本画面は `00_common.md` §5「詳細画面共通パターン」に準拠する。構造は `17_academic_book_detail.md`（学術書・専門書）と同一パターン（`rail-facts` が非インタラクティブな静的表示）のため、詳細はそちらを参照。

## 差分サマリ

- ルート: `/papers/:id`。パンくずは「論文・文献」（一覧のみ）。編集ボタンあり（遷移先: `11_paper_form.md`）
- `rail-facts` は `17_academic_book_detail.md` と同様に非インタラクティブ表示（ステータス・評価・お気に入りはすべて読み取り専用の `<span>`）
- タグ/カテゴリに削除ボタンなし（読み取り専用表示）
- 登録方法は「手動登録」
- 任意セクション有無: 種別固有情報 ✓（5項目） / エピソード・巻構成 ✗ / スタッフ ✗ / 配信 ✗
- セクション順序: 概要 → 種別固有情報 → 関連作品 → リソース
- リソースタブは「リンク」「ファイル」のみ。リンクラベルが「出版社ページ(DOI解決)」でURLが `https://doi.org/{DOI}` になる点が学術書と異なる

## 種別固有情報（`prop-row` 5項目、`11_paper_form.md` の「種別固有情報」欄と同一フィールド）

DOI / 掲載誌名 / 巻号 / ページ範囲 / 著者一覧（複数著者は `/` 区切り表示）

```ts
interface PaperDetail extends ItemDetailBase {
  mediaType: 'paper';
  detail: {
    doi: string;
    journalName: string;
    volumeIssue: string;     // 例: "Vol.32 No.4"
    pageRange: string;       // 例: "123-140"
    authors: string[];       // フォームでは改行区切り入力 → 表示は "/" 結合
  };
}
```

## API連携

- 詳細取得: `GET /items/{id}`（`detail(paper)` + tags + categories + calibre_links）
- リソース: `POST/DELETE /items/{id}/links`, `/items/{id}/files`（DOIから自動生成される「出版社ページ(DOI解決)」リンクを既定で1件表示する設計を想定）
- その他は `17_academic_book_detail.md` と同様の【要確認】事項（タグ/カテゴリ編集可否）を引き継ぐ

参照: [items.md](../../backend/mediavault-api/items.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md)
