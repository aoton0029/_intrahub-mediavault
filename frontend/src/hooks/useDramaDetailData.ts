import type { PropertyItem } from "@/components/shared";
import { useItemDetailData } from "./useItemDetailData";

type DramaDetail = {
  number_of_seasons: number | null;
  number_of_episodes: number | null;
  networks: string[];
  status: string | null;
  original_language: string | null;
  first_air_date: string | null;
  last_air_date: string | null;
  genres: string[];
  rating: number | null;
};

function formatValue(value: string | number | null | undefined, fallback = "未登録") {
  if (value === null || value === undefined || value === "") {
    return fallback;
  }
  return String(value);
}

function mapPropertyItems(detail: DramaDetail | null): PropertyItem[] {
  return [
    { key: "number_of_seasons", label: "シーズン数", value: detail?.number_of_seasons ? `${detail.number_of_seasons}期` : "未登録", muted: !detail?.number_of_seasons },
    { key: "number_of_episodes", label: "話数", value: detail?.number_of_episodes ? `${detail.number_of_episodes}話` : "未登録", muted: !detail?.number_of_episodes },
    { key: "networks", label: "放送局", value: detail?.networks?.length ? detail.networks.join(", ") : "未登録", muted: !detail?.networks?.length },
    { key: "status", label: "放送状況", value: formatValue(detail?.status), muted: !detail?.status },
    { key: "original_language", label: "原語", value: formatValue(detail?.original_language), muted: !detail?.original_language },
    { key: "first_air_date", label: "放送開始日", value: formatValue(detail?.first_air_date), muted: !detail?.first_air_date },
    { key: "last_air_date", label: "放送終了日", value: formatValue(detail?.last_air_date), muted: !detail?.last_air_date },
    { key: "genres", label: "ジャンル", value: detail?.genres?.length ? detail.genres.join("・") : "未登録", muted: !detail?.genres?.length },
  ];
}

export function useDramaDetailData(id: string | undefined) {
  return useItemDetailData<DramaDetail>("drama", id, {
    includeGroups: true,
    groupType: "season",
    mapPropertyItems,
  });
}

export type UseDramaDetailDataResult = ReturnType<typeof useDramaDetailData>;
