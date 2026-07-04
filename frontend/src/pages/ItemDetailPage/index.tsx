import { useState } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { useItemDetailQuery, useUpdateItemStatus, useDeleteItem } from '@/features/items/hooks'
import { PropertiesPanel } from '@/components/PropertiesPanel'
import { SeasonEpisodeList } from '@/components/SeasonEpisodeList'
import { ItemMediaLinks } from '@/components/ItemMediaLinks'
import { ItemMetaSections } from '@/components/ItemMetaSections'
import { DeleteConfirmDialog } from '@/components/DeleteConfirmDialog'
import type { ApiClientError } from '@/lib/apiClient'
import type { ItemGroup, MediaType } from '@/features/items/types'

/**
 * 【機能概要】: メディア種別コードを日本語のパンくずラベルへ変換する
 * 【実装方針】: 02_item_detail.htmlのパンくず表示（例:「アニメ / 星屑のシンフォニア」）に合わせた最小限のマッピング
 * 【テスト対応】: TC-N-05（パンくずのaria-label・タイトル包含確認）
 * 🟡 信頼性レベル: 02_item_detail.htmlの表示例からの妥当な推測（全media_type分のラベルは要件に明記なし）
 */
const MEDIA_TYPE_LABELS: Record<MediaType, string> = {
  anime: 'アニメ',
  movie: '映画',
  drama: 'ドラマ',
  manga: '漫画',
  novel: '小説',
  game: 'ゲーム',
  academic_book: '学術書・専門書',
  paper: '論文・文献',
}

// 【エラーメッセージ定数】: 404/汎用エラーの文言を一箇所に集約し、表示側とテストの参照元をぶらさない 🔵
// 【調整可能性】: 文言変更時はここのみ修正すればよい（テストは testcases.md の期待文言と一致させる）
const ERROR_MESSAGES = {
  notFound: 'アイテムが見つかりません',
  generic: 'エラーが発生しました',
} as const

/**
 * 【ヘルパー関数】: ロード中に表示するレイアウト骨格スケルトンを描画する
 * 【再利用性】: ItemDetailPage内のisLoading分岐から呼び出される専用パーツ
 * 【単一責任】: 「ロード中の見た目」のみを担当し、データ取得・エラー処理には関与しない
 * 🟡 信頼性レベル: UI/UX要件（スケルトン）は妥当な推測、role="status"の検証手法はHomePage.test.tsx既存パターンに準拠
 */
function ItemDetailSkeleton() {
  return (
    <main className="main" role="status" aria-label="読み込み中">
      <div className="titlebar">
        <div className="doc-title-skeleton" />
      </div>
      <div className="content">
        <div className="doc-cover" />
        <div className="doc-title-skeleton" />
      </div>
    </main>
  )
}

/**
 * 【ヘルパー関数】: 404（ITEM_NOT_FOUND）／汎用エラーの2種類のエラー画面を描画する
 * 【再利用性】: isError分岐から呼び出され、エラー種別ごとの文言・導線切り替えを一箇所に集約する
 * 【単一責任】: 「エラー時の表示」のみを担当し、データ取得ロジックには関与しない
 * 🔵 信頼性レベル: TC-004-E01・items.md 404・dataflow.mdより
 */
function ItemDetailErrorView({
  error,
  onRetry,
}: {
  error: ApiClientError | null | undefined
  onRetry: () => void
}) {
  // 【分岐条件】: ITEM_NOT_FOUNDは専用画面（一覧へ戻る導線）、それ以外は汎用エラー+再試行ボタン 🔵
  if (error?.code === 'ITEM_NOT_FOUND') {
    return (
      <main className="main">
        <p>{ERROR_MESSAGES.notFound}</p>
        <Link to="/">一覧へ戻る</Link>
      </main>
    )
  }

  return (
    <main className="main">
      <p>{ERROR_MESSAGES.generic}</p>
      <button type="button" className="btn btn-sm" onClick={onRetry}>
        再試行
      </button>
    </main>
  )
}

/**
 * 【機能概要】: 詳細画面（ItemDetailPage）本体
 * 【改善内容】: ロード中スケルトンとエラー画面をそれぞれ専用の小コンポーネントへ分離し、
 * 本体コンポーネントの責務を「3状態の分岐制御＋データ表示時の統合」に絞った
 * 【設計方針】: 02_item_detail.htmlのレイアウト（.app-shell.has-properties省略形として.titlebar/.content/.properties）に
 * 沿ってPropertiesPanel/SeasonEpisodeList/ItemMediaLinks/ItemMetaSectionsを統合する
 * 【保守性】: エラー文言はERROR_MESSAGESに集約し、スケルトン/エラー表示は独立部品化したことでテスト対象箇所の特定が容易
 * 【テスト対応】: TASK-0020 Red フェーズの11テストケース全てを通すための実装（Refactor後も同一挙動を維持）
 * 🔵 信頼性レベル: red-phase.md「Greenフェーズで実装すべき内容」・requirements.mdより
 */
export interface ItemDetailPageProps {
  /**
   * 【シーズン構成seam】: ItemDetail型にはgroupsがネストされないため、テスト/将来のデータ供給用に
   * ページコンポーネントの外部からシーズングループを注入できるseam propとして公開する 🟡
   */
  seasonGroups?: ItemGroup[]
}

export default function ItemDetailPage({ seasonGroups = [] }: ItemDetailPageProps = {}) {
  // 【入力値取得】: URLパスパラメータからidを取得する 🔵
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()

  // 【データ取得】: id===''の場合はuseItemDetailQuery内部のenabled:falseによりフェッチされない 🔵
  const { data, isLoading, isError, error, refetch } = useItemDetailQuery(id ?? '')

  // 【ステータス更新配線】: onStatusChangeはmutateを呼ぶのみとし、API連携の本実装はTASK-0021の責務とする 🟡
  const { mutate: updateStatus, isPending: isStatusUpdating } = useUpdateItemStatus()

  // 【削除ダイアログ開閉状態】: 初期状態はfalse（非表示）とし、削除ボタン押下でtrueに切り替える 🔵
  // 【テスト対応】: TC-006-B02（初期非表示）、TC-006-01/CANCEL（開閉制御）
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)

  // 【削除処理配線】: useDeleteItemからmutate/isPendingを取得し、確定ボタン押下時にidを渡してmutateする 🔵
  // 【命名方針】: updateStatus（status変更）と対になる命名としてdeleteItemに統一し、呼び出し側の可読性を揃える
  // 【テスト対応】: TC-006-01（成功時navigate）、TC-006-E01/E02（エラートースト）、TC-006-B01（二重送信防止）
  const { mutate: deleteItem, isPending: isDeleting } = useDeleteItem()

  // 【ローディング状態】: role="status"のスケルトンのみ表示し、本体・エラー文言は出さない 🟡
  if (isLoading) {
    return <ItemDetailSkeleton />
  }

  // 【エラー状態】: ITEM_NOT_FOUNDは専用画面、それ以外は汎用エラー+再試行ボタン 🔵
  if (isError) {
    return <ItemDetailErrorView error={error} onRetry={() => refetch()} />
  }

  // 【データ未到達時の安全表示】: idが空文字等でdataが無い場合、クラッシュせず何も描画しない 🔵
  const item = data?.data
  if (!item) {
    return <main className="main" />
  }

  const mediaTypeLabel = MEDIA_TYPE_LABELS[item.media_type] ?? item.media_type

  return (
    <div className="app-shell has-properties">
      <main className="main">
        <div className="titlebar">
          <div>
            {/* 【パンくず】: aria-label="パンくずリスト"のnavにmedia_typeラベルとタイトルを含める 🟡 */}
            <nav aria-label="パンくずリスト" className="breadcrumb">
              {mediaTypeLabel} <span>/ {item.title}</span>
            </nav>
            <h1>{item.title}</h1>
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            {/* 【編集ボタン】: 編集画面へnavigateするのみ。遷移先ルートは別要件で未確定のため404の可能性あり 🟡 */}
            <button
              type="button"
              className="btn btn-sm"
              onClick={() => navigate(`/items/${item.id}/edit`)}
            >
              編集
            </button>
            {/* 【削除ボタン】: クリックでダイアログを開く。実削除処理はDeleteConfirmDialogのonConfirmで行う 🔵 */}
            <button
              type="button"
              className="btn btn-sm btn-danger"
              onClick={() => setDeleteDialogOpen(true)}
            >
              削除
            </button>
          </div>
        </div>

        <div className="content">
          <div className="doc-cover">
            {!item.cover_image_url && <span className="doc-cover-placeholder" aria-hidden="true" />}
          </div>
          {item.original_title && (
            <div className="doc-original">
              原題: <span>{item.original_title}</span>
            </div>
          )}

          {item.description && (
            <div className="doc-section">
              <h3>概要</h3>
              <p>{item.description}</p>
            </div>
          )}

          {/* 【シーズン構成seam】: ItemDetailにはgroupsがネストされないため、seasonGroups propから渡す 🟡 */}
          <SeasonEpisodeList mediaType={item.media_type} groups={seasonGroups} />

          {/* 【リンク・ファイル・トレーラーseam】: ItemDetailにはネストされないため空配列を渡す 🟡 */}
          <ItemMediaLinks links={[]} files={[]} trailers={[]} />
        </div>
      </main>

      <PropertiesPanel
        item={item}
        onStatusChange={(next) =>
          updateStatus({ id: item.id, body: { status: next.status, consumed_date: next.consumedDate } })
        }
        isStatusUpdating={isStatusUpdating}
      />

      <aside className="properties">
        <ItemMetaSections
          tags={item.tags}
          categories={item.categories}
          relations={[]}
          staff={[]}
          mylists={[]}
        />
      </aside>

      {/* 【削除確認ダイアログ】: openで開閉制御し、onConfirmでmutateを呼び出す 🔵 */}
      <DeleteConfirmDialog
        open={deleteDialogOpen}
        itemTitle={item.title}
        isDeleting={isDeleting}
        onConfirm={() => deleteItem(item.id)}
        onCancel={() => setDeleteDialogOpen(false)}
      />
    </div>
  )
}
