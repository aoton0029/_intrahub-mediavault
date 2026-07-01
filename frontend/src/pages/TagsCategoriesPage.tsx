import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Form, FormField, FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form';
import {
  useTagsQuery,
  useCategoriesQuery,
  useCreateTagMutation,
  useCreateCategoryMutation,
} from '@/api/tags';
import { ApiClientError } from '@/types';

const nameSchema = z.object({
  name: z.string().min(1, '名称を入力してください'),
});
type NameInput = z.infer<typeof nameSchema>;

function CreateForm({
  label,
  onSubmit,
  isPending,
}: {
  label: string;
  onSubmit: (name: string) => Promise<void>;
  isPending: boolean;
}) {
  const form = useForm<NameInput>({
    resolver: zodResolver(nameSchema),
    defaultValues: { name: '' },
  });

  const handleSubmit = form.handleSubmit(async (values) => {
    await onSubmit(values.name);
    form.reset();
  });

  return (
    <Form {...form}>
      <form onSubmit={handleSubmit} className="flex gap-2 items-end">
        <FormField
          control={form.control}
          name="name"
          render={({ field }) => (
            <FormItem className="flex-1">
              <FormLabel>{label}</FormLabel>
              <FormControl>
                <Input placeholder={`${label}名`} {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit" disabled={isPending || !form.formState.isValid && form.formState.isSubmitted || !form.watch('name')}>
          作成
        </Button>
      </form>
    </Form>
  );
}

export default function TagsCategoriesPage() {
  const { data: tagsData, isLoading: tagsLoading } = useTagsQuery();
  const { data: categoriesData, isLoading: categoriesLoading } = useCategoriesQuery();
  const createTagMutation = useCreateTagMutation();
  const createCategoryMutation = useCreateCategoryMutation();

  const tags = tagsData?.data ?? [];
  const categories = categoriesData?.data ?? [];

  const handleCreateTag = async (name: string) => {
    try {
      await createTagMutation.mutateAsync({ name });
      toast.success(`タグ「${name}」を作成しました`);
    } catch (err) {
      if (err instanceof ApiClientError && err.code === 'VALIDATION_ERROR') {
        toast.error(err.message);
      } else {
        toast.error('タグの作成に失敗しました');
      }
      throw err;
    }
  };

  const handleCreateCategory = async (name: string) => {
    try {
      await createCategoryMutation.mutateAsync({ name });
      toast.success(`カテゴリ「${name}」を作成しました`);
    } catch (err) {
      if (err instanceof ApiClientError && err.code === 'VALIDATION_ERROR') {
        toast.error(err.message);
      } else {
        toast.error('カテゴリの作成に失敗しました');
      }
      throw err;
    }
  };

  return (
    <div className="p-6 max-w-2xl mx-auto space-y-8">
      <h1 className="text-2xl font-bold">タグ・カテゴリ管理</h1>

      {/* タグセクション */}
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">タグ</h2>
        <CreateForm
          label="タグ"
          onSubmit={handleCreateTag}
          isPending={createTagMutation.isPending}
        />
        {tagsLoading ? (
          <p className="text-muted-foreground text-sm">読み込み中...</p>
        ) : tags.length === 0 ? (
          <p className="text-muted-foreground text-sm">タグがありません</p>
        ) : (
          <ul className="space-y-1">
            {tags.map((tag) => (
              <li key={tag.id} className="flex items-center justify-between rounded border px-3 py-2">
                <span>{tag.name}</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* カテゴリセクション */}
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">カテゴリ</h2>
        <CreateForm
          label="カテゴリ"
          onSubmit={handleCreateCategory}
          isPending={createCategoryMutation.isPending}
        />
        {categoriesLoading ? (
          <p className="text-muted-foreground text-sm">読み込み中...</p>
        ) : categories.length === 0 ? (
          <p className="text-muted-foreground text-sm">カテゴリがありません</p>
        ) : (
          <ul className="space-y-1">
            {categories.map((category) => (
              <li key={category.id} className="flex items-center justify-between rounded border px-3 py-2">
                <span>{category.name}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
