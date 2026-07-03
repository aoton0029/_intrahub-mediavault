# frontend-ui-compliance データフロー図

**作成日**: 2026-07-03
**関連アーキテクチャ**: [architecture.md](architecture.md)
**関連要件定義**: [requirements.md](../../spec/frontend-ui-compliance/requirements.md)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: EARS要件定義書・設計文書・ユーザヒアリングを参考にした確実なフロー
- 🟡 **黄信号**: EARS要件定義書・設計文書・ユーザヒアリングから妥当な推測によるフロー
- 🔴 **赤信号**: EARS要件定義書・設計文書・ユーザヒアリングにない推測によるフロー

---

## トークン反映の全体フロー 🔵

**信頼性**: 🔵 *requirements.md REQ-001・REQ-402・ヒアリング回答（トークン反映方法）より*

```mermaid
flowchart TD
    A["_shared.css の値\n（--bg-app, --text-primary 等）"] --> B["frontend/src/index.css の\n:root トークンを直接置換"]
    B --> C["Tailwind CSS 4 @theme で\nCSS変数をユーティリティに連携"]
    C --> D["shadcnコンポーネントの\noklch系トークン（--primary等）を\n置換後の値で上書き（REQ-402）"]
    D --> E["各ページ・コンポーネントが\n再ビルドなしで新トークンを反映"]

    F["既存 media_type別アクセントカラー\n（--accent-anime 等8色）"] -->|"独立トークンとして維持\n(REQ-002)"| C
```

## コンポーネント拡張のデータフロー

### Sidebar拡張（REQ-003） 🔵

**信頼性**: 🔵 *requirements.md REQ-003・PRD「全体一覧」節より*

```mermaid
flowchart TD
    A[Sidebar.tsx 既存実装] --> B{モックアップ要素を追加}
    B --> C[ブランドロゴ表示\nドットアイコン+MediaVault]
    B --> D[ナビ項目の件数バッジ\n.nav-item .count]
    B --> E[サブカテゴリのインデント階層\n.nav-item.indent]
    B --> F["ライブラリ」セクション見出し\n.nav-section-label]
    B --> G[設定の下部固定配置]
    C --> H[Sidebar 表示更新]
    D --> H
    E --> H
    F --> H
    G --> H
```

**備考**: 件数バッジの値は既存のAPI取得データ（items一覧のcount）をそのまま表示に利用する想定 🟡（表示ロジックのみ追加、データ取得フローは変更なし）

### MediaCard拡張（REQ-004） 🔵

**信頼性**: 🔵 *requirements.md REQ-004・受け入れ基準より*

```mermaid
flowchart TD
    A[MediaCard.tsx 既存実装\n（アイテムデータ props）] --> B{視覚要素を追加}
    B --> C[.cover グラデーション\nプレースホルダ表示]
    B --> D[.badge media_typeバッジ\nオーバーレイ]
    B --> E["★ .fav\nfavorite=true の場合表示"]
    B --> F[.status-dot\nprogress/done/none 色分け]
    C --> G[MediaCard 表示更新]
    D --> G
    E --> G
    F --> G
```

### RootLayout / app-shell グリッド化（REQ-006） 🔵

**信頼性**: 🔵 *requirements.md REQ-006・受け入れ基準より*

```mermaid
flowchart TD
    A[RootLayout.tsx] --> B{画面種別を判定}
    B -->|一覧・検索・フォーム・設定画面| C[".app-shell\n(sidebar + main の2カラム)"]
    B -->|アイテム詳細画面| D[".app-shell.has-properties\n(sidebar + main + properties列 3カラム)"]
    D --> E["properties列は空間のみ確保\n（中身は次回以降、スコープ外）"]
    C --> F[Outlet でページ描画]
    E --> F
```

**備考**: properties列は`--properties-w: 300px`分のグリッド列を確保するのみで、中身のコンポーネント実装はスコープ外（ヒアリングQ1/Q4）🔵

## 画面別スタイル適用フロー 🔵

**信頼性**: 🔵 *requirements.md画面別要件（REQ-005〜010）より*

```mermaid
sequenceDiagram
    participant U as ユーザー
    participant P as 各ページ (HomePage等)
    participant C as 共通コンポーネント (TagPill/EmptyState/FilterBar)
    participant S as shadcn/ui (Button variant拡張)
    participant CSS as index.css (置換済みトークン)

    U->>P: 画面を開く
    P->>CSS: CSS変数を参照（背景・文字・アクセント色）
    P->>C: 新規共通コンポーネントを描画\n（フィルタバー・タグピル・空状態等）
    P->>S: 既存Button等のvariantで\nボタン群を描画（.btn-accent等相当）
    C-->>P: 描画結果
    S-->>P: 描画結果
    P-->>U: モックアップ準拠の画面表示
```

**注記**: このフローはAPIデータ取得・状態管理には関与しない。既存の[frontend-collection-ui](../frontend-collection-ui/dataflow.md)のデータ取得フローに視覚表現層を上乗せするのみ 🔵

## エラーハンドリングフロー 🔴

**信頼性**: 🔴 *要件定義に記載なし*

本要件は視覚表現の変更のみでビジネスロジック・エラーハンドリングを変更しないため、エラーハンドリングフローの新規追加は発生しない。既存の[frontend-collection-ui](../frontend-collection-ui/architecture.md)のエラーハンドリング（`apiClient` → `ApiClientError` → sonner toast）をそのまま利用する。

## 関連文書

- **アーキテクチャ**: [architecture.md](architecture.md)
- **要件定義**: [requirements.md](../../spec/frontend-ui-compliance/requirements.md)
- **参照設計（既存データフロー）**: [frontend-collection-ui/dataflow.md](../frontend-collection-ui/dataflow.md)

## 信頼性レベルサマリー

- 🔵 青信号: 10件 (77%)
- 🟡 黄信号: 2件 (15%)
- 🔴 赤信号: 1件 (8%)

**品質評価**: 高品質（要件定義書・受け入れ基準との対応が明確。件数バッジのデータソースのみ推測🟡）
