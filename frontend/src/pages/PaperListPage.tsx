import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { FiHeart, FiMoreHorizontal } from "react-icons/fi";
import {
  FilterToolbar,
  LiteratureList,
  LoadMoreSentinel,
  MediaContextMenu,
  MediaFilterPanel,
  QuickViewSheet,
  useInfiniteScroll,
  type ContextMenuStatus,
  type FilterChip,
  type FilterOption,
  type LiteratureRowProps,
  type MediaContextMenuTarget,
} from "@/components/shared";
import { usePageChrome } from "@/components/layout/usePageChrome";
import { useMediaItemActions } from "@/hooks/useQuickViewData";
import { useMediaListData, type MediaListFilters } from "@/hooks/useMediaListData";

const SORT_OPTIONS: FilterOption[] = [
  { label: "追加日順", value: "created_at" },
  { label: "更新日順", value: "updated_at" },
  { label: "タイトル順", value: "title" },
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
  not_started: "未読",
  in_progress: "読書中",
  completed: "読了",
};

export function PaperListPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const filters = parseFilters(searchParams);
  const { items, hasNextPage, fetchNextPage, isFetchingNextPage, tags, categories } = useMediaListData(filters, {
    mediaTypeOverride: "paper",
    getBadgeLabel: () => "論文",
    getItemHref: (item) => `/research/papers/${item.id}`,
  });
  const itemActions = useMediaItemActions();

  const [filterPanelOpen, setFilterPanelOpen] = useState(false);
  const [quickViewId, setQuickViewId] = useState<string | null>(null);
  const [menuTarget, setMenuTarget] = useState<MediaContextMenuTarget | null>(null);

  usePageChrome({
    actions: (
      <button type="button" className="btn btn-accent" onClick={() => toast.info("論文の追加機能は今後対応予定です。")}>
        ＋ 論文を追加
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
    if (!window.confirm("この論文を削除しますか？この操作は取り消せません。")) return;
    try {
      await itemActions.deleteItem(id);
      toast.success("論文を削除しました。");
      if (quickViewId === id) setQuickViewId(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "削除に失敗しました。");
    }
  }

  const rows: LiteratureRowProps[] = items.map((item) => {
    const status = toContextMenuStatus(item.status);

    return {
      id: item.id,
      title: item.original_title || item.title,
      authors: item.authors || undefined,
      year: item.publication_year ?? undefined,
      journal: item.journal || undefined,
      doi: item.doi ?? undefined,
      rating: item.rating ?? undefined,
      tags: item.tags.length > 0 ? (
        <div className="prop-taglist">
          {item.tags.map((tag) => (
            <span key={tag.id} className="tag-pill">
              {tag.name}
            </span>
          ))}
        </div>
      ) : undefined,
      aside: (
        <>
          <span className={`table-row-status ${status}`}>{STATUS_LABELS[status]}</span>
          <button
            type="button"
            className="table-row-menu-btn"
            title="メニュー"
            aria-label="メニュー"
            onClick={(event) => {
              event.preventDefault();
              event.stopPropagation();
              handleOpenMenu(item.id, item.original_title || item.title, event.currentTarget);
            }}
          >
            <FiMoreHorizontal className="icon" />
          </button>
        </>
      ),
    };
  });

  return (
    <>
      <FilterToolbar
        chips={chips}
        sortOptions={SORT_OPTIONS}
        selectedSort={filters.sort ?? "created_at"}
        onSortChange={(value) => updateSearchParams({ sort: value })}
        searchValue={filters.title ?? ""}
        searchPlaceholder="タイトル・著者・DOIで検索…"
        onSearchChange={(value) => updateSearchParams({ title: value || undefined })}
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

      <LiteratureList items={rows} onRowClick={(id) => setQuickViewId(id)} />

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
    </>
  );
}
