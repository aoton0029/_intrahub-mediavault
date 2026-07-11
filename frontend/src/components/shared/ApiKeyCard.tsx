import { useState } from "react";
import { FiKey, FiLink, FiSave } from "react-icons/fi";

type ApiKeyCardProps = {
  provider: string;
  keyMasked: string;
  onEdit?: () => void;
  variant?: "edit-link" | "inline-save";
  onSave?: (value: string) => void;
  saving?: boolean;
  requiresKey?: boolean;
};

export function ApiKeyCard({
  provider,
  keyMasked,
  onEdit,
  variant = "edit-link",
  onSave,
  saving = false,
  requiresKey = true,
}: ApiKeyCardProps) {
  const [value, setValue] = useState("");

  return (
    <div className="kv-card">
      <div>
        <div className="provider">{provider}</div>
        <div className="key">{keyMasked}</div>
      </div>
      {variant === "inline-save" ? (
        requiresKey ? (
          <div className="filter-bar" style={{ justifyContent: "flex-end" }}>
            <input
              aria-label={`${provider} APIキー`}
              type="password"
              placeholder="APIキーを入力"
              value={value}
              disabled={saving}
              onChange={(event) => setValue(event.target.value)}
              style={{ width: 220 }}
            />
            <button
              type="button"
              className="btn btn-accent btn-sm"
              disabled={saving}
              onClick={() => {
                onSave?.(value);
                setValue("");
              }}
            >
              <FiSave className="icon" />
              {saving ? "保存中..." : "保存"}
            </button>
          </div>
        ) : (
          <span className="field-hint">
            <FiKey className="icon" />
            設定不要
          </span>
        )
      ) : (
        <button type="button" className="btn btn-ghost btn-sm" onClick={onEdit}>
          <FiLink className="icon" />
          編集
        </button>
      )}
    </div>
  );
}
