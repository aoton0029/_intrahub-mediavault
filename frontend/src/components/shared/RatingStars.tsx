import { useState } from "react";
import { FiStar } from "react-icons/fi";
import { cn } from "@/lib/cn";

type RatingStarsProps = {
  value: number;
  onChange?: (value: number) => void;
  readOnly?: boolean;
};

export function RatingStars({ value, onChange, readOnly = false }: RatingStarsProps) {
  const [hoverValue, setHoverValue] = useState<number | null>(null);
  const displayValue = hoverValue ?? value;

  if (readOnly) {
    return <RatingStarsMini value={displayValue} className="rating-stars" />;
  }

  return (
    <span className="rating-stars" data-rating={value} onMouseLeave={() => setHoverValue(null)}>
      {Array.from({ length: 5 }, (_, index) => {
        const rating = index + 1;
        return (
          <button
            key={rating}
            type="button"
            className="star-btn"
            data-rating={rating}
            onMouseEnter={() => setHoverValue(rating)}
            onClick={() => onChange?.(rating)}
          >
            <FiStar className={cn("icon", rating <= displayValue && "is-full")} />
          </button>
        );
      })}
      <span className="val">{displayValue.toFixed(1)}</span>
    </span>
  );
}

export function RatingStarsMini({ value, className = "rating-stars-mini" }: { value: number; className?: string }) {
  return (
    <span className={className}>
      {Array.from({ length: 5 }, (_, index) => (
        <FiStar key={index} className={cn("icon", index < Math.floor(value) && "is-full")} />
      ))}
      <span className="val">{value.toFixed(1)}</span>
    </span>
  );
}
