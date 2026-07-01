/**
 * TASK-0018: GroupSection - メディア種別によるグループ表示分岐コンポーネント
 * 🔵 REQ-101/102、EDGE-004より
 */
import type { Item } from '@/types';
import SeasonGroupList from './SeasonGroupList';
import VolumeGroupList from './VolumeGroupList';
import ChapterGroupList from './ChapterGroupList';

interface GroupSectionProps {
  item: Item;
}

/**
 * 【機能概要】: mediaTypeに応じてグループ表示コンポーネントを分岐する
 * 【実装方針】:
 *   - anime/drama → SeasonGroupList（話数登録UI付き） 🔵 REQ-101
 *   - manga/novel → VolumeGroupList（話数登録UIなし・EDGE-004） 🔵 REQ-102
 *   - movie → ChapterGroupList（最小実装） 🟡 REQ-017/REQ-301
 *   - game/academic_book/paper → null（グループ管理対象外） 🟡
 * 【テスト対応】: TC-GS-N-01〜06
 * 🔵 dataflow.md 機能4フロー図より
 */
export default function GroupSection({ item }: GroupSectionProps) {
  switch (item.mediaType) {
    case 'anime':
    case 'drama':
      // 【分岐】: REQ-101 anime/drama → SeasonGroupList（話数登録UI付き） 🔵
      return <SeasonGroupList itemId={item.id} />;

    case 'manga':
    case 'novel':
      // 【分岐】: REQ-102 manga/novel → VolumeGroupList（話数登録UIなし） 🔵
      // EDGE-004: VolumeGroupListには話数登録ボタンを含めない設計
      return <VolumeGroupList itemId={item.id} />;

    case 'movie':
      // 【分岐】: REQ-017/REQ-301 movie → ChapterGroupList（オプション・最小実装） 🟡
      return <ChapterGroupList itemId={item.id} />;

    default:
      // 【分岐】: game/academic_book/paperはグループ管理対象外のためnullを返す 🟡
      return null;
  }
}
