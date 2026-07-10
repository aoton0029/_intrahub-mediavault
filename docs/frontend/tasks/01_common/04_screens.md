# 04. フェーズ3: 画面別実装タスク

依存: フェーズ0〜2（[01_foundation.md](01_foundation.md) / [02_shared_components.md](02_shared_components.md) / [03_detail_layout.md](03_detail_layout.md)）。詳細画面（16, 17, 18, 20〜24）は[03_detail_layout.md](03_detail_layout.md)完了が前提。

画面同士は互いに独立して実装可能（並行着手可）。各画面の詳細は対応する設計書が正であり、ここでは「どのフェーズ1/2成果物を使うか」「どのAPIドキュメントを参照するか」の橋渡しのみ記載する。

---

## 01. ホーム
- 設計書: [design/01_home.md](../design/01_home.md) / モック: [ui/01_home.html](../ui/01_home.html)
- 構成要素: `MediaGrid`, `MediaCard`, `EmptyState`
- API参照: [items.md](../../backend/mediavault-api/items.md), [mylists.md](../../backend/mediavault-api/mylists.md)

## 02. 一般メディア一覧
- 設計書: [design/02_general_media.md](../design/02_general_media.md) / モック: [ui/02_general_media.html](../ui/02_general_media.html)
- 構成要素: `FilterToolbar`, `MediaGrid`, `MediaCard`, `LoadMoreSentinel`+`useInfiniteScroll`, `EmptyState`
- API参照: [items.md](../../backend/mediavault-api/items.md), [categories.md](../../backend/mediavault-api/categories.md), [tags.md](../../backend/mediavault-api/tags.md)

## 03. 学術書一覧
- 設計書: [design/03_academic_books.md](../design/03_academic_books.md) / モック: [ui/03_academic_books.html](../ui/03_academic_books.html)
- 構成要素: `FilterToolbar`, `LiteratureList`/`LiteratureRow`, `LoadMoreSentinel`, `EmptyState`
- API参照: [items.md](../../backend/mediavault-api/items.md), [categories.md](../../backend/mediavault-api/categories.md), [tags.md](../../backend/mediavault-api/tags.md)

## 04. 論文一覧
- 設計書: [design/04_papers.md](../design/04_papers.md) / モック: [ui/04_papers.html](../ui/04_papers.html)
- 構成要素: `FilterToolbar`, `LiteratureList`/`LiteratureRow`, `LoadMoreSentinel`, `EmptyState`
- API参照: [items.md](../../backend/mediavault-api/items.md), [tags.md](../../backend/mediavault-api/tags.md)

## 06. マイリスト一覧
- 設計書: [design/06_mylists.md](../design/06_mylists.md) / モック: [ui/06_mylists.html](../ui/06_mylists.html)
- 構成要素: `MylistCover`, `Modal`（作成・削除確認）, `EmptyState`
- API参照: [mylists.md](../../backend/mediavault-api/mylists.md)

## 11. 論文登録フォーム
- 設計書: [design/11_paper_form.md](../design/11_paper_form.md) / モック: [ui/11_paper_form.html](../ui/11_paper_form.html)
- 構成要素: `FormSection`/`FormGrid`/`FormField`/`FormActions`（react-hook-form + zod）
- API参照: [items.md](../../backend/mediavault-api/items.md), [import.md](../../backend/mediavault-api/import.md), [data-model.md](../../backend/mediavault-api/data-model.md)

## 12. 一般メディア検索
- 設計書: [design/12_general_media_search.md](../design/12_general_media_search.md) / モック: [ui/12_general_media_search.html](../ui/12_general_media_search.html)
- 構成要素: `FilterToolbar`（検索ボックス中心）, `MediaCard variant="search-result"`, `MediaGrid`, `EmptyState`
- API参照: [items.md](../../backend/mediavault-api/items.md), [import.md](../../backend/mediavault-api/import.md)

## 13. 学術書検索
- 設計書: [design/13_academic_book_search.md](../design/13_academic_book_search.md) / モック: [ui/13_academic_book_search.html](../ui/13_academic_book_search.html)
- 構成要素: `FilterToolbar`, `LiteratureList`/`LiteratureRow`（検索結果表示）, `EmptyState`
- API参照: [items.md](../../backend/mediavault-api/items.md), [import.md](../../backend/mediavault-api/import.md)

## 15. マイリスト詳細
- 設計書: [design/15_mylist_detail.md](../design/15_mylist_detail.md) / モック: [ui/15_mylist_detail.html](../ui/15_mylist_detail.html)
- 構成要素: `MediaGrid`/`MediaCard`（収録作品一覧）, `Modal`（削除確認）, `EmptyState`
- API参照: [mylists.md](../../backend/mediavault-api/mylists.md), [items.md](../../backend/mediavault-api/items.md)

## 16. アニメ詳細
- 設計書: [design/16_anime_detail.md](../design/16_anime_detail.md) / モック: [ui/16_anime_detail.html](../ui/16_anime_detail.html)
- 構成要素: `DetailLayout`（[03_detail_layout.md](03_detail_layout.md)）— セクション: 概要, 構成(Group), スタッフ, 関連作品, 配信, リソース（種別固有情報なし）
- API参照: [items.md](../../backend/mediavault-api/items.md), [item-episodes.md](../../backend/mediavault-api/item-episodes.md), [staff.md](../../backend/mediavault-api/staff.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [item-streaming-links.md](../../backend/mediavault-api/item-streaming-links.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md), [item-trailers.md](../../backend/mediavault-api/item-trailers.md)

## 17. 学術書詳細
- 設計書: [design/17_academic_book_detail.md](../design/17_academic_book_detail.md) / モック: [ui/17_academic_book_detail.html](../ui/17_academic_book_detail.html)
- 構成要素: `DetailLayout` — セクション: 概要, 種別固有情報(4項目), 関連作品, リソース（「出版社ページ」等ラベル変更に注意）
- API参照: [items.md](../../backend/mediavault-api/items.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md)

## 18. 論文詳細
- 設計書: [design/18_paper_detail.md](../design/18_paper_detail.md) / モック: [ui/18_paper_detail.html](../ui/18_paper_detail.html)
- 構成要素: `DetailLayout` — セクション: 概要, 種別固有情報(5項目), 関連作品, リソース（「出版社ページ」等ラベル変更に注意）
- API参照: [items.md](../../backend/mediavault-api/items.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md)

## 19. 設定
- 設計書: [design/19_settings.md](../design/19_settings.md) / モック: [ui/19_settings.html](../ui/19_settings.html)
- 構成要素: `SettingsShell`+`SettingsTabs`（タブstate化）, `ApiKeyCard`, `EmptyState`（APIキー未設定時）
- API参照: [settings.md](../../backend/mediavault-api/settings.md)

## 20. 映画詳細
- 設計書: [design/20_movie_detail.md](../design/20_movie_detail.md) / モック: [ui/20_movie_detail.html](../ui/20_movie_detail.html)
- 構成要素: `DetailLayout` — セクション: 概要, 種別固有情報(6項目), スタッフ, 関連作品, 配信, リソース（構成(Group)なし）
- API参照: [items.md](../../backend/mediavault-api/items.md), [staff.md](../../backend/mediavault-api/staff.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [item-streaming-links.md](../../backend/mediavault-api/item-streaming-links.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md), [item-trailers.md](../../backend/mediavault-api/item-trailers.md)

## 21. ドラマ詳細
- 設計書: [design/21_drama_detail.md](../design/21_drama_detail.md) / モック: [ui/21_drama_detail.html](../ui/21_drama_detail.html)
- 構成要素: `DetailLayout` — セクション: 概要, 種別固有情報(7項目), 構成(Group), スタッフ, 関連作品, 配信, リソース（全オプションセクションあり）
- API参照: [items.md](../../backend/mediavault-api/items.md), [item-episodes.md](../../backend/mediavault-api/item-episodes.md), [staff.md](../../backend/mediavault-api/staff.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [item-streaming-links.md](../../backend/mediavault-api/item-streaming-links.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md), [item-trailers.md](../../backend/mediavault-api/item-trailers.md)

## 22. マンガ詳細
- 設計書: [design/22_manga_detail.md](../design/22_manga_detail.md) / モック: [ui/22_manga_detail.html](../ui/22_manga_detail.html)
- 構成要素: `DetailLayout` — セクション: 概要, 種別固有情報(4項目), 構成(Group), 関連作品, リソース（スタッフ・配信なし）
- API参照: [items.md](../../backend/mediavault-api/items.md), [item-groups.md](../../backend/mediavault-api/item-groups.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md)

## 23. 小説詳細
- 設計書: [design/23_novel_detail.md](../design/23_novel_detail.md) / モック: [ui/23_novel_detail.html](../ui/23_novel_detail.html)
- 構成要素: `DetailLayout` — セクション: 概要, 種別固有情報(4項目), 構成(Group), 関連作品, リソース（スタッフ・配信なし）
- API参照: [items.md](../../backend/mediavault-api/items.md), [item-groups.md](../../backend/mediavault-api/item-groups.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md)

## 24. ゲーム詳細
- 設計書: [design/24_game_detail.md](../design/24_game_detail.md) / モック: [ui/24_game_detail.html](../ui/24_game_detail.html)
- 構成要素: `DetailLayout` — セクション: 概要, 種別固有情報(5項目), 関連作品, リソース（構成(Group)・スタッフ・配信なし）
- API参照: [items.md](../../backend/mediavault-api/items.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md)

---

## 補足

- `item-groups.md` / `item-episodes.md` はモック上の「構成(Group)」（`GroupList`/`EpisodeRow`）に対応する2つのリソースドキュメント。画面によりシーズン単位（anime/drama）か巻単位（manga/novel）かでどちらを主に参照するか異なるため、実装時に両方を確認すること。
- 各画面のAPI参照は「関連しそうなリソースドキュメントの列挙」であり、正式なフィールド対応は各設計書の「API連携」節末尾のリンクと実装時の突き合わせで確定させる（[00_common.md §7](../design/00_common.md#7-api連携についての注記)）。
