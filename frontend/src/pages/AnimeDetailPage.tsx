import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import { FiCalendar, FiExternalLink, FiLink2, FiPlus } from "react-icons/fi";
import { toast } from "sonner";
import { DetailLayout, DetailMain, DetailRail } from "@/components/detail";
import { usePageChrome } from "@/components/layout/usePageChrome";
import { EmptyState, FavoriteToggle, InlineAddForm, RatingStars, StatusSwitcher } from "@/components/shared";
import { useAnimeDetailData } from "@/hooks/useAnimeDetailData";

function CoverImage({ src, alt }: { src: string | null | undefined; alt: string }) {
  if (!src) {
    return <div className="doc-cover" />;
  }

  return <img className="doc-cover" src={src} alt={alt} />;
}

export function AnimeDetailPage() {
  const { id } = useParams();
  const detail = useAnimeDetailData(id);
  const pageChrome = useMemo(() => ({
    breadcrumbs: [
      { label: "一般メディア", to: "/media" },
      { label: "アニメ" },
    ],
  }), []);
  usePageChrome(pageChrome);

  if (!id) {
    return <EmptyState title="作品IDが見つかりません" description="URL を確認してからもう一度開いてください。" />;
  }

  if (detail.isLoading) {
    return <EmptyState title="読み込み中です" description="アニメ詳細を取得しています。" />;
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

  const streaming = detail.streaming.map((link) => ({
    ...link,
    actionLabel: "削除",
    onAction: (linkId: string) => {
      void runAction(() => detail.removeStreamingLink(linkId), "配信リンクを削除しました。");
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

  const resourceFooter = (
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
      <InlineAddForm
        triggerLabel="ファイルを追加"
        fields={[
          { name: "path", placeholder: "ファイルパス" },
          { name: "label", placeholder: "ファイル名", defaultValue: "パンフレットPDF" },
          {
            name: "fileType",
            placeholder: "種別",
            type: "select",
            defaultValue: "pdf",
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
      <InlineAddForm
        triggerLabel="トレーラーを追加"
        fields={[
          { name: "url", placeholder: "トレーラー URL", defaultValue: "https://" },
          { name: "label", placeholder: "トレーラー名", defaultValue: "本予告編" },
        ]}
        onSubmit={(values) => {
          if (!values.url) return;
          void runAction(() => detail.addTrailer(values.url, values.label || undefined), "トレーラーを追加しました。");
        }}
      />
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

  return (
    <DetailLayout
      rail={(
        <DetailRail
          cover={<CoverImage src={item.cover_image_url} alt={item.title} />}
          title={item.title}
          originalTitle={detail.actionLabel}
          facts={[
            <StatusSwitcher key="status" value={item.status} onChange={(status) => void runAction(() => detail.updateStatus(status), "ステータスを更新しました。")} />,
            <RatingStars key="rating" value={item.rating ?? 0} onChange={(rating) => void runAction(() => detail.updateRating(rating), "評価を更新しました。")} />,
            <FavoriteToggle key="favorite" value={item.is_favorite} onChange={(value) => void runAction(() => detail.updateFavorite(value), "お気に入りを更新しました。")} />,
            <span key="release" className="meta-item">
              <FiCalendar className="icon" />
              {item.release_date ?? "公開日未登録"}
            </span>,
            <span key="source" className="meta-item muted">
              <FiExternalLink className="icon" />
              {item.source === "manual" ? "手動登録" : `API(Annict) / external_id: ${item.external_id ?? "未設定"}`}
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
          overview={detail.overview || "概要はまだ登録されていません。"}
          groups={detail.groups}
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
          staffList={staffList}
          staffFooter={(
            <InlineAddForm
              triggerLabel="スタッフを追加"
              fields={[
                { name: "staffId", placeholder: "スタッフ ID" },
                { name: "role", placeholder: "役職", defaultValue: "監督" },
                { name: "characterName", placeholder: "キャラクター名" },
              ]}
              onSubmit={(values) => {
                if (!values.staffId || !values.role) return;
                void runAction(
                  () => detail.addStaff(values.staffId, values.role, values.characterName || undefined),
                  "スタッフを追加しました。",
                );
              }}
            />
          )}
          relatedWorks={relatedWorks}
          relatedWorksFooter={(
            <InlineAddForm
              triggerLabel="関連作品を追加"
              fields={[
                { name: "relatedItemId", placeholder: "関連作品の item_id" },
                {
                  name: "relationType",
                  placeholder: "関係性",
                  type: "select",
                  defaultValue: "reference",
                  options: [
                    { value: "reference", label: "reference" },
                    { value: "dlc", label: "dlc" },
                  ],
                },
              ]}
              onSubmit={(values) => {
                if (!values.relatedItemId || !values.relationType) return;
                void runAction(
                  () => detail.addRelation(values.relatedItemId, values.relationType as "reference" | "dlc"),
                  "関連作品を追加しました。",
                );
              }}
            />
          )}
          streaming={streaming}
          streamingFooter={(
            <InlineAddForm
              triggerLabel="配信サイトを追加"
              fields={[
                {
                  name: "platform",
                  placeholder: "配信サービス",
                  type: "select",
                  defaultValue: "netflix",
                  options: [
                    { value: "netflix", label: "Netflix" },
                    { value: "amazon_prime", label: "Amazon Prime Video" },
                    { value: "disney_plus", label: "Disney+" },
                    { value: "dmm_tv", label: "DMM TV" },
                    { value: "apple_tv", label: "Apple TV" },
                  ],
                },
                { name: "url", placeholder: "配信 URL", defaultValue: "https://" },
              ]}
              onSubmit={(values) => {
                if (!values.platform || !values.url) return;
                void runAction(
                  () => detail.addStreamingLink(values.platform as Parameters<typeof detail.addStreamingLink>[0], values.url),
                  "配信リンクを追加しました。",
                );
              }}
            />
          )}
          resourceTabs={resourceTabs}
          resourceFooter={resourceFooter}
        />
      )}
    />
  );
}
