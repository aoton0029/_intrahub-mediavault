import { useRef, useState } from "react";
import { FiFile, FiUpload } from "react-icons/fi";
import { toast } from "sonner";
import { Modal } from "./Modal";

type FileAddMode = "upload" | "path";

export function FileAddModal({
  open,
  onClose,
  onAddByPath,
  onUpload,
}: {
  open: boolean;
  onClose: () => void;
  onAddByPath: (path: string, label?: string) => Promise<unknown>;
  onUpload: (file: File, label?: string) => Promise<unknown>;
}) {
  const [mode, setMode] = useState<FileAddMode>("upload");
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [path, setPath] = useState("");
  const [label, setLabel] = useState("");
  const [isSubmitting, setSubmitting] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  async function handleSubmit() {
    if (mode === "upload" && !selectedFile) {
      toast.error("アップロードするファイルを選択してください。");
      return;
    }
    if (mode === "path" && !path.trim()) {
      toast.error("ファイルパスを入力してください。");
      return;
    }

    setSubmitting(true);
    try {
      if (mode === "upload") {
        await onUpload(selectedFile!, label.trim() || undefined);
      } else {
        await onAddByPath(path.trim(), label.trim() || undefined);
      }
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "ファイルの追加に失敗しました。");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="ファイルを追加" maxWidth={520}>
      <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
        <button
          type="button"
          className={mode === "upload" ? "btn btn-accent btn-sm" : "btn btn-ghost btn-sm"}
          onClick={() => setMode("upload")}
        >
          <FiUpload className="icon" />
          ファイルをアップロード
        </button>
        <button
          type="button"
          className={mode === "path" ? "btn btn-accent btn-sm" : "btn btn-ghost btn-sm"}
          onClick={() => setMode("path")}
        >
          <FiFile className="icon" />
          パスを入力
        </button>
      </div>

      {mode === "upload" ? (
        <div style={{ marginBottom: 14 }}>
          <input
            ref={fileInputRef}
            type="file"
            style={{ display: "none" }}
            onChange={(event) => setSelectedFile(event.target.files?.[0] ?? null)}
          />
          <button type="button" className="btn btn-ghost" onClick={() => fileInputRef.current?.click()}>
            <FiUpload className="icon" />
            ファイルを選択…
          </button>
          {selectedFile ? (
            <p style={{ marginTop: 8, fontSize: 12 }}>
              選択中: {selectedFile.name}（{(selectedFile.size / 1024 / 1024).toFixed(2)} MB）
            </p>
          ) : (
            <p style={{ marginTop: 8, fontSize: 12, opacity: 0.7 }}>ファイルが選択されていません（最大 100MB）</p>
          )}
        </div>
      ) : (
        <div className="form-grid" style={{ marginBottom: 14 }}>
          <div className="form-field full">
            <label>
              ファイルパス <span className="required">*</span>
            </label>
            <input
              type="text"
              placeholder="例: /srv/files/pdf/example.pdf"
              value={path}
              onChange={(event) => setPath(event.target.value)}
            />
          </div>
        </div>
      )}

      <div className="form-grid">
        <div className="form-field full">
          <label>ファイル名（任意）</label>
          <input
            type="text"
            placeholder="例: パンフレットPDF"
            value={label}
            onChange={(event) => setLabel(event.target.value)}
          />
        </div>
      </div>

      <p style={{ marginTop: 10, fontSize: 12, opacity: 0.7 }}>種別（pdf / image / video など）は拡張子から自動判定されます。</p>

      <div className="form-actions">
        <button type="button" className="btn btn-accent" disabled={isSubmitting} onClick={() => void handleSubmit()}>
          {isSubmitting ? "追加中…" : "追加"}
        </button>
        <button type="button" className="btn btn-ghost" onClick={onClose}>
          キャンセル
        </button>
      </div>
    </Modal>
  );
}
