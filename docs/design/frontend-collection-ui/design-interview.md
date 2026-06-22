# frontend-collection-ui 設計ヒアリング記録

**作成日**: 2026-06-22
**ヒアリング実施**: step2〜step4 既存情報ベースの差分ヒアリング（kairo-design）

## ヒアリング目的

既存の要件定義（requirements.md）・ユーザーストーリー・受け入れ基準・コンテキストノート、frontend/PRD.md・frontend/tech-stack.md、確定済みのバックエンド設計（docs/design/mediavault-backend/）・既存デザインシステム（docs/frontend/ui/01_components.html）を確認した上で、技術設計に必要な未確定事項（作業規模・フォーム実装方式・型設計方針・コンポーネント粒度・API層構成・通知UI）を明確化するためのヒアリングを実施した。

## 質問と回答

### Q1: 作業規模

**質問日時**: 2026-06-22
**カテゴリ**: 優先順位
**背景**: 設計文書の詳細度（フル/軽量/カスタム）を決定する必要があった。

**回答**: フル設計（推奨）。

**信頼性への影響**: architecture.md・dataflow.md・design-interview.md・interfaces.tsの4種類を作成する方針に確定。DBスキーマ・独自API仕様はフロントエンドでは不要と判断し対象外とした。

---

### Q2: フォーム実装方式

**質問日時**: 2026-06-22
**カテゴリ**: 技術選択
**背景**: 手動追加・編集・APIキー登録・タグ作成等、多数のフォーム画面（NFR-201のバリデーション要件含む）があり、実装方式が未確定だった。

**回答**: react-hook-form + zod（推奨）。

**信頼性への影響**: architecture.mdの「フォーム」項目を🔵で確定。NFR-201（フィールド近傍エラー表示）の実現方式が明確化された。

---

### Q3: メディア別詳細の型設計方針

**質問日時**: 2026-06-22
**カテゴリ**: 技術選択
**背景**: バックエンドの`CreateItemRequest.details`は`serde_json::Value`（型消去）だが、フロントエンドの型定義（interfaces.ts）でどう表現するか未確定だった。

**回答**: media_typeによる判別共用体（Discriminated Union）を採用（推奨）。

**信頼性への影響**: interfaces.tsで`Item`型をmedia_type別のUnion型として定義する方針に確定（🔵）。anime_details/manga_details等の型はbackend types.rsのRust構造体を直接参照してTypeScript化した。

---

### Q4: コンポーネント設計の粒度

**質問日時**: 2026-06-22
**カテゴリ**: アーキテクチャ
**背景**: shadcn/uiをベースにどこまで独自コンポーネント化するか（Atomic分割 or pages中心）が未確定だった。

**回答**: Atomic的に分割（推奨）。`components/ui`（shadcn基盤）・`components/common`（MediaCard等の再利用コンポーネント）・`features/*`（機能固有コンポーネント）に分離。

**信頼性への影響**: architecture.mdのディレクトリ構造・コンポーネント粒度項目を🔵で確定。

---

### Q5: API通信層の構成

**質問日時**: 2026-06-22
**カテゴリ**: 技術選択
**背景**: fetchラップ + TanStack Queryのcustom hooksをapi/に集約する方針か、axios等を追加導入するかが未確定だった。

**回答**: fetch + TanStack Query hooks（推奨）。axios等の追加依存は導入しない。

**信頼性への影響**: architecture.mdの「バックエンド連携（API層）」項目を🔵で確定。依存を最小限に保つ方針が明文化された。

---

### Q6: トースト/エラー通知UI

**質問日時**: 2026-06-22
**カテゴリ**: 技術選択
**背景**: API成功/失敗時の通知UIをライブラリ導入するか自作するかが未確定だった。

**回答**: sonner導入（推奨）。

**信頼性への影響**: architecture.md・dataflow.mdのエラーハンドリングフローにsonner toastを明記（🔵）。

---

## ヒアリング結果サマリー

### 確認できた事項
- フロントエンドはPRD・バックエンドAPI仕様・既存デザインシステムをもとに新規設計する（既存実装なし）。
- 13画面構成（一般メディア/学術書専門書/論文文献の3グループ分割）はrequirements.md REQ-004/404のとおり採用。
- バックエンドの`details: serde_json::Value`はフロントエンドではmedia_type別の判別共用体として厳密型付けする。

### 設計方針の決定事項
- フォーム: react-hook-form + zod
- 型設計: Discriminated Union（media_type判別）
- コンポーネント: Atomic的分割（ui/common/features）
- API層: fetch + TanStack Query hooks（axios不使用）
- 通知UI: sonner

### 残課題
- ページング方式（ページ送り vs 無限スクロール）はPRDに明記がなく、本設計ではページ送りUIを暫定採用（🟡）。実装時に再確認が望ましい。
- 楽観的更新の適用範囲（お気に入り・status等）は一般的パターンからの推測（🟡）であり、実装フェーズでの検証が必要。

### 信頼性レベル分布

**ヒアリング前（要件定義書ベース）**:
- 🔵 青信号: 32件 (62%)
- 🟡 黄信号: 19件 (37%)
- 🔴 赤信号: 0件 (0%)

**ヒアリング後（設計文書全体）**:
- 🔵 青信号: 30件 (61%) （+ フォーム/型/コンポーネント/API層/通知UI方針が新たに🔵化）
- 🟡 黄信号: 14件 (29%) （ページング方式・楽観的更新等のUI実装詳細）
- 🔴 赤信号: 3件 (6%) （可用性要件・バッチ処理対象外の明記等、PRDに記載がない判断）

## 関連文書

- **アーキテクチャ設計**: [architecture.md](architecture.md)
- **データフロー**: [dataflow.md](dataflow.md)
- **型定義**: [interfaces.ts](interfaces.ts)
- **要件定義**: [requirements.md](../../spec/frontend-collection-ui/requirements.md)
