import { useState, useCallback } from 'react'

export function useConfirmDialog() {
  const [open, setOpen] = useState(false)
  const [pendingAction, setPendingAction] = useState<(() => void) | null>(null)

  const confirm = useCallback((action: () => void) => {
    setPendingAction(() => action)
    setOpen(true)
  }, [])

  const handleConfirm = useCallback(() => {
    pendingAction?.()
    setOpen(false)
    setPendingAction(null)
  }, [pendingAction])

  const handleCancel = useCallback(() => {
    setOpen(false)
    setPendingAction(null)
  }, [])

  return { open, confirm, handleConfirm, handleCancel }
}
