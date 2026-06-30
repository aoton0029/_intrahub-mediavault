# TASK-0002 設定作業実行

## 作業概要

- **タスクID**: TASK-0002
- **作業内容**: Tailwind CSS v4デザイントークン（ダークテーマ・media_type別アクセントカラー）定義
- **実行日時**: 2026-06-30
- **実行者**: Claude Code (kairo-implement)

## 設計文書参照

- **参照文書**: docs/tasks/frontend-collection-ui/TASK-0002.md, docs/spec/frontend-collection-ui/note.md, docs/design/frontend-collection-ui/interfaces.ts
- **関連要件**: note.md「既存デザインシステム」、MediaType型（8種別）

## ⚠️ 推測値であることの明記

このタスクは参照元CSS（`docs/frontend/ui/01_components.html`、`_shared.css`）がリポジトリから削除済みのため、**具体的な色の16進値はTASK-0002.md記載の推測値をそのまま採用**した（🔴信頼性）。デザイン最終確定時は本タスクの値を差し替える前提とする。

## 実行した作業

### 1. ダークテーマCSS変数の追加

`frontend/src/index.css`の`:root`に以下を追加（TASK-0002.md記載の推測値どおり）:

```css
--bg-base: #0f1115;
--bg-surface: #1a1d23;
--bg-elevated: #232730;
--text-primary: #f5f5f5;
--text-secondary: #9ca3af;
--border-default: #2d313a;
```

### 2. media_type別アクセントカラー（MediaType型8種別）の追加

`docs/design/frontend-collection-ui/interfaces.ts`の`MediaType`型を確認し、8種別（anime/movie/drama/manga/novel/game/academic_book/paper）すべてに対応:

```css
--accent-anime: #f97316;
--accent-movie: #3b82f6;
--accent-manga: #ec4899;
--accent-novel: #8b5cf6;
--accent-game: #10b981;
--accent-drama: #ef4444;
--accent-academic-book: #14b8a6;
--accent-paper: #64748b;
```

note.mdの「book/paper」表記は`interfaces.ts`の`MediaType`定義に合わせ`academic_book`と解釈した（TASK-0002.md記載の解釈方針に準拠）。

### 3. Tailwind v4 `@theme inline`連携

`@theme inline`ブロックに上記トークンを`--color-*`命名規則でマッピングし、`bg-bg-base`・`text-accent-anime`等のユーティリティクラスとして利用可能にした。

### 4. MediaType→アクセントカラーのマッピングユーティリティ

`frontend/src/lib/media-type-accent.ts`を新規作成。`getMediaTypeAccentClass(mediaType: MediaType): string`関数を実装（TASK-0004で正式な型定義ファイルが配置されるまでの暫定として、ローカルに`MediaType`型を定義）。

## 作業結果

- [x] ダークテーマ用CSS変数定義完了
- [x] media_type別アクセントカラー8種別すべて定義完了
- [x] `@theme inline`でのユーティリティクラス化完了
- [x] マッピングユーティリティ関数実装完了

## 遭遇した問題と解決方法

特になし（TASK-0001で基盤が整備済みだったため設定作業自体はスムーズだった）。

## 次のステップ

- `/tsumiki:direct-verify` を実行して設定を確認
- TASK-0004（型定義ファイル配置）完了後、`media-type-accent.ts`内のローカル`MediaType`型を正式な型定義からのインポートに置き換える
- TASK-0006（共通UIコンポーネント実装）でMediaTypeBadge等から本トークンを参照
