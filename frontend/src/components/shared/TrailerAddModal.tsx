import { useState } from "react";
import { toast } from "sonner";
import { Modal } from "./Modal";

export function TrailerAddModal({
  open,
  onClose,
  onAdd,
}: {
  open: boolean;
  onClose: () => void;
  onAdd: (url: string, label?: string) => Promise<unknown>;
}) {
  const [url, setUrl] = useState("https://");
  const [label, setLabel] = useState("");
  const [isSubmitting, setSubmitting] = useState(false);

  async function handleSubmit() {
    if (!url.trim() || url.trim() === "https://") {
      toast.error("トレーラー URL を入力してください。");
      return;
    }

    setSubmitting(true);
    try {
      await onAdd(url.trim(), label.trim() || undefined);
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "トレーラーの追加に失敗しました。");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="トレーラーを追加" maxWidth={480}>
      <div className="form-grid">
        <div className="form-field full">
          <label>
            トレーラー URL <span className="required">*</span>
          </label>
          <input
            type="text"
            placeholder="https://www.youtube.com/watch?v=..."
            value={url}
            onChange={(event) => setUrl(event.target.value)}
          />
        </div>
        <div className="form-field full">
          <label>トレーラー名</label>
          <input
            type="text"
            placeholder="本予告編"
            value={label}
            onChange={(event) => setLabel(event.target.value)}
          />
        </div>
      </div>

      <div className="form-actions">
        <button type="button" className="btn btn-accent" disabled={isSubmitting} onClick={() => void handleSubmit()}>
          追加
        </button>
        <button type="button" className="btn btn-ghost" onClick={onClose}>
          キャンセル
        </button>
      </div>
    </Modal>
  );
}
