import type { Item } from '../types';
import { mediaTypeLabels } from '../types';

export function MediaCard({ item }: { item: Item }) {
  return (
    <div className="media-card overflow-hidden rounded-app border border-border-soft bg-bg-surface transition-[border-color,transform] hover:-translate-y-0.5 hover:border-accent">
      <div
        className="relative flex aspect-[2/3] items-end justify-end bg-[linear-gradient(160deg,#33304a,#232323)] p-1.5"
        style={
          item.cover_image_url
            ? { backgroundImage: `url(${item.cover_image_url})`, backgroundSize: 'cover' }
            : undefined
        }
      >
        <span className="badge absolute top-1.5 left-1.5 rounded bg-black/55 px-1.5 py-0.5 font-mono text-[10px] tracking-wide text-text-muted">
          {mediaTypeLabels[item.media_type]}
        </span>
        {item.is_favorite && (
          <span className="fav text-sm text-favorite" aria-hidden="true">
            ❤
          </span>
        )}
      </div>
      <div className="p-2.5 pt-2.5 pb-3">
        <p className="title m-0 mb-1 line-clamp-2 font-display text-[13.5px] font-semibold text-text-primary">
          {item.title}
        </p>
        {item.rating !== undefined && (
          <div className="meta flex items-center gap-1.5 font-mono text-[11px] text-favorite">
            ★ {item.rating}
          </div>
        )}
        {(item.tags?.length ?? 0) > 0 && (
          <div className="prop-taglist mt-1.5 flex flex-wrap gap-1.5">
            {item.tags!.map((tag) => (
              <span
                key={tag.id}
                className="tag-pill inline-flex items-center rounded bg-accent-soft px-1.5 py-0.5 font-mono text-[11px] text-accent-strong"
              >
                #{tag.name}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
