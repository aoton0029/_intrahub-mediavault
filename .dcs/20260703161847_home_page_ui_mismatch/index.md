# バグ調査インデックス: HomePage表示がデザインカンプ(01_home.html)と大きくかけ離れている

## バグ情報

- 概要: `docs/frontend/ui/01_home.html` をベースに設計したはずのUIが、`http://localhost` で実際に表示されるページ（`frontend/src/pages/HomePage.tsx` 等）と大きくかけ離れている。
- 種類: UI/表示バグ
- 発生状況: 常に発生（`http://localhost` を開くと必ず発生）
- エラー情報: なし（見た目の乖離のみ）
- 再現手順: ブラウザで `http://localhost` を開くだけ

## 分析結果ファイル一覧

- [初期調査](./initial_investigation.md) — 実施済み
- [処理フロー分析](./flow_analysis.md) — 実施済み
- [詳細原因分析（root_cause_analysis）](./root_cause_analysis.md) — 実施済み
- 影響範囲分析（impact-analysis） — まだ実行されていません
- シーケンス図分析（sequence-diagram-analysis） — まだ実行されていません
- 状態遷移分析（state-transition-analysis） — まだ実行されていません
- エッジケース分析（edgecase-analysis） — まだ実行されていません
- パフォーマンス分析（performance-analysis） — まだ実行されていません

## クイックサマリー

### 初期仮説（確度順）

1. **⭐⭐⭐ `Sidebar.tsx` が使用するクラス（`.sidebar`, `.brand`, `.nav-item`, `.nav-section`, `.count` 等）に対応するCSSルールが `frontend/src/index.css` に一切定義されていない。**
   モックアップの正解定義 `docs/frontend/ui/_shared.css:75-131` には存在するが、`frontend/src/index.css`（全348行）には該当スタイルが存在せず、`@media (max-width:980px)`内で`.sidebar`を非表示にする記述のみが存在する。サイドバーはページ左側の主要領域であり、これが未装飾になることは「見た目が大きくかけ離れている」という報告と強く整合する。

2. **⭐⭐ `MediaCard.tsx` のモック固有クラス（`.cover`, `.badge`, `.fav`, `.body`, `.title`, `.meta`）に対応するCSSが index.css になく、Tailwindユーティリティで部分的に代替されているのみ。**
   フォント（Source Serif 4のタイトル、JetBrains Monoのバッジ）やレイアウト細部がモックと異なる可能性がある。

3. （処理フロー分析で除外済み）ルーティング設定の問題。`frontend/src/routes.tsx`・`App.tsx` を確認した結果、`/` → `RootLayout` → `HomePage` のマウント経路に誤りは見つからなかった。

### 処理フロー分析での更新（確度順）

1. **⭐⭐⭐⭐⭐ `index.css` に `.sidebar`/`.brand`/`.nav-item`/`.nav-section`/`.count` 系の実スタイルが一切存在しない。**
   他タスク（TASK-0004/0005/0009/0010/0011）は「_shared.css移植＋信頼性コメント」付きでCSSが追加されているが、TASK-0007（Sidebar拡張）分だけこのパターンが完全に欠落している一貫性の崩れを確認。
2. **⭐⭐⭐⭐ `Sidebar.test.tsx`（全12ケース）はクラス名文字列の存在確認のみで、CSSルール本体やComputed Styleは検証していない。**
   このためCSS移植漏れがあってもテストはグリーンになり、`overview.md` 上で「完了」と誤って記録された可能性が高い。
3. **⭐⭐⭐ `--color-sidebar` 等のshadcn由来トークン（index.css:259-266）は `.sidebar` クラスの欠落を救済しない。** 表面的な確認では見過ごされやすい。
4. **⭐⭐ `MediaCard.tsx` のモック固有クラスに対応するCSSがなく、Tailwindで部分的に代替されているのみ。**（継続、深刻度は候補1より低い）
5. **除外: ルーティング設定・ビルドキャッシュ等の問題。** `routes.tsx`/`App.tsx` の確認により根拠なしと判断。

### 次のアクション

1. `docs/tasks/frontend-ui-compliance/TASK-0007.md`・`TASK-0008.md` の完了条件本文と `docs/implements/frontend-ui-compliance/TASK-0007/`・`TASK-0008/` 配下の実装フェーズ文書を確認し、CSS移植が完了条件に含まれていたか、なぜ漏れたのかを特定する。
2. `MediaCard.test.tsx` についても同様にCSS実体の検証範囲を確認する。
3. 詳細バグ分析（`tsumiki:dcs:bug-analysis` の詳細原因分析ステップ）を実行し、根本原因と修正方針を確定させる。

詳細は [初期調査](./initial_investigation.md)・[処理フロー分析](./flow_analysis.md) を参照してください。
