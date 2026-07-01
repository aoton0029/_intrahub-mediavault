import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Form, FormField, FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form';
import {
  useStaffListQuery,
  useCreateStaffMutation,
  useAttachStaffMutation,
  useDetachStaffMutation,
} from '@/api/staff';
import { useItemsQuery } from '@/api/items';
import type { Staff, ItemStaff } from '@/types';
import { ApiClientError } from '@/types';

const createStaffSchema = z.object({
  name: z.string().min(1, 'スタッフ名を入力してください'),
});
type CreateStaffInput = z.infer<typeof createStaffSchema>;

const attachSchema = z.object({
  itemId: z.string().min(1, 'アイテムを選択してください'),
  role: z.string().min(1, '役割を入力してください'),
  characterName: z.string().optional(),
});
type AttachInput = z.infer<typeof attachSchema>;

function AttachStaffDialog({
  staff,
  onClose,
}: {
  staff: Staff;
  onClose: () => void;
}) {
  const { data: itemsData } = useItemsQuery({});
  const attachMutation = useAttachStaffMutation();
  const items = itemsData?.data ?? [];

  const form = useForm<AttachInput>({
    resolver: zodResolver(attachSchema),
    defaultValues: { itemId: '', role: '', characterName: '' },
  });

  const handleSubmit = form.handleSubmit(async (values) => {
    try {
      await attachMutation.mutateAsync({
        itemId: values.itemId,
        staffId: staff.id,
        role: values.role,
        characterName: values.characterName || undefined,
      });
      toast.success(`${staff.name} を作品に紐付けました`);
      onClose();
    } catch (err) {
      toast.error(err instanceof ApiClientError ? err.message : '紐付けに失敗しました');
    }
  });

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-background rounded-lg p-6 w-full max-w-md space-y-4">
        <h3 className="text-lg font-semibold">{staff.name} を作品に紐付け</h3>
        <Form {...form}>
          <form onSubmit={handleSubmit} className="space-y-4">
            <FormField
              control={form.control}
              name="itemId"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>作品</FormLabel>
                  <FormControl>
                    <select
                      className="w-full border rounded px-3 py-2 text-sm bg-background"
                      {...field}
                    >
                      <option value="">選択してください</option>
                      {items.map((item) => (
                        <option key={item.id} value={item.id}>
                          {item.title}
                        </option>
                      ))}
                    </select>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="role"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>役割（必須）</FormLabel>
                  <FormControl>
                    <Input placeholder="例: 監督、声優、著者" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="characterName"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>キャラクター名（任意）</FormLabel>
                  <FormControl>
                    <Input placeholder="キャラクター名" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <div className="flex gap-2 justify-end">
              <Button type="button" variant="outline" onClick={onClose}>
                キャンセル
              </Button>
              <Button type="submit" disabled={attachMutation.isPending}>
                紐付け
              </Button>
            </div>
          </form>
        </Form>
      </div>
    </div>
  );
}

function StaffItemRow({
  staff,
  itemStaffList,
}: {
  staff: Staff;
  itemStaffList: ItemStaff[];
}) {
  const [showAttach, setShowAttach] = useState(false);
  const detachMutation = useDetachStaffMutation();
  const { data: itemsData } = useItemsQuery({});
  const items = itemsData?.data ?? [];

  const myRelations = itemStaffList.filter((is) => is.staffId === staff.id);

  const handleDetach = async (itemId: string, itemStaffId: string) => {
    try {
      await detachMutation.mutateAsync({ itemId, itemStaffId });
      toast.success('紐付けを解除しました');
    } catch {
      toast.error('紐付け解除に失敗しました');
    }
  };

  return (
    <li className="border rounded p-3 space-y-2">
      <div className="flex items-center justify-between">
        <span className="font-medium">{staff.name}</span>
        <Button size="sm" variant="outline" onClick={() => setShowAttach(true)}>
          作品に紐付け
        </Button>
      </div>
      {myRelations.length > 0 && (
        <ul className="space-y-1 pl-2">
          {myRelations.map((rel) => {
            const item = items.find((i) => i.id === rel.itemId);
            return (
              <li key={rel.id} className="flex items-center gap-2 text-sm text-muted-foreground">
                <span>{item?.title ?? rel.itemId}</span>
                <span className="text-xs border rounded px-1">{rel.role}</span>
                {rel.characterName && <span className="text-xs">({rel.characterName})</span>}
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-5 px-1 text-xs text-destructive"
                  disabled={detachMutation.isPending}
                  onClick={() => handleDetach(rel.itemId, rel.id)}
                >
                  解除
                </Button>
              </li>
            );
          })}
        </ul>
      )}
      {showAttach && <AttachStaffDialog staff={staff} onClose={() => setShowAttach(false)} />}
    </li>
  );
}

export default function StaffPage() {
  const { data: staffData, isLoading } = useStaffListQuery();
  const createMutation = useCreateStaffMutation();
  const staffList = staffData?.data ?? [];

  const form = useForm<CreateStaffInput>({
    resolver: zodResolver(createStaffSchema),
    defaultValues: { name: '' },
  });

  const handleCreate = form.handleSubmit(async (values) => {
    try {
      await createMutation.mutateAsync({ name: values.name });
      toast.success(`スタッフ「${values.name}」を追加しました`);
      form.reset();
    } catch (err) {
      toast.error(err instanceof ApiClientError ? err.message : 'スタッフの追加に失敗しました');
    }
  });

  return (
    <div className="p-6 max-w-2xl mx-auto space-y-6">
      <h1 className="text-2xl font-bold">スタッフ管理</h1>

      <section className="space-y-3">
        <h2 className="text-lg font-semibold">スタッフ追加</h2>
        <Form {...form}>
          <form onSubmit={handleCreate} className="flex gap-2 items-end">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem className="flex-1">
                  <FormLabel>スタッフ名</FormLabel>
                  <FormControl>
                    <Input placeholder="スタッフ名を入力" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button type="submit" disabled={createMutation.isPending || !form.watch('name')}>
              追加
            </Button>
          </form>
        </Form>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg font-semibold">スタッフ一覧</h2>
        {isLoading ? (
          <p className="text-muted-foreground text-sm">読み込み中...</p>
        ) : staffList.length === 0 ? (
          <p className="text-muted-foreground text-sm">スタッフがいません</p>
        ) : (
          <ul className="space-y-2">
            {staffList.map((staff) => (
              <StaffItemRow key={staff.id} staff={staff} itemStaffList={[]} />
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
