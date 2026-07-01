import { useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useForm, FormProvider } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Form, FormField, FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form';
import { ItemBaseFields } from '@/features/item-form/components/ItemBaseFields';
import { AnimeDetailsFields } from '@/features/item-form/components/AnimeDetailsFields';
import { MovieDetailsFields } from '@/features/item-form/components/MovieDetailsFields';
import { DramaDetailsFields } from '@/features/item-form/components/DramaDetailsFields';
import { MangaDetailsFields } from '@/features/item-form/components/MangaDetailsFields';
import { NovelDetailsFields } from '@/features/item-form/components/NovelDetailsFields';
import { GameDetailsFields } from '@/features/item-form/components/GameDetailsFields';
import { AcademicBookDetailsFields } from '@/features/item-form/components/AcademicBookDetailsFields';
import { PaperDetailsFields } from '@/features/item-form/components/PaperDetailsFields';
import { createItemRequestSchema, type CreateItemRequestInput } from '@/lib/itemSchema';
import { useItemQuery, useCreateItemMutation, useUpdateItemMutation } from '@/api/items';
import { ApiClientError, type MediaType } from '@/types';

type ItemGroup = 'general' | 'academic' | 'paper';
type ItemFormMode = 'create' | 'edit';

type ItemFormPageProps = {
  mode: ItemFormMode;
  group?: ItemGroup;
};

function groupToDefaultMediaType(group?: ItemGroup): MediaType {
  if (group === 'academic') return 'academic_book';
  if (group === 'paper') return 'paper';
  return 'anime';
}

function DetailsFields({ mediaType }: { mediaType: MediaType }) {
  switch (mediaType) {
    case 'anime': return <AnimeDetailsFields />;
    case 'movie': return <MovieDetailsFields />;
    case 'drama': return <DramaDetailsFields />;
    case 'manga': return <MangaDetailsFields />;
    case 'novel': return <NovelDetailsFields />;
    case 'game': return <GameDetailsFields />;
    case 'academic_book': return <AcademicBookDetailsFields />;
    case 'paper': return <PaperDetailsFields />;
  }
}

const MEDIA_TYPE_OPTIONS: { value: MediaType; label: string }[] = [
  { value: 'anime', label: 'アニメ' },
  { value: 'movie', label: '映画' },
  { value: 'drama', label: 'ドラマ' },
  { value: 'manga', label: '漫画' },
  { value: 'novel', label: '小説' },
  { value: 'game', label: 'ゲーム' },
  { value: 'academic_book', label: '学術書・専門書' },
  { value: 'paper', label: '論文・文献' },
];

export default function ItemFormPage({ mode, group }: ItemFormPageProps) {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data: itemData, isLoading: isItemLoading, error: itemError } = useItemQuery(
    mode === 'edit' && id ? id : ''
  );

  const createMutation = useCreateItemMutation();
  const updateMutation = useUpdateItemMutation(id ?? '');

  const form = useForm<CreateItemRequestInput>({
    resolver: zodResolver(createItemRequestSchema),
    defaultValues: {
      mediaType: groupToDefaultMediaType(group),
      title: '',
      originalTitle: '',
      description: '',
      coverImageUrl: '',
      releaseDate: '',
      homepageUrl: '',
      source: 'manual',
      externalId: '',
      details: {},
    },
  });

  const mediaType = form.watch('mediaType');
  const source = form.watch('source');

  // edit mode: populate form with existing item data
  useEffect(() => {
    if (mode === 'edit' && itemData?.data) {
      const item = itemData.data;
      form.reset({
        mediaType: item.mediaType,
        title: item.title,
        originalTitle: item.originalTitle ?? '',
        description: item.description ?? '',
        coverImageUrl: item.coverImageUrl ?? '',
        releaseDate: item.releaseDate ?? '',
        homepageUrl: item.homepageUrl ?? '',
        source: item.source,
        externalId: item.externalId ?? '',
        details: item.details as Record<string, unknown>,
      });
    }
  }, [mode, itemData, form]);

  // edit mode: redirect on 404
  useEffect(() => {
    if (itemError instanceof ApiClientError && itemError.code === 'ITEM_NOT_FOUND') {
      toast.error('アイテムが見つかりませんでした');
      navigate('/');
    }
  }, [itemError, navigate]);

  const onSubmit = async (values: CreateItemRequestInput) => {
    try {
      if (mode === 'create') {
        const result = await createMutation.mutateAsync({
          mediaType: values.mediaType,
          title: values.title,
          originalTitle: values.originalTitle || undefined,
          description: values.description || undefined,
          coverImageUrl: values.coverImageUrl || undefined,
          releaseDate: values.releaseDate || undefined,
          homepageUrl: values.homepageUrl || undefined,
          source: values.source,
          externalId: values.externalId || undefined,
          details: values.details,
        });
        toast.success('アイテムを作成しました');
        navigate(`/items/${(result.data as { id: string }).id}`);
      } else {
        await updateMutation.mutateAsync({
          title: values.title,
          description: values.description || undefined,
        });
        toast.success('アイテムを更新しました');
        navigate(`/items/${id}`);
      }
    } catch (err) {
      if (err instanceof ApiClientError) {
        if (err.code === 'VALIDATION_ERROR') {
          toast.error('入力内容を確認してください');
          // field-level errors would be set here if the API returns field details
        } else if (err.code === 'ITEM_NOT_FOUND') {
          toast.error('アイテムが見つかりませんでした');
          navigate('/');
        } else {
          toast.error(err.message);
        }
      } else {
        toast.error('エラーが発生しました');
      }
    }
  };

  const isPending = createMutation.isPending || updateMutation.isPending;

  if (mode === 'edit' && isItemLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl p-4 md:p-6">
      <h1 className="mb-6 text-2xl font-bold">
        {mode === 'create' ? 'アイテムを追加' : 'アイテムを編集'}
      </h1>

      <FormProvider {...form}>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
            {/* media_type selector: create only */}
            {mode === 'create' && (
              <FormField
                control={form.control}
                name="mediaType"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>メディア種別 *</FormLabel>
                    <FormControl>
                      <select
                        className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                        {...field}
                      >
                        {MEDIA_TYPE_OPTIONS.map((opt) => (
                          <option key={opt.value} value={opt.value}>
                            {opt.label}
                          </option>
                        ))}
                      </select>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            )}

            {/* base fields */}
            <ItemBaseFields />

            {/* external_id: api source only, read-only */}
            {source === 'api' && (
              <FormField
                control={form.control}
                name="externalId"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>外部ID</FormLabel>
                    <FormControl>
                      <Input {...field} readOnly disabled className="bg-muted" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            )}

            {/* media-type specific fields */}
            <DetailsFields mediaType={mediaType} />

            <div className="flex gap-3 pt-2">
              <Button type="button" variant="outline" onClick={() => navigate(-1)}>
                キャンセル
              </Button>
              <Button type="submit" disabled={isPending}>
                {isPending && (
                  <span className="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                )}
                保存
              </Button>
            </div>
          </form>
        </Form>
      </FormProvider>
    </div>
  );
}
