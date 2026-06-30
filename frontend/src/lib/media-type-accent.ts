export type MediaType =
  | 'anime'
  | 'movie'
  | 'drama'
  | 'manga'
  | 'novel'
  | 'game'
  | 'academic_book'
  | 'paper'

const MEDIA_TYPE_ACCENT_CLASS: Record<MediaType, string> = {
  anime: 'text-accent-anime',
  movie: 'text-accent-movie',
  manga: 'text-accent-manga',
  novel: 'text-accent-novel',
  game: 'text-accent-game',
  drama: 'text-accent-drama',
  academic_book: 'text-accent-academic-book',
  paper: 'text-accent-paper',
}

export function getMediaTypeAccentClass(mediaType: MediaType): string {
  return MEDIA_TYPE_ACCENT_CLASS[mediaType]
}
