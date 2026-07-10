import { cn } from "@/lib/cn";

type StatCardProps = {
  label: string;
  value: number;
  isFavorite?: boolean;
};

export function StatCard({ label, value, isFavorite = false }: StatCardProps) {
  return (
    <div className={cn("stat-card", isFavorite && "is-favorite")}>
      <div className="stat-value">{value}</div>
      <div className="stat-label">{label}</div>
    </div>
  );
}
