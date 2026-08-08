import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Modal } from "./Modal";

export type LocatorType = "page" | "timestamp" | "location" | "chapter" | "none";

export type CitationFormValues = {
  quoteText: string;
  note: string;
  locatorType: LocatorType;
  pageNumber: number | null;
  timestampSeconds: number | null;
  locationNumber: number | null;
  chapter: string;
};

export type CitationFormInitial = {
  quoteText: string;
  note: string | null;
  locatorType: LocatorType;
  pageNumber: number | null;
  timestampSeconds: number | null;
  locationNumber: number | null;
  chapter: string | null;
};

const LOCATOR_TYPE_LABELS: Record<LocatorType, string> = {
  page: "ページ番号",
  timestamp: "再生秒数",
  location: "位置No.（電子書籍）",
  chapter: "章・話数",
  none: "なし",
};

const EMPTY_VALUES: CitationFormValues = {
  quoteText: "",
  note: "",
  locatorType: "none",
  pageNumber: null,
  timestampSeconds: null,
  locationNumber: null,
  chapter: "",
};

function toFormValues(initial?: CitationFormInitial): CitationFormValues {
  if (!initial) {
    return EMPTY_VALUES;
  }
  return {
    quoteText: initial.quoteText,
    note: initial.note ?? "",
    locatorType: initial.locatorType,
    pageNumber: initial.pageNumber,
    timestampSeconds: initial.timestampSeconds,
    locationNumber: initial.locationNumber,
    chapter: initial.chapter ?? "",
  };
}

export function CitationFormModal({
  open,
  onClose,
  onSubmit,
  initial,
}: {
  open: boolean;
  onClose: () => void;
  onSubmit: (values: CitationFormValues) => Promise<unknown>;
  initial?: CitationFormInitial;
}) {
  const isEditing = Boolean(initial);
  const [values, setValues] = useState<CitationFormValues>(() => toFormValues(initial));
  const [isSubmitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (open) {
      setValues(toFormValues(initial));
    }
  }, [open, initial]);

  async function handleSubmit() {
    if (!values.quoteText.trim()) {
      toast.error("引用文を入力してください。");
      return;
    }

    setSubmitting(true);
    try {
      await onSubmit({ ...values, quoteText: values.quoteText.trim() });
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "引用の保存に失敗しました。");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title={isEditing ? "引用を編集" : "引用を追加"} maxWidth={520}>
      <div className="form-grid">
        <div className="form-field full">
          <label>
            引用文 <span className="required">*</span>
          </label>
          <textarea
            rows={4}
            placeholder="引用したい文章を入力してください"
            value={values.quoteText}
            onChange={(event) => setValues((prev) => ({ ...prev, quoteText: event.target.value }))}
          />
        </div>
        <div className="form-field full">
          <label>メモ（任意）</label>
          <textarea
            rows={2}
            placeholder="自分のコメント・所感など"
            value={values.note}
            onChange={(event) => setValues((prev) => ({ ...prev, note: event.target.value }))}
          />
        </div>
        <div className="form-field full">
          <label>付加情報の種類</label>
          <select
            value={values.locatorType}
            onChange={(event) =>
              setValues((prev) => ({
                ...prev,
                locatorType: event.target.value as LocatorType,
                pageNumber: null,
                timestampSeconds: null,
                locationNumber: null,
                chapter: "",
              }))
            }
          >
            {(Object.keys(LOCATOR_TYPE_LABELS) as LocatorType[]).map((type) => (
              <option key={type} value={type}>
                {LOCATOR_TYPE_LABELS[type]}
              </option>
            ))}
          </select>
        </div>
        {values.locatorType === "page" ? (
          <div className="form-field full">
            <label>ページ番号</label>
            <input
              type="number"
              min={1}
              placeholder="例: 128"
              value={values.pageNumber ?? ""}
              onChange={(event) =>
                setValues((prev) => ({ ...prev, pageNumber: event.target.value ? Number(event.target.value) : null }))
              }
            />
          </div>
        ) : null}
        {values.locatorType === "timestamp" ? (
          <div className="form-field full">
            <label>再生秒数</label>
            <input
              type="number"
              min={0}
              placeholder="例: 754（秒）"
              value={values.timestampSeconds ?? ""}
              onChange={(event) =>
                setValues((prev) => ({
                  ...prev,
                  timestampSeconds: event.target.value ? Number(event.target.value) : null,
                }))
              }
            />
          </div>
        ) : null}
        {values.locatorType === "location" ? (
          <div className="form-field full">
            <label>位置No.</label>
            <input
              type="number"
              min={1}
              placeholder="例: 1234"
              value={values.locationNumber ?? ""}
              onChange={(event) =>
                setValues((prev) => ({
                  ...prev,
                  locationNumber: event.target.value ? Number(event.target.value) : null,
                }))
              }
            />
          </div>
        ) : null}
        {values.locatorType === "chapter" ? (
          <div className="form-field full">
            <label>章・話数</label>
            <input
              type="text"
              placeholder="例: 第3章"
              value={values.chapter}
              onChange={(event) => setValues((prev) => ({ ...prev, chapter: event.target.value }))}
            />
          </div>
        ) : null}
      </div>

      <div className="form-actions">
        <button type="button" className="btn btn-accent" disabled={isSubmitting} onClick={() => void handleSubmit()}>
          {isSubmitting ? "保存中…" : isEditing ? "保存" : "追加"}
        </button>
        <button type="button" className="btn btn-ghost" onClick={onClose}>
          キャンセル
        </button>
      </div>
    </Modal>
  );
}
