import { useState } from "react";
import { Modal } from "./Modal";

export function ConfirmDialog({
  open,
  onClose,
  onConfirm,
  title,
  message,
  confirmLabel = "削除",
}: {
  open: boolean;
  onClose: () => void;
  onConfirm: () => Promise<unknown>;
  title: string;
  message: string;
  confirmLabel?: string;
}) {
  const [isSubmitting, setSubmitting] = useState(false);

  async function handleConfirm() {
    setSubmitting(true);
    try {
      await onConfirm();
      onClose();
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title={title} maxWidth={420}>
      <p>{message}</p>
      <div className="form-actions">
        <button type="button" className="btn btn-danger" disabled={isSubmitting} onClick={() => void handleConfirm()}>
          {isSubmitting ? "処理中…" : confirmLabel}
        </button>
        <button type="button" className="btn btn-ghost" onClick={onClose}>
          キャンセル
        </button>
      </div>
    </Modal>
  );
}
