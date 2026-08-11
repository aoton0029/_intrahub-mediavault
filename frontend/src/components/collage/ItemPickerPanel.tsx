import { useState } from "react";
import { FiSearch } from "react-icons/fi";
import { LoadMoreSentinel, useInfiniteScroll } from "@/components/shared";
import { useMediaListData, type MediaType } from "@/hooks/useMediaListData";
import type { MediaType as CollageMediaType } from "@/config/mediaTypes";
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
  mediaType: CollageMediaType | "all";
  onSelectItem: (item: CollageSelectedItem) => void;
}) {
  const [title, setTitle] = useState("");
  const { items, hasNextPage, fetchNextPage, isFetchingNextPage } = useMediaListData({
    mediaType: mediaType === "all" ? undefined : mediaType,
    title,
  });

  const sentinelRef = useInfiniteScroll(() => {
    if (hasNextPage && !isFetchingNextPage) {
      void fetchNextPage();
    }
  }, Boolean(hasNextPage) && !isFetchingNextPage);

  return (
    <aside className="item-picker-panel">
      <label className="search-box">
        <FiSearch className="icon" />
        <input
          type="text"
          placeholder="作品名で検索…"
          value={title}
          onChange={(event) => setTitle(event.target.value)}
        />
      </label>

      <div className="picker-results">
        {items.length === 0 ? (
          <p style={{ gridColumn: "1 / -1", textAlign: "center", padding: "20px 0", color: "var(--color-text-faint)", fontSize: 12.5 }}>
            該当する作品が見つかりません
          </p>
        ) : (
          items.map((item) => (
            <div className="picker-card" key={item.id}>
              <div className="cover">
                {item.cover_image_url ? <img src={item.cover_image_url} alt="" loading="lazy" /> : null}
                <span className="badge">{MEDIA_TYPE_LABELS[item.media_type] ?? item.media_type}</span>
                <button
                  type="button"
                  className="assign-btn"
                  onClick={() => onSelectItem({ id: item.id, title: item.title, imageUrl: item.cover_image_url })}
                >
                  このマスに反映
                </button>
              </div>
              <p className="title">{item.title}</p>
            </div>
          ))
        )}
      </div>

      <div ref={sentinelRef}>
        <LoadMoreSentinel loading={Boolean(hasNextPage) && isFetchingNextPage} text={hasNextPage ? "もっと見る" : "すべて読み込みました"} />
      </div>
    </aside>
  );
}
