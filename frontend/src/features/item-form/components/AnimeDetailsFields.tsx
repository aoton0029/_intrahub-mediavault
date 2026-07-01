import { useFormContext } from 'react-hook-form'
import { FormField, FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { TagInput } from './TagInput'
import type { CreateItemRequestInput } from '@/lib/itemSchema'

export function AnimeDetailsFields() {
  const form = useFormContext<CreateItemRequestInput>()
  const source = form.watch('source')
  const isApi = source === 'api'

  const details = (form.watch('details') ?? {}) as Record<string, unknown>

  const getDetailsValue = (key: string) => details[key]
  const setDetailsValue = (key: string, value: unknown) => {
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
            value={(getDetailsValue('episodeCount') as number | undefined) ?? ''}
            onChange={(e) => setDetailsValue('episodeCount', e.target.value === '' ? undefined : Number(e.target.value))}
          />
        </FormControl>
        <FormField
          control={form.control}
          name="details"
          render={() => <FormMessage />}
        />
      </FormItem>

      <FormItem>
        <FormLabel>シーズン数</FormLabel>
        <FormControl>
          <Input
            type="number"
            min={0}
            value={(getDetailsValue('seasonCount') as number | undefined) ?? ''}
            onChange={(e) => setDetailsValue('seasonCount', e.target.value === '' ? undefined : Number(e.target.value))}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>スタジオ</FormLabel>
        <FormControl>
          <Input
            value={(getDetailsValue('studio') as string | undefined) ?? ''}
            onChange={(e) => setDetailsValue('studio', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>ジャンル</FormLabel>
        <TagInput
          value={(getDetailsValue('genreList') as string[] | undefined) ?? []}
          onChange={(v) => setDetailsValue('genreList', v)}
        />
      </FormItem>

      <FormItem>
        <FormLabel>原作種別</FormLabel>
        <FormControl>
          <Input
            value={(getDetailsValue('sourceType') as string | undefined) ?? ''}
            onChange={(e) => setDetailsValue('sourceType', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      {isApi && (
        <FormItem>
          <FormLabel>Jikan ID</FormLabel>
          <FormControl>
            <Input value={(getDetailsValue('jikanId') as string | undefined) ?? ''} readOnly disabled />
          </FormControl>
        </FormItem>
      )}
    </div>
  )
}
