import { FiLink } from "react-icons/fi";

export function ApiKeyCard({ provider, keyMasked, onEdit }: { provider: string; keyMasked: string; onEdit: () => void }) {
  return (
    <div className="kv-card">
      <div>
        <div className="provider">{provider}</div>
        <div className="key">{keyMasked}</div>
      </div>
      <button type="button" className="btn btn-ghost btn-sm" onClick={onEdit}>
        <FiLink className="icon" />
        編集
      </button>
    </div>
  );
}
