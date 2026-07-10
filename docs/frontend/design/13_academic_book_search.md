# 13. 検索して追加（学術書・専門書）

対応モック: `docs/frontend/ui/13_academic_book_search.html`

本画面は `12_general_media_search.md` と同一構造のため、フルスペックの記述はそちらに譲り、ここでは差分のみを記載する（`12_general_media_search.md` 末尾の「差分: 13_academic_book_search」と同内容）。

## 差分サマリ

- ルート: `/academic-books/search`。遷移元は `03_academic_books.md`
- 種別が `academic_book` 単一のため `FilterSelect（種別）` は無く、`SearchBox`（プレースホルダ「タイトル・著者名で検索…」）+「検索」ボタンのみ
- 検索結果の `source` は `NDL`（国立国会図書館）固定
- 取り込み済み例は和書のみのため `originalTitle`（原題）行を省略
- APIキー未設定時の空状態メッセージ: 「学術書・専門書の検索には国立国会図書館(NDL)のAPIキーが必要です。設定画面から登録してください。」
- 「手動で入力する」の遷移先は学術書用フォーム（`10_academic_book_form.html`、design未作成 = 別タスク）

## API連携

- 検索: `GET /external-search?media_type=academic_book&title=...`【要確認→ `items.md` 記載の `GET /items/search`（外部API横断検索）が該当する可能性】
- 取り込み: `POST /items`（`409 ITEM_ALREADY_IMPORTED` / `422 API_KEY_NOT_CONFIGURED` はモックHTMLコメントより高確度）【要確認→ `items.md` 記載の `POST /items/import` が該当する可能性】

参照: [items.md](../../backend/mediavault-api/items.md)
