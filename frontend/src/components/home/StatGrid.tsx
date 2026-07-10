import { StatCard } from "./StatCard";

export type HomeStats = {
  totalCount: number;
  inProgressCount: number;
  doneCount: number;
  favoriteCount: number;
};

type StatGridProps = {
  stats: HomeStats;
};

export function StatGrid({ stats }: StatGridProps) {
  return (
    <div className="stat-grid">
      <StatCard label="総件数" value={stats.totalCount} />
      <StatCard label="進行中" value={stats.inProgressCount} />
      <StatCard label="読了・視聴済み" value={stats.doneCount} />
      <StatCard label="❤ お気に入り" value={stats.favoriteCount} isFavorite />
    </div>
  );
}
