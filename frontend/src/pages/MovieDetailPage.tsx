import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import { FiCalendar, FiExternalLink, FiLink2, FiPlus } from "react-icons/fi";
import { toast } from "sonner";
import { DetailLayout, DetailMain, DetailRail } from "@/components/detail";
import { usePageChrome } from "@/components/layout/usePageChrome";
import { detailSectionMatrix } from "@/config/detailSections";
import { EmptyState, FavoriteToggle, RatingStars, StatusSwitcher } from "@/components/shared";
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
  const detail = useMovieDetailData(id);
  const pageChrome = useMemo(() => ({
    breadcrumbs: [
      { label: "一般メディア", to: "/media" },
      { label: "映画" },
    ],
    actions: id ? <Link className="btn btn-accent" to={`/media/${id}/edit`}>編集する</Link> : undefined,
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

  const resourceFooter = (
    <div className="filter-bar" style={{ marginTop: 10 }}>
      <button
        type="button"
        className="btn btn-ghost btn-sm"
        onClick={() => {
          const label = promptValue("リンク名を入力してください", "公式サイト");
          const url = promptValue("URL を入力してください", item.homepage_url ?? "https://");
          if (!label || !url) {
            return;
          }
          void runAction(() => detail.addLink(label, url), "リンクを追加しました。");
        }}
      >
        <FiPlus className="icon" />
        リンクを追加
      </button>
      <button
        type="button"
        className="btn btn-ghost btn-sm"
        onClick={() => {
          const label = promptValue("ファイル名を入力してください", "パンフレット画像");
          const path = promptValue("ファイルパスを入力してください");
          const fileType = promptValue("file_type を入力してください (pdf / image / other)", "image") as "pdf" | "image" | "other";
          if (!path || !fileType) {
            return;
          }
          void runAction(() => detail.addFile(path, label || undefined, fileType), "ファイルを追加しました。");
        }}
      >
        <FiPlus className="icon" />
        ファイルを追加
      </button>
      <button
        type="button"
        className="btn btn-ghost btn-sm"
        onClick={() => {
          const label = promptValue("トレーラー名を入力してください", "本予告編");
          const url = promptValue("トレーラー URL を入力してください", "https://");
          if (!url) {
            return;
          }
          void runAction(() => detail.addTrailer(url, label || undefined), "トレーラーを追加しました。");
        }}
      >
        <FiPlus className="icon" />
        トレーラーを追加
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

  return (
    <DetailLayout
      rail={(
        <DetailRail
          cover={<CoverImage src={item.cover_image_url} alt={item.title} />}
          title={item.title}
          originalTitle={detail.actionLabel}
          facts={[
            <StatusSwitcher
              key="status"
              value={item.status}
              labels={MOVIE_STATUS_LABELS}
              onChange={(status) => void runAction(() => detail.updateStatus(status), "ステータスを更新しました。")}
            />,
            <RatingStars key="rating" value={item.rating ?? 0} onChange={(rating) => void runAction(() => detail.updateRating(rating), "評価を更新しました。")} />,
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
          overview={detail.overview || "概要はまだ登録されていません。"}
          propertyList={detailSectionMatrix.movie.propertyList ? detail.propertyItems : undefined}
          staffList={detailSectionMatrix.movie.staffList ? staffList : undefined}
          relatedWorks={relatedWorks}
          streaming={detailSectionMatrix.movie.streaming ? streaming : undefined}
          resourceTabs={detail.resourceTabs}
          staffFooter={(
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              onClick={() => {
                const staffId = promptValue("スタッフ ID を入力してください");
                const role = promptValue("役職を入力してください", "監督");
                const characterName = promptValue("キャラクター名があれば入力してください");
                if (!staffId || !role) {
                  return;
                }
                void runAction(() => detail.addStaff(staffId, role, characterName || undefined), "スタッフを追加しました。");
              }}
            >
              <FiPlus className="icon" />
              スタッフを追加
            </button>
          )}
          relatedWorksFooter={(
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              onClick={() => {
                const relatedItemId = promptValue("関連作品の item_id を入力してください");
                const relationType = promptValue("relation_type を入力してください (reference / dlc)", "reference") as "reference" | "dlc";
                if (!relatedItemId || !relationType) {
                  return;
                }
                void runAction(() => detail.addRelation(relatedItemId, relationType), "関連作品を追加しました。");
              }}
            >
              <FiPlus className="icon" />
              関連作品を追加
            </button>
          )}
          streamingFooter={(
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              onClick={() => {
                const platform = promptValue("platform を入力してください (netflix / amazon_prime / disney_plus / dmm_tv / apple_tv)", "disney_plus") as Parameters<typeof detail.addStreamingLink>[0];
                const url = promptValue("配信 URL を入力してください", "https://");
                if (!platform || !url) {
                  return;
                }
                void runAction(() => detail.addStreamingLink(platform, url), "配信リンクを追加しました。");
              }}
            >
              <FiPlus className="icon" />
              配信サイトを追加
            </button>
          )}
          resourceFooter={resourceFooter}
        />
      )}
    />
  );
}
