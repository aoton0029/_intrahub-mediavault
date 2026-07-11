# 16_anime_detail 未決事項

設計書中の【要確認】項目、およびタスク実装中にCodexが仮決定した事項を記録する。

## 未決事項

- [x] `DetailMain`の`groups`セクション見出しが共通コンポーネント側で「構成」固定になっており、設計書が指定する「シーズン構成」という文言と一致しない。共通コンポーネント（`00_common/04_detail_layout.md`）側でセクションタイトルをprops化するか、本画面のみ許容差分とするかを決定する
- [x] `GroupList`/`EpisodeRow`が「話数を追加」「シーズンを追加」ボタンのslotを持たないため、追加ボタンをどう組み込むか（共通コンポーネント拡張 or 画面側で別途`DetailSection`を組み立てるか）を決定する
- [x] `ItemStaff`（`{id, item_id, staff_id, role, character_name}`）単独ではスタッフ氏名が取得できず、`Staff`（`{id, external_id, name, image_url, created_at}`）との結合が必要。結合方法（`GET /items/{id}/staff`のレスポンスに`Staff`情報が含まれるか、別途`staff_id`ごとに取得が必要か）を`staff.md`で確認し決定する
- [x] アニメの「概要」セクションの出典データが設計書の`AnimeDetail`形状に含まれていない（あらすじフィールドが無い）。`Item`側の既存フィールドから表示するか、`AnimeDetail`に概要フィールドを追加すべきか決定する
- [x] `frontend/src/routes.tsx`の`/media/:id`ルートは現状anime専用実装とするが、`mediaType`に応じて他詳細画面（movie/drama等）へ振り分ける仕組みは未実装。振り分け方針（`ItemDetail.media_type`を見て内部でコンポーネント切り替え等）を決定する
- [x] Calibre連携（`PATCH /items/{id}/files/{file_id}/calibre-link`）のUI・フローは本タスクで未実装（ボタン配置のみ）。連携フロー自体の実装タイミングを決定する
- [x] マイリスト所属取得APIが`00_common`側の設計書注記でも「現状ドキュメントに無く、UIのみ先行実装」とされている（モックHTMLコメント参照）。バックエンド実装時にAPI追加が必要な旨を記録する

## ユーザーによる決定事項

- [x] `groups`セクション見出し: 共通コンポーネント側でセクションタイトルをprops化し、anime詳細では「シーズン構成」を表示する
- [x] シーズン/話数の追加ボタン配置: `GroupList`/`EpisodeRow` を共通拡張し、actions / footer などの差し込み口を追加して対応する
- [x] スタッフ氏名の取得方法: `GET /items/{id}/staff` のレスポンスを拡張し、`Staff` 情報を含めて 1 回の取得で氏名を返せるようにする
- [x] 概要セクションの出典: `Item.description` を正式な表示元とし、`AnimeDetail` への概要フィールド追加は行わない
- [x] `/media/:id` の振り分け方針: 当面は `AnimeDetailPage` 直結とし、他 media_type の詳細画面が実装された段階でディスパッチャ化する
- [x] Calibre連携の実装タイミング: 今回タスクではボタン配置までに留め、連携フロー本体は別タスクで実装する
- [x] マイリスト所属取得APIの扱い: `GET /items/{id}/mylists` は `mylists.md` に既存定義があるため、API追加懸念は解消済みとして扱う

## Codexによる仮決定ログ

- (Codexが仮決定した場合、ここに追記される)
