import { useFormContext } from 'react-hook-form'
import { FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import type { CreateItemRequestInput } from '@/lib/itemSchema'

function validateIsbn(value: string): string | undefined {
  if (!value) return undefined
  const digits = value.replace(/-/g, '')
  if (!/^\d+$/.test(digits)) return 'ISBNは数字とハイフンのみ使用できます'
  if (digits.length !== 10 && digits.length !== 13) return 'ISBNはISBN-10またはISBN-13形式で入力してください'
  return undefined
}

export function AcademicBookDetailsFields() {
  const form = useFormContext<CreateItemRequestInput>()
  const source = form.watch('source')
  const isApi = source === 'api'

  const details = (form.watch('details') ?? {}) as Record<string, unknown>

  const getV = (key: string) => details[key]
  const setV = (key: string, value: unknown) => {
    form.setValue('details', { ...details, [key]: value })
  }

  const isbnValue = (getV('isbn') as string | undefined) ?? ''
  const isbnError = isbnValue ? validateIsbn(isbnValue) : undefined

  return (
    <div className="space-y-4">
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
            value={isbnValue}
            onChange={(e) => setV('isbn', e.target.value || undefined)}
            placeholder="例: 978-4-00-000000-0"
          />
        </FormControl>
        {isbnError && <FormMessage>{isbnError}</FormMessage>}
      </FormItem>

      {isApi && (
        <>
          <FormItem>
            <FormLabel>NDL ID</FormLabel>
            <FormControl>
              <Input value={(getV('ndlId') as string | undefined) ?? ''} readOnly disabled />
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
