import { useRef, useState } from 'react';
import { toast } from 'sonner';
import { useUploadItemFileMutation } from '@/api/links-files';
import type { FileType } from '@/types';
import UploadProgressIndicator from './UploadProgressIndicator';

interface FileUploadDropzoneProps {
  itemId: string;
}

function detectFileType(file: File): FileType {
  if (file.type === 'application/pdf') return 'pdf';
  if (file.type.startsWith('image/')) return 'image';
  return 'other';
}

export default function FileUploadDropzone({ itemId }: FileUploadDropzoneProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const [progress, setProgress] = useState<number | null>(null);

  const uploadMutation = useUploadItemFileMutation(itemId, (p) => setProgress(p));

  async function handleFiles(files: FileList | null) {
    if (!files || files.length === 0) return;
    const file = files[0];
    const fileType = detectFileType(file);

    setProgress(0);
    try {
      await uploadMutation.mutateAsync({ file, fileType });
      setProgress(null);
    } catch (err: unknown) {
      setProgress(null);
      const code = (err as { code?: string })?.code;
      if (code === 'FILE_STORAGE_WRITE_FAILED') {
        toast.error('ファイルの保存に失敗しました。ストレージに書き込めませんでした。');
      } else {
        toast.error('ファイルのアップロードに失敗しました。');
      }
    }
  }

  function onDragOver(e: React.DragEvent) {
    e.preventDefault();
    setIsDragOver(true);
  }

  function onDragLeave() {
    setIsDragOver(false);
  }

  function onDrop(e: React.DragEvent) {
    e.preventDefault();
    setIsDragOver(false);
    handleFiles(e.dataTransfer.files);
  }

  function onClick() {
    inputRef.current?.click();
  }

  return (
    <div data-testid="file-upload-dropzone">
      <div
        role="button"
        tabIndex={0}
        aria-label="ファイルをドロップまたはクリックして選択"
        onDragOver={onDragOver}
        onDragLeave={onDragLeave}
        onDrop={onDrop}
        onClick={onClick}
        onKeyDown={(e) => e.key === 'Enter' && onClick()}
        style={{
          border: `2px dashed ${isDragOver ? '#3b82f6' : '#d1d5db'}`,
          borderRadius: '8px',
          padding: '24px',
          textAlign: 'center',
          cursor: 'pointer',
          background: isDragOver ? '#eff6ff' : 'transparent',
          transition: 'border-color 0.15s, background 0.15s',
        }}
      >
        <p style={{ margin: 0, color: '#6b7280', fontSize: '14px' }}>
          ここにファイルをドロップ、またはクリックして選択
        </p>
        <input
          ref={inputRef}
          type="file"
          style={{ display: 'none' }}
          aria-hidden="true"
          onChange={(e) => handleFiles(e.target.files)}
          data-testid="file-input"
        />
      </div>
      {progress !== null && <UploadProgressIndicator progress={progress} />}
    </div>
  );
}
