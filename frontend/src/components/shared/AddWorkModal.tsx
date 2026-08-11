import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMutation } from "@tanstack/react-query";
import { FiKey, FiPlusCircle, FiRepeat, FiSearch } from "react-icons/fi";
import { Modal } from "./Modal";
import { EmptyState } from "./EmptyState";
import { MediaCard, type MediaCardProps } from "./MediaCard";
import { MediaTypeDropdown } from "./MediaTypeDropdown";
import { MediaSearchError, useMediaSearch, type SearchMediaType } from "@/hooks/useMediaSearch";
import { importItem, ImportItemError } from "@/pages/MediaSearchPage";

export type GeneralSearchMediaType = Exclude<SearchMediaType, "academic_book">;

const MEDIA_TYPE_OPTIONS: Array<{ value: GeneralSearchMediaType; label: string }> = [
  { value: "anime", label: "アニメ" },
  { value: "movie", label: "映画" },
  { value: "drama", label: "ドラマ" },
  { value: "manga", label: "漫画" },
  { value: "novel", label: "小説" },
  { value: "game", label: "ゲーム" },
];

const PROVIDER_LABELS: Record<SearchMediaType, string> = {
  anime: "Annict",
  movie: "TMDb",
  drama: "TMDb",
  manga: "楽天ブックス",
  novel: "楽天ブックス",
  game: "Steam",
  academic_book: "楽天ブックス",
};

export type AddWorkModalScope = "general" | "academic_book";

export function AddWorkModal({
  open,
  onClose,
  scope = "general",
  initialMediaType = "movie",
}: {
  open: boolean;
  onClose: () => void;
  scope?: AddWorkModalScope;
  /** Media type preselected in the type dropdown; ignored when scope is "academic_book". Defaults to 映画 (movie). */
  initialMediaType?: GeneralSearchMediaType;
}) {
  const navigate = useNavigate();
  const isAcademic = scope === "academic_book";
  const [stayMode, setStayMode] = useState(false);
  const [selectedMediaType, setSelectedMediaType] = useState<GeneralSearchMediaType>(initialMediaType);
  const [query, setQuery] = useState("");
  const [submittedMediaType, setSubmittedMediaType] = useState<SearchMediaType>(isAcademic ? "academic_book" : initialMediaType);
  const [importedIds, setImportedIds] = useState<Set<string>>(new Set());
  const searchMutation = useMediaSearch();
  const importMutation = useMutation({ mutationFn: importItem });

  // Re-seed the selected type from `initialMediaType` each time the modal opens, without an effect
  // (see https://react.dev/learn/you-might-not-need-an-effect#adjusting-some-state-when-a-prop-changes).
  const [wasOpen, setWasOpen] = useState(open);
  if (open !== wasOpen) {
    setWasOpen(open);
    if (open && !isAcademic) {
      setSelectedMediaType(initialMediaType);
    }
  }

  const effectiveMediaType: SearchMediaType = isAcademic ? "academic_book" : selectedMediaType;

  const searchError = searchMutation.error;
  const apiKeyMissing = searchError instanceof MediaSearchError && searchError.status === 422 && searchError.code === "API_KEY_NOT_CONFIGURED";
  const searchResults = apiKeyMissing ? [] : (searchMutation.data ?? []);

  function handleClose() {
    setQuery("");
    setImportedIds(new Set());
    searchMutation.reset();
    onClose();
  }

  function handleSearch() {
    setSubmittedMediaType(effectiveMediaType);
    searchMutation.mutate({ mediaType: effectiveMediaType, query });
  }

  async function handleImport(item: { id: string; provider: string | null; mediaType: SearchMediaType }) {
    try {
      const imported = await importMutation.mutateAsync({
        mediaType: item.mediaType,
        provider: item.provider,
        externalId: item.id,
      });
      setImportedIds((current) => new Set(current).add(item.id));
      if (!stayMode) {
        const detailPath = item.mediaType === "academic_book" ? "academic-books" : "media";
        navigate(`/${detailPath}/${imported.data.id}`);
        handleClose();
      }
    } catch (error) {
      if (error instanceof ImportItemError && error.status === 409 && error.code === "ITEM_ALREADY_IMPORTED") {
        setImportedIds((current) => new Set(current).add(item.id));
        return;
      }

      throw error;
    }
  }

  const mediaCards: MediaCardProps[] = searchResults.map((item) => ({
    title: item.title,
    badge: isAcademic ? "学術書" : (MEDIA_TYPE_OPTIONS.find((option) => option.value === item.media_type)?.label ?? item.media_type),
    meta: item.year ? String(item.year) : undefined,
    imageUrl: item.thumbnail_url,
    variant: "search-result",
    imported: importedIds.has(item.id),
    actionLabel: "取り込む",
    onAction: () => void handleImport({ id: item.id, provider: item.provider, mediaType: item.media_type }),
  }));

  return (
    <Modal open={open} onClose={handleClose} title="作品を追加" maxWidth="min(1400px, 94vw)" height="min(1000px, 90vh)">
      <div className="filter-bar" style={{ marginBottom: 16 }}>
        {isAcademic ? null : (
          <MediaTypeDropdown value={selectedMediaType} onChange={(value) => setSelectedMediaType(value as GeneralSearchMediaType)} />
        )}
        <label className="search-box" style={{ marginLeft: 0, flex: 1 }}>
          <FiSearch className="icon" aria-hidden="true" />
          <input
            aria-label="作品名"
            placeholder={isAcademic ? "タイトル・著者名で検索…" : "作品名で検索…"}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") handleSearch();
            }}
          />
        </label>
        <button type="button" className="btn btn-accent" onClick={handleSearch}>
          <FiSearch aria-hidden="true" />
          検索
        </button>
        <button
          type="button"
          className="media-type-chip"
          role="switch"
          aria-checked={stayMode}
          data-active={stayMode}
          title="取り込み後もこのモーダルを開いたままにする"
          onClick={() => setStayMode((current) => !current)}
        >
          <FiRepeat className="icon" aria-hidden="true" />
          <span className="label">連続取り込み</span>
        </button>
      </div>

      <div className="modal-scroll-area">
        {apiKeyMissing ? (
          <EmptyState
            title="APIキーが設定されていません"
            description={`この種別の検索には ${PROVIDER_LABELS[submittedMediaType]} のAPIキーが必要です。設定画面から登録してください。`}
            action={
              <button
                type="button"
                className="btn btn-accent btn-sm"
                style={{ marginTop: 12 }}
                onClick={() => {
                  handleClose();
                  navigate("/settings?tab=api");
                }}
              >
                <FiKey aria-hidden="true" />
                設定を開く
              </button>
            }
          />
        ) : (
          <>
            {searchMutation.isIdle ? (
              <EmptyState
                title="検索して作品を追加"
                description={isAcademic ? "タイトルや著者名で検索すると取り込み候補が表示されます。" : "種別を選び、作品名で検索すると取り込み候補が表示されます。"}
                action={
                  <div style={{ display: "inline-flex", alignItems: "center", gap: 8 }}>
                    <FiPlusCircle aria-hidden="true" />
                    <span>検索結果からワンクリックで追加できます。</span>
                  </div>
                }
              />
            ) : null}
            {searchMutation.isPending ? <p>検索中...</p> : null}
            {!searchMutation.isIdle && !searchMutation.isPending ? (
              <div className="card-grid is-dense">
                {mediaCards.map((item, index) => (
                  <MediaCard key={item.id ?? `${item.badge}-${item.title}-${index}`} {...item} thumbnailOrientation="vertical" />
                ))}
              </div>
            ) : null}
          </>
        )}
      </div>

      <div className="form-actions">
        <button type="button" className="btn btn-ghost" onClick={handleClose}>
          閉じる
        </button>
      </div>
    </Modal>
  );
}
