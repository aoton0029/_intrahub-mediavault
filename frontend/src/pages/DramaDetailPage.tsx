import { useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { FiCalendar, FiExternalLink, FiLink2, FiPlus } from "react-icons/fi";
import { toast } from "sonner";
import { DetailLayout, DetailMain, DetailRail } from "@/components/detail";
import { usePageChrome } from "@/components/layout/usePageChrome";
import { detailSectionMatrix } from "@/config/detailSections";
import { CastAddModal, CitationFormModal, ConfirmDialog, ConsumedDateEditor, EmptyState, FavoriteToggle, InlineAddForm, RatingStars, RelatedItemSearchModal, StaffAddModal, StatusSwitcher, StreamingLinkAddModal, FileAddModal, ThemeSongAddModal, TrailerAddModal, LinkAddModal } from "@/components/shared";
import { useDramaDetailData } from "@/hooks/useDramaDetailData";

function CoverImage({ src, alt }: { src: string | null | undefined; alt: string }) {
  if (!src) {
    return <div className="doc-cover" />;
  }

  return <img className="doc-cover" src={src} alt={alt} />;
}

export function DramaDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const detail = useDramaDetailData(id);
  const [isRelatedModalOpen, setRelatedModalOpen] = useState(false);
  const [isStaffModalOpen, setStaffModalOpen] = useState(false);
  const [isCastModalOpen, setCastModalOpen] = useState(false);
  const [isThemeSongModalOpen, setThemeSongModalOpen] = useState(false);
  const [isStreamingModalOpen, setStreamingModalOpen] = useState(false);
  const [isFileModalOpen, setFileModalOpen] = useState(false);
  const [isTrailerModalOpen, setTrailerModalOpen] = useState(false);
  const [isLinkModalOpen, setLinkModalOpen] = useState(false);
  const [isCitationModalOpen, setCitationModalOpen] = useState(false);
  const [editingCitation, setEditingCitation] = useState<(typeof detail.citations)[number] | null>(null);
  const [deletingCitationId, setDeletingCitationId] = useState<string | null>(null);

  async function handleDelete() {
    if (!id) return;
    if (!window.confirm("この作品を削除しますか？この操作は取り消せません。")) return;
    try {
      await detail.deleteItem();
      toast.success("作品を削除しました。");
      navigate("/media");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "削除に失敗しました。");
    }
  }

  const pageChrome = useMemo(() => ({
    breadcrumbs: [
      { label: "メディア", to: "/media" },
      { label: "ドラマ" },
    ],
    actions: id ? (
      <div style={{ display: "flex", gap: 8 }}>
        <Link className="btn btn-accent" to={`/media/${id}/edit`}>編集する</Link>
        <button type="button" className="btn btn-danger" onClick={() => void handleDelete()}>削除する</button>
      </div>
    ) : undefined,
  }), [id]);
  usePageChrome(pageChrome);

  if (!id) {
    return <EmptyState title="作品IDが見つかりません" description="URL を確認してからもう一度開いてください。" />;
  }

  if (detail.isLoading) {
    return <EmptyState title="読み込み中です" description="ドラマ詳細を取得しています。" />;
  }

  if (detail.isError || !detail.item) {
    return <EmptyState title="詳細を読み込めませんでした" description="時間をおいて再読み込みしてください。" />;
  }

  const item = detail.item;

  async function runAction(action: () => Promise<unknown>, successMessage: string) {
    try {
      await action();
      toast.success(successMessage);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "操作に失敗しました。");
    }
  }

  const relatedWorks = detail.relatedWorks.map((relation) => ({
    ...relation,
    onRemove: (relationId: string) => {
      void runAction(() => detail.removeRelation(relationId), "関連作品を解除しました。");
    },
  }));

  const staffList = detail.staffList.map((staff) => ({
    ...staff,
    actionLabel: "解除",
    onAction: (staffId: string) => {
      void runAction(() => detail.removeStaff(staffId), "スタッフを解除しました。");
    },
  }));

  const castList = detail.castList.map((cast) => ({
    ...cast,
    actionLabel: "解除",
    onAction: (castId: string) => {
      void runAction(() => detail.removeCast(castId), "キャストを解除しました。");
    },
  }));

  const themeSongs = detail.themeSongs.map((group) => ({
    ...group,
    songs: group.songs.map((song) => ({
      ...song,
      actionLabel: "解除",
      onAction: (itemThemeSongId: string) => {
        void runAction(() => detail.removeThemeSong(itemThemeSongId), "テーマソングを解除しました。");
      },
    })),
  }));

  const streaming = detail.streaming.map((link) => ({
    ...link,
    actionLabel: "削除",
    onAction: (linkId: string) => {
      void runAction(() => detail.removeStreamingLink(linkId), "配信リンクを削除しました。");
    },
  }));

  const images = detail.images.map((image) => ({
    ...image,
    onSetCover: (url: string) => void runAction(() => detail.setCoverImage(url), "サムネイルを設定しました。"),
    onRemove: (imageId: string) => void runAction(() => detail.removeImage(imageId), "画像を削除しました。"),
  }));

  const resourceTabs = {
    links: detail.resourceTabs.links?.map((entry) => ({
      ...entry,
      onRemove: (linkId: string) => void runAction(() => detail.removeLink(linkId), "リンクを削除しました。"),
    })),
    files: detail.resourceTabs.files?.map((entry) => ({
      ...entry,
      onRemove: (fileId: string) => void runAction(() => detail.removeFile(fileId), "ファイルを削除しました。"),
    })),
    trailers: detail.resourceTabs.trailers?.map((entry) => ({
      ...entry,
      onRemove: (trailerId: string) => void runAction(() => detail.removeTrailer(trailerId), "トレーラーを削除しました。"),
    })),
  };

  const citations = detail.citations.map((citation) => ({
    id: citation.id,
    quoteText: citation.quote_text,
    note: citation.note,
    locatorType: citation.locator_type,
    pageNumber: citation.page_number,
    timestampSeconds: citation.timestamp_seconds,
    locationNumber: citation.location_number,
    chapter: citation.chapter,
    onEdit: (citationId: string) => {
      setEditingCitation(detail.citations.find((entry) => entry.id === citationId) ?? null);
    },
    onRemove: (citationId: string) => setDeletingCitationId(citationId),
  }));

  const citationsFooter = (
    <div className="filter-bar" style={{ marginTop: 10, gap: 8 }}>
      <button
        type="button"
        className="btn btn-ghost btn-sm"
        onClick={() => {
          setEditingCitation(null);
          setCitationModalOpen(true);
        }}
      >
        <FiPlus className="icon" />
        引用を追加
      </button>
    </div>
  );

  const linksFooter = (
    <div className="filter-bar" style={{ marginTop: 10, gap: 8 }}>
      <button type="button" className="btn btn-ghost btn-sm" onClick={() => setLinkModalOpen(true)}>
        <FiPlus className="icon" />
        リンクを追加
      </button>
    </div>
  );

  const filesFooter = (
    <div className="filter-bar" style={{ marginTop: 10, gap: 8 }}>
      <button type="button" className="btn btn-ghost btn-sm" onClick={() => setFileModalOpen(true)}>
        <FiPlus className="icon" />
        ファイルを追加
      </button>
      {detail.files.some((file) => file.file_type === "pdf") ? (
        <button
          type="button"
          className="btn btn-ghost btn-sm"
          onClick={() => toast.info("Calibre 連携フロー本体は別タスクで実装予定です。")}
        >
          <FiLink2 className="icon" />
          Calibre に連携
        </button>
      ) : null}
    </div>
  );

  const trailersFooter = (
    <div className="filter-bar" style={{ marginTop: 10, gap: 8 }}>
      <button type="button" className="btn btn-ghost btn-sm" onClick={() => setTrailerModalOpen(true)}>
        <FiPlus className="icon" />
        トレーラーを追加
      </button>
    </div>
  );

  return (
    <>
    <DetailLayout
      rail={(
        <DetailRail
          cover={<CoverImage src={item.cover_image_url} alt={item.title} />}
          title={item.original_title || item.title}
          originalTitle={detail.actionLabel}
          facts={[
            <StatusSwitcher key="status" value={item.status} onChange={(status) => void runAction(() => detail.updateStatus(status), "ステータスを更新しました。")} />,
            <RatingStars key="rating" value={item.rating ?? 0} onChange={(rating) => void runAction(() => detail.updateRating(rating), "評価を更新しました。")} />,
            <ConsumedDateEditor
              key="consumed-date"
              value={item.consumed_date}
              onChange={(date) => void runAction(() => detail.updateConsumedDate(date), "視聴日を更新しました。")}
            />,
            <FavoriteToggle key="favorite" value={item.is_favorite} onChange={(value) => void runAction(() => detail.updateFavorite(value), "お気に入りを更新しました。")} />,
            <span key="release" className="meta-item">
              <FiCalendar className="icon" />
              {item.release_date ?? "公開日未登録"}
            </span>,
            <span key="source" className="meta-item muted">
              <FiExternalLink className="icon" />
              {item.source === "manual" ? "手動登録" : `API(TMDb) / external_id: ${item.external_id ?? "未設定"}`}
            </span>,
          ]}
          tags={detail.tags}
          categories={detail.categories}
          mylists={detail.mylists.map((mylist) => ({ id: mylist.id, label: mylist.name, actionLabel: "解除" }))}
          onAddTag={(name) => void runAction(() => detail.addTag(name), "タグを追加しました。")}
          onRemoveTag={(tagId) => void runAction(() => detail.removeTag(tagId), "タグを削除しました。")}
          onAddCategory={(name) => void runAction(() => detail.addCategory(name), "カテゴリを追加しました。")}
          onRemoveCategory={(categoryId) => void runAction(() => detail.removeCategory(categoryId), "カテゴリを削除しました。")}
          onRemoveMylist={(mylistId) => void runAction(() => detail.removeMylist(mylistId), "マイリストから解除しました。")}
          mylistsFooter={(
            <Link className="btn btn-ghost btn-sm" to="/mylists" style={{ marginTop: 6 }}>
              <FiPlus className="icon" />
              マイリストに追加
            </Link>
          )}
        />
      )}
      main={(
        <DetailMain
          overview={detail.overview}
          onUpdateOverview={(value) => void runAction(() => detail.updateDescription(value), "概要を更新しました。")}
          propertyList={detailSectionMatrix.drama.propertyList ? detail.propertyItems : undefined}
          groups={detailSectionMatrix.drama.groupList ? detail.groups : undefined}
          groupTitle="シーズン構成"
          groupActions={(group) => (
            <InlineAddForm
              triggerLabel="話数を追加"
              fields={[
                { name: "episodeNumber", placeholder: "話数番号", type: "number", defaultValue: String(group.episodes.length + 1) },
                { name: "title", placeholder: "話数タイトル" },
              ]}
              onSubmit={(values) => {
                const episodeNumber = Number(values.episodeNumber);
                if (!Number.isFinite(episodeNumber)) {
                  toast.error("話数番号は数値で入力してください。");
                  return;
                }
                void runAction(() => detail.addEpisode(group.id, episodeNumber, values.title || undefined), "話数を追加しました。");
              }}
            />
          )}
          groupFooter={(
            <InlineAddForm
              triggerLabel="シーズンを追加"
              fields={[{ name: "name", placeholder: "シーズン名", defaultValue: `シーズン${detail.groups.length + 1}` }]}
              onSubmit={(values) => {
                if (!values.name) return;
                void runAction(() => detail.addGroup(values.name, detail.groups.length + 1), "シーズンを追加しました。");
              }}
            />
          )}
          staffList={detailSectionMatrix.drama.staffList ? staffList : undefined}
          staffFooter={(
            <button type="button" className="btn btn-ghost btn-sm" style={{ marginTop: 10 }} onClick={() => setStaffModalOpen(true)}>
              スタッフを追加
            </button>
          )}
          castList={detailSectionMatrix.drama.castList ? castList : undefined}
          themeSongs={detailSectionMatrix.drama.themeSongs ? themeSongs : undefined}
          castFooter={(
            <button type="button" className="btn btn-ghost btn-sm" style={{ marginTop: 10 }} onClick={() => setCastModalOpen(true)}>
              キャストを追加
            </button>
          )}
          relatedWorks={relatedWorks}
          themeSongsFooter={(
            <button type="button" className="btn btn-ghost btn-sm" onClick={() => setThemeSongModalOpen(true)}>
              <FiPlus className="icon" />
              テーマソングを追加
            </button>
          )}
          relatedWorksFooter={(
            <button type="button" className="btn btn-ghost btn-sm" onClick={() => setRelatedModalOpen(true)}>
              <FiPlus className="icon" />
              関連作品を追加
            </button>
          )}
          streaming={detailSectionMatrix.drama.streaming ? streaming : undefined}
          streamingFooter={(
            <button type="button" className="btn btn-ghost btn-sm" style={{ marginTop: 10 }} onClick={() => setStreamingModalOpen(true)}>
              <FiPlus className="icon" />
              配信サイトを追加
            </button>
          )}
          images={detailSectionMatrix.drama.images ? images : undefined}
          citations={citations}
          citationsFooter={citationsFooter}
          imagesFooter={(
            <InlineAddForm
              triggerLabel="画像URLを追加"
              fields={[{ name: "url", placeholder: "画像URL", defaultValue: "https://" }]}
              onSubmit={(values) => {
                if (!values.url) return;
                void runAction(() => detail.addImage(values.url), "画像を追加しました。");
              }}
            />
          )}
          resourceTabs={resourceTabs}
          linksFooter={linksFooter}
          filesFooter={filesFooter}
          trailersFooter={trailersFooter}
        />
      )}
    />
    {isRelatedModalOpen ? (
      <RelatedItemSearchModal
        open={isRelatedModalOpen}
        onClose={() => setRelatedModalOpen(false)}
        excludeItemIds={[id]}
        alreadyRelatedIds={detail.relatedWorks.map((relation) => relation.relatedItemId)}
        onSelect={async (itemId, relationType) => {
          await detail.addRelation(itemId, relationType);
          toast.success("関連作品を追加しました。");
        }}
      />
    ) : null}
    {isStaffModalOpen ? (
      <StaffAddModal
        open={isStaffModalOpen}
        onClose={() => setStaffModalOpen(false)}
        onLink={async (staffId, role, characterName) => {
          await detail.addStaff(staffId, role, characterName);
          toast.success("スタッフを追加しました。");
        }}
      />
    ) : null}
    {isThemeSongModalOpen ? (
      <ThemeSongAddModal
        open={isThemeSongModalOpen}
        onClose={() => setThemeSongModalOpen(false)}
        onLink={async (themeSongId, themeType, displayOrder) => {
          await detail.addThemeSong(themeSongId, themeType, displayOrder);
          toast.success("テーマソングを追加しました。");
        }}
      />
    ) : null}
    {isCastModalOpen ? (
      <CastAddModal
        open={isCastModalOpen}
        onClose={() => setCastModalOpen(false)}
        onLink={async (castId, characterName) => {
          await detail.addCast(castId, characterName);
          toast.success("キャストを追加しました。");
        }}
      />
    ) : null}
    {isStreamingModalOpen ? (
      <StreamingLinkAddModal
        open={isStreamingModalOpen}
        onClose={() => setStreamingModalOpen(false)}
        registeredPlatforms={detail.streaming.flatMap((link) => (link.platform ? [link.platform] : []))}
        onAdd={async (platform, url) => {
          await detail.addStreamingLink(platform, url);
          toast.success("配信リンクを追加しました。");
        }}
      />
    ) : null}
    {isFileModalOpen ? (
      <FileAddModal
        open={isFileModalOpen}
        onClose={() => setFileModalOpen(false)}
        onAddByPath={async (path, label) => {
          await detail.addFile(path, label);
          toast.success("ファイルを追加しました。");
        }}
        onUpload={async (file, label) => {
          await detail.uploadFile(file, label);
          toast.success("ファイルをアップロードしました。");
        }}
      />
    ) : null}
    {isTrailerModalOpen ? (
      <TrailerAddModal
        open={isTrailerModalOpen}
        onClose={() => setTrailerModalOpen(false)}
        onAdd={async (url, label) => {
          await detail.addTrailer(url, label);
          toast.success("トレーラーを追加しました。");
        }}
      />
    ) : null}
    {isLinkModalOpen ? (
      <LinkAddModal
        open={isLinkModalOpen}
        onClose={() => setLinkModalOpen(false)}
        defaultUrl={item.homepage_url ?? "https://"}
        onAdd={async (label, url) => {
          await detail.addLink(label, url);
          toast.success("リンクを追加しました。");
        }}
      />
    ) : null}
    {isCitationModalOpen || editingCitation ? (
      <CitationFormModal
        open
        onClose={() => {
          setCitationModalOpen(false);
          setEditingCitation(null);
        }}
        initial={editingCitation ? {
          quoteText: editingCitation.quote_text,
          note: editingCitation.note,
          locatorType: editingCitation.locator_type,
          pageNumber: editingCitation.page_number,
          timestampSeconds: editingCitation.timestamp_seconds,
          locationNumber: editingCitation.location_number,
          chapter: editingCitation.chapter,
        } : undefined}
        onSubmit={async (values) => {
          const payload = {
            quote_text: values.quoteText,
            note: values.note || undefined,
            locator_type: values.locatorType,
            page_number: values.pageNumber ?? undefined,
            timestamp_seconds: values.timestampSeconds ?? undefined,
            location_number: values.locationNumber ?? undefined,
            chapter: values.chapter || undefined,
          };
          if (editingCitation) {
            await detail.updateCitation(editingCitation.id, payload);
            toast.success("引用を更新しました。");
          } else {
            await detail.addCitation(payload);
            toast.success("引用を追加しました。");
          }
        }}
      />
    ) : null}
    {deletingCitationId ? (
      <ConfirmDialog
        open
        onClose={() => setDeletingCitationId(null)}
        title="引用を削除"
        message="この引用を削除しますか？この操作は取り消せません。"
        onConfirm={() => runAction(() => detail.removeCitation(deletingCitationId), "引用を削除しました。")}
      />
    ) : null}
    </>
  );
}
