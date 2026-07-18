import { useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { FiCalendar, FiExternalLink, FiLink2, FiPlus } from "react-icons/fi";
import { toast } from "sonner";
import { DetailLayout, DetailMain, DetailRail } from "@/components/detail";
import { usePageChrome } from "@/components/layout/usePageChrome";
import { detailSectionMatrix } from "@/config/detailSections";
import { CastAddModal, ConsumedDateEditor, EmptyState, FavoriteToggle, RatingStars, RelatedItemSearchModal, StaffAddModal, StatusSwitcher, StreamingLinkAddModal, FileAddModal, TrailerAddModal, LinkAddModal } from "@/components/shared";
import { useMovieDetailData } from "@/hooks/useMovieDetailData";

const MOVIE_STATUS_LABELS = {
  not_started: "未着手",
  in_progress: "視聴中",
  completed: "視聴済",
} as const;

function CoverImage({ src, alt }: { src: string | null | undefined; alt: string }) {
  if (!src) {
    return <div className="doc-cover" />;
  }

  return <img className="doc-cover" src={src} alt={alt} />;
}

export function MovieDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const detail = useMovieDetailData(id);
  const [isRelatedModalOpen, setRelatedModalOpen] = useState(false);
  const [isStaffModalOpen, setStaffModalOpen] = useState(false);
  const [isCastModalOpen, setCastModalOpen] = useState(false);
  const [isStreamingModalOpen, setStreamingModalOpen] = useState(false);
  const [isFileModalOpen, setFileModalOpen] = useState(false);
  const [isTrailerModalOpen, setTrailerModalOpen] = useState(false);
  const [isLinkModalOpen, setLinkModalOpen] = useState(false);

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
      { label: "映画" },
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
    return <EmptyState title="読み込み中です" description="映画詳細を取得しています。" />;
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

  function promptValue(message: string, defaultValue = "") {
    const value = window.prompt(message, defaultValue);
    return value?.trim() || "";
  }

  const relatedWorks = detail.relatedWorks.map((relation) => ({
    ...relation,
    onRemove: (relationId: string) => {
      if (!window.confirm("この関連作品を解除しますか？")) {
        return;
      }
      void runAction(() => detail.removeRelation(relationId), "関連作品を解除しました。");
    },
  }));

  const staffList = detail.staffList.map((staff) => ({
    ...staff,
    actionLabel: "解除",
    onAction: (staffId: string) => {
      if (!window.confirm("このスタッフを解除しますか？")) {
        return;
      }
      void runAction(() => detail.removeStaff(staffId), "スタッフを解除しました。");
    },
  }));

  const castList = detail.castList.map((cast) => ({
    ...cast,
    actionLabel: "解除",
    onAction: (castId: string) => {
      if (!window.confirm("このキャストを解除しますか？")) {
        return;
      }
      void runAction(() => detail.removeCast(castId), "キャストを解除しました。");
    },
  }));

  const streaming = detail.streaming.map((link) => ({
    ...link,
    actionLabel: "削除",
    onAction: (linkId: string) => {
      if (!window.confirm("この配信リンクを削除しますか？")) {
        return;
      }
      void runAction(() => detail.removeStreamingLink(linkId), "配信リンクを削除しました。");
    },
  }));

  const images = detail.images.map((image) => ({
    ...image,
    onSetCover: (url: string) => void runAction(() => detail.setCoverImage(url), "サムネイルを設定しました。"),
    onRemove: (imageId: string) => {
      if (!window.confirm("この画像を削除しますか？")) {
        return;
      }
      void runAction(() => detail.removeImage(imageId), "画像を削除しました。");
    },
  }));

  const linksFooter = (
    <div className="filter-bar" style={{ marginTop: 10 }}>
      <button type="button" className="btn btn-ghost btn-sm" onClick={() => setLinkModalOpen(true)}>
        <FiPlus className="icon" />
        リンクを追加
      </button>
    </div>
  );

  const filesFooter = (
    <div className="filter-bar" style={{ marginTop: 10 }}>
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
    <div className="filter-bar" style={{ marginTop: 10 }}>
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
            <StatusSwitcher
              key="status"
              value={item.status}
              labels={MOVIE_STATUS_LABELS}
              onChange={(status) => void runAction(() => detail.updateStatus(status), "ステータスを更新しました。")}
            />,
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
          propertyList={detailSectionMatrix.movie.propertyList ? detail.propertyItems : undefined}
          staffList={detailSectionMatrix.movie.staffList ? staffList : undefined}
          castList={detailSectionMatrix.movie.castList ? castList : undefined}
          relatedWorks={relatedWorks}
          streaming={detailSectionMatrix.movie.streaming ? streaming : undefined}
          images={detailSectionMatrix.movie.images ? images : undefined}
          resourceTabs={detail.resourceTabs}
          staffFooter={(
            <button type="button" className="btn btn-ghost btn-sm" onClick={() => setStaffModalOpen(true)}>
              <FiPlus className="icon" />
              スタッフを追加
            </button>
          )}
          castFooter={(
            <button type="button" className="btn btn-ghost btn-sm" onClick={() => setCastModalOpen(true)}>
              <FiPlus className="icon" />
              キャストを追加
            </button>
          )}
          relatedWorksFooter={(
            <button type="button" className="btn btn-ghost btn-sm" onClick={() => setRelatedModalOpen(true)}>
              <FiPlus className="icon" />
              関連作品を追加
            </button>
          )}
          streamingFooter={(
            <button type="button" className="btn btn-ghost btn-sm" onClick={() => setStreamingModalOpen(true)}>
              <FiPlus className="icon" />
              配信サイトを追加
            </button>
          )}
          imagesFooter={(
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              onClick={() => {
                const url = promptValue("画像URLを入力してください", "https://");
                if (!url) {
                  return;
                }
                void runAction(() => detail.addImage(url), "画像を追加しました。");
              }}
            >
              <FiPlus className="icon" />
              画像URLを追加
            </button>
          )}
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
    </>
  );
}
