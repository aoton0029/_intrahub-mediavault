import { z } from 'zod'

const MEDIA_TYPES = ['anime', 'movie', 'drama', 'manga', 'novel', 'game', 'academic_book', 'paper'] as const
const ITEM_STATUSES = ['not_started', 'in_progress', 'completed'] as const
const ITEM_SOURCES = ['api', 'manual'] as const

const releaseDateRegex = /^\d{4}-\d{2}-\d{2}$/

const optionalUrl = z.string().url().or(z.literal('')).optional()

export const createItemRequestSchema = z.object({
  mediaType: z.enum(MEDIA_TYPES),
  title: z.string().min(1),
  originalTitle: z.string().optional(),
  description: z.string().optional(),
  coverImageUrl: optionalUrl,
  releaseDate: z
    .string()
    .regex(releaseDateRegex, 'YYYY-MM-DD形式で入力してください')
    .optional()
    .or(z.literal('')),
  homepageUrl: optionalUrl,
  source: z.enum(ITEM_SOURCES),
  externalId: z.string().optional(),
  details: z.record(z.string(), z.unknown()),
})

export const updateItemRequestSchema = z.object({
  title: z.string().min(1).optional(),
  description: z.string().optional(),
  status: z.enum(ITEM_STATUSES).optional(),
  consumedDate: z.string().optional(),
  rating: z.number().min(0).max(10).optional(),
  isFavorite: z.boolean().optional(),
})

export type CreateItemRequestInput = z.infer<typeof createItemRequestSchema>
export type UpdateItemRequestInput = z.infer<typeof updateItemRequestSchema>
