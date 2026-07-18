import type { PropertyItem } from "@/components/shared";
import { useItemDetailData } from "./useItemDetailData";

type BookDetail = {
  authors: string | null;
  publisher: string | null;
  isbn: string | null;
  series_name?: string | null;
};

function formatValue(value: string | number | null | undefined, fallback = "未登録") {
  if (value === null || value === undefined || value === "") {
    return fallback;
  }
  return String(value);
}

function mapPropertyItems(detail: BookDetail | null): PropertyItem[] {
  return [
    { key: "authors", label: "著者", value: formatValue(detail?.authors), muted: !detail?.authors },
    { key: "publisher", label: "出版社", value: formatValue(detail?.publisher), muted: !detail?.publisher },
    { key: "isbn", label: "ISBN", value: formatValue(detail?.isbn), muted: !detail?.isbn },
    { key: "series_name", label: "シリーズ名", value: formatValue(detail?.series_name), muted: !detail?.series_name },
  ];
}

export function useAcademicBookDetailData(id: string | undefined) {
  return useItemDetailData<BookDetail>("academic_book", id, {
    includeGroups: false,
    mapPropertyItems,
  });
}

export type UseAcademicBookDetailDataResult = ReturnType<typeof useAcademicBookDetailData>;
