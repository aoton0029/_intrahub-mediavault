import { useFormContext } from 'react-hook-form'
import { FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { TagInput } from './TagInput'
import type { CreateItemRequestInput } from '@/lib/itemSchema'

const doiPattern = /^10\./

function validateDoi(value: string): string | undefined {
  if (!value) return undefined
  if (!doiPattern.test(value)) return 'DOIは10.で始まる形式で入力してください'
  return undefined
}

export function PaperDetailsFields() {
  const form = useFormContext<CreateItemRequestInput>()
  const source = form.watch('source')
  const isApi = source === 'api'

  const details = (form.watch('details') ?? {}) as Record<string, unknown>

  const getV = (key: string) => details[key]
  const setV = (key: string, value: unknown) => {
    form.setValue('details', { ...details, [key]: value })
  }

  const doiValue = (getV('doi') as string | undefined) ?? ''
  const doiError = doiValue ? validateDoi(doiValue) : undefined

  return (
    <div className="space-y-4">
      <FormItem>
        <FormLabel>DOI</FormLabel>
        <FormControl>
          <Input
            value={doiValue}
            onChange={(e) => setV('doi', e.target.value || undefined)}
            placeholder="例: 10.1234/example.doi"
          />
        </FormControl>
        {doiError && <FormMessage>{doiError}</FormMessage>}
      </FormItem>

      <FormItem>
        <FormLabel>掲載誌</FormLabel>
        <FormControl>
          <Input
            value={(getV('journalName') as string | undefined) ?? ''}
            onChange={(e) => setV('journalName', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>巻号</FormLabel>
        <FormControl>
          <Input
            value={(getV('volumeIssue') as string | undefined) ?? ''}
            onChange={(e) => setV('volumeIssue', e.target.value || undefined)}
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>ページ範囲</FormLabel>
        <FormControl>
          <Input
            value={(getV('pageRange') as string | undefined) ?? ''}
            onChange={(e) => setV('pageRange', e.target.value || undefined)}
            placeholder="例: 123-145"
          />
        </FormControl>
      </FormItem>

      <FormItem>
        <FormLabel>著者リスト</FormLabel>
        <FormControl>
          <TagInput
            value={(getV('authorList') as string[] | undefined) ?? []}
            onChange={(val) => setV('authorList', val)}
            placeholder="著者名を入力してEnterで追加"
          />
        </FormControl>
      </FormItem>

      {isApi && (
        <FormItem>
          <FormLabel>NDL ID</FormLabel>
          <FormControl>
            <Input value={(getV('ndlId') as string | undefined) ?? ''} readOnly disabled />
          </FormControl>
        </FormItem>
      )}
    </div>
  )
}
