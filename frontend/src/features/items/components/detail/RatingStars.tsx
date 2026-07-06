import { FiStar } from 'react-icons/fi';
import { useUpdateItemMutation } from '../../api-detail';

export function RatingStars({ itemId, rating }: { itemId: string; rating: number | null }) {
  const updateItem = useUpdateItemMutation(itemId);
  const rounded = rating ? Math.round(rating) : 0;

  return (
    <span className="inline-flex items-center gap-0.5">
      {[1, 2, 3, 4, 5].map((value) => (
        <button
          key={value}
          type="button"
          onClick={() => updateItem.mutate({ rating: value })}
          aria-label={`評価を${value}にする`}
          className="flex items-center"
        >
          <FiStar
            className={`h-3.5 w-3.5 ${
              value <= rounded ? 'fill-favorite text-favorite' : 'fill-border text-border'
            }`}
          />
        </button>
      ))}
      <span className="ml-1 font-mono text-text-muted">{rating ?? '-'}</span>
    </span>
  );
}
