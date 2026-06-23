# TASK-0011 テストケース一覧

## 単体テスト（DB不要、純粋関数・パース処理）

### TC-0011-U01: UUIDパース成功
**Given** 正しい形式のUUID文字列
**When** parse_item_idを呼ぶ
**Then** Ok(Uuid)が返る

### TC-0011-U02: UUIDパース失敗
**Given** "abc"のような不正な文字列
**When** parse_item_idを呼ぶ
**Then** Err(ApiError) でcode=VALIDATION_ERROR, status=400

## 統合テスト（実DB, #[ignore]）

### TC-0011-N01: 存在するitemの詳細取得（詳細テーブルあり）
**Given** anime media_typeのitemとanime_detailsレコードが存在
**When** GET /items/:id相当のリポジトリ関数を呼ぶ
**Then** Item基本情報 + detail(anime_detailsの内容)が返る

### TC-0011-N02: 存在するitemの詳細取得（タグ・カテゴリあり）
**Given** itemにtag, categoryが紐付いている
**When** 取得処理を呼ぶ
**Then** tags, categoriesが配列で返る

### TC-0011-N03: 詳細テーブルにレコードが無い場合
**Given** itemsレコードはあるが対応する詳細テーブルにレコードが無い
**When** 取得処理を呼ぶ
**Then** detail=nullで返る（エラーにしない）

### TC-0011-N04: タグ・カテゴリが紐付いていない場合
**Given** item_tags/item_categoriesに紐付けが無い
**When** 取得処理を呼ぶ
**Then** tags=[], categories=[]

### TC-0011-E01: 存在しないitemで404
**Given** 存在しないUUID
**When** ハンドラを呼ぶ
**Then** 404 ApiErrorCode::ItemNotFound（code="ITEM_NOT_FOUND"）

### TC-0011-E02: 不正なUUID形式で400
**Given** "not-a-uuid"
**When** ハンドラを呼ぶ
**Then** 400 ApiErrorCode::ValidationError

### TC-0011-N05: media_typeごとの詳細テーブル分岐（8種）
**Given** 8つのmedia_typeそれぞれでitemと対応detailレコードを作成
**When** 取得処理を呼ぶ
**Then** 各media_typeに対応した詳細テーブルから正しくデータが取得される

## カバレッジ
- 正常系: TC-0011-N01〜N05
- 異常系: TC-0011-E01, E02
- 境界値/単体: TC-0011-U01, U02
