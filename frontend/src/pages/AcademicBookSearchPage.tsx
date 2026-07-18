import { useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { useMutation } from "@tanstack/react-query";
import { FiKey, FiPlusCircle, FiRepeat, FiSearch } from "react-icons/fi";
import { EmptyState, MediaGrid, type MediaCardProps } from "@/components/shared";
import { MediaSearchError, useMediaSearch } from "@/hooks/useMediaSearch";
import { ImportItemError, importItem } from "./MediaSearchPage";

const MEDIA_TYPE: "academic_book" = "academic_book";
const BADGE_LABEL = "学術書";
const PROVIDER_LABEL = "楽天ブックス";

export function AcademicBookSearchPage() {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const stayMode = searchParams.get("stay") === "1";
  const [query, setQuery] = useState("");
  const [importedIds, setImportedIds] = useState<Set<string>>(new Set());
  const searchMutation = useMediaSearch();
  const importMutation = useMutation({
    mutationFn: importItem,
  });

  const searchError = searchMutation.error;
  const apiKeyMissing = searchError instanceof MediaSearchError && searchError.status === 422 && searchError.code === "API_KEY_NOT_CONFIGURED";

  const searchResults = apiKeyMissing ? [] : (searchMutation.data ?? []);

  const handleSearch = () => {
    searchMutation.mutate({ mediaType: MEDIA_TYPE, query });
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

  const handleImport = async (item: { id: string; provider: string | null }) => {
    try {
      const imported = await importMutation.mutateAsync({
        mediaType: MEDIA_TYPE,
        provider: item.provider,
        externalId: item.id,
      });
      setImportedIds((current) => new Set(current).add(item.id));
      if (!stayMode) {
        navigate(`/academic-books/${imported.data.id}`);
      }
    } catch (error) {
      if (error instanceof ImportItemError && error.status === 409 && error.code === "ITEM_ALREADY_IMPORTED") {
        setImportedIds((current) => new Set(current).add(item.id));
        return;
      }

      throw error;
    }
  };

  const mediaCards: Array<MediaCardProps & { id: string }> = searchResults.map((item) => ({
    id: item.id,
    title: item.title,
    badge: BADGE_LABEL,
    meta: item.year ? String(item.year) : undefined,
    imageUrl: item.thumbnail_url,
    variant: "search-result",
    imported: importedIds.has(item.id),
    actionLabel: "取り込む",
    onAction: () => void handleImport({ id: item.id, provider: item.provider }),
  }));

  return (
    <>
      <div className="filter-bar">
        <label className="search-box" style={{ marginLeft: 0, flex: 1 }}>
          <FiSearch className="icon" aria-hidden="true" />
          <input aria-label="作品名" placeholder="タイトル・著者名で検索…" value={query} onChange={(event) => setQuery(event.target.value)} />
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
          description={`学術書・専門書の検索には ${PROVIDER_LABEL} のAPIキーが必要です。設定画面から登録してください。`}
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
              description="タイトルや著者名で検索すると取り込み候補が表示されます。"
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
