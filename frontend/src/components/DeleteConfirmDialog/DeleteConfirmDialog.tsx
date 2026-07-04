import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'

export interface DeleteConfirmDialogProps {
  /** ダイアログの表示/非表示 */
  open: boolean
  /** 確認文言に表示する作品タイトル */
  itemTitle: string
  /** 削除API呼び出し中の表示制御用（TASK-0021で使用） */
  isDeleting?: boolean
  /** 確定ボタン押下時のコールバック */
  onConfirm: () => void
  /** キャンセルボタン押下時のコールバック */
  onCancel: () => void
}

export function DeleteConfirmDialog({
  open,
  itemTitle,
  isDeleting = false,
  onConfirm,
  onCancel,
}: DeleteConfirmDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={(next) => !next && onCancel()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>作品を削除しますか?</AlertDialogTitle>
          <AlertDialogDescription>
            {`『${itemTitle}』を削除します。この操作は取り消せません。`}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>キャンセル</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            disabled={isDeleting}
            onClick={onConfirm}
          >
            削除
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
