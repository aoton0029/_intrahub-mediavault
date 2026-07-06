import { FiCalendar, FiLink, FiPlayCircle } from 'react-icons/fi';
import { statusLabels } from '../../types';
import type { ItemDetail } from '../../types-detail';
import { RatingStars } from './RatingStars';
import { FavoriteToggle } from './FavoriteToggle';
import { TagCategorySection } from './TagCategorySection';
import { MylistSection } from './MylistSection';

export function DetailRail({ item }: { item: ItemDetail }) {
  return (
    <aside className="sticky top-0">
      <div
        className="mb-4 w-full max-w-[220px] rounded-app bg-[linear-gradient(160deg,#33304a,#232323)]"
        style={{
          aspectRatio: '2/3',
          ...(item.cover_image_url
            ? { backgroundImage: `url(${item.cover_image_url})`, backgroundSize: 'cover' }
            : {}),
        }}
      />
      <h1 className="mb-1.5 font-display text-[21px] font-semibold text-text-primary">
        {item.title}
      </h1>
      <div className="mb-4.5 text-[13px] text-text-faint">
        {item.original_title}
        {item.original_title && item.release_date ? ' ・ ' : ''}
        {item.release_date ? `${item.release_date.slice(0, 4)}年` : ''}
      </div>

      <div className="flex flex-col gap-2.5">
        <span className="flex items-center gap-2 text-[12.5px] text-text-muted">
          <FiPlayCircle className="h-4 w-4 text-text-faint" />
          {statusLabels[item.status]}
        </span>
        <RatingStars itemId={item.id} rating={item.rating} />
        <FavoriteToggle itemId={item.id} isFavorite={item.is_favorite} />
        {item.release_date && (
          <span className="flex items-center gap-2 text-[12.5px] text-text-muted">
            <FiCalendar className="h-4 w-4 text-text-faint" />
            {item.release_date}
          </span>
        )}
        <span className="flex items-center gap-2 text-[12.5px] text-text-faint">
          <FiLink className="h-4 w-4 text-text-faint" />
          {item.source === 'api'
            ? `API / external_id: ${item.external_id ?? '-'}`
            : '手動登録'}
        </span>
      </div>

      <hr className="my-4.5 border-t border-border-soft" />

      <TagCategorySection itemId={item.id} kind="tag" items={item.tags} />
      <TagCategorySection itemId={item.id} kind="category" items={item.categories} />
      <MylistSection itemId={item.id} />
    </aside>
  );
}
