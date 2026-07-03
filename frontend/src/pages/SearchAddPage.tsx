import { useState, useEffect } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { toast } from 'sonner'
import { useExternalSearchQuery, useImportItemMutation } from '@/api/search'
import { ApiClientError } from '@/types'
import type { MediaType, ExternalSearchResultItem, ExternalSearchQuery } from '@/types'
import { Button } from '@/components/ui/button'
import { EmptyState } from '@/components/common/EmptyState'

type ItemGroup = 'general' | 'academic' | 'paper'

const GROUP_MEDIA_TYPES: Record<ItemGroup, MediaType[]> = {
  general: ['anime', 'movie', 'drama', 'manga', 'novel', 'game'],
  academic: ['academic_book'],
  paper: ['paper'],
}

const MEDIA_TYPE_LABELS: Record<MediaType, string> = {
  anime: 'アニメ',
  movie: '映画',
  drama: 'ドラマ',
  manga: '漫画',
  novel: '小説',
  game: 'ゲーム',
  academic_book: '学術書',
  paper: '論文',
}

type SearchFormValues = {
  q: string
  mediaType: MediaType
}

type SearchAddPageProps = {
  group: ItemGroup
}

export default function SearchAddPage({ group }: SearchAddPageProps) {
  const navigate = useNavigate()
  const mediaTypes = GROUP_MEDIA_TYPES[group]

  const { register, handleSubmit } = useForm<SearchFormValues>({
    defaultValues: {
      q: '',
      mediaType: mediaTypes[0],
    },
  })

  const [searchQuery, setSearchQuery] = useState<ExternalSearchQuery | null>(null)

  const activeQuery: ExternalSearchQuery = searchQuery ?? { q: '', mediaType: mediaTypes[0] }
  const { data: results, isLoading: isSearching, isError, error, refetch } =
    useExternalSearchQuery(activeQuery)

  const importMutation = useImportItemMutation()

  useEffect(() => {
    if (isError && error) {
      if (
        !(error instanceof ApiClientError) ||
        (error.code !== 'API_KEY_NOT_CONFIGURED' && error.code !== 'EXTERNAL_API_TIMEOUT')
      ) {
        toast.error('検索に失敗しました')
      }
    }
  }, [isError, error])

  const onSubmit = (values: SearchFormValues) => {
    setSearchQuery({ q: values.q, mediaType: values.mediaType })
  }

  const handleImport = (result: ExternalSearchResultItem) => {
    const mediaType = searchQuery?.mediaType ?? mediaTypes[0]
    importMutation.mutate(
      { mediaType, externalId: result.externalId, raw: result.raw },
      {
        onSuccess: (item) => {
          toast.success('追加しました')
          navigate(`/items/${item.id}`)
        },
        onError: (err) => {
          if (err instanceof ApiClientError) {
            toast.error(err.message)
          } else {
            toast.error('インポートに失敗しました')
          }
        },
      }
    )
  }

  const renderErrorUI = () => {
    if (!isError || !error || !(error instanceof ApiClientError)) return null

    if (error.code === 'API_KEY_NOT_CONFIGURED') {
      return (
        <div role="alert" className="flex flex-col items-center gap-4 p-8 text-center">
          <p className="text-sm text-destructive">APIキーが設定されていません</p>
          <Button asChild>
            <Link to={`/items/new/${group}`}>手動で追加する</Link>
          </Button>
        </div>
      )
    }

    if (error.code === 'EXTERNAL_API_TIMEOUT') {
      return (
        <div role="alert" className="flex flex-col items-center gap-4 p-8 text-center">
          <p className="text-sm text-destructive">検索がタイムアウトしました</p>
          <Button type="button" onClick={() => refetch()}>再試行</Button>
        </div>
      )
    }

    return null
  }

  return (
    <div className="flex flex-col gap-4 p-4">
      <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-2">
        <div className="flex flex-col gap-1">
          <label htmlFor="search-q" className="text-sm font-medium">検索語</label>
          <input
            id="search-q"
            type="text"
            className="rounded border border-input bg-background px-3 py-2 text-sm"
            placeholder="タイトルを入力..."
            {...register('q', { required: true })}
          />
        </div>
        <div className="flex gap-2">
          <label htmlFor="search-media-type" className="sr-only">メディアタイプ</label>
          <select
            id="search-media-type"
            className="rounded border border-input bg-background px-3 py-2 text-sm"
            aria-label="メディアタイプ"
            {...register('mediaType')}
          >
            {mediaTypes.map((mt) => (
              <option key={mt} value={mt}>{MEDIA_TYPE_LABELS[mt]}</option>
            ))}
          </select>
          <Button
            type="submit"
            disabled={isSearching || importMutation.isPending}
            aria-label="検索"
          >
            {isSearching ? '検索中...' : '検索'}
          </Button>
        </div>
      </form>

      {isSearching && (
        <div className="flex justify-center p-8">
          <div
            className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"
            aria-label="ローディング"
          />
        </div>
      )}

      {renderErrorUI()}

      {!isSearching && !isError && results && results.length > 0 && (
        <div className="flex flex-col">
          {results.map((result) => (
            <div key={result.externalId} className="result-row" onClick={() => handleImport(result)}>
              <div
                className="thumb"
                style={result.coverImageUrl ? { backgroundImage: `url(${result.coverImageUrl})`, backgroundSize: 'cover', backgroundPosition: 'center' } : undefined}
              />
              <div className="info">
                <div className="title">{result.title}</div>
                {result.releaseDate && <div className="sub">{result.releaseDate}</div>}
              </div>
              <Button
                type="button"
                onClick={(e) => { e.stopPropagation(); handleImport(result) }}
                disabled={importMutation.isPending}
              >
                {importMutation.isPending ? '追加中...' : '追加'}
              </Button>
            </div>
          ))}
        </div>
      )}

      {!isSearching && !isError && results && results.length === 0 && searchQuery && (
        <EmptyState message="検索結果がありません" />
      )}
    </div>
  )
}
