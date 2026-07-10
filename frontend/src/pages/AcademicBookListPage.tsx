import { useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { FiHeart, FiTag } from "react-icons/fi";
import { FilterToolbar, LoadMoreSentinel, MediaGrid, useInfiniteScroll, type FilterChip, type FilterOption } from "@/components/shared";
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

export function AcademicBookListPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const filters = parseFilters(searchParams);
  const { mediaCards, hasNextPage, fetchNextPage, isFetchingNextPage, tags, categories } = useMediaListData(filters, {
    mediaTypeOverride: "academic_book",
    getBadgeLabel: () => "学術書",
    getItemHref: (item) => `/academic-books/${item.id}`,
  });

  const activeTag = tags.find((tag) => tag.id === filters.tagId);
  const activeCategory = categories.find((category) => category.id === filters.categoryId);

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
      id: "all",
      label: "すべて",
      active: !filters.isFavorite && !filters.tagId && !filters.categoryId,
      onClick: () => updateSearchParams({ is_favorite: undefined, tag_id: undefined, category_id: undefined }),
    },
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
          label: activeTag.name,
          icon: <FiTag className="icon" />,
          active: true,
          removable: true,
          onRemove: () => updateSearchParams({ tag_id: undefined }),
        } satisfies FilterChip]
      : []),
    ...(activeCategory
      ? [{
          id: "category",
          label: activeCategory.name,
          active: true,
          removable: true,
          onRemove: () => updateSearchParams({ category_id: undefined }),
        } satisfies FilterChip]
      : []),
    {
      id: "tag-add",
      label: "タグ",
      add: true,
      onClick: () => toast.info("タグ追加UIは後続タスクで実装します。"),
    },
    {
      id: "category-add",
      label: "カテゴリ",
      add: true,
      onClick: () => toast.info("カテゴリ追加UIは後続タスクで実装します。"),
    },
  ];

  const sentinelRef = useInfiniteScroll(() => {
    if (hasNextPage && !isFetchingNextPage) {
      void fetchNextPage();
    }
  }, Boolean(hasNextPage) && !isFetchingNextPage);

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
      />

      <MediaGrid items={mediaCards} density="compact" />

      <div ref={sentinelRef}>
        <LoadMoreSentinel loading={Boolean(hasNextPage) && isFetchingNextPage} text={hasNextPage ? "読み込み中…" : "すべて読み込みました"} />
      </div>
    </>
  );
}
