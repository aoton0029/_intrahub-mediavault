import { useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { toast } from 'sonner';
import { useItemQuery, useDeleteItemMutation } from '@/api/items';
import { useConfirmDialog } from '@/hooks/useConfirmDialog';
import { ConfirmDialog } from '@/components/common/ConfirmDialog';
import { ApiClientError } from '@/types';
import type { MediaType } from '@/types';
import { Button } from '@/components/ui/button';

// 【ラベル変換テーブル】: mediaType → パンくず用日本語ラベルの決定的マッピング
// 🟡 信頼性レベル: MediaTypeBadge.tsxの既存マッピングを流用（設計文書に階層ラベル導出ロジックの明記なし）
const MEDIA_TYPE_LABEL: Record<MediaType, string> = {
  anime: 'アニメ',
  movie: '映画',
  drama: 'ドラマ',
  manga: '漫画',
  novel: '小説',
  game: 'ゲーム',
  academic_book: '専門書',
  paper: '論文',
};

export default function ItemDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data, isLoading, error } = useItemQuery(id ?? '');
  const { open, confirm, handleConfirm, handleCancel } = useConfirmDialog();
  const deleteMutation = useDeleteItemMutation();

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

  // 【削除処理】: 削除確認ダイアログの確定操作で既存の削除APIを呼び出す 🔵
  const handleDeleteClick = () => {
    confirm(() => {
      deleteMutation.mutate(item.id);
    });
  };

  return (
    <div className="mx-auto max-w-2xl p-4 md:p-6">
      {/* 【パンくずリスト】: ホーム › メディアタイプ › アイテムタイトル 🔵 */}
      <nav className="breadcrumb">
        <Link to="/">ホーム</Link>
        <span>›</span>
        <span>{MEDIA_TYPE_LABEL[item.mediaType] ?? item.mediaType}</span>
        <span>›</span>
        <span>{item.title}</span>
      </nav>

      {/* 【タイトルバー】: 編集・削除ボタンを右側に配置。左側は本文側の.doc-titleがタイトル表示を兼ねるため空要素で確保する 🔵 */}
      <div className="titlebar">
        <div aria-hidden="true" />
        <div className="actions">
          <Button asChild variant="outline" size="sm" className="btn">
            <Link to={`/items/${id}/edit`}>編集</Link>
          </Button>
          <Button variant="danger" size="sm" className="btn-danger" onClick={handleDeleteClick}>
            削除
          </Button>
        </div>
      </div>

      {/* 【ドキュメント本文】: カバー・タイトル・原題・概要 🔵 */}
      <article className="doc">
        <div
          className="doc-cover"
          style={item.coverImageUrl ? { backgroundImage: `url(${item.coverImageUrl})` } : undefined}
        />
        <h1 className="doc-title">{item.title}</h1>
        {item.originalTitle && <p className="doc-original">{item.originalTitle}</p>}
        {item.description && (
          <section className="doc-section">
            <h2>概要</h2>
            <p>{item.description}</p>
          </section>
        )}
      </article>

      <ConfirmDialog
        open={open}
        title="アイテムを削除しますか？"
        description="この操作は取り消せません。"
        onConfirm={handleConfirm}
        onCancel={handleCancel}
      />
    </div>
  );
}
