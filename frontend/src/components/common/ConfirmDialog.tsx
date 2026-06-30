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
