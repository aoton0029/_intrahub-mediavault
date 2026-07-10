# 03_academic_books 未決事項

設計書中の【要確認】項目、およびタスク実装中にCodexが仮決定した事項を記録する。

## 未決事項

- [ ] `sort`パラメータはバックエンド未実装。UIのみ先行実装し、有効化は別タスクで行う（`02_general_media.md` §3, §5と同様）
- [ ] カードバッジ「学術書」/「専門書」の2種類の表示名の由来: `docs/backend/mediavault-api/items.md`上、`media_type`は`academic_book`単一で判別フィールドが見当たらない。モック（`03_academic_books.html`）は5件中3件が「専門書」・2件が「学術書」と混在表示のみで、データ上の判別方法の記載が無い
- [ ] 「＋ 作品を追加」の遷移先 `13_academic_book_search.md` は本タスクの実装範囲外。ルート未実装のままリンク/ボタンだけ設置してよいか

## Codexによる仮決定ログ

- [x] `FilterToolbar` は `filterOptions` 未指定時に種別セレクトを描画しないことを確認したため、`AcademicBookListPage` 側では props を省略して実装した
- [x] カードバッジ表示名は現行APIレスポンスで「学術書」/「専門書」を判別できないため、本タスクでは暫定的に全件 `学術書` 固定表示とした。判別フィールド追加または表示ルール定義が必要
- [x] 「＋ 作品を追加」は `/academic-books/search` へのプレースホルダ `Link` を設置した。遷移先ルート実装は本タスク範囲外
