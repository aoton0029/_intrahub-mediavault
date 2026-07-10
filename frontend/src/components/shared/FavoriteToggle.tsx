import { FiHeart } from "react-icons/fi";
import { cn } from "@/lib/cn";

type FavoriteToggleProps = {
  value: boolean;
  onChange: (value: boolean) => void;
  label?: string;
};

export function FavoriteToggle({ value, onChange, label = "お気に入り" }: FavoriteToggleProps) {
  return (
    <button
      type="button"
      className={cn("meta-item favorite-toggle", value && "is-active")}
      data-favorite-toggle
      aria-pressed={value}
      onClick={() => onChange(!value)}
    >
      <FiHeart className="icon" />
      {label}
    </button>
  );
}
