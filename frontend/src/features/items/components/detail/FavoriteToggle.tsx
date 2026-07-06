import { FiHeart } from 'react-icons/fi';
import { useUpdateItemMutation } from '../../api-detail';

export function FavoriteToggle({ itemId, isFavorite }: { itemId: string; isFavorite: boolean }) {
  const updateItem = useUpdateItemMutation(itemId);

  return (
    <button
      type="button"
      onClick={() => updateItem.mutate({ is_favorite: !isFavorite })}
      aria-pressed={isFavorite}
      className={`flex items-center gap-1.5 text-[12.5px] ${
        isFavorite ? 'text-favorite' : 'text-text-muted'
      }`}
    >
      <FiHeart className={`h-4 w-4 ${isFavorite ? 'fill-favorite text-favorite' : ''}`} />
      お気に入り
    </button>
  );
}
