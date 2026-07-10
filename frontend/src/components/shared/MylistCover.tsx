import { cn } from "@/lib/cn";

export function MylistCover({ count, covers }: { count: 1 | 2 | 3 | 4; covers: string[] }) {
  return (
    <div className={cn("mylist-cover", `n${count}`)}>
      {covers.slice(0, count).map((cover, index) => (
        <div key={`${cover}-${index}`} style={cover ? { backgroundImage: `url(${cover})` } : undefined} />
      ))}
    </div>
  );
}
