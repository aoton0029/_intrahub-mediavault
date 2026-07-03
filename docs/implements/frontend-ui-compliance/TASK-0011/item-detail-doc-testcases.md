# TASK-0011: ItemDetailPage テストケース定義

対象: `frontend/src/pages/ItemDetailPage.tsx`
テストファイル: `frontend/src/pages/ItemDetailPage.test.tsx`（新規）

## 1. 正常系テストケース

- **TC-IDP-N-01**: パンくずリストが表示される 🔵
  - 何をテストするか: `.breadcrumb`要素内に「ホーム」「mediaTypeラベル」「アイテムタイトル」が表示されること
  - 入力値: `makeItem('1')` (`mediaType: 'anime'`, `title: 'Item 1'`)
  - 期待される結果: `.breadcrumb`内に「ホーム」「アニメ」「Item 1」に相当するテキストが存在する
  - 確認ポイント: 既存の`Sidebar`のカテゴリ表記（mediaType単位）と整合すること

- **TC-IDP-N-02**: パンくずの「ホーム」リンクが`/`を指す 🔵
  - 入力値: 同上
  - 期待される結果: `.breadcrumb`内の`<a>`(Link)の`href`が`/`

- **TC-IDP-N-03**: タイトルバーに「編集」ボタン（`.btn`）が表示され、クリックで`/items/:id/edit`へ遷移する 🔵
  - 入力値: `id='item-abc'`
  - 期待される結果: `編集`ボタンの`href`（またはクリック時のnavigate先）が`/items/item-abc/edit`

- **TC-IDP-N-04**: タイトルバーに「削除」ボタン（`.btn-danger`）が表示される 🔵
  - 期待される結果: `role=button`かつ`class`に`btn-danger`を含む要素が「削除」というテキストで存在する

- **TC-IDP-N-05**: 「削除」ボタン押下でConfirmDialog（`confirm`）が開く 🔵
  - 何をテストするか: `useConfirmDialog`の`confirm`が呼ばれ、ダイアログ表示状態になること
  - 期待される結果: `data-testid="confirm-dialog"`が表示される

- **TC-IDP-N-06**: ConfirmDialogで確定すると`useDeleteItemMutation`のmutateが呼ばれる 🔵
  - 入力値: 削除ボタン→確定ボタンクリック
  - 期待される結果: `mutate`が`id`引数で呼び出される

- **TC-IDP-N-07**: `.doc-title`にitem.titleが表示される 🔵
- **TC-IDP-N-08**: `item.originalTitle`が設定されている場合`.doc-original`に表示される 🔵
- **TC-IDP-N-09**: `.doc-section`内にitem.descriptionが表示される 🔵
- **TC-IDP-N-10**: `.doc-cover`が存在し、`item.coverImageUrl`が設定されている場合背景画像スタイルが設定される 🔵

## 2. 異常系テストケース

- **TC-IDP-E-01**: `error.code==='ITEM_NOT_FOUND'`のとき一覧へリダイレクトされる（既存挙動の回帰確認） 🔵
  - 何をテストするか: 視覚変更後も既存のエラーハンドリングが壊れていないこと
  - 期待される結果: `navigate('/')`が呼ばれ、toast.errorが呼ばれる

- **TC-IDP-E-02**: 上記以外のエラー時、エラーメッセージと「一覧へ戻る」ボタンが表示される（既存挙動の回帰確認） 🔵

- **TC-IDP-E-03**: 削除mutation失敗時もConfirmDialogが正しく閉じ、エラーが握りつぶされない 🟡
  - 実際の発生シナリオ: API側で削除失敗（409等）
  - 期待される結果: 既存のエラー通知ロジック（存在する場合）が動作、なければ最低限例外が伝播しアプリがクラッシュしない

## 3. 境界値テストケース

- **TC-IDP-B-01**: `item.originalTitle`が`undefined`の場合`.doc-original`が表示されない 🔵
  - 境界値選択の根拠: オプショナルフィールドの有無分岐
- **TC-IDP-B-02**: `item.coverImageUrl`が`undefined`の場合`.doc-cover`はプレースホルダ（背景画像なし）として表示される 🟡
- **TC-IDP-B-03**: `item.description`が`undefined`の場合`.doc-section`の概要テキストは空表示（エラーにならない） 🟡
- **TC-IDP-B-04**: Propertiesパネル用の右カラム（`RootLayout`の`.properties`）が本コンポーネント変更後も崩れず表示される（統合確認、RootLayout側は変更しない） 🔵

## 4. 開発言語・フレームワーク

- **プログラミング言語**: TypeScript 🔵（プロジェクト標準）
- **テストフレームワーク**: Vitest + @testing-library/react 🔵（`frontend/src/pages/HomePage.test.tsx`と同一パターンを踏襲）
- **モック対象**: `@/api/items`（`useItemQuery`, `useDeleteItemMutation`）, `react-router-dom`（`useNavigate`）, `sonner`（`toast`）, `@/hooks/useConfirmDialog`

## 5. 要件定義との対応関係

- 参照した機能概要: `item-detail-doc-requirements.md` セクション1
- 参照した入力・出力仕様: 同セクション2
- 参照した制約条件: 同セクション3（ConfirmDialog/RootLayout非変更）
- 参照した使用例: 同セクション4

## 品質判定

- テスト分類: 正常系10・異常系3・境界値4で網羅性あり。
- 期待値: 明確。
- 技術選択: 確定（Vitest/RTL、既存HomePage.test.tsxパターン踏襲）。
- 実装可能性: 確実。
- 信頼性レベル: 🔵多数、🟡は削除失敗時・coverプレースホルダ・description空表示の3件のみ。
- **総合判定: 高品質**
