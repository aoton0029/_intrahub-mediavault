import { useState } from "react";
import { FiTv } from "react-icons/fi";
import { toast } from "sonner";
import { Modal } from "./Modal";

export type StreamingPlatform = "netflix" | "amazon_prime" | "disney_plus" | "dmm_tv" | "apple_tv";

const PLATFORM_OPTIONS: { value: StreamingPlatform; label: string; iconSrc: string | null }[] = [
  { value: "netflix", label: "Netflix", iconSrc: "/netflix.png" },
  { value: "amazon_prime", label: "Amazon Prime Video", iconSrc: "/prime_video.png" },
  { value: "disney_plus", label: "Disney+", iconSrc: "/disney_plus.png" },
  { value: "dmm_tv", label: "DMM TV", iconSrc: "/dmm_tv.png" },
  { value: "apple_tv", label: "Apple TV", iconSrc: null },
];

export function StreamingLinkAddModal({
  open,
  onClose,
  onAdd,
  registeredPlatforms = [],
}: {
  open: boolean;
  onClose: () => void;
  onAdd: (platform: StreamingPlatform, url: string) => Promise<unknown>;
  registeredPlatforms?: StreamingPlatform[];
}) {
  const [platform, setPlatform] = useState<StreamingPlatform | null>(null);
  const [url, setUrl] = useState("https://");
  const [isSubmitting, setSubmitting] = useState(false);

  async function handleSubmit() {
    if (!platform) {
      toast.error("配信サービスを選択してください。");
      return;
    }
    if (!url.trim() || url.trim() === "https://") {
      toast.error("配信 URL を入力してください。");
      return;
    }

    setSubmitting(true);
    try {
      await onAdd(platform, url.trim());
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "配信リンクの追加に失敗しました。");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="配信サイトを追加" maxWidth={520}>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(140px, 1fr))",
          gap: 10,
          marginBottom: 18,
        }}
      >
        {PLATFORM_OPTIONS.map((option) => {
          const isSelected = platform === option.value;
          const isRegistered = registeredPlatforms.includes(option.value);
          return (
            <button
              key={option.value}
              type="button"
              className={isSelected ? "btn btn-accent" : "btn btn-ghost"}
              style={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: 6,
                padding: "12px 8px",
                height: "auto",
              }}
              onClick={() => setPlatform(option.value)}
            >
              {option.iconSrc ? (
                <img
                  src={option.iconSrc}
                  alt={option.label}
                  style={{ width: 36, height: 36, objectFit: "contain", borderRadius: 8 }}
                />
              ) : (
                <FiTv style={{ width: 36, height: 36 }} />
              )}
              <span style={{ fontSize: 12 }}>{option.label}</span>
              {isRegistered ? <span style={{ fontSize: 10, opacity: 0.7 }}>登録済み</span> : null}
            </button>
          );
        })}
      </div>

      <div className="form-grid">
        <div className="form-field full">
          <label>
            配信 URL <span className="required">*</span>
          </label>
          <input
            type="text"
            placeholder="https://www.netflix.com/title/..."
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
