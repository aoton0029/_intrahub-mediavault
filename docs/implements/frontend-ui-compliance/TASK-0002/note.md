# TASK-0002 実装ノート: Tailwind @theme連携とshadcn oklchトークンの上書き

## 実施内容

- `frontend/src/index.css` の `:root` 内shadcn oklch系トークン（`--background`, `--foreground`, `--card`, `--popover`, `--primary`, `--secondary`, `--muted`, `--accent`, `--destructive`, `--input`, `--ring`等）を、`_shared.css`準拠のCSS変数参照（`var(--bg-app)`等）に置換した（REQ-402）。
- `_shared.css`由来の単一アクセント色トークンをshadcn標準の`--accent`（hover背景用ニュートラルトークン）との名前衝突を避けるため、`--accent` → `--brand-accent`にリネームした。shadcnの`--primary`は`var(--brand-accent)`を参照し、shadcnの`--accent`自体は`var(--accent-soft)`（ニュートラルなhover背景用途）を参照するようにした。
- `.dark`クラスブロックを`:root`と同一のトークン参照に揃え、方針B（`:root`をダークテーマの正とし、`.dark`クラスの有無で見た目が変わらないようにする。将来のライトモード実装まで凍結）をコメントで明記した。
- `@theme inline`内の`--color-accent-anime`等media_type別マッピングは変更していない（回帰確認テスト追加）。
- `--border`は既存 `:root` 定義（`#383838`）をそのまま維持（重複定義を避け、`--background`等の上書きブロックには含めていない）。

## 変更ファイル

- `frontend/src/index.css`
- `frontend/src/design-tokens.test.ts`（TASK-0002用テストケース TC-01〜TC-04を追加、TASK-0001のTC-04-4アサーションを`--brand-accent`に更新）

## 影響範囲確認

- `frontend/src/App.css`（Viteテンプレート由来の未使用ファイル、どこからもimportされていないため対象外）
- `frontend/src/lib/media-type-accent.ts` は `--accent-{mediatype}` 名前空間のトークンのみ使用しており、`--accent`単体を参照していないため影響なし
- コードベース内に `bg-accent` / `text-accent`（単体）のTailwindユーティリティクラス使用箇所なし（`text-accent-{mediatype}`のみ使用）のため、リネームによる既存コンポーネントへの影響なし

## テスト結果

- `yarn test`: 22 test files / 196 tests すべてパス
- `yarn build`: エラーなく完了（`tsc -b && vite build`）
