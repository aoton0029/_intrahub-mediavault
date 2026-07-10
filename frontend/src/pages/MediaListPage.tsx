import { useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { FiHeart } from "react-icons/fi";
import { FilterToolbar, LoadMoreSentinel, MediaGrid, useInfiniteScroll, type FilterChip, type FilterOption } from "@/components/shared";
import { useMediaListData, type MediaListFilters } from "@/hooks/useMediaListData";

const FILTER_OPTIONS: FilterOption[] = [
  { label: "All", value: "" },
  { label: "Anime", value: "anime" },
  { label: "Movie", value: "movie" },
  { label: "Drama", value: "drama" },
  { label: "Manga", value: "manga" },
  { label: "Novel", value: "novel" },
  { label: "Game", value: "game" },
];

const SORT_OPTIONS: FilterOption[] = [
  { label: "Recently added", value: "created_at" },
  { label: "Recently updated", value: "updated_at" },
  { label: "Rating", value: "rating" },
  { label: "Title", value: "title" },
  { label: "Release date", value: "release_date" },
];

function parseFilters(searchParams: URLSearchParams): MediaListFilters {
  const mediaType = searchParams.get("media_type");

  return {
    isFavorite: searchParams.get("is_favorite") === "true" ? true : undefined,
    mediaType: mediaType ? (mediaType as MediaListFilters["mediaType"]) : undefined,
    tagId: searchParams.get("tag_id") ?? undefined,
    categoryId: searchParams.get("category_id") ?? undefined,
    title: searchParams.get("title") ?? undefined,
    sort: searchParams.get("sort") ?? undefined,
    status: searchParams.get("status") ?? undefined,
  };
}

export function MediaListPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const filters = parseFilters(searchParams);
  const { mediaCards, hasNextPage, fetchNextPage, isFetchingNextPage, tags, categories } = useMediaListData(filters);

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
      label: "All",
      active: !filters.isFavorite && !filters.tagId && !filters.categoryId,
      onClick: () => updateSearchParams({ is_favorite: undefined, tag_id: undefined, category_id: undefined }),
    },
    {
      id: "favorite",
      label: "Favorites",
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
      id: "tag-add",
      label: "Tag",
      add: true,
      onClick: () => toast.info("Tag add UI will be implemented in a later task."),
    },
    {
      id: "category-add",
      label: "Category",
      add: true,
      onClick: () => toast.info("Category add UI will be implemented in a later task."),
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
        filterOptions={FILTER_OPTIONS}
        selectedFilter={filters.mediaType ?? ""}
        onFilterChange={(value) => updateSearchParams({ media_type: value || undefined })}
        sortOptions={SORT_OPTIONS}
        selectedSort={filters.sort ?? "created_at"}
        onSortChange={(value) => updateSearchParams({ sort: value })}
        searchValue={filters.title ?? ""}
        onSearchChange={(value) => updateSearchParams({ title: value || undefined })}
      />

      <MediaGrid items={mediaCards} density="compact" />

      <div ref={sentinelRef}>
        <LoadMoreSentinel loading={Boolean(hasNextPage) && isFetchingNextPage} text={hasNextPage ? "Load more results" : "All items loaded"} />
      </div>
    </>
  );
}
