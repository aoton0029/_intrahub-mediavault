import { useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { toast } from 'sonner';
import { useItemQuery } from '@/api/items';
import { ApiClientError } from '@/types';
import { Button } from '@/components/ui/button';

export default function ItemDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data, isLoading, error } = useItemQuery(id ?? '');

  useEffect(() => {
    if (error instanceof ApiClientError && error.code === 'ITEM_NOT_FOUND') {
      toast.error('アイテムが見つかりませんでした');
      navigate('/');
    }
  }, [error, navigate]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
      </div>
    );
  }

  if (error && !(error instanceof ApiClientError && error.code === 'ITEM_NOT_FOUND')) {
    return (
      <div className="flex flex-col items-center gap-4 p-8">
        <p className="text-sm text-destructive">データの取得に失敗しました</p>
        <Button asChild variant="outline">
          <Link to="/">一覧へ戻る</Link>
        </Button>
      </div>
    );
  }

  const item = data?.data;
  if (!item) return null;

  return (
    <div className="mx-auto max-w-2xl p-4 md:p-6">
      <div className="mb-4 flex items-center justify-between">
        <Button asChild variant="outline" size="sm">
          <Link to="/">← 一覧へ戻る</Link>
        </Button>
        <Button asChild size="sm">
          <Link to={`/items/${id}/edit`}>編集</Link>
        </Button>
      </div>

      <h1 className="mb-2 text-2xl font-bold">{item.title}</h1>
      {item.originalTitle && (
        <p className="mb-4 text-sm text-muted-foreground">{item.originalTitle}</p>
      )}
      {item.description && <p className="mb-4">{item.description}</p>}
    </div>
  );
}
