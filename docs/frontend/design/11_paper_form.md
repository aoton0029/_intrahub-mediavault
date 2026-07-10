# 11. 作品を追加（論文・文献）

対応モック: `docs/frontend/ui/11_paper_form.html`

## 1. 画面概要 / ルート

論文・文献をAPI検索を使わずフォーム入力で新規作成する画面（`media_type: paper` 固定、種別セレクトなし）。ルート: `/papers/new`（編集時は `/papers/:id/edit` を同一フォームで再利用する想定）。

## 2. レイアウト構成

```
<AppShell>
  <Titlebar title="作品を追加" />
  <Content>
    <Form onSubmit={handleSubmit}>
      <FormSectionTitle>基本情報</FormSectionTitle>
      <FormGrid>
        <FormField full label="タイトル" required />
        <FormField label="発表日" type="date" />
        <FormField label="評価" type="number" min={0} max={5} step={0.1} />
        <FormField full label="概要" as="textarea" />
        <FormField label="関連サイトURL" />
        <FormField label="閲読状態" as="select" options={[未着手, 視聴中, 視聴済]} />
        <FormField label="お気に入り" as="select" options={[登録しない, お気に入りに登録する]} />
      </FormGrid>

      <FormSectionTitle>種別固有情報</FormSectionTitle>
      <FormGrid>
        <FormField label="DOI" />
        <FormField label="掲載誌名" />
        <FormField label="巻号" />
        <FormField label="掲載ページ範囲" />
        <FormField full label="著者一覧" as="textarea" hint="著者が複数いる場合は改行で区切って入力してください" />
      </FormGrid>

      <FormActions>
        <Button type="submit" variant="accent">保存する</Button>
        <Button as={Link} to="/papers" variant="ghost">キャンセル</Button>
      </FormActions>
    </Form>
  </Content>
</AppShell>
```

## 3. 表示データ / Props型

```ts
interface PaperFormValues {
  title: string;
  publishedDate?: string;      // date
  rating?: number;              // 0.0-5.0, step 0.1
  overview?: string;
  relatedUrl?: string;
  status: 'not_started' | 'in_progress' | 'done';
  isFavorite: boolean;
  detail: {
    doi?: string;
    journalName?: string;
    volumeIssue?: string;
    pageRange?: string;
    authors?: string;           // textarea入力（改行区切り）→ 送信時に配列へ変換
  };
}
```

zodスキーマ: `title` は必須。`rating` は0.0〜5.0の範囲。`authors` は改行区切りテキストを `string[]` にパースする transform を適用。

## 4. 画面固有コンポーネント

なし（`FormSection` / `FormGrid` / `FormField` / `FormActions` は `00_common.md` の共通コンポーネント）

## 5. インタラクション仕様

- react-hook-form + zod resolver でクライアントバリデーション（`.form-field.error` / `.field-error` に対応する表示）
- 「著者一覧」は複数行テキストエリア。プレースホルダは「1行に1名ずつ入力してください」の例示付き
- 保存後は作成時ステータス変更を別リクエストとして送る設計（フォームからの初期作成は `POST /items` で `status`/`consumed_date` を含めず、その後別途 `PATCH /items/{id}/status` で変更する運用。モックHTMLコメントに準拠）

## 6. API連携

- 作成: `POST /items`（`media_type: paper` 固定。種別固有情報は `details` キーにネストして送信。モックHTMLコメントより高確度）
- ステータス変更: 作成後に変更する場合は `PATCH /items/{id}/status { status, consumed_date }`（モックHTMLコメントより高確度）

参照: [items.md](../../backend/mediavault-api/items.md)

## 7. Tailwindスタイリング上の注意

- `.form-grid` は2カラムグリッド、`.form-field.full` は2カラム分（`grid-column: 1 / -1`）
- 「著者一覧」の `<textarea>` は `placeholder` に改行を含む（`&#10;` エンティティ）ため、React実装では通常の複数行文字列で表現する
