import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { SeasonEpisodeList } from './index'
import type { ItemGroup } from '@/features/items/types'

function makeGroup(overrides: Partial<ItemGroup> = {}): ItemGroup {
  return {
    id: 'group-1',
    group_type: 'season',
    group_name: 'シーズン1',
    display_order: 1,
    episodes: [],
    ...overrides,
  }
}

describe('SeasonEpisodeList', () => {
  it('renders grouped seasons with episode counts for anime (TC-004-01)', () => {
    const groups: ItemGroup[] = [
      makeGroup({
        id: 'g1',
        group_name: 'シーズン1',
        display_order: 1,
        episodes: [
          { id: 'e1', episode_number: 1, title: '漂流のはじまり' },
          { id: 'e2', episode_number: 2, title: '船団のしきたり' },
        ],
      }),
      makeGroup({
        id: 'g2',
        group_name: 'シーズン2（視聴中）',
        display_order: 2,
        episodes: [{ id: 'e3', episode_number: 1, title: '再会の軌道' }],
      }),
    ]

    render(<SeasonEpisodeList mediaType="anime" groups={groups} />)

    expect(screen.getByText('シーズン1')).toBeInTheDocument()
    expect(screen.getByText('2話')).toBeInTheDocument()
    expect(screen.getByText('漂流のはじまり')).toBeInTheDocument()
    expect(screen.getByText('シーズン2（視聴中）')).toBeInTheDocument()
    expect(screen.getByText('1話')).toBeInTheDocument()
    expect(screen.getByText('再会の軌道')).toBeInTheDocument()
  })

  it('renders for drama media type as well', () => {
    const groups: ItemGroup[] = [
      makeGroup({ episodes: [{ id: 'e1', episode_number: 1, title: '第一話' }] }),
    ]
    render(<SeasonEpisodeList mediaType="drama" groups={groups} />)
    expect(screen.getByText('第一話')).toBeInTheDocument()
  })

  it('orders seasons and episodes by display_order / episode_number ascending', () => {
    const groups: ItemGroup[] = [
      makeGroup({
        id: 'g2',
        group_name: 'シーズン2',
        display_order: 2,
        episodes: [
          { id: 'e2', episode_number: 2, title: 'B' },
          { id: 'e1', episode_number: 1, title: 'A' },
        ],
      }),
      makeGroup({ id: 'g1', group_name: 'シーズン1', display_order: 1, episodes: [] }),
    ]

    render(<SeasonEpisodeList mediaType="anime" groups={groups} />)

    const headers = screen.getAllByRole('heading', { level: 4 })
    expect(headers[0]).toHaveTextContent('シーズン1')
    expect(headers[1]).toHaveTextContent('シーズン2')

    const episodeRows = document.querySelectorAll('.episode-row')
    expect(Array.from(episodeRows).map((el) => el.textContent)).toEqual(['01A', '02B'])
  })

  it('renders nothing when mediaType is not anime/drama', () => {
    const groups: ItemGroup[] = [makeGroup()]
    const { container } = render(<SeasonEpisodeList mediaType="movie" groups={groups} />)
    expect(container).toBeEmptyDOMElement()
  })

  it('renders nothing when groups is empty', () => {
    const { container } = render(<SeasonEpisodeList mediaType="anime" groups={[]} />)
    expect(container).toBeEmptyDOMElement()
  })

  it('ignores non-season groups (volume/chapter)', () => {
    const groups: ItemGroup[] = [
      makeGroup({ group_type: 'volume', group_name: '巻1' }),
      makeGroup({ id: 'g-season', group_type: 'season', group_name: 'シーズン1' }),
    ]
    render(<SeasonEpisodeList mediaType="anime" groups={groups} />)
    expect(screen.queryByText('巻1')).not.toBeInTheDocument()
    expect(screen.getByText('シーズン1')).toBeInTheDocument()
  })

  it('renders season header only when episodes is empty', () => {
    const groups: ItemGroup[] = [makeGroup({ episodes: [] })]
    render(<SeasonEpisodeList mediaType="anime" groups={groups} />)
    expect(screen.getByText('シーズン1')).toBeInTheDocument()
    expect(screen.queryByText(/話$/)).toHaveTextContent('0話')
  })

  it('renders episode_number only when title is not set', () => {
    const groups: ItemGroup[] = [
      makeGroup({ episodes: [{ id: 'e1', episode_number: 3 }] }),
    ]
    render(<SeasonEpisodeList mediaType="anime" groups={groups} />)
    expect(screen.getByText('03')).toBeInTheDocument()
  })
})
