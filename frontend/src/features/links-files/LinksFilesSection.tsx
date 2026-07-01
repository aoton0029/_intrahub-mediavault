import { useItemFilesQuery } from '@/api/links-files';
import ItemLinksList from './ItemLinksList';
import ItemFilesList from './ItemFilesList';
import ItemTrailersList from './ItemTrailersList';

const CALIBRE_WEB_BASE_URL =
  import.meta.env.VITE_CALIBRE_WEB_BASE_URL ?? 'http://localhost:8083';

interface LinksFilesSectionProps {
  itemId: string;
}

export default function LinksFilesSection({ itemId }: LinksFilesSectionProps) {
  const { data: filesData } = useItemFilesQuery(itemId);
  const files = filesData?.data ?? [];

  // REQ-302: pdf+calibreBookId紐付け済みファイルが存在する場合にCalibre-Web遷移ボタンを表示
  const linkedPdf = files.find((f) => f.fileType === 'pdf' && f.calibreBookId);

  return (
    <div data-testid="links-files-section">
      {linkedPdf && (
        <a
          href={`${CALIBRE_WEB_BASE_URL}/book/${linkedPdf.calibreBookId}`}
          target="_blank"
          rel="noopener noreferrer"
          data-testid="calibre-web-button"
          aria-label="Calibre-Webで開く"
        >
          Calibre-Webで開く
        </a>
      )}
      <ItemLinksList itemId={itemId} />
      <ItemFilesList itemId={itemId} />
      <ItemTrailersList itemId={itemId} />
    </div>
  );
}
