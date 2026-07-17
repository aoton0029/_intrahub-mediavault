import { useState } from "react";
import { toast } from "sonner";
import { Modal } from "./Modal";

export function LinkAddModal({
  open,
  onClose,
  onAdd,
  defaultUrl = "https://",
}: {
  open: boolean;
  onClose: () => void;
  onAdd: (label: string, url: string) => Promise<unknown>;
  defaultUrl?: string;
}) {
  const [label, setLabel] = useState("公式サイト");
  const [url, setUrl] = useState(defaultUrl);
  const [isSubmitting, setSubmitting] = useState(false);

  async function handleSubmit() {
    if (!label.trim()) {
      toast.error("リンク名を入力してください。");
      return;
    }
    if (!url.trim() || url.trim() === "https://") {
      toast.error("URL を入力してください。");
      return;
    }

    setSubmitting(true);
    try {
      await onAdd(label.trim(), url.trim());
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "リンクの追加に失敗しました。");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="リンクを追加" maxWidth={480}>
      <div className="form-grid">
        <div className="form-field full">
          <label>
            リンク名 <span className="required">*</span>
          </label>
          <input type="text" placeholder="公式サイト" value={label} onChange={(event) => setLabel(event.target.value)} />
        </div>
        <div className="form-field full">
          <label>
            URL <span className="required">*</span>
          </label>
          <input
            type="text"
            placeholder="https://..."
            value={url}
            onChange={(event) => setUrl(event.target.value)}
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
