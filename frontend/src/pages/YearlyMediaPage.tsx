import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  DEFAULT_MEDIA_SORT,
  DisplaySettingsDropdown,
  EmptyState,
  FilterToolbar,
  LoadMoreSentinel,
  MEDIA_SORT_OPTIONS,
  MediaGrid,
  MediaTypeDropdown,
  useInfiniteScroll,
} from "@/components/shared";
import {
  useYearItems,
  useYearlyMediaData,
  type YearDateField,
  type YearlyMediaFilters,
} from "@/hooks/useYearlyMediaData";

const INITIAL_VISIBLE_YEARS = 3;
const VISIBLE_YEARS_STEP = 3;

const DATE_FIELD_OPTIONS: { label: string; value: YearDateField }[] = [
  { label: "リリース年", value: "release" },
  { label: "視聴・読了年", value: "consumed" },
];

function parseFilters(searchParams: URLSearchParams): YearlyMediaFilters {
  const mediaType = searchParams.get("media_type");

  return {
    dateField: searchParams.get("date_field") === "consumed" ? "consumed" : "release",
    mediaType: mediaType ? (mediaType as YearlyMediaFilters["mediaType"]) : undefined,
    sort: searchParams.get("sort") ?? DEFAULT_MEDIA_SORT,
  };
}

type YearSectionProps = {
  filters: YearlyMediaFilters;
  year: number;
  count: number;
  thumbnailOrientation: "vertical" | "horizontal";
  columns?: number;
};

function YearSection({ filters, year, count, thumbnailOrientation, columns }: YearSectionProps) {
  const { mediaCards, hasNextPage, fetchNextPage, isFetchingNextPage } = useYearItems(filters, year);

  return (
    <section aria-label={`${year}年`} style={{ marginBottom: "2rem" }}>
      <h2 style={{ margin: "0 0 0.75rem" }}>
        {year}年 <span style={{ fontWeight: "normal", opacity: 0.7 }}>({count})</span>
      </h2>
      <MediaGrid items={mediaCards} density="compact" thumbnailOrientation={thumbnailOrientation} columns={columns} />
      {hasNextPage ? (
        <button
          type="button"
          className="btn"
          disabled={isFetchingNextPage}
          onClick={() => void fetchNextPage()}
          style={{ marginTop: "0.5rem" }}
        >
          {isFetchingNextPage ? "読み込み中..." : "もっと見る"}
        </button>
      ) : null}
    </section>
  );
}

export function YearlyMediaPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const filters = parseFilters(searchParams);
  const { years, isLoading, isError } = useYearlyMediaData(filters);
  const [visibleYearCount, setVisibleYearCount] = useState(INITIAL_VISIBLE_YEARS);
  const thumbnailOrientation = searchParams.get("thumbnail") === "vertical" ? "vertical" : "horizontal";
  const colsParam = Number(searchParams.get("cols"));
  const columns = Number.isInteger(colsParam) && colsParam >= 2 && colsParam <= 16 ? colsParam : undefined;

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

  const visibleYears = years.slice(0, visibleYearCount);
  const hasMoreYears = years.length > visibleYearCount;

  const sentinelRef = useInfiniteScroll(() => {
    if (hasMoreYears) {
      setVisibleYearCount((current) => current + VISIBLE_YEARS_STEP);
    }
  }, hasMoreYears);

  return (
    <>
      <FilterToolbar
        chips={DATE_FIELD_OPTIONS.map((option) => ({
          id: option.value,
          label: option.label,
          active: filters.dateField === option.value,
          onClick: () =>
            updateSearchParams({ date_field: option.value === "release" ? undefined : option.value }),
        }))}
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
        trailing={
          <DisplaySettingsDropdown
            thumbnailOrientation={thumbnailOrientation}
            onThumbnailOrientationChange={(value) => updateSearchParams({ thumbnail: value === "vertical" ? "vertical" : undefined })}
            columns={columns}
            onColumnsChange={(value) => updateSearchParams({ cols: value ? String(value) : undefined })}
          />
        }
      />

      {isError ? (
        <EmptyState title="読み込みエラー" description="年別データの取得に失敗しました。時間をおいて再度お試しください。" />
      ) : !isLoading && years.length === 0 ? (
        <EmptyState
          title="表示できる作品がありません"
          description={
            filters.dateField === "consumed"
              ? "視聴日・読了日が登録された作品がありません。"
              : "リリース日が登録された作品がありません。"
          }
        />
      ) : (
        visibleYears.map((entry) => (
          <YearSection
            key={entry.year}
            filters={filters}
            year={entry.year}
            count={entry.count}
            thumbnailOrientation={thumbnailOrientation}
            columns={columns}
          />
        ))
      )}

      {hasMoreYears ? (
        <div ref={sentinelRef}>
          <LoadMoreSentinel loading text="さらに過去の年を読み込み中..." />
        </div>
      ) : null}
    </>
  );
}
