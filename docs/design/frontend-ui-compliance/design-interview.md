# frontend-ui-compliance 設計ヒアリング記録

**作成日**: 2026-07-03
**ヒアリング実施**: step4 既存情報ベースの差分ヒアリング（軽量設計）

## ヒアリング目的

要件定義書（`docs/spec/frontend-ui-compliance/requirements.md`）とPRDのヒアリング記録（`docs/spec/frontend-ui-compliance/interview-record.md`）はスコープ境界（Propertiesパネル除外等）を明確化済みだったため、本設計段階では設計上の技術選択（トークン反映方法・コンポーネント実装方針・優先順位）に絞ってヒアリングを実施した。

## 質問と回答

### Q1: デザイントークンの適用方法について、`_shared.css`の値をどう反映するか

**質問日時**: 2026-07-03
**カテゴリ**: 技術選択
**背景**: `_shared.css`をそのままインポートして二重管理するか、既存`index.css`の変数を置換するかで実装コスト・保守性が大きく異なるため確認が必要だった。

**回答**: `index.css`のCSS変数を直接置換（推奨）

**信頼性への影響**:
- architecture.md「デザイントークン層」の適用方法を🔵で確定。Tailwind CSS 4の`@theme`経由での連携方針もこの回答に基づき確定

---

### Q2: 共通コンポーネント（ボタン・タグピル・空状態等）の実装方針

**質問日時**: 2026-07-03
**カテゴリ**: 技術選択
**背景**: 既存shadcn/uiコンポーネントを拡張するか、モックアップのCSSクラスをそのまま素のHTML+CSSとして移植するかで、既存コードベースとの整合性が変わるため確認した。

**回答**: 既存shadcn/uiコンポーネントを拡張（推奨）

**信頼性への影響**:
- architecture.md「共通コンポーネント層」を🔵で確定。`Button`のvariant拡張、新規は`TagPill`/`EmptyState`/`FilterBar`のみという分担が明確化された

---

### Q3: 軽量スコープでの実装優先順位

**質問日時**: 2026-07-03
**カテゴリ**: 優先順位
**背景**: requirements.mdのヒアリング記録Q3で全画面が対象と確定済みだが、設計段階でフェーズ分けの要否を再確認した。

**回答**: 全画面同時対応（推奨）

**信頼性への影響**:
- architecture.md「画面別スタイル適用」を🔵で確定。フェーズ分けは行わず、5画面すべてを1つの設計スコープとして扱う

## ヒアリング結果サマリー

### 確認できた事項
- デザイントークンは`index.css`の変数を直接置換し、`_shared.css`との二重管理はしない
- 共通コンポーネントは既存shadcn/uiの拡張を基本とし、新規コンポーネントは最小限（TagPill/EmptyState/FilterBar）に留める
- 5画面すべてを同時にスタイル対応する（フェーズ分けなし）

### 設計方針の決定事項
- Tailwind CSS 4 `@theme`経由でCSS変数とユーティリティクラス・shadcnトークンを連携させる
- `Sidebar.tsx` / `MediaCard.tsx` / `RootLayout.tsx`は既存コンポーネントを拡張し、新規ファイル追加は行わない

### 残課題
- 件数バッジ（Sidebarのnav-item .count）の具体的なデータ取得元（既存API集計値の利用可否）は未確認 🟡
- Tailwind CSS 4の`@theme`とCSS変数直接参照のどちらを優先するかの実装詳細は次工程（タスク分割・実装）で確定

### 信頼性レベル分布

**ヒアリング前**（要件定義書・PRDのみからの推測時点）:
- 🔵 青信号: 約10件
- 🟡 黄信号: 約4件
- 🔴 赤信号: 約2件

**ヒアリング後**:
- 🔵 青信号: 25件（architecture.md 15件 + dataflow.md 10件、+15）
- 🟡 黄信号: 5件（architecture.md 3件 + dataflow.md 2件、+1）
- 🔴 赤信号: 3件（architecture.md 2件 + dataflow.md 1件、要件定義に元々ないスケーラビリティ・可用性・エラーハンドリング項目）

## 関連文書

- **アーキテクチャ設計**: [architecture.md](architecture.md)
- **データフロー**: [dataflow.md](dataflow.md)
- **要件定義**: [requirements.md](../../spec/frontend-ui-compliance/requirements.md)
- **要件定義ヒアリング記録（前段）**: [../../spec/frontend-ui-compliance/interview-record.md](../../spec/frontend-ui-compliance/interview-record.md)
