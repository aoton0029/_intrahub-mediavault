import { useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { FiCalendar, FiExternalLink, FiPlus } from "react-icons/fi";
import { toast } from "sonner";
import { DetailLayout, DetailMain, DetailRail } from "@/components/detail";
import { usePageChrome } from "@/components/layout/usePageChrome";
import { detailSectionMatrix } from "@/config/detailSections";
import { ConsumedDateEditor, EmptyState, FavoriteToggle, InlineAddForm, RatingStars, RelatedItemSearchModal, StatusSwitcher } from "@/components/shared";
import { useGameDetailData } from "@/hooks/useGameDetailData";

function CoverImage({ src, alt }: { src: string | null | undefined; alt: string }) {
  if (!src) {
    return <div className="doc-cover" />;
  }

  return <img className="doc-cover" src={src} alt={alt} />;
}

export function GameDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const detail = useGameDetailData(id);
  const [isRelatedModalOpen, setRelatedModalOpen] = useState(false);

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
      { label: "一般メディア", to: "/media" },
      { label: "ゲーム" },
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
    return <EmptyState title="読み込み中です" description="ゲーム詳細を取得しています。" />;
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

  const linksFooter = (
    <div className="filter-bar" style={{ marginTop: 10, gap: 8 }}>
      <InlineAddForm
        triggerLabel="リンクを追加"
        fields={[
          { name: "label", placeholder: "リンク名", defaultValue: "公式サイト" },
          { name: "url", placeholder: "URL", defaultValue: item.homepage_url ?? "https://" },
        ]}
        onSubmit={(values) => {
          if (!values.label || !values.url) return;
          void runAction(() => detail.addLink(values.label, values.url), "リンクを追加しました。");
        }}
      />
    </div>
  );

  const filesFooter = (
    <div className="filter-bar" style={{ marginTop: 10, gap: 8 }}>
      <InlineAddForm
        triggerLabel="ファイルを追加"
        fields={[
          { name: "path", placeholder: "ファイルパス" },
          { name: "label", placeholder: "ファイル名", defaultValue: "スクリーンショット" },
          {
            name: "fileType",
            placeholder: "種別",
            type: "select",
            defaultValue: "image",
            options: [
              { value: "pdf", label: "pdf" },
              { value: "image", label: "image" },
              { value: "other", label: "other" },
            ],
          },
        ]}
        onSubmit={(values) => {
          if (!values.path || !values.fileType) return;
          void runAction(
            () => detail.addFile(values.path, values.label || undefined, values.fileType as "pdf" | "image" | "other"),
            "ファイルを追加しました。",
          );
        }}
      />
    </div>
  );

  const trailersFooter = (
    <div className="filter-bar" style={{ marginTop: 10, gap: 8 }}>
      <InlineAddForm
        triggerLabel="トレーラーを追加"
        fields={[
          { name: "url", placeholder: "トレーラー URL", defaultValue: "https://" },
          { name: "label", placeholder: "トレーラー名", defaultValue: "トレーラー" },
        ]}
        onSubmit={(values) => {
          if (!values.url) return;
          void runAction(() => detail.addTrailer(values.url, values.label || undefined), "トレーラーを追加しました。");
        }}
      />
    </div>
  );

  return (
    <>
    <DetailLayout
      rail={(
        <DetailRail
          cover={<CoverImage src={item.cover_image_url} alt={item.title} />}
          title={item.title}
          originalTitle={detail.actionLabel}
          facts={[
            <StatusSwitcher key="status" value={item.status} onChange={(status) => void runAction(() => detail.updateStatus(status), "ステータスを更新しました。")} />,
            <RatingStars key="rating" value={item.rating ?? 0} onChange={(rating) => void runAction(() => detail.updateRating(rating), "評価を更新しました。")} />,
            <ConsumedDateEditor
              key="consumed-date"
              value={item.consumed_date}
              onChange={(date) => void runAction(() => detail.updateConsumedDate(date), "プレイ日を更新しました。")}
            />,
            <FavoriteToggle key="favorite" value={item.is_favorite} onChange={(value) => void runAction(() => detail.updateFavorite(value), "お気に入りを更新しました。")} />,
            <span key="release" className="meta-item">
              <FiCalendar className="icon" />
              {item.release_date ?? "発売日未登録"}
            </span>,
            <span key="source" className="meta-item muted">
              <FiExternalLink className="icon" />
              {item.source === "manual" ? "手動登録" : `API(Steam) / external_id: ${item.external_id ?? "未設定"}`}
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
          propertyList={detailSectionMatrix.game.propertyList ? detail.propertyItems : undefined}
          relatedWorks={relatedWorks}
          relatedWorksFooter={(
            <button type="button" className="btn btn-ghost btn-sm" onClick={() => setRelatedModalOpen(true)}>
              <FiPlus className="icon" />
              関連作品を追加
            </button>
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
    </>
  );
}
