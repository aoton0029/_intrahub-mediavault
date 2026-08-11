import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { FiHeart } from "react-icons/fi";
import {
  AddWorkModal,
  DisplaySettingsDropdown,
  FilterToolbar,
  LoadMoreSentinel,
  MediaContextMenu,
  MediaFilterPanel,
  MediaGrid,
  MediaTable,
  QuickViewSheet,
  ViewModeToggle,
  useInfiniteScroll,
  type ContextMenuStatus,
  type FilterChip,
  type FilterOption,
  type MediaContextMenuTarget,
  type MediaTableRow,
  type ViewMode,
} from "@/components/shared";
import { usePageChrome } from "@/components/layout/usePageChrome";
import { useDisplaySettings } from "@/hooks/useDisplaySettings";
import { useMediaItemActions } from "@/hooks/useQuickViewData";
import { useMediaListData, type MediaListFilters } from "@/hooks/useMediaListData";

const SORT_OPTIONS: FilterOption[] = [
  { label: "追加日順", value: "created_at" },
  { label: "更新日順", value: "updated_at" },
  { label: "タイトル順", value: "title" },
  { label: "発売日順", value: "release_date" },
];

function parseFilters(searchParams: URLSearchParams): MediaListFilters {
  return {
    isFavorite: searchParams.get("is_favorite") === "true" ? true : undefined,
    tagId: searchParams.get("tag_id") ?? undefined,
    categoryId: searchParams.get("category_id") ?? undefined,
    title: searchParams.get("title") ?? undefined,
    sort: searchParams.get("sort") ?? undefined,
  };
}

function anchorToMenuPosition(anchor: HTMLElement) {
  const rect = anchor.getBoundingClientRect();
  return { top: rect.bottom + 6, right: window.innerWidth - rect.right };
}

function toContextMenuStatus(status: string): ContextMenuStatus {
  if (status === "in_progress") return "in_progress";
  if (status === "completed" || status === "done") return "completed";
  return "not_started";
}

const STATUS_LABELS: Record<ContextMenuStatus, string> = {
  not_started: "未着手",
  in_progress: "視聴中",
  completed: "視聴済",
};

export function AcademicBookListPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const filters = parseFilters(searchParams);
  const { items, mediaCards, hasNextPage, fetchNextPage, isFetchingNextPage, tags, categories } = useMediaListData(filters, {
    mediaTypeOverride: "academic_book",
    getBadgeLabel: () => "学術書",
    getItemHref: (item) => `/academic-books/${item.id}`,
  });
  const { thumbnailOrientation, columns, showTitle, showRating, setThumbnailOrientation, setColumns, setShowTitle, setShowRating } = useDisplaySettings();
  const itemActions = useMediaItemActions();

  const [viewMode, setViewMode] = useState<ViewMode>("grid");
  const [filterPanelOpen, setFilterPanelOpen] = useState(false);
  const [quickViewId, setQuickViewId] = useState<string | null>(null);
  const [menuTarget, setMenuTarget] = useState<MediaContextMenuTarget | null>(null);
  const [isAddModalOpen, setAddModalOpen] = useState(false);

  usePageChrome({
    actions: (
      <button type="button" className="btn btn-accent" onClick={() => setAddModalOpen(true)}>
        ＋ 作品を追加
      </button>
    ),
  });

  const activeTag = tags.find((tag) => tag.id === filters.tagId);
  const activeCategory = categories.find((category) => category.id === filters.categoryId);
  const activeFilterCount = (filters.tagId ? 1 : 0) + (filters.categoryId ? 1 : 0);

  const updateSearchParams = (updates: Record<string, string | undefined>) => {
    const next = new URLSearchParams(searchParams);

    for (const [key, value] of Object.entries(updates)) {
      if (!value) {
        next.delete(key);
      } else {
        next.set(key, value);
      }
    }

    setSearchParams(next);
  };

  const chips: FilterChip[] = [
    {
      id: "favorite",
      label: "お気に入り",
      icon: <FiHeart className="icon" />,
      active: Boolean(filters.isFavorite),
      onClick: () => updateSearchParams({ is_favorite: filters.isFavorite ? undefined : "true" }),
    },
    ...(activeTag
      ? [{
          id: "tag",
          label: `# ${activeTag.name}`,
          active: true,
          removable: true,
          onRemove: () => updateSearchParams({ tag_id: undefined }),
        } satisfies FilterChip]
      : []),
    ...(activeCategory
      ? [{
          id: "category",
          label: `Category: ${activeCategory.name}`,
          active: true,
          removable: true,
          onRemove: () => updateSearchParams({ category_id: undefined }),
        } satisfies FilterChip]
      : []),
    {
      id: "filter-panel-toggle",
      label: "フィルター",
      active: filterPanelOpen || activeFilterCount > 0,
      count: activeFilterCount,
      onClick: () => setFilterPanelOpen((current) => !current),
    },
  ];

  const sentinelRef = useInfiniteScroll(() => {
    if (hasNextPage && !isFetchingNextPage) {
      void fetchNextPage();
    }
  }, Boolean(hasNextPage) && !isFetchingNextPage);

  function handleAddToCollection() {
    toast.info("コレクション追加UIは今後の対応予定です。");
  }

  function handleOpenMenu(id: string, title: string, anchor: HTMLElement) {
    const source = items.find((item) => item.id === id);
    setMenuTarget({
      id,
      title,
      status: toContextMenuStatus(source?.status ?? "not_started"),
      favorite: source?.is_favorite ?? false,
      position: anchorToMenuPosition(anchor),
    });
  }

  async function handleSetStatus(id: string, status: ContextMenuStatus) {
    try {
      await itemActions.updateStatus(id, status);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "ステータスの更新に失敗しました。");
    }
  }

  async function handleToggleFavorite(id: string, next: boolean) {
    try {
      await itemActions.updateFavorite(id, next);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "お気に入りの更新に失敗しました。");
    }
  }

  async function handleDelete(id: string) {
    if (!window.confirm("この作品を削除しますか？この操作は取り消せません。")) return;
    try {
      await itemActions.deleteItem(id);
      toast.success("作品を削除しました。");
      if (quickViewId === id) setQuickViewId(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "削除に失敗しました。");
    }
  }

  const tableRows: MediaTableRow[] = items.map((item) => ({
    id: item.id,
    title: item.original_title || item.title,
    badge: "学術書",
    imageUrl: item.cover_image_url,
    status: toContextMenuStatus(item.status),
    statusLabel: STATUS_LABELS[toContextMenuStatus(item.status)],
    rating: item.rating ?? undefined,
    updatedLabel: item.updated_at ? item.updated_at.slice(0, 10) : "更新日未登録",
  }));

  return (
    <>
      <FilterToolbar
        chips={chips}
        sortOptions={SORT_OPTIONS}
        selectedSort={filters.sort ?? "created_at"}
        onSortChange={(value) => updateSearchParams({ sort: value })}
        searchValue={filters.title ?? ""}
        searchPlaceholder="タイトルで検索…"
        onSearchChange={(value) => updateSearchParams({ title: value || undefined })}
        trailing={
          <>
            <ViewModeToggle value={viewMode} onChange={setViewMode} />
            <DisplaySettingsDropdown
              thumbnailOrientation={thumbnailOrientation}
              onThumbnailOrientationChange={setThumbnailOrientation}
              columns={columns}
              onColumnsChange={setColumns}
              showTitle={showTitle}
              onShowTitleChange={setShowTitle}
              showRating={showRating}
              onShowRatingChange={setShowRating}
            />
          </>
        }
      />

      <MediaFilterPanel
        hidden={!filterPanelOpen}
        tags={tags.map((tag) => ({ id: tag.id, label: tag.name }))}
        categories={categories.map((category) => ({ id: category.id, label: category.name }))}
        activeTagId={filters.tagId}
        activeCategoryId={filters.categoryId}
        onSelectTag={(id) => updateSearchParams({ tag_id: id })}
        onSelectCategory={(id) => updateSearchParams({ category_id: id })}
      />

      {viewMode === "grid" ? (
        <MediaGrid
          items={mediaCards}
          density="compact"
          thumbnailOrientation={thumbnailOrientation}
          columns={columns}
          showTitle={showTitle}
          showRating={showRating}
          onQuickView={(item) => item.id && setQuickViewId(item.id)}
          onAddToCollection={handleAddToCollection}
          onOpenMenu={(item, anchor) => item.id && handleOpenMenu(item.id, item.title, anchor)}
        />
      ) : (
        <MediaTable
          rows={tableRows}
          onRowClick={(id) => setQuickViewId(id)}
          onOpenMenu={(id, anchor) => {
            const row = tableRows.find((entry) => entry.id === id);
            if (row) handleOpenMenu(id, row.title, anchor);
          }}
        />
      )}

      {hasNextPage ? (
        <div ref={sentinelRef}>
          <LoadMoreSentinel loading={isFetchingNextPage} text="読み込み中…" />
        </div>
      ) : null}

      <MediaContextMenu
        target={menuTarget}
        onClose={() => setMenuTarget(null)}
        onQuickView={(id) => setQuickViewId(id)}
        onToggleFavorite={(id, next) => void handleToggleFavorite(id, next)}
        onSetStatus={(id, status) => void handleSetStatus(id, status)}
        onAddToMylist={() => toast.info("マイリスト追加UIは今後の対応予定です。")}
        onDelete={(id) => void handleDelete(id)}
      />

      <QuickViewSheet itemId={quickViewId} onClose={() => setQuickViewId(null)} onDeleted={() => setQuickViewId(null)} />

      <AddWorkModal open={isAddModalOpen} onClose={() => setAddModalOpen(false)} scope="academic_book" />
    </>
  );
}
