import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { FiBook, FiCalendar, FiCheck, FiChevronDown, FiFileText } from "react-icons/fi";
import {
  DEFAULT_MEDIA_SORT,
  DisplaySettingsDropdown,
  EmptyState,
  FilterToolbar,
  MEDIA_SORT_OPTIONS,
  MediaGrid,
} from "@/components/shared";
import { mediaTypeConfig } from "@/config/mediaTypes";
import { useDisplaySettings } from "@/hooks/useDisplaySettings";
import { useOutsideClick } from "@/hooks/useOutsideClick";
import type { MediaType } from "@/hooks/useMediaListData";
import {
  useYearItems,
  useYearlyMediaData,
  type YearCount,
  type YearDateField,
  type YearlyMediaFilters,
} from "@/hooks/useYearlyMediaData";
import { cn } from "@/lib/cn";

const DATE_FIELD_OPTIONS: { label: string; value: "release" | "consumed" }[] = [
  { label: "リリース年", value: "release" },
  { label: "視聴・読了年", value: "consumed" },
];

function parseDateField(searchParams: URLSearchParams): YearDateField {
  const raw = searchParams.get("date_field");
  return raw === "release" || raw === "consumed" ? raw : "any";
}

function mediaTypeLabel(mediaType: MediaType) {
  if (mediaType === "academic_book") {
    return "学術書";
  }
  if (mediaType === "paper") {
    return "論文";
  }
  return mediaTypeConfig[mediaType].label;
}

function MediaTypeIcon({ mediaType, className }: { mediaType: MediaType; className?: string }) {
  if (mediaType === "academic_book") {
    return <FiBook className={className} />;
  }
  if (mediaType === "paper") {
    return <FiFileText className={className} />;
  }
  const Icon = mediaTypeConfig[mediaType].icon;
  return <Icon className={className} />;
}

type YearDropdownProps = {
  years: YearCount[];
  value: number | undefined;
  onChange: (year: number) => void;
};

function YearDropdown({ years, value, onChange }: YearDropdownProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useOutsideClick<HTMLDivElement>(() => setOpen(false), open);

  return (
    <div className="media-type-dropdown" ref={rootRef}>
      <button
        type="button"
        className="chip media-type-dropdown-trigger"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label="年"
        onClick={() => setOpen((current) => !current)}
      >
        <FiCalendar className="icon" />
        {value != null ? `${value}年` : "年"}
        <FiChevronDown className="icon" />
      </button>
      {open ? (
        <div className="media-type-dropdown-popover" role="listbox" aria-label="年">
          {years.map((entry) => (
            <button
              key={entry.year}
              type="button"
              role="option"
              aria-selected={value === entry.year}
              className={cn("media-type-option", value === entry.year && "active")}
              onClick={() => {
                onChange(entry.year);
                setOpen(false);
              }}
            >
              <span className="label">
                {entry.year}年 ({entry.count})
              </span>
              {value === entry.year ? <FiCheck className="icon check" /> : null}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

type MediaTypeSectionProps = {
  filters: YearlyMediaFilters;
  year: number;
  mediaType: MediaType;
  count: number;
  thumbnailOrientation: "vertical" | "horizontal";
  columns?: number;
  showTitle: boolean;
  showRating: boolean;
};

function MediaTypeSection({ filters, year, mediaType, count, thumbnailOrientation, columns, showTitle, showRating }: MediaTypeSectionProps) {
  const { mediaCards, hasNextPage, fetchNextPage, isFetchingNextPage } = useYearItems(
    { ...filters, mediaType },
    year,
  );
  const label = mediaTypeLabel(mediaType);

  return (
    <section aria-label={label} style={{ marginBottom: "2rem" }}>
      <h2 style={{ margin: "0 0 0.75rem", display: "flex", alignItems: "center", gap: "0.5rem" }}>
        <MediaTypeIcon mediaType={mediaType} />
        {label} <span style={{ fontWeight: "normal", opacity: 0.7 }}>({count})</span>
      </h2>
      <MediaGrid
        items={mediaCards}
        density="compact"
        thumbnailOrientation={thumbnailOrientation}
        columns={columns}
        showTitle={showTitle}
        showRating={showRating}
      />
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

function parseYearParam(searchParams: URLSearchParams) {
  const raw = searchParams.get("year");
  if (!raw) {
    return undefined;
  }
  const parsed = Number.parseInt(raw, 10);
  return Number.isNaN(parsed) ? undefined : parsed;
}

export function YearlyMediaPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const dateField = parseDateField(searchParams);
  const sort = searchParams.get("sort") ?? DEFAULT_MEDIA_SORT;
  const { years, isLoading, isError } = useYearlyMediaData({ dateField });
  const { thumbnailOrientation, columns, showTitle, showRating, setThumbnailOrientation, setColumns, setShowTitle, setShowRating } = useDisplaySettings();

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

  const selectedYear = parseYearParam(searchParams) ?? years[0]?.year;
  const selectedEntry = years.find((entry) => entry.year === selectedYear);
  const itemFilters: YearlyMediaFilters = { dateField, sort };

  return (
    <>
      <FilterToolbar
        chips={DATE_FIELD_OPTIONS.map((option) => {
          const active = dateField === "any" || dateField === option.value;
          const other = option.value === "release" ? "consumed" : "release";
          return {
            id: option.value,
            label: option.label,
            active,
            onClick: () => {
              // 最低1つは選択された状態を維持する（単独選択中のチップはOFFにできない）
              if (active && dateField !== "any") {
                return;
              }
              // OFFにする → もう一方のみ、ONにする → 両方（= any = パラメータ削除）
              updateSearchParams({ date_field: active ? other : undefined });
            },
          };
        })}
        filterSlot={
          <YearDropdown
            years={years}
            value={selectedYear}
            onChange={(year) => updateSearchParams({ year: String(year) })}
          />
        }
        sortOptions={MEDIA_SORT_OPTIONS}
        selectedSort={sort}
        onSortChange={(value) => updateSearchParams({ sort: value })}
        trailing={
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
        }
      />

      {isError ? (
        <EmptyState title="読み込みエラー" description="年別データの取得に失敗しました。時間をおいて再度お試しください。" />
      ) : !isLoading && (years.length === 0 || !selectedEntry) ? (
        <EmptyState
          title="表示できる作品がありません"
          description={
            dateField === "consumed"
              ? "視聴日・読了日が登録された作品がありません。"
              : dateField === "release"
                ? "リリース日が登録された作品がありません。"
                : "リリース日・視聴日・読了日が登録された作品がありません。"
          }
        />
      ) : (
        selectedEntry?.media_types
          .filter((entry) => entry.count > 0)
          .map((entry) => (
            <MediaTypeSection
              key={entry.media_type}
              filters={itemFilters}
              year={selectedEntry.year}
              mediaType={entry.media_type}
              count={entry.count}
              thumbnailOrientation={thumbnailOrientation}
              columns={columns}
              showTitle={showTitle}
              showRating={showRating}
            />
          ))
      )}
    </>
  );
}
