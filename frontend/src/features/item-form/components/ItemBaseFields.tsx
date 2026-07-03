import { useFormContext } from 'react-hook-form'
import { FormField, FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import type { CreateItemRequestInput } from '@/lib/itemSchema'

export function ItemBaseFields() {
  const form = useFormContext<CreateItemRequestInput>()
  const coverImageUrl = form.watch('coverImageUrl')

  return (
    <>
      <FormField
        control={form.control}
        name="title"
        render={({ field }) => (
          <FormItem className="form-field">
            <FormLabel>タイトル<span className="required">*</span></FormLabel>
            <FormControl>
              <Input placeholder="タイトルを入力" {...field} />
            </FormControl>
            <p className="field-hint">作品の正式名称を入力してください</p>
            <FormMessage className="field-error" />
          </FormItem>
        )}
      />

      <FormField
        control={form.control}
        name="originalTitle"
        render={({ field }) => (
          <FormItem className="form-field">
            <FormLabel>原題</FormLabel>
            <FormControl>
              <Input placeholder="原題（任意）" {...field} />
            </FormControl>
            <FormMessage className="field-error" />
          </FormItem>
        )}
      />

      <FormField
        control={form.control}
        name="description"
        render={({ field }) => (
          <FormItem className="form-field full">
            <FormLabel>説明</FormLabel>
            <FormControl>
              <Textarea placeholder="説明（任意）" rows={3} {...field} />
            </FormControl>
            <FormMessage className="field-error" />
          </FormItem>
        )}
      />

      <FormField
        control={form.control}
        name="coverImageUrl"
        render={({ field }) => (
          <FormItem className="form-field">
            <FormLabel>カバー画像URL</FormLabel>
            <FormControl>
              <Input placeholder="https://..." {...field} />
            </FormControl>
            <FormMessage className="field-error" />
            {coverImageUrl && (
              <img
                src={coverImageUrl}
                alt="カバープレビュー"
                className="mt-2 h-32 w-auto rounded object-cover"
              />
            )}
          </FormItem>
        )}
      />

      <FormField
        control={form.control}
        name="releaseDate"
        render={({ field }) => (
          <FormItem className="form-field">
            <FormLabel>公開日</FormLabel>
            <FormControl>
              <Input type="date" {...field} />
            </FormControl>
            <FormMessage className="field-error" />
          </FormItem>
        )}
      />

      <FormField
        control={form.control}
        name="homepageUrl"
        render={({ field }) => (
          <FormItem className="form-field">
            <FormLabel>公式サイトURL</FormLabel>
            <FormControl>
              <Input placeholder="https://..." {...field} />
            </FormControl>
            <FormMessage className="field-error" />
          </FormItem>
        )}
      />
    </>
  )
}
