import { useItemFilesQuery, useDeleteItemFileMutation } from '@/api/links-files';
import CalibreLinkBadge from './CalibreLinkBadge';
import FileUploadDropzone from './FileUploadDropzone';

interface ItemFilesListProps {
  itemId: string;
}

export default function ItemFilesList({ itemId }: ItemFilesListProps) {
  const { data, isLoading, isError } = useItemFilesQuery(itemId);
  const deleteMutation = useDeleteItemFileMutation(itemId);

  if (isLoading) return <div aria-live="polite">読み込み中...</div>;
  if (isError) return <div role="alert">ファイル情報の取得に失敗しました</div>;

  const files = data?.data ?? [];

  return (
    <div data-testid="item-files-list">
      <h4>ファイル</h4>
      <FileUploadDropzone itemId={itemId} />
      {files.length === 0 ? (
        <p>ファイルが登録されていません</p>
      ) : (
        <ul>
          {files.map((file) => (
            <li key={file.id} data-testid="file-item">
              <span>{file.label ?? file.path}</span>
              <span data-testid="file-type-badge">{file.fileType}</span>
              {file.fileType === 'pdf' && (
                <CalibreLinkBadge
                  status={file.calibreBookId ? 'linked' : 'unlinked'}
                  calibreBookId={file.calibreBookId}
                />
              )}
              <button
                onClick={() => deleteMutation.mutate(file.id)}
                disabled={deleteMutation.isPending}
                aria-label="ファイルを削除する"
                data-testid="file-delete-button"
              >
                削除
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
