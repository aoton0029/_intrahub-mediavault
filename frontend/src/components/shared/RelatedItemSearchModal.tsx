import { useState } from "react";
import { toast } from "sonner";
import { Modal } from "./Modal";
import { MediaCard } from "./MediaCard";
import { useItemRelationSearch } from "@/hooks/useItemRelationSearch";

const MEDIA_TYPE_LABELS: Record<string, string> = {
  anime: "アニメ",
  movie: "映画",
  drama: "ドラマ",
  manga: "漫画",
  novel: "小説",
  game: "ゲーム",
  academic_book: "学術書",
  paper: "論文",
};

export type RelationType = "reference" | "dlc";

export function RelatedItemSearchModal({
  open,
  onClose,
  excludeItemIds,
  alreadyRelatedIds,
  onSelect,
}: {
  open: boolean;
  onClose: () => void;
  excludeItemIds: string[];
  alreadyRelatedIds: string[];
  onSelect: (itemId: string, relationType: RelationType) => Promise<unknown>;
}) {
  const [title, setTitle] = useState("");
  const [relationType, setRelationType] = useState<RelationType>("reference");
  const { results, isLoading } = useItemRelationSearch(title);

  const visibleResults = results.filter((item) => !excludeItemIds.includes(item.id));

  async function handleSelect(itemId: string) {
    try {
      await onSelect(itemId, relationType);
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "関連作品の追加に失敗しました。");
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="関連作品を追加" maxWidth={640}>
      <div className="filter-bar" style={{ marginBottom: 16 }}>
        <div className="search-box" style={{ flex: 1 }}>
          🔍 <input
            type="text"
            placeholder="作品名で検索…"
            value={title}
            onChange={(event) => setTitle(event.target.value)}
          />
        </div>
        <div className="filter-select">
          関係
          <select value={relationType} onChange={(event) => setRelationType(event.target.value as RelationType)}>
            <option value="reference">参照(reference)</option>
            <option value="dlc">DLC</option>
          </select>
        </div>
      </div>

      <div className="card-grid is-compact">
        {isLoading ? <p>検索中です...</p> : null}
        {!isLoading && title.trim().length > 0 && visibleResults.length === 0 ? <p>該当する作品が見つかりませんでした。</p> : null}
        {visibleResults.map((item) => (
          <MediaCard
            key={item.id}
            title={item.title}
            badge={MEDIA_TYPE_LABELS[item.media_type] ?? item.media_type}
            meta={<span>{item.release_date ? item.release_date.slice(0, 4) : "年不明"}年</span>}
            variant="search-result"
            imageUrl={item.cover_image_url}
            imported={alreadyRelatedIds.includes(item.id)}
            importedLabel="登録済み"
            actionLabel="選択"
            onAction={() => void handleSelect(item.id)}
          />
        ))}
      </div>

      <div className="form-actions">
        <button type="button" className="btn btn-ghost" onClick={onClose}>
          キャンセル
        </button>
      </div>
    </Modal>
  );
}
