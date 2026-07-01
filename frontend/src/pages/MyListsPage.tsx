import { useState } from 'react'
import { toast } from 'sonner'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { zodResolver } from '@hookform/resolvers/zod'
import {
  useMyListsQuery,
  useMyListQuery,
  useCreateMyListMutation,
  useRemoveItemFromMyListMutation,
} from '@/api/mylists'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

const createListSchema = z.object({
  name: z.string().min(1, 'リスト名を入力してください'),
})
type CreateListForm = z.infer<typeof createListSchema>

export default function MyListsPage() {
  const [selectedListId, setSelectedListId] = useState<string>('')

  const { data: listsData, isLoading: listsLoading } = useMyListsQuery()
  const { data: listDetailData } = useMyListQuery(selectedListId)
  const createMutation = useCreateMyListMutation()
  const removeMutation = useRemoveItemFromMyListMutation()

  const lists = listsData?.data ?? []
  const listDetail = listDetailData?.data
  const listItems = listDetail?.items ?? []

  const { register, handleSubmit, reset, formState: { errors, isValid } } = useForm<CreateListForm>({
    resolver: zodResolver(createListSchema),
    mode: 'onChange',
  })

  const onCreateList = handleSubmit(async ({ name }) => {
    try {
      await createMutation.mutateAsync(name)
      toast.success(`「${name}」を作成しました`)
      reset()
    } catch {
      toast.error('リストの作成に失敗しました')
    }
  })

  const handleRemoveItem = async (itemId: string) => {
    if (!selectedListId) return
    try {
      await removeMutation.mutateAsync({ listId: selectedListId, itemId })
      toast.success('アイテムをリストから削除しました')
    } catch {
      toast.error('削除に失敗しました')
    }
  }

  return (
    <div className="flex h-full gap-4 p-4">
      {/* サイドバー: リスト一覧 */}
      <aside className="w-64 flex-shrink-0 flex flex-col gap-3">
        <h2 className="text-sm font-semibold">マイリスト</h2>

        <form onSubmit={onCreateList} className="flex flex-col gap-1">
          <Input
            placeholder="リスト名"
            {...register('name')}
            aria-label="リスト名"
          />
          {errors.name && (
            <p className="text-xs text-destructive">{errors.name.message}</p>
          )}
          <Button type="submit" disabled={!isValid || createMutation.isPending}>
            作成
          </Button>
        </form>

        {listsLoading ? (
          <div className="text-sm text-muted-foreground">読み込み中...</div>
        ) : lists.length === 0 ? (
          <div className="text-sm text-muted-foreground">リストがありません</div>
        ) : (
          <ul className="flex flex-col gap-1">
            {lists.map(list => (
              <li key={list.id}>
                <button
                  type="button"
                  onClick={() => setSelectedListId(list.id)}
                  className={`w-full rounded px-3 py-2 text-left text-sm transition-colors hover:bg-muted ${
                    selectedListId === list.id ? 'bg-muted font-medium' : ''
                  }`}
                >
                  {list.name}
                </button>
              </li>
            ))}
          </ul>
        )}
      </aside>

      {/* メインエリア: リスト詳細 */}
      <main className="flex-1">
        {!selectedListId ? (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            リストを選択してください
          </div>
        ) : (
          <div className="flex flex-col gap-4">
            <h2 className="text-lg font-semibold">{listDetail?.name}</h2>
            {listItems.length === 0 ? (
              <div className="text-sm text-muted-foreground">アイテムがありません</div>
            ) : (
              <ul className="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4">
                {listItems.map(item => (
                  <li key={item.id} className="flex flex-col gap-2 rounded-lg border p-3">
                    {item.coverImageUrl && (
                      <img
                        src={item.coverImageUrl}
                        alt={item.title}
                        className="aspect-[2/3] w-full rounded object-cover"
                      />
                    )}
                    <span className="text-sm font-medium line-clamp-2">{item.title}</span>
                    <Button
                      type="button"
                      variant="destructive"
                      size="sm"
                      onClick={() => handleRemoveItem(item.id)}
                      disabled={removeMutation.isPending}
                    >
                      削除
                    </Button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </main>
    </div>
  )
}
