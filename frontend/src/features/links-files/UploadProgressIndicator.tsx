interface UploadProgressIndicatorProps {
  progress: number;
}

export default function UploadProgressIndicator({ progress }: UploadProgressIndicatorProps) {
  return (
    <div data-testid="upload-progress-indicator" aria-label={`アップロード進捗: ${progress}%`}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <div
          role="progressbar"
          aria-valuenow={progress}
          aria-valuemin={0}
          aria-valuemax={100}
          style={{
            flex: 1,
            height: '8px',
            background: '#e5e7eb',
            borderRadius: '4px',
            overflow: 'hidden',
          }}
        >
          <div
            style={{
              width: `${progress}%`,
              height: '100%',
              background: '#3b82f6',
              transition: 'width 0.2s ease',
            }}
          />
        </div>
        <span style={{ fontSize: '12px', color: '#6b7280', minWidth: '36px' }}>{progress}%</span>
      </div>
    </div>
  );
}
