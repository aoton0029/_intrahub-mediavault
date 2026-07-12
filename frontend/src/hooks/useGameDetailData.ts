import type { PropertyItem } from "@/components/shared";
import { useItemDetailData } from "./useItemDetailData";

type GameDetail = {
  platforms: string[];
  developers: string[];
  publishers: string[];
  screenshots: string[];
  metacritic: number | null;
  genres: string[];
};

export function useGameDetailData(id: string | undefined) {
  return useItemDetailData<GameDetail>("game", id, {
    mapPropertyItems: (detail) => [
      { key: "platforms", label: "プラットフォーム", value: detail?.platforms?.length ? detail.platforms.join(", ") : "未登録", muted: !detail?.platforms?.length },
      { key: "developers", label: "開発元", value: detail?.developers?.length ? detail.developers.join(", ") : "未登録", muted: !detail?.developers?.length },
      { key: "publishers", label: "販売元", value: detail?.publishers?.length ? detail.publishers.join(", ") : "未登録", muted: !detail?.publishers?.length },
      { key: "metacritic", label: "Metacritic", value: detail?.metacritic ? `${detail.metacritic}点` : "未登録", muted: !detail?.metacritic },
      { key: "genres", label: "ジャンル", value: detail?.genres?.length ? detail.genres.join("・") : "未登録", muted: !detail?.genres?.length },
    ] satisfies PropertyItem[],
  });
}

export type UseGameDetailDataResult = ReturnType<typeof useGameDetailData>;
