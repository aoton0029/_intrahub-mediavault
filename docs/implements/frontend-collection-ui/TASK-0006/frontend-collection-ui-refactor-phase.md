# TASK-0006: 共通UIコンポーネント実装 - Refactorフェーズ記録

**機能名**: 共通UIコンポーネント（MediaCard / MediaTypeBadge / FilterBar / EmptyState / ConfirmDialog）
**タスクID**: TASK-0006
**要件名**: frontend-collection-ui
**フェーズ**: Refactor（品質改善）
**作成日**: 2026-06-30

---

## 1. リファクタリング方針

Greenフェーズの課題・改善点（`frontend-collection-ui-green-phase.md` 5節）に基づき、以下を実施した。

- `npx shadcn@latest add badge dialog` を `frontend/` ディレクトリで実行し、shadcn/ui の Badge / Dialog コンポーネントを正式導入（ネットワーク・CLIともに利用可能だった）
- `MediaTypeBadge` を独自 `span` 実装から shadcn/ui `Badge` ベースに置き換え
- `ConfirmDialog` を独自モーダル DOM 実装から shadcn/ui `Dialog`（Radix UI Dialog Primitive ベース）に置き換え
- 機能的な変更（新機能追加）は行わず、既存の44テスト（実際は他コンポーネント分含め52テスト）をすべて維持

`MediaCard` / `FilterBar` / `EmptyState` は Green フェーズ実装のままで変更不要と判断した（`MediaCard`・`EmptyState` は既に `Button` 等の shadcn/ui 基底コンポーネントを利用済み、`FilterBar` は Phase 2 で詳細実装予定のコンテナのため）。

---

## 2. 追加された shadcn/ui コンポーネント

`npx shadcn@latest add badge dialog` の実行により以下を新規生成（`components.json` の既存設定に準拠）。

- `frontend/src/components/ui/badge.tsx`: cva ベースの `Badge`（variant: default/secondary/destructive/outline/ghost/link）
- `frontend/src/components/ui/dialog.tsx`: Radix UI `Dialog` Primitive ラッパー（`Dialog`, `DialogContent`, `DialogHeader`, `DialogFooter`, `DialogTitle`, `DialogDescription`, `DialogOverlay`, `DialogPortal`, `DialogClose`, `DialogTrigger`）

既存の `frontend/src/components/ui/button.tsx` は内容が同一のためCLIにより自動的にスキップされた（上書きなし）。

---

## 3. 改善後コード

### 3.1 MediaTypeBadge.tsx（リファクタ後）

```typescript
import { cn } from '@/lib/utils'
import { Badge } from '@/components/ui/badge'
import { getMediaTypeAccentClass } from '@/lib/media-type-accent'
import type { MediaType } from '@/types'

interface MediaTypeBadgeProps {
  mediaType: MediaType
}

// 【ラベル変換テーブル】: mediaType → 日本語ラベルの決定的マッピング
// 🟡 信頼性レベル: 設計文書に日本語ラベルの明記なし、妥当な推測
const MEDIA_TYPE_LABEL: Record<MediaType, string> = {
  anime: 'アニメ',
  movie: '映画',
  drama: 'ドラマ',
  manga: '漫画',
  novel: '小説',
  game: 'ゲーム',
  academic_book: '専門書',
  paper: '論文',
}

/**
 * 【機能概要】: MediaType を受け取り、対応するアクセントカラー・日本語ラベルでバッジ表示する
 * 【改善内容】: 独自 span 実装から shadcn/ui の Badge コンポーネントベースに置き換え。Badge の
 *   variant="outline" をベースとし、media_type 固有のアクセントカラーは className で上乗せする
 * 【設計方針】: タスク完了条件「各コンポーネントが shadcn/ui の基底コンポーネントを利用している」を満たすため、
 *   Badge の cva バリアント管理を活用しつつ、getMediaTypeAccentClass() の動的クラスを cn() で合成する
 * 【パフォーマンス】: レンダリングコストは従来の span 実装と同等（追加の状態・副作用なし）
 * 【保守性】: Badge 側の基本スタイル（角丸・パディング・フォント）は shadcn/ui 側に集約され、
 *   本コンポーネントは media_type 固有の関心事（ラベル・アクセント色）のみを担当する
 * 【テスト対応】: TC-MB-N-01, TC-MB-N-02, TC-MB-E-01, TC-MB-B-01
 * 🔵 信頼性レベル: media-type-accent.ts を直接参照。Badge への置換は note.md「アーキテクチャ制約」より
 */
export function MediaTypeBadge({ mediaType }: MediaTypeBadgeProps) {
  // 【型外入力防御】: MediaType の8値に含まれない値が渡されてもクラッシュしないようフォールバック
  const accentClass = getMediaTypeAccentClass(mediaType) ?? ''
  const label = MEDIA_TYPE_LABEL[mediaType] ?? String(mediaType)

  return (
    <Badge
      variant="outline"
      data-testid="media-type-badge"
      className={cn(accentClass)}
    >
      {label}
    </Badge>
  )
}
```

**改善ポイント**:
- shadcn/ui `Badge`（cva ベース variant 管理）を基底に利用することで、デザインシステムとの一貫性が向上
- `data-testid="media-type-badge"` を維持し、既存テストとの互換性を確保
- アクセントカラーは引き続き `getMediaTypeAccentClass()` から取得し、ハードコーディングを排除

### 3.2 ConfirmDialog.tsx（リファクタ後）

```typescript
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

interface ConfirmDialogProps {
  open: boolean
  title: string
  description?: string
  onConfirm: () => void
  onCancel: () => void
  confirmLabel?: string
  cancelLabel?: string
}

/**
 * 【機能概要】: アイテム削除等の操作に対する確認ダイアログを表示する
 * 【改善内容】: 独自実装の最小モーダル DOM から、shadcn/ui の Dialog（Radix UI Dialog Primitive ベース）に置き換えた。
 *   これによりフォーカストラップ・Escキーでの閉鎖・ポータル化・スクリーンリーダー向け aria 属性が
 *   Radix 側で自動的に提供されるようになり、アクセシビリティが向上した
 * 【設計方針】: open props を Dialog の制御 props にそのまま渡し、onOpenChange で閉鎖要求（背景クリック・Esc）を
 *   onCancel に集約する。タスク完了条件「各コンポーネントが shadcn/ui の基底コンポーネントを利用している」を満たす
 * 【パフォーマンス】: open=false 時は Radix Dialog が内容を DOM にレンダリングしないため、
 *   従来の早期 return 実装と同等にレンダリングコストが抑えられる
 * 【保守性】: DialogHeader/DialogTitle/DialogDescription/DialogFooter の役割分担により、
 *   レイアウト変更時も shadcn/ui 側の更新追従がしやすい
 * 【テスト対応】: TC-CD-N-01〜04, TC-CD-E-01, TC-CD-B-01〜02
 * 🟡 信頼性レベル: requirements.md「2.5 ConfirmDialog」より。shadcn/ui Dialog への置換は note.md「アーキテクチャ制約」より
 */
export function ConfirmDialog({
  open,
  title,
  description,
  onConfirm,
  onCancel,
  confirmLabel,
  cancelLabel,
}: ConfirmDialogProps) {
  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        // 【閉鎖要求の一元化】: 背景クリック・Esc キー等、Radix 起点の閉鎖要求を onCancel に集約する
        if (!nextOpen) {
          onCancel()
        }
      }}
    >
      <DialogContent data-testid="confirm-dialog" showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>

          {/* 【補足説明】: description は任意。指定時のみ表示する */}
          {description ? <DialogDescription>{description}</DialogDescription> : null}
        </DialogHeader>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={onCancel}>
            {cancelLabel ?? 'キャンセル'}
          </Button>
          <Button type="button" onClick={onConfirm}>
            {confirmLabel ?? 'OK'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
```

**改善ポイント**:
- Radix UI `Dialog` Primitive により、フォーカストラップ・Esc キー閉鎖・背景クリック閉鎖・ポータル化（`document.body` 直下へのレンダリング）が自動的に提供される
- `showCloseButton={false}` を指定し、`DialogContent` 既定の右上クローズボタン（「Close」）を非表示化。これにより既存テスト `TC-CD-B-02`（ボタン数2件の期待）との互換性を維持
- `onOpenChange` で Esc・背景クリック等の Radix 起点の閉鎖要求を `onCancel` に一元集約し、呼び出し側の責務を単純化
- Radix Dialog は `open=false` 時に内容を DOM から除去するため、`TC-CD-B-01`（`queryByText` が `null`）との互換性も維持

---

## 4. セキュリティレビュー

- **XSS**: `title` / `description` / `confirmLabel` / `cancelLabel` / `mediaType` ラベルはすべて React の JSX テキストノードとして描画されており、`dangerouslySetInnerHTML` 等の使用はない。React の自動エスケープにより XSS リスクは低い。
- **入力値検証**: `MediaTypeBadge` は型外の `mediaType` が渡されてもクラッシュしないようフォールバック処理済み（`?? ''`, `?? String(mediaType)`）。
- **認証・認可**: 本コンポーネント群は表示専用 Pure Component であり、認証・認可ロジックを含まない（対象外）。
- **CSRF/SQLi**: ネットワーク通信・DB アクセスを含まないため対象外。
- **フォーカス管理**: 旧実装は `aria-modal="true"` のみで実際のフォーカストラップがなく、キーボード操作で背景要素にフォーカスが漏れるリスクがあった。Radix Dialog Primitive への置き換えによりこの問題を解消。

重大な脆弱性は検出されなかった。

---

## 5. パフォーマンスレビュー

- **計算量**: いずれのコンポーネントも O(1) の表示処理のみで、ループ等の重い処理は含まれない（`MediaTypeBadge` のテーブル参照、`ConfirmDialog` の条件分岐レンダリングのみ）。
- **再レンダリング**: `Badge` / `Dialog` の置き換えにより props 構造・依存配列は変化しておらず、再レンダリングコストの増加はない。
- **バンドルサイズ**: `radix-ui`（`Dialog` Primitive 含む）は既に `package.json` の dependencies に含まれていたため、新規の重量級依存追加はなし。
- **メモリ**: `Dialog` は `open=false` 時に DOM から内容を除去するため、不要なポータル要素の常駐はない。

重大な性能課題は検出されなかった。

---

## 6. テスト実行結果

### リファクタ前（ベースライン確認）

```bash
yarn test
```

```
Test Files  7 passed (7)
     Tests  52 passed (52)
  Duration  885ms
```

個別テストで2秒以上要するものはなし。

### リファクタ後

```bash
yarn test
```

```
Test Files  7 passed (7)
     Tests  52 passed (52)
  Duration  1.03s
```

```bash
yarn lint
```

```
Done（エラー・警告なし）
```

```bash
npx tsc -b --noEmit
```

型エラーなし。

全52テスト（common コンポーネント分44件含む）がリファクタ前後で継続して成功し、機能破綻は確認されなかった。

---

## 7. 品質判定

```
✅ 高品質:
- テスト結果: リファクタ前後とも Test Files 7 passed / Tests 52 passed で継続成功
- セキュリティ: 重大な脆弱性なし（むしろ Dialog のフォーカストラップ導入でアクセシビリティ・セキュリティ姿勢が向上）
- パフォーマンス: 重大な性能課題なし（依存追加なし、再レンダリングコスト不変）
- リファクタ目標: 達成（shadcn/ui Badge/Dialog CLI導入・移行が完了、完了条件「各コンポーネントがshadcn/uiの基底コンポーネントを利用している」を充足）
- コード品質: lint・typecheck ともにクリーン
- ファイルサイズ: MediaTypeBadge.tsx 50行、ConfirmDialog.tsx 74行（いずれも500行制限内）
- ドキュメント: 本ファイルおよびメモファイルに改善内容を記録済み
```

---

## 8. 残課題（将来検討事項）

- `MediaCard`: `coverImageUrl` 未設定時の `src=""` の扱い改善（プレースホルダ画像・`loading="lazy"`）は本タスクのテスト要件外のため未対応のまま据え置き
- `EmptyState`: アイコン表示（lucide-react）等のビジュアル強化はテスト要件外のため未実装のまま据え置き
- `FilterBar`: 詳細 UI 実装は Phase 2（TASK-0010）で対応予定

---

## 9. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-verify-complete` で完全性検証を実行します。
