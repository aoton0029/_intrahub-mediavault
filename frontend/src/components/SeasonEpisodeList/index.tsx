import type { ItemGroup, MediaType } from '@/features/items/types'

export interface SeasonEpisodeListProps {
  mediaType: MediaType
  groups: ItemGroup[]
}

/**
 * 詳細画面「シーズン構成」セクション（.doc-section > .group-block）
 *
 * TASK-0016: media_typeがanime/dramaの場合のみ、ItemGroup(group_type: 'season')を
 * display_order昇順でグループ化して表示する。それ以外のmedia_type・groupsが空の場合は
 * セクション自体を描画しない。
 */
export function SeasonEpisodeList({ mediaType, groups }: SeasonEpisodeListProps) {
  if (mediaType !== 'anime' && mediaType !== 'drama') {
    return null
  }

  const seasons = groups
    .filter((group) => group.group_type === 'season')
    .slice()
    .sort((a, b) => a.display_order - b.display_order)

  if (seasons.length === 0) {
    return null
  }

  return (
    <div className="doc-section">
      <h3>シーズン構成</h3>
      {seasons.map((season) => {
        const episodes = (season.episodes ?? []).slice().sort((a, b) => a.episode_number - b.episode_number)
        return (
          <div className="group-block" key={season.id}>
            <h4 className="group-header">
              <span>{season.group_name}</span>
              <span>{episodes.length}話</span>
            </h4>
            {episodes.map((episode) => (
              <div className="episode-row" key={episode.id}>
                <span className="num">{String(episode.episode_number).padStart(2, '0')}</span>
                {episode.title ?? episode.episode_number}
              </div>
            ))}
          </div>
        )
      })}
    </div>
  )
}
