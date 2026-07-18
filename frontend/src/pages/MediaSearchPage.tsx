import { useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { useMutation } from "@tanstack/react-query";
import { FiKey, FiPlusCircle, FiRepeat, FiSearch } from "react-icons/fi";
import { EmptyState, MediaGrid, MediaTypeDropdown, type MediaCardProps } from "@/components/shared";
import { MediaSearchError, useMediaSearch, type SearchMediaType } from "@/hooks/useMediaSearch";
import { apiFetch } from "@/lib/apiClient";

const MEDIA_TYPE_OPTIONS: Array<{ value: SearchMediaType; label: string }> = [
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
};

type ImportResponse = {
  success: boolean;
  data: {
    id: string;
  };
};

type ImportApiErrorResponse = {
  code?: string;
  error?: {
    code?: string;
    message?: string;
  };
  message?: string;
};

class ImportItemError extends Error {
  status: number;
  code?: string;

  constructor(message: string, status: number, code?: string) {
    super(message);
    this.name = "ImportItemError";
    this.status = status;
    this.code = code;
  }
}

async function importItem({ mediaType, provider, externalId }: { mediaType: SearchMediaType; provider: string | null; externalId: string }) {
  const response = await apiFetch("/items/import", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      media_type: mediaType,
      provider,
      external_id: externalId,
    }),
  });

  if (!response.ok) {
    let code: string | undefined;
    let message = `Failed to import item: ${response.status}`;

    try {
      const json = (await response.json()) as ImportApiErrorResponse;
      code = json.code ?? json.error?.code;
      message = json.error?.message ?? json.message ?? message;
    } catch {
      // Ignore JSON parse errors and fall back to the default message.
    }

    throw new ImportItemError(message, response.status, code);
  }

  return (await response.json()) as ImportResponse;
}

export function MediaSearchPage() {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const stayMode = searchParams.get("stay") === "1";
  const [selectedMediaType, setSelectedMediaType] = useState<SearchMediaType>("anime");
  const [query, setQuery] = useState("");
  const [submittedMediaType, setSubmittedMediaType] = useState<SearchMediaType>("anime");
  const [importedIds, setImportedIds] = useState<Set<string>>(new Set());
  const searchMutation = useMediaSearch();
  const importMutation = useMutation({
    mutationFn: importItem,
  });

  const searchError = searchMutation.error;
  const apiKeyMissing = searchError instanceof MediaSearchError && searchError.status === 422 && searchError.code === "API_KEY_NOT_CONFIGURED";

  const searchResults = apiKeyMissing ? [] : (searchMutation.data ?? []);

  const handleSearch = () => {
    setSubmittedMediaType(selectedMediaType);
    searchMutation.mutate({ mediaType: selectedMediaType, query });
  };

  const handleToggleStayMode = () => {
    setSearchParams((params) => {
      const next = new URLSearchParams(params);
      if (stayMode) {
        next.delete("stay");
      } else {
        next.set("stay", "1");
      }
      return next;
    });
  };

  const handleImport = async (item: MediaCardProps & { id: string; provider: string | null; mediaType: SearchMediaType }) => {
    try {
      const imported = await importMutation.mutateAsync({
        mediaType: item.mediaType,
        provider: item.provider,
        externalId: item.id,
      });
      setImportedIds((current) => new Set(current).add(item.id));
      if (!stayMode) {
        navigate(`/media/${imported.data.id}`);
      }
    } catch (error) {
      if (error instanceof ImportItemError && error.status === 409 && error.code === "ITEM_ALREADY_IMPORTED") {
        setImportedIds((current) => new Set(current).add(item.id));
        return;
      }

      throw error;
    }
  };

  const mediaCards: Array<MediaCardProps & { id: string; provider: string | null; mediaType: SearchMediaType }> = searchResults.map((item) => ({
    id: item.id,
    provider: item.provider,
    mediaType: item.media_type,
    title: item.title,
    badge: MEDIA_TYPE_OPTIONS.find((option) => option.value === item.media_type)?.label ?? item.media_type,
    meta: item.year ? String(item.year) : undefined,
    imageUrl: item.thumbnail_url,
    variant: "search-result",
    imported: importedIds.has(item.id),
    actionLabel: "取り込む",
    onAction: () =>
      void handleImport({
        id: item.id,
        provider: item.provider,
        mediaType: item.media_type,
        title: item.title,
        badge: MEDIA_TYPE_OPTIONS.find((option) => option.value === item.media_type)?.label ?? item.media_type,
        meta: item.year ? String(item.year) : undefined,
        imageUrl: item.thumbnail_url,
        variant: "search-result",
        imported: importedIds.has(item.id),
      }),
  }));

  return (
    <>
      <div className="filter-bar">
        <MediaTypeDropdown value={selectedMediaType} onChange={(value) => setSelectedMediaType(value as SearchMediaType)} />
        <label className="search-box" style={{ marginLeft: 0, flex: 1 }}>
          <FiSearch className="icon" aria-hidden="true" />
          <input aria-label="作品名" placeholder="作品名で検索…" value={query} onChange={(event) => setQuery(event.target.value)} />
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
          title="取り込み後もこのページに留まる"
          onClick={handleToggleStayMode}
        >
          <FiRepeat className="icon" aria-hidden="true" />
          <span className="label">連続取り込み</span>
        </button>
      </div>

      {apiKeyMissing ? (
        <EmptyState
          title="APIキーが設定されていません"
          description={`この種別の検索には ${PROVIDER_LABELS[submittedMediaType]} のAPIキーが必要です。設定画面から登録してください。`}
          action={
            <Link className="btn btn-accent btn-sm" style={{ marginTop: 12 }} to="/settings?tab=api">
              <FiKey aria-hidden="true" />
              設定を開く
            </Link>
          }
        />
      ) : (
        <>
          {searchMutation.isIdle ? (
            <EmptyState
              title="検索して作品を追加"
              description="種別を選び、作品名で検索すると取り込み候補が表示されます。"
              action={
                <div style={{ display: "inline-flex", alignItems: "center", gap: 8 }}>
                  <FiPlusCircle aria-hidden="true" />
                  <span>検索結果からワンクリックで追加できます。</span>
                </div>
              }
            />
          ) : null}
          {searchMutation.isPending ? <p>検索中...</p> : null}
          {!searchMutation.isIdle && !searchMutation.isPending ? <MediaGrid items={mediaCards} density="compact" /> : null}
        </>
      )}
    </>
  );
}
