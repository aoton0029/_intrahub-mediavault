# 15. マイリスト詳細

対応モック: `docs/frontend/ui/15_mylist_detail.html`

## 1. 画面概要 / ルート

特定マイリストの収録作品を書誌情報中心の行リストで表示し、削除（マイリストからの除外）を行う画面。ルート: `/mylists/:id`。サイドバー「マイリスト」がactive。

## 2. レイアウト構成

```
<AppShell>
  <Titlebar>
    <Breadcrumb><Link to="/mylists">マイリスト</Link> / {mylist.name}</Breadcrumb>
    <h1>{mylist.name} <CountBadge>{itemCount} 件</CountBadge></h1>
    <Button variant="accent" as={Link} to="/media/search">＋ 作品を追加</Button>
  </Titlebar>
  <Content>
    {items.length > 0 ? (
      <LiteratureList>
        <LiteratureRow
          title byline={[mediaTypeLabel, <RatingStars readOnly value={rating} />]}
          tags={tags}
          action={<Button variant="danger" size="sm" onClick={openRemoveModal}>削除</Button>} />
      </LiteratureList>
    ) : (
      <EmptyState title="このマイリストにはまだ作品がありません"
        description="「＋ 作品を追加」から、このリストに収録したい作品を検索して追加しましょう。" />
    )}
  </Content>

  <Modal open={isRemoveModalOpen} onClose={closeRemoveModal} title="マイリストから削除しますか？">
    <p>「{targetTitle}」をこのマイリストから削除します。作品自体は削除されません。</p>
    <FormActions>
      <Button variant="danger" size="sm">削除する</Button>
      <Button variant="ghost" size="sm" onClick={closeRemoveModal}>キャンセル</Button>
    </FormActions>
  </Modal>
</AppShell>
```

## 3. 表示データ / Props型

```ts
interface MylistDetailItem {
  itemId: string;
  title: string;
  mediaTypeLabel: string;   // 例: "漫画", "アニメ"
  rating?: number;
  tags: string[];
}
```

## 4. 画面固有コンポーネント

- `CountBadge`: タイトル横の件数表示（`font-mono`, `text-faint`）。他画面未使用のため画面固有

## 5. インタラクション仕様

- 「削除」ボタン押下で確認モーダルを開く（破壊的操作のため誤操作防止に確認を挟む）。モーダルは通常非表示（モックは静的HTMLのため開いた状態で配置）
- 「＋ 作品を追加」の遷移先はモックでは `12_general_media_search.html` 固定だが、実装時はこのマイリストへの追加であることをコンテキストとして引き継ぐ必要がある【要確認】（クエリパラメータ等で `mylist_id` を渡す設計を検討）

## 6. API連携

- 詳細・収録アイテム取得: `GET /mylists/{id}`【要確認】（推測）
- マイリストから除外: `DELETE /mylists/{id}/items/{item_id}`【要確認】（推測。作品自体（Item）は削除しない点に注意）

参照: [mylists.md](../../backend/mediavault-api/mylists.md)

## 7. Tailwindスタイリング上の注意

- `LiteratureRow` の `.rating-stars` はこの画面では読み取り専用表示（`.val` に数値付き）
