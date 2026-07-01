import { useFormContext } from 'react-hook-form'
import { FormItem, FormLabel, FormControl } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { TagInput } from './TagInput'
import type { CreateItemRequestInput } from '@/lib/itemSchema'

export function GameDetailsFields() {
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
        <FormLabel>プラットフォーム</FormLabel>
        <TagInput
          value={(getV('platformList') as string[] | undefined) ?? []}
          onChange={(v) => setV('platformList', v)}
        />
      </FormItem>

      <FormItem>
        <FormLabel>開発元</FormLabel>
        <FormControl>
          <Input
            value={(getV('developer') as string | undefined) ?? ''}
            onChange={(e) => setV('developer', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>発売元</FormLabel>
        <FormControl>
          <Input
            value={(getV('publisher') as string | undefined) ?? ''}
            onChange={(e) => setV('publisher', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      {isApi && (
        <>
          <FormItem>
            <FormLabel>Steam App ID</FormLabel>
            <FormControl>
              <Input value={(getV('steamAppid') as string | undefined) ?? ''} readOnly disabled />
            </FormControl>
          </FormItem>

          <FormItem>
            <FormLabel>IGDB ID</FormLabel>
            <FormControl>
              <Input value={(getV('igdbId') as string | undefined) ?? ''} readOnly disabled />
            </FormControl>
          </FormItem>
        </>
      )}
    </div>
  )
}
