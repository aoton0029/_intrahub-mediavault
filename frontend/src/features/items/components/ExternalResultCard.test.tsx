import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ExternalResultCard } from './ExternalResultCard';
import type { MovieDetails } from '../types';

const apiFetchMock = vi.fn();
vi.mock('@/lib/api-client', async (importOriginal) => {
  const original = await importOriginal<typeof import('@/lib/api-client')>();
  return {
    ...original,
    apiFetch: (...args: unknown[]) => apiFetchMock(...args),
  };
});

const movieResult: MovieDetails = {
  media_type: 'movie',
  provider: 'tmdb',
  external_id: '603',
  title: 'マトリックス',
  original_title: 'The Matrix',
  alternative_titles: [],
  description: 'あらすじ',
  release_date: '1999-03-31',
  image_url: 'https://image.tmdb.org/t/p/w342/poster.jpg',
  genres: [],
  rating: 8.2,
  url: null,
  runtime_minutes: null,
  original_language: 'en',
  vote_count: 100,
  collection: null,
  production_companies: [],
};

function renderCard(result: MovieDetails) {
  const queryClient = new QueryClient({ defaultOptions: { mutations: { retry: false } } });
  return render(
    <QueryClientProvider client={queryClient}>
      <ExternalResultCard result={result} mediaType="movie" />
    </QueryClientProvider>,
  );
}

describe('ExternalResultCard', () => {
  beforeEach(() => {
    apiFetchMock.mockReset();
  });

  it('MediaDetailsの正規化フィールドからタイトル・原題・公開年・プロバイダを表示する', () => {
    renderCard(movieResult);

    expect(screen.getByText('マトリックス')).toBeInTheDocument();
    expect(screen.getByText('The Matrix')).toBeInTheDocument();
    expect(screen.getByText('1999年')).toBeInTheDocument();
    expect(screen.getByText('TMDB')).toBeInTheDocument();
  });

  it('provider=null（Jikan）はJikan表示になる', () => {
    renderCard({ ...movieResult, provider: null });
    expect(screen.getByText('Jikan')).toBeInTheDocument();
  });

  it('取り込むでMediaDetails全体をPOST /items/importへ送る', async () => {
    apiFetchMock.mockResolvedValue({ data: { id: 'x' } });
    renderCard(movieResult);

    await userEvent.click(screen.getByRole('button', { name: '取り込む' }));

    expect(apiFetchMock).toHaveBeenCalledWith(
      '/items/import',
      expect.objectContaining({ method: 'POST' }),
    );
    const options = apiFetchMock.mock.calls[0][1] as { body: string };
    expect(JSON.parse(options.body)).toEqual(movieResult);
  });
});
