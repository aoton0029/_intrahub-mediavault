import { useFormContext } from 'react-hook-form'
import { FormItem, FormLabel, FormControl } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import type { CreateItemRequestInput } from '@/lib/itemSchema'

export function NovelDetailsFields() {
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
        <FormLabel>巻数</FormLabel>
        <FormControl>
          <Input
            type="number"
            min={0}
            value={(getV('volumeCount') as number | undefined) ?? ''}
            onChange={(e) => setV('volumeCount', e.target.value === '' ? undefined : Number(e.target.value))}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>著者</FormLabel>
        <FormControl>
          <Input
            value={(getV('author') as string | undefined) ?? ''}
            onChange={(e) => setV('author', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>出版社</FormLabel>
        <FormControl>
          <Input
            value={(getV('publisher') as string | undefined) ?? ''}
            onChange={(e) => setV('publisher', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>ISBN</FormLabel>
        <FormControl>
          <Input
            value={(getV('isbn') as string | undefined) ?? ''}
            onChange={(e) => setV('isbn', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      {isApi && (
        <>
          <FormItem>
            <FormLabel>Open Library ID</FormLabel>
            <FormControl>
              <Input value={(getV('openlibraryId') as string | undefined) ?? ''} readOnly disabled />
            </FormControl>
          </FormItem>

          <FormItem>
            <FormLabel>Google Books ID</FormLabel>
            <FormControl>
              <Input value={(getV('googleBooksId') as string | undefined) ?? ''} readOnly disabled />
            </FormControl>
          </FormItem>
        </>
      )}
    </div>
  )
}
