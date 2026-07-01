import { useFormContext } from 'react-hook-form'
import { FormItem, FormLabel, FormControl } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { TagInput } from './TagInput'
import type { CreateItemRequestInput } from '@/lib/itemSchema'

export function DramaDetailsFields() {
  const form = useFormContext<CreateItemRequestInput>()
  const source = form.watch('source')
  const isApi = source === 'api'

  const details = (form.watch('details') ?? {}) as Record<string, unknown>

  const getV = (key: string) => details[key]
  const setV = (key: string, value: unknown) => {
    form.setValue('details', { ...details, [key]: value })
  }

  return (
    <div className="space-y-4">
      <FormItem>
        <FormLabel>話数</FormLabel>
        <FormControl>
          <Input
            type="number"
            min={0}
            value={(getV('episodeCount') as number | undefined) ?? ''}
            onChange={(e) => setV('episodeCount', e.target.value === '' ? undefined : Number(e.target.value))}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>シーズン数</FormLabel>
        <FormControl>
          <Input
            type="number"
            min={0}
            value={(getV('seasonCount') as number | undefined) ?? ''}
            onChange={(e) => setV('seasonCount', e.target.value === '' ? undefined : Number(e.target.value))}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>放送局/配信網</FormLabel>
        <FormControl>
          <Input
            value={(getV('network') as string | undefined) ?? ''}
            onChange={(e) => setV('network', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>ジャンル</FormLabel>
        <TagInput
          value={(getV('genreList') as string[] | undefined) ?? []}
          onChange={(v) => setV('genreList', v)}
        />
      </FormItem>

      {isApi && (
        <FormItem>
          <FormLabel>TMDB ID</FormLabel>
          <FormControl>
            <Input value={(getV('tmdbId') as string | undefined) ?? ''} readOnly disabled />
          </FormControl>
        </FormItem>
      )}
    </div>
  )
}
