import { useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { FiHeart } from "react-icons/fi";
import { DEFAULT_MEDIA_SORT, DisplaySettingsDropdown, FilterToolbar, LoadMoreSentinel, MEDIA_SORT_OPTIONS, MediaGrid, MediaTypeDropdown, useInfiniteScroll, type FilterChip } from "@/components/shared";
import { useDisplaySettings } from "@/hooks/useDisplaySettings";
import { useMediaListData, type MediaListFilters } from "@/hooks/useMediaListData";

function parseFilters(searchParams: URLSearchParams): MediaListFilters {
  const mediaType = searchParams.get("media_type");

  return {
    isFavorite: searchParams.get("is_favorite") === "true" ? true : undefined,
    mediaType: mediaType ? (mediaType as MediaListFilters["mediaType"]) : undefined,
    tagId: searchParams.get("tag_id") ?? undefined,
    categoryId: searchParams.get("category_id") ?? undefined,
    title: searchParams.get("title") ?? undefined,
    sort: searchParams.get("sort") ?? DEFAULT_MEDIA_SORT,
    status: searchParams.get("status") ?? undefined,
  };
}

export function MediaListPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const filters = parseFilters(searchParams);
  const { mediaCards, hasNextPage, fetchNextPage, isFetchingNextPage, tags, categories } = useMediaListData(filters);
  const { thumbnailOrientation, columns, setThumbnailOrientation, setColumns } = useDisplaySettings();

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
      id: "favorite",
      label: "Favorites",
      icon: <FiHeart className="icon" />,
      iconOnly: true,
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
        filterSlot={
          <MediaTypeDropdown
            includeAll
            value={filters.mediaType && filters.mediaType !== "academic_book" ? filters.mediaType : "all"}
            onChange={(value) => updateSearchParams({ media_type: value === "all" ? undefined : value })}
          />
        }
        sortOptions={MEDIA_SORT_OPTIONS}
        selectedSort={filters.sort ?? DEFAULT_MEDIA_SORT}
        onSortChange={(value) => updateSearchParams({ sort: value })}
        searchValue={filters.title ?? ""}
        onSearchChange={(value) => updateSearchParams({ title: value || undefined })}
        trailing={
          <DisplaySettingsDropdown
            thumbnailOrientation={thumbnailOrientation}
            onThumbnailOrientationChange={setThumbnailOrientation}
            columns={columns}
            onColumnsChange={setColumns}
          />
        }
      />

      <MediaGrid items={mediaCards} density="compact" thumbnailOrientation={thumbnailOrientation} columns={columns} />

      <div ref={sentinelRef}>
        <LoadMoreSentinel loading={Boolean(hasNextPage) && isFetchingNextPage} text={hasNextPage ? "Load more results" : "All items loaded"} />
      </div>
    </>
  );
}
