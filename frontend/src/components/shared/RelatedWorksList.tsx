import { FiGitBranch, FiTrash2 } from "react-icons/fi";

export type RelatedWork = { id: string; title: string; relation: string; onRemove?: (id: string) => void };

export function RelatedWorksList({ items }: { items: RelatedWork[] }) {
  return (
    <div>
      {items.map((item) => (
        <div key={item.id} className="result-row">
          <div className="thumb" />
          <div className="info">
            <div className="title">{item.title}</div>
            <div className="sub"><FiGitBranch className="icon" /> {item.relation}</div>
          </div>
          {item.onRemove ? (
            <button type="button" className="btn btn-danger btn-sm" onClick={() => item.onRemove?.(item.id)}>
              <FiTrash2 className="icon" />
              解除
            </button>
          ) : null}
        </div>
      ))}
    </div>
  );
}
