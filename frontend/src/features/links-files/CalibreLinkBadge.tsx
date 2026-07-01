const CALIBRE_WEB_BASE_URL =
  import.meta.env.VITE_CALIBRE_WEB_BASE_URL ?? 'http://localhost:8083';

interface CalibreLinkBadgeProps {
  status: 'linked' | 'unlinked';
  calibreBookId?: string;
}

export default function CalibreLinkBadge({ status, calibreBookId }: CalibreLinkBadgeProps) {
  if (status === 'linked' && calibreBookId) {
    const url = `${CALIBRE_WEB_BASE_URL}/book/${calibreBookId}`;
    return (
      <a
        href={url}
        target="_blank"
        rel="noopener noreferrer"
        data-testid="calibre-link"
        aria-label="Calibre-Webで閲覧する"
      >
        Calibre-Webで閲覧
      </a>
    );
  }

  return (
    <span data-testid="calibre-unlinked-badge" aria-label="Calibre未連携">
      未連携
    </span>
  );
}
