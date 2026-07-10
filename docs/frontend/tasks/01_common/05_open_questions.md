# 05. 要確認事項の追跡

設計書 `docs/frontend/design/00_common.md` 中の【要確認】項目、および実装中に新たに生じた未確定事項を記録する。Codexは実装をブロックする判断が必要な場合、ここに追記してからClaudeのレビューを待つ。ブロッキングでない場合は妥当な仮決定を行いメモを残した上で先に進んでよい。

## 記入フォーマット

```
### <連番>. <一言タイトル>
- 出典: 設計書 §<節番号> / 実装中に発見
- 論点: ...
- 選択肢: A) ... B) ...
- 仮決定（あれば）: ...
- 状態: [ ] 未決定 / [x] 決定
- 決定内容（決定後に記入）: ...
```

## 一覧

### 1. ライトモード切替方式（Tailwind `dark:` を使わない件）
- 出典: 設計書 §2
- 論点: `prefers-color-scheme`ではなく`data-theme`属性による明示的トグルのため、Tailwindの`dark:`バリアントを使わず`[data-theme="light"]`セレクタベースに寄せる方針が示されている
- 状態: [x] 決定
- 決定内容: [01_foundation.md](01_foundation.md) の方針に従い実装する。追加の意思決定は不要。

### 2. フォント調達方法（Inter / Source Serif 4 / JetBrains Mono）
- 出典: 設計書 §2, [01_foundation.md](01_foundation.md)
- 論点: `--font-ui`/`--font-display`/`--font-mono`に対応する実フォントが現行`package.json`に未導入（`@fontsource-variable/geist`のみ導入済み）
- 選択肢: A) `@fontsource-variable/inter`等を追加導入する B) 既存の`geist`等で代替する
- 仮決定（あれば）: `npm install`前提の新規依存追加は避け、既存の `@fontsource-variable/geist` を UI / Display に流用し、Mono は system monospace fallback を使用
- 状態: [x] 決定
- 決定内容（決定後に記入）: `frontend/src/index.css` で `--font-ui` / `--font-display` に `Geist Variable` を採用し、`--font-mono` は `ui-monospace` 系 fallback とした。

### 3. 解除アイコン（マイリスト/関連作品からの解除ボタン）
- 出典: 設計書 §4
- 論点: モックSVGは箱アイコンで`react-icons`に厳密一致がない
- 選択肢: A) `FiPackage` B) `FiX`
- 仮決定（あれば）: destructive action と視認性を優先して `FiTrash2` を採用
- 状態: [x] 決定
- 決定内容（決定後に記入）: 関連作品・リスト解除ボタンは `FiTrash2` ベースで統一した。意味が明確で、既存モックの破壊的操作ボタンとも整合するため。

### 4. 並び替えアイコン
- 出典: 設計書 §4
- 論点: モックはカスタムpathで`FiArrowUpDown`相当だが厳密一致がない場合がある
- 仮決定（あれば）: `react-icons/fi` に厳密一致がないため `FiArrowDown` を近似採用
- 状態: [x] 決定
- 決定内容（決定後に記入）: `FilterToolbar` の sort icon は `FiArrowDown` を使用。上下ソート専用アイコン不在のため、最も近い Feather 系の矢印アイコンで代替した。

### 5. バックエンドAPI仕様との突き合わせ
- 出典: 設計書 §7
- 論点: 各画面ドキュメントのAPI連携章はPRDからの推測。`docs/backend/mediavault-api/`の実仕様と突き合わせて確定させる必要がある
- 状態: [ ] 未決定（本common実装タスクの範囲では影響なし。画面タスク側で解消する）
