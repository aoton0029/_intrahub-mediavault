import { SectionHeading } from "@/components/home/SectionHeading";
import { StatGrid } from "@/components/home/StatGrid";
import { MediaGrid } from "@/components/shared";
import { useHomeData } from "@/hooks/useHomeData";

const EMPTY_STATS = {
  totalCount: 0,
  inProgressCount: 0,
  doneCount: 0,
  favoriteCount: 0,
};

export function HomePage() {
  const { data } = useHomeData();

  return (
    <>
      <StatGrid stats={data?.stats ?? EMPTY_STATS} />

      <SectionHeading title="最近追加した作品" seeAllHref="/media" />
      <MediaGrid items={data?.recentItems ?? []} density="default" />

      <SectionHeading title="進行中" seeAllHref="/media?status=in_progress" />
      <MediaGrid items={data?.inProgressItems ?? []} density="default" />
    </>
  );
}
