import { useFormContext } from 'react-hook-form'
import { FormField, FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import type { CreateItemRequestInput } from '@/lib/itemSchema'

export function ItemBaseFields() {
  const form = useFormContext<CreateItemRequestInput>()
  const coverImageUrl = form.watch('coverImageUrl')

  return (
    <div className="space-y-4">
      <FormField
        control={form.control}
        name="title"
        render={({ field }) => (
          <FormItem>
            <FormLabel>タイトル *</FormLabel>
            <FormControl>
              <Input placeholder="タイトルを入力" {...field} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={form.control}
        name="originalTitle"
        render={({ field }) => (
          <FormItem>
            <FormLabel>原題</FormLabel>
            <FormControl>
              <Input placeholder="原題（任意）" {...field} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={form.control}
        name="description"
        render={({ field }) => (
          <FormItem>
            <FormLabel>説明</FormLabel>
            <FormControl>
              <Textarea placeholder="説明（任意）" rows={3} {...field} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={form.control}
        name="coverImageUrl"
        render={({ field }) => (
          <FormItem>
            <FormLabel>カバー画像URL</FormLabel>
            <FormControl>
              <Input placeholder="https://..." {...field} />
            </FormControl>
            <FormMessage />
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
          <FormItem>
            <FormLabel>公開日</FormLabel>
            <FormControl>
              <Input type="date" {...field} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={form.control}
        name="homepageUrl"
        render={({ field }) => (
          <FormItem>
            <FormLabel>公式サイトURL</FormLabel>
            <FormControl>
              <Input placeholder="https://..." {...field} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
    </div>
  )
}
