import { useState, type ReactNode } from 'react';
import { Link } from 'react-router-dom';
import {
  FiFileText,
  FiFolder,
  FiGitBranch,
  FiLayers,
  FiLink2,
  FiPlus,
  FiTv,
  FiUsers,
} from 'react-icons/fi';
import {
  DetailLayout,
  DetailRail,
  DetailSection,
  GroupList,
  PropertyList,
  RelatedWorksList,
  StaffList,
  StreamingLinks,
} from '@/components/detail';
import {
  EmptyState,
  FilterToolbar,
  FormActions,
  FormField,
  FormGrid,
  FormSection,
  LiteratureList,
  LoadMoreSentinel,
  MediaGrid,
  Modal,
  MylistCover,
  ResourceTabs,
  useInfiniteScroll,
} from '@/components/shared';
import {
  animeGroups,
  animeResourceTabs,
  detailCategories,
  detailMylists,
  detailTags,
  filterChips,
  literatureItems,
  mediaCards,
  movieProperties,
  paperProperties,
  paperResourceTabs,
  railMetaItems,
  relatedWorks,
  searchResults,
  staffMembers,
  streamingLinks,
} from '@/mocks/data';
import type { ItemStatus, TagLikeItem } from '@/types/ui';

function useEditableTags(initialItems: TagLikeItem[]) {
  const [items, setItems] = useState(initialItems);

  return {
    items,
    add: (label: string) =>
      setItems((current) => [...current, { id: `${label}-${current.length}`, label }]),
    remove: (id: string) => setItems((current) => current.filter((item) => item.id !== id)),
  };
}

function OverviewSection({ children }: { children: ReactNode }) {
  return (
    <DetailSection icon={<FiFileText className="icon" />} title="概要">
      <p>{children}</p>
    </DetailSection>
  );
}

export function HomePage() {
  return (
    <div className="space-y-8">
      <div className="stat-grid">
        <div className="stat-card">
          <div className="stat-value">128</div>
          <div className="stat-label">総アイテム数</div>
        </div>
        <div className="stat-card is-favorite">
          <div className="stat-value">37</div>
          <div className="stat-label">お気に入り</div>
        </div>
        <div className="stat-card">
          <div className="stat-value">12</div>
          <div className="stat-label">進行中</div>
        </div>
      </div>

      <div className="section-heading">
        <span>最近追加した作品</span>
        <Link className="see-all" to="/media">
          すべて見る →
        </Link>
      </div>
      <MediaGrid items={mediaCards} />
    </div>
  );
}

function MediaIndexPage({ title, addTo, density = 'compact' as const }: { title: string; addTo: string; density?: 'default' | 'compact' }) {
  const [search, setSearch] = useState('');
  const [sort, setSort] = useState('recent');
  const [category, setCategory] = useState('all');
  const [page, setPage] = useState(1);
  const { ref } = useInfiniteScroll({ onReachEnd: () => setPage((current) => current + 1), enabled: page < 2 });

  return (
    <div>
      <div className="mb-5 flex justify-end">
        <Link className="btn btn-accent" to={addTo}>
          <FiPlus className="icon" />
          作品を追加
        </Link>
      </div>
      <FilterToolbar
        chips={filterChips}
        filters={[
          {
            id: 'category',
            label: 'カテゴリ',
            value: category,
            options: [
              { value: 'all', label: 'すべて' },
              { value: 'watchlist', label: 'watchlist' },
            ],
            onChange: setCategory,
          },
        ]}
        sortOptions={[
          { value: 'recent', label: '最近追加' },
          { value: 'rating', label: '評価順' },
        ]}
        sortValue={sort}
        onSortChange={setSort}
        searchValue={search}
        onSearchChange={setSearch}
        searchPlaceholder={`${title}を検索`}
      />
      <MediaGrid items={mediaCards} density={density} />
      <LoadMoreSentinel innerRef={ref} label={page < 2 ? 'さらに読み込み中' : 'これ以上ありません'} loading={page < 2} />
    </div>
  );
}

export function GeneralMediaPage() {
  return <MediaIndexPage title="一般メディア" addTo="/media/search" />;
}

export function AcademicBooksPage() {
  return (
    <div>
      <div className="mb-5 flex justify-end">
        <Link className="btn btn-accent" to="/academic-books/search">
          <FiPlus className="icon" />
          作品を追加
        </Link>
      </div>
      <FilterToolbar
        chips={filterChips}
        sortOptions={[
          { value: 'recent', label: '最近追加' },
          { value: 'title', label: 'タイトル順' },
        ]}
        sortValue="recent"
        onSortChange={() => undefined}
        searchValue=""
        onSearchChange={() => undefined}
        searchPlaceholder="学術書を検索"
      />
      <LiteratureList items={literatureItems.filter((item) => item.id.startsWith('book'))} />
    </div>
  );
}

export function PapersPage() {
  return (
    <div>
      <div className="mb-5 flex justify-end">
        <Link className="btn btn-accent" to="/papers/new">
          <FiPlus className="icon" />
          作品を追加
        </Link>
      </div>
      <FilterToolbar
        chips={filterChips}
        sortOptions={[
          { value: 'recent', label: '最近追加' },
          { value: 'year', label: '発表年順' },
        ]}
        sortValue="recent"
        onSortChange={() => undefined}
        searchValue=""
        onSearchChange={() => undefined}
        searchPlaceholder="論文を検索"
      />
      <LiteratureList items={literatureItems.filter((item) => item.id.startsWith('paper'))} />
    </div>
  );
}

export function MylistsPage() {
  const [open, setOpen] = useState(false);

  return (
    <div>
      <div className="mb-5 flex justify-end">
        <button type="button" className="btn btn-accent" onClick={() => setOpen(true)}>
          <FiPlus className="icon" />
          新しいマイリスト
        </button>
      </div>
      <div className="card-grid">
        {['週末に見る', '研究用インスピレーション', '2026 replay'].map((title, index) => (
          <Link className="media-card" key={title} to={`/mylists/mylist-00${index + 1}`} style={{ color: 'inherit', textDecoration: 'none' }}>
            <MylistCover count={(index === 0 ? 4 : index === 1 ? 3 : 2) as 2 | 3 | 4 | 1} covers={[]} />
            <div className="body">
              <div className="title">{title}</div>
              <div className="meta">{index + 4} items</div>
            </div>
          </Link>
        ))}
      </div>
      <Modal open={open} onClose={() => setOpen(false)} title="新しいマイリスト">
        <FormGrid>
          <FormField label="リスト名" required full>
            <input defaultValue="週末に見る" />
          </FormField>
        </FormGrid>
        <FormActions>
          <button type="button" className="btn btn-accent" onClick={() => setOpen(false)}>
            作成
          </button>
          <button type="button" className="btn btn-ghost" onClick={() => setOpen(false)}>
            キャンセル
          </button>
        </FormActions>
      </Modal>
    </div>
  );
}

export function GeneralMediaSearchPage() {
  return (
    <div>
      <div className="mb-5 flex justify-end">
        <Link className="btn btn-ghost" to="/media/search">
          手動で入力する
        </Link>
      </div>
      <FilterToolbar
        chips={filterChips}
        sortOptions={[
          { value: 'relevance', label: '関連度順' },
          { value: 'year', label: '年順' },
        ]}
        sortValue="relevance"
        onSortChange={() => undefined}
        searchValue="science fiction"
        onSearchChange={() => undefined}
      />
      <MediaGrid items={searchResults} density="compact" variant="search-result" />
    </div>
  );
}

export function AcademicBookSearchPage() {
  return (
    <div>
      <div className="mb-5 flex justify-end">
        <Link className="btn btn-ghost" to="/academic-books/search">
          手動で入力する
        </Link>
      </div>
      <FilterToolbar
        chips={filterChips}
        sortOptions={[
          { value: 'relevance', label: '関連度順' },
          { value: 'year', label: '年順' },
        ]}
        sortValue="relevance"
        onSortChange={() => undefined}
        searchValue="machine learning systems"
        onSearchChange={() => undefined}
      />
      <LiteratureList items={literatureItems.filter((item) => item.id.startsWith('book'))} />
    </div>
  );
}

export function PaperFormPage() {
  return (
    <form>
      <FormSection title="基本情報">
        <FormGrid>
          <FormField label="タイトル" required full>
            <input defaultValue="Attention Is All You Need" />
          </FormField>
          <FormField label="著者" required full>
            <input defaultValue="Ashish Vaswani et al." />
          </FormField>
          <FormField label="掲載誌 / 学会" required>
            <input defaultValue="NIPS 2017" />
          </FormField>
          <FormField label="発表年">
            <input defaultValue="2017" />
          </FormField>
          <FormField label="DOI">
            <input defaultValue="10.48550/arXiv.1706.03762" />
          </FormField>
          <FormField label="関連サイトURL">
            <input defaultValue="https://doi.org/10.48550/arXiv.1706.03762" />
          </FormField>
        </FormGrid>
      </FormSection>
      <FormSection title="メモ">
        <FormGrid>
          <FormField label="概要" full>
            <textarea defaultValue="Transformer アーキテクチャの原典論文。" />
          </FormField>
        </FormGrid>
      </FormSection>
      <FormActions>
        <button type="submit" className="btn btn-accent">
          保存する
        </button>
        <Link className="btn btn-ghost" to="/papers">
          キャンセル
        </Link>
      </FormActions>
    </form>
  );
}

export function MylistDetailPage() {
  return (
    <div>
      <div className="section-heading">
        <span>週末に見る</span>
        <Link className="see-all" to="/mylists">
          一覧に戻る →
        </Link>
      </div>
      <MediaGrid items={mediaCards} />
    </div>
  );
}

function DetailPageTemplate({
  title,
  originalTitle,
  summary,
  properties,
  showGroups,
  showStaff,
  showStreaming,
  resourceTabs,
}: {
  title: string;
  originalTitle?: string;
  summary: string;
  properties?: typeof paperProperties;
  showGroups?: boolean;
  showStaff?: boolean;
  showStreaming?: boolean;
  resourceTabs: typeof animeResourceTabs;
}) {
  const [status, setStatus] = useState<ItemStatus>('in_progress');
  const [rating, setRating] = useState(4);
  const [favorite, setFavorite] = useState(true);
  const tagState = useEditableTags(detailTags);
  const categoryState = useEditableTags(detailCategories);
  const [mylists, setMylists] = useState(detailMylists);

  return (
    <DetailLayout
      rail={
        <DetailRail
          title={title}
          originalTitle={originalTitle}
          status={status}
          rating={rating}
          favorite={favorite}
          metaItems={railMetaItems}
          tags={tagState.items}
          categories={categoryState.items}
          mylists={mylists}
          onStatusChange={setStatus}
          onRatingChange={setRating}
          onFavoriteChange={setFavorite}
          onAddTag={tagState.add}
          onRemoveTag={tagState.remove}
          onAddCategory={categoryState.add}
          onRemoveCategory={categoryState.remove}
          onRemoveMylist={(id) => setMylists((current) => current.filter((item) => item.id !== id))}
        />
      }
      main={
        <>
          <OverviewSection>{summary}</OverviewSection>
          {properties ? (
            <DetailSection icon={<FiFolder className="icon" />} title="種別固有情報">
              <PropertyList items={properties} />
            </DetailSection>
          ) : null}
          {showGroups ? (
            <DetailSection icon={<FiLayers className="icon" />} title="構成">
              <GroupList groups={animeGroups} />
            </DetailSection>
          ) : null}
          {showStaff ? (
            <DetailSection icon={<FiUsers className="icon" />} title="スタッフ">
              <StaffList members={staffMembers} />
            </DetailSection>
          ) : null}
          <DetailSection icon={<FiGitBranch className="icon" />} title="関連作品">
            <RelatedWorksList items={relatedWorks} />
          </DetailSection>
          {showStreaming ? (
            <DetailSection icon={<FiTv className="icon" />} title="配信">
              <StreamingLinks links={streamingLinks} />
            </DetailSection>
          ) : null}
          <DetailSection icon={<FiLink2 className="icon" />} title="リソース">
            <ResourceTabs tabs={resourceTabs} />
          </DetailSection>
        </>
      }
    />
  );
}

export function AnimeDetailPage() {
  return (
    <DetailPageTemplate
      title="攻殻機動隊 SAC_2045"
      originalTitle="Ghost in the Shell: SAC_2045"
      summary="シーズン・スタッフ・配信・リソースをすべて持つ詳細画面の正準形。左レールで状態管理し、右側で本文セクションを積む。"
      showGroups
      showStaff
      showStreaming
      resourceTabs={animeResourceTabs}
    />
  );
}

export function MovieDetailPage() {
  return (
    <DetailPageTemplate
      title="Dune: Part Two"
      summary="映画詳細は種別固有情報、スタッフ、配信、リソースを持つが、シーズン構成は持たない。"
      properties={movieProperties}
      showStaff
      showStreaming
      resourceTabs={animeResourceTabs}
    />
  );
}

export function DramaDetailPage() {
  return (
    <DetailPageTemplate
      title="The Leftovers"
      summary="ドラマ詳細は映画と同様の情報に加え、シーズン構成を持つ。"
      properties={movieProperties}
      showGroups
      showStaff
      showStreaming
      resourceTabs={animeResourceTabs}
    />
  );
}

export function MangaDetailPage() {
  return (
    <DetailPageTemplate
      title="チ。-地球の運動について-"
      summary="漫画詳細は構成セクションと関連作品、リソースを持つ。"
      properties={paperProperties.slice(0, 4)}
      showGroups
      resourceTabs={paperResourceTabs}
    />
  );
}

export function NovelDetailPage() {
  return (
    <DetailPageTemplate
      title="The Dispossessed"
      summary="小説詳細も漫画と同じ骨格だが、配信やスタッフは表示しない。"
      properties={paperProperties.slice(0, 4)}
      showGroups
      resourceTabs={paperResourceTabs}
    />
  );
}

export function GameDetailPage() {
  return (
    <DetailPageTemplate
      title="Baldur’s Gate 3"
      summary="ゲーム詳細はプロパティと関連作品、リソースだけを持つ。"
      properties={paperProperties}
      resourceTabs={paperResourceTabs}
    />
  );
}

export function AcademicBookDetailPage() {
  return (
    <DetailPageTemplate
      title="Designing Data-Intensive Applications"
      summary="学術書詳細は論文詳細と近い構造で、配信・スタッフ・構成を持たない。"
      properties={paperProperties.slice(0, 4)}
      resourceTabs={paperResourceTabs}
    />
  );
}

export function PaperDetailPage() {
  return (
    <DetailPageTemplate
      title="Attention Is All You Need"
      summary="論文詳細は種別固有情報と関連作品、リソースを中心に構成する。"
      properties={paperProperties}
      resourceTabs={paperResourceTabs}
    />
  );
}

export function SettingsPage() {
  const tabs = [
    { id: 'api', label: 'API キー' },
    { id: 'import', label: 'Import' },
    { id: 'system', label: 'System' },
  ];
  const [active, setActive] = useState(tabs[0].id);

  return (
    <div className="settings-shell">
      <div className="settings-tabs">
        {tabs.map((tab) => (
          <button
            type="button"
            key={tab.id}
            className={active === tab.id ? 'settings-tab active' : 'settings-tab'}
            onClick={() => setActive(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      <div className="settings-panel">
        {active === 'api' ? (
          <>
            <h2>API キー</h2>
            <p className="desc">インポート元サービスの API キーを管理する。</p>
            <div className="kv-card">
              <div>
                <div className="provider">TMDB</div>
                <div className="key">***********ce1</div>
              </div>
              <button type="button" className="btn btn-ghost btn-sm">
                編集
              </button>
            </div>
            <EmptyState title="MAL キー未設定" description="Anime import の前に設定が必要です。" />
          </>
        ) : null}
        {active === 'import' ? (
          <>
            <h2>Import</h2>
            <p className="desc">検索インポート時の既定動作を定義する。</p>
            <div className="kv-card">
              <div>
                <div className="provider">Default source</div>
                <div className="key">TMDB / OpenLibrary / DOI</div>
              </div>
            </div>
          </>
        ) : null}
        {active === 'system' ? (
          <>
            <h2>System</h2>
            <p className="desc">UI トーンと既定表示設定。</p>
            <div className="kv-card">
              <div>
                <div className="provider">Theme handling</div>
                <div className="key">Explicit data-theme toggle</div>
              </div>
            </div>
          </>
        ) : null}
      </div>
    </div>
  );
}
