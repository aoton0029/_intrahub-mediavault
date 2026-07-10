# 06. マイリスト（一覧）

対応モック: `docs/frontend/ui/06_mylists.html`

## 1. 画面概要 / ルート

作成済みマイリストをカードグリッドで一覧表示し、新規マイリスト作成モーダルを提供する画面。ルート: `/mylists`。サイドバー「マイリスト」がactive。

## 2. レイアウト構成

```
<AppShell>
  <Titlebar title="マイリスト" action={<Button onClick={openCreateModal}>＋ 新しいマイリスト</Button>} />
  <Content>
    {mylists.length > 0 ? (
      <MediaGrid>
        <MediaCard>
          <MylistCover count={coverCount} covers={coverImages} badge={`${itemCount}件`} />
          <Body title meta={`${itemCount} 件`} />
        </MediaCard>
      </MediaGrid>
    ) : (
      <EmptyState title="マイリストがありません"
        description="お気に入りの作品をまとめる、あなただけのリストを作りましょう。"
        action={<Button size="sm" onClick={openCreateModal}>＋ 新しいマイリストを作成</Button>} />
    )}
  </Content>

  <Modal open={isCreateModalOpen} onClose={closeCreateModal} title="新しいマイリスト">
    <FormField label="マイリスト名" required placeholder="例: 積読" />
    <FormActions>
      <Button variant="accent" size="sm">作成する</Button>
      <Button variant="ghost" size="sm" onClick={closeCreateModal}>キャンセル</Button>
    </FormActions>
  </Modal>
</AppShell>
```

一覧とEmptyStateは排他表示（モックコメントに明記）。モーダルは通常非表示で、「＋ 新しいマイリスト」押下時のみ開く（モックは静的HTMLのため常時開いた状態で配置されているが、実装は `isCreateModalOpen` stateで制御する）。

## 3. 表示データ / Props型

```ts
interface MylistSummary {
  id: string;
  name: string;
  itemCount: number;
  coverImages: string[];   // 先頭1-4件のカバー画像URL。n1〜n4のコラージュに使用
}
```

## 4. 画面固有コンポーネント

なし（`MylistCover` / `Modal` / `EmptyState` は `00_common.md` の共通コンポーネント）

## 5. インタラクション仕様

- カードクリックで `15_mylist_detail.md` へ遷移
- 作成モーダルの「作成する」押下で `POST /mylists` を呼び、成功後モーダルを閉じて一覧を再取得（TanStack Queryの invalidate想定）
- 「マイリスト名」は必須バリデーション（react-hook-form + zod）

## 6. API連携

- 一覧取得: `GET /mylists`【要確認】（PRDのバックエンドAPI節は未記載のため推測。件数・カバー画像を含むレスポンス形状は要確定）
- 作成: `POST /mylists { name }`（モックHTMLコメントより高確度）

参照: [mylists.md](../../backend/mediavault-api/mylists.md)

## 7. Tailwindスタイリング上の注意

- `MylistCover` は収録作品数に応じて `n1`(1枚) / `n2`(2枚横並び) / `n3`(上1枚+下2枚) / `n4`(2x2) のグリッドパターンを切り替える
