import { useState } from "react";
import { LoadMoreSentinel, MediaCard, useInfiniteScroll } from "@/components/shared";
import { useMediaListData, type MediaType } from "@/hooks/useMediaListData";
import type { CollageSelectedItem } from "@/hooks/useCollageBuilder";

const MEDIA_TYPE_LABELS: Record<MediaType, string> = {
  anime: "アニメ",
  movie: "映画",
  drama: "ドラマ",
  manga: "漫画",
  novel: "小説",
  game: "ゲーム",
  academic_book: "学術書",
};

export function ItemPickerPanel({
  mediaType,
  onSelectItem,
}: {
  mediaType: MediaType;
  onSelectItem: (item: CollageSelectedItem) => void;
}) {
  const [title, setTitle] = useState("");
  const { items, hasNextPage, fetchNextPage, isFetchingNextPage } = useMediaListData({ mediaType, title });

  const sentinelRef = useInfiniteScroll(() => {
    if (hasNextPage && !isFetchingNextPage) {
      void fetchNextPage();
    }
  }, Boolean(hasNextPage) && !isFetchingNextPage);

  return (
    <div className="collage-picker">
      <div className="search-box">
        🔍 <input
          type="text"
          placeholder="作品名で検索…"
          value={title}
          onChange={(event) => setTitle(event.target.value)}
        />
      </div>

      <div className="card-grid is-compact">
        {items.map((item) => (
          <MediaCard
            key={item.id}
            title={item.title}
            badge={MEDIA_TYPE_LABELS[item.media_type] ?? item.media_type}
            variant="search-result"
            imageUrl={item.cover_image_url}
            actionLabel="このマスに反映"
            onAction={() => onSelectItem({ id: item.id, title: item.title, imageUrl: item.cover_image_url })}
          />
        ))}
      </div>

      <div ref={sentinelRef}>
        <LoadMoreSentinel loading={Boolean(hasNextPage) && isFetchingNextPage} text={hasNextPage ? "もっと見る" : "すべて読み込みました"} />
      </div>
    </div>
  );
}
