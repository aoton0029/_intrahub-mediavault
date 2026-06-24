# TASK-0020 テストケース定義書: スタッフ管理CRUD実装

**タスクID**: TASK-0020
**機能名**: staff（スタッフ管理CRUD）
**要件名**: mediavault-backend
**出力ファイル**: docs/implements/mediavault-backend/TASK-0020/staff-testcases.md
**作成日**: 2026-06-24

---

## 対象エンドポイント

| # | メソッド | パス | 概要 |
|---|---|---|---|
| 1 | POST | `/staff` | スタッフ作成 |
| 2 | POST | `/items/:id/staff` | itemへのスタッフ紐付け |
| 3 | DELETE | `/items/:id/staff/:item_staff_id` | 紐付け解除 |

## 対象関数・レイヤー

- **models/staff.rs**: `Staff`, `ItemStaff` 構造体、`parse_create_staff_request()`, `parse_create_item_staff_request()`（バリデーション）
- **repositories/staff_repository.rs**: `create_staff()`, `link_staff()`, `unlink_staff()`
- **handlers/staff.rs**: `create_staff_handler()`, `create_item_staff_handler()`, `delete_item_staff_handler()`
- **models/response.rs**: `ApiErrorCode::StaffNotFound`（新規追加）

---

## 1. 正常系テストケース（基本的な動作）

### TC-N-01: スタッフ作成（必須フィールドのみ）

- **テスト名**: 必須フィールド`name`のみでスタッフを作成できる
  - **何をテストするか**: `POST /staff` に `name` のみを渡したとき、staffテーブルへINSERTされUUID付きのStaffが返ること
  - **期待される動作**: バリデーション成功 → repository `create_staff()` → 201でStaffを返す
- **入力値**: `{ "name": "監督A" }`
  - **入力データの意味**: REQ-009の最小ユースケース（名前だけのスタッフ登録）を代表
- **期待される結果**: HTTP 201, `{ success: true, data: { id: <UUID>, external_id: null, name: "監督A", image_url: null, created_at: <timestamp> } }`
  - **期待結果の理由**: external_id/image_urlはoptionalでNULL許容、idはgen_random_uuid()、created_atはCURRENT_TIMESTAMPで自動付与（database-schema.sql）
- **テストの目的**: スタッフ作成の最小成功パスを確認
  - **確認ポイント**: idがUUID形式、external_id/image_urlがnull、201ステータス
- 🔵 信頼性レベル: staff-requirements.md 4.2 / TASK-0020 テストケース1 に明記

### TC-N-02: スタッフ作成（全フィールド指定）

- **テスト名**: name/external_id/image_url 全指定でスタッフを作成できる
  - **何をテストするか**: optional含む全フィールドが正しく保存されること
  - **期待される動作**: 全フィールドがそのままINSERTされ、201で返る
- **入力値**: `{ "name": "声優B", "external_id": "anilist-12345", "image_url": "https://example.com/b.png" }`
  - **入力データの意味**: external_idは外部API由来の重複防止ID、image_urlは画像URLの代表値
- **期待される結果**: HTTP 201, `data.external_id == "anilist-12345"`, `data.image_url == "https://example.com/b.png"`, `data.name == "声優B"`
  - **期待結果の理由**: optionalフィールドも渡されればそのまま保持される（external_idはバリデーション不要・保持のみ）
- **テストの目的**: optionalフィールドの保持を確認
  - **確認ポイント**: external_id/image_urlが欠落せず保存される
- 🔵 信頼性レベル: staff-requirements.md 2.1 / note.md セクション4 入力仕様に基づく

### TC-N-03: itemへのスタッフ紐付け（監督役・character_nameなし）

- **テスト名**: 既存item・既存staffに対しrole指定で紐付けできる
  - **何をテストするか**: `POST /items/:id/staff` がitem_staffへINSERTし201を返すこと
  - **期待される動作**: item_id/staff_idの存在確認 → item_staffへINSERT → 201
- **入力値**: パス `id=<既存item UUID>`, ボディ `{ "staff_id": "<既存staff UUID>", "role": "監督" }`
  - **入力データの意味**: 最も典型的な紐付け（監督役、キャラ名不要）を代表
- **期待される結果**: HTTP 201, `{ success: true, data: { id: <UUID>, item_id, staff_id, role: "監督", character_name: null } }`
  - **期待結果の理由**: character_nameはoptionalなので未指定時はnull、item_staffにレコード1件作成
- **テストの目的**: 紐付け作成の基本成功パスを確認
  - **確認ポイント**: item_staff.idが新規発行、character_nameがnull、201ステータス
- 🔵 信頼性レベル: staff-requirements.md 4.2 / TASK-0020 テストケース2 に明記

### TC-N-04: character_name付きの紐付け（声優役）

- **テスト名**: 声優役でcharacter_nameを含めて紐付けできる
  - **何をテストするか**: character_nameが正しく保存されること
  - **期待される動作**: character_name付きでINSERT → 201、保存値が返る
- **入力値**: `{ "staff_id": "<既存staff UUID>", "role": "声優", "character_name": "主人公" }`
  - **入力データの意味**: 声優役はキャラ名を伴う代表的ユースケース
- **期待される結果**: HTTP 201, `data.role == "声優"`, `data.character_name == "主人公"`
  - **期待結果の理由**: character_nameはNULLable列だが、指定された場合は正しく永続化される必要がある
- **テストの目的**: optional character_nameの保持を確認
  - **確認ポイント**: character_nameが欠落せず保存される
- 🟡 信頼性レベル: staff-requirements.md 4.2 / TASK-0020 テストケース3（🟡）

### TC-N-05: 紐付け削除の正常動作

- **テスト名**: 既存item_staffレコードを削除できる
  - **何をテストするか**: `DELETE /items/:id/staff/:item_staff_id` が該当レコードを削除し204を返すこと
  - **期待される動作**: item_id整合性チェック → DELETE → 204 No Content
- **入力値**: パス `id=<既存item UUID>`, `item_staff_id=<該当item_staff.id>`
  - **入力データの意味**: 正しいitem配下の正しい紐付けID（整合する組み合わせ）
- **期待される結果**: HTTP 204（ボディなし）、当該item_staffレコードがDBから消える
  - **期待結果の理由**: 削除成功は204 No Content（note.md レスポンス形式）
- **テストの目的**: 紐付け解除の基本成功パスを確認
  - **確認ポイント**: 204ステータス、削除後に該当レコードがSELECTで取得不可
- 🟡 信頼性レベル: staff-requirements.md 4.2 / TASK-0020 テストケース4（🟡）

### TC-N-06: parse_create_staff_request の正常パース（models単体）

- **テスト名**: 有効なnameでparse_create_staff_requestが成功する
  - **何をテストするか**: バリデーション関数が正常入力をOkで返すこと
  - **期待される動作**: 検証済みCreateStaffRequestを`Ok`で返す
- **入力値**: `CreateStaffRequest { name: "監督A".to_string(), external_id: None, image_url: None }`
  - **入力データの意味**: ハンドラ手前のpure関数を単体で検証（DB不要）
- **期待される結果**: `Ok(req)` が返り、`req.name == "監督A"`
  - **期待結果の理由**: items CRUDの`parse_create_item_request`と同等のパターン（note.md）
- **テストの目的**: DB非依存のバリデーションロジックを高速に検証
  - **確認ポイント**: 戻り値がOk、フィールド値が保持される
- 🔵 信頼性レベル: note.md セクション2「parse_*関数」パターン / item.rs参考実装に基づく

---

## 2. 異常系テストケース（エラーハンドリング）

### TC-E-01: name空文字でスタッフ作成 → 400 VALIDATION_ERROR

- **テスト名**: 空のnameはバリデーションエラーになる
  - **エラーケースの概要**: 必須項目nameが空文字
  - **エラー処理の重要性**: nameはNOT NULL・空文字不可。空登録はデータ品質を損なう
- **入力値**: `{ "name": "" }`
  - **不正な理由**: `name VARCHAR(255) NOT NULL`かつ空文字不可（staff-requirements.md 2.1）
  - **実際の発生シナリオ**: フロントの入力漏れ、空白のみ送信
- **期待される結果**: HTTP 400, `{ success: false, error: { code: "VALIDATION_ERROR", message: <分かりやすい説明> } }`
  - **エラーメッセージの内容**: nameが必須である旨の明確なメッセージ
  - **システムの安全性**: INSERTに到達せず早期リターン、DB状態は不変
- **テストの目的**: 必須項目バリデーションの確認
  - **品質保証の観点**: 不正データのDB混入を防ぐ
- 🔵 信頼性レベル: staff-requirements.md 2.1 / 4.3「role空文字 / 不正UUID → 400」と同方針

### TC-E-02: 存在しないstaff_idで紐付け → 404 STAFF_NOT_FOUND

- **テスト名**: 不存在のstaff_idでの紐付けはSTAFF_NOT_FOUNDを返す
  - **エラーケースの概要**: staff_idがDBに存在しないUUID
  - **エラー処理の重要性**: FK制約だけだと汎用エラーになるため、アプリ側で事前確認し詳細なコードを返す
- **入力値**: `{ "staff_id": "<DBに存在しないUUID>", "role": "監督" }`
  - **不正な理由**: staffテーブルに該当行がない
  - **実際の発生シナリオ**: 削除済みstaffの参照、誤ったID送信
- **期待される結果**: HTTP 404, `error.code == "STAFF_NOT_FOUND"`
  - **エラーメッセージの内容**: 指定スタッフが存在しない旨
  - **システムの安全性**: item_staffへINSERTされない、整合性維持
- **テストの目的**: 新規エラーコードSTAFF_NOT_FOUNDの動作確認
  - **品質保証の観点**: クライアントが原因を特定できる明示的エラー
- 🔵 信頼性レベル: TASK-0020 完了条件・テストケース5 / staff-requirements.md 2.2・4.3 に明記

### TC-E-03: 存在しないitem_idで紐付け → 404 ITEM_NOT_FOUND

- **テスト名**: 不存在のitem_idでの紐付けはITEM_NOT_FOUNDを返す
  - **エラーケースの概要**: パスのitem idがDBに存在しない
  - **エラー処理の重要性**: itemとstaffの両方の存在確認が必要
- **入力値**: パス `id=<不存在item UUID>`, ボディ `{ "staff_id": "<既存staff UUID>", "role": "監督" }`
  - **不正な理由**: itemsテーブルに該当行がない
  - **実際の発生シナリオ**: 削除済みitemへの紐付け試行
- **期待される結果**: HTTP 404, `error.code == "ITEM_NOT_FOUND"`
  - **エラーメッセージの内容**: 指定アイテムが存在しない旨
  - **システムの安全性**: INSERTに到達せず整合性維持
- **テストの目的**: 既存ITEM_NOT_FOUNDコードの流用確認
  - **品質保証の観点**: item/staffで異なるエラーコードを返し原因を区別
- 🟡 信頼性レベル: staff-requirements.md 2.2・4.3（🟡、ITEM_NOT_FOUNDは既存流用）

### TC-E-04: 存在しないitem_staff_idで削除 → 404

- **テスト名**: 不存在のitem_staff_idでの削除は404を返す
  - **エラーケースの概要**: 削除対象のitem_staff.idが存在しない
  - **エラー処理の重要性**: 影響行0件を成功扱いにせず明示的に404とする
- **入力値**: パス `id=<既存item UUID>`, `item_staff_id=<不存在UUID>`
  - **不正な理由**: item_staffテーブルに該当行なし
  - **実際の発生シナリオ**: 二重削除、誤ったID指定
- **期待される結果**: HTTP 404
  - **エラーメッセージの内容**: 該当紐付けが存在しない旨
  - **システムの安全性**: DELETEの影響行0件を検出して404、サイレント成功を回避
- **テストの目的**: 削除時の存在確認を検証
  - **品質保証の観点**: べき等でない誤操作をクライアントへ通知
- 🟡 信頼性レベル: TASK-0020 完了条件 / staff-requirements.md 4.3（🟡）

### TC-E-05: item_idに属さないitem_staff_idで削除 → 404（整合性チェック）

- **テスト名**: item_staffは存在するが別itemに属する場合は404を返す
  - **エラーケースの概要**: item_staff.idは存在するが、パスのitem_idと不一致
  - **エラー処理の重要性**: 他itemの紐付けを誤って削除する事故を防ぐ整合性ガード
- **入力値**: パス `id=<itemA UUID>`, `item_staff_id=<itemBに属するitem_staff.id>`
  - **不正な理由**: item_staff.item_id != パスのitem_id
  - **実際の発生シナリオ**: クライアントのID取り違え、URL改ざん
- **期待される結果**: HTTP 404、itemBの紐付けは削除されず残存
  - **エラーメッセージの内容**: 該当item配下に紐付けが存在しない旨
  - **システムの安全性**: 別itemのデータを保護、誤削除を防止
- **テストの目的**: item_id整合性チェックの確認
  - **品質保証の観点**: 横断的な不正削除の防止（権限・整合性の境界）
- 🟡 信頼性レベル: TASK-0020 実装詳細3 / staff-requirements.md 4.3（🟡、妥当推測）

### TC-E-06: role空文字で紐付け → 400 VALIDATION_ERROR

- **テスト名**: 空のroleはバリデーションエラーになる
  - **エラーケースの概要**: 必須項目roleが空文字
  - **エラー処理の重要性**: roleはNOT NULL・空文字不可
- **入力値**: `{ "staff_id": "<既存staff UUID>", "role": "" }`
  - **不正な理由**: `role VARCHAR(100) NOT NULL`、空文字不可
  - **実際の発生シナリオ**: 役割未選択のまま送信
- **期待される結果**: HTTP 400, `error.code == "VALIDATION_ERROR"`
  - **エラーメッセージの内容**: roleが必須である旨
  - **システムの安全性**: INSERT前に早期リターン、DB不変
- **テストの目的**: role必須バリデーションの確認
  - **品質保証の観点**: 役割なし紐付けの混入防止
- 🔵 信頼性レベル: staff-requirements.md 2.2・4.3 に明記

### TC-E-07: 不正なUUID形式のstaff_idで紐付け → 400 VALIDATION_ERROR

- **テスト名**: UUIDとして解釈不能なstaff_idは400になる
  - **エラーケースの概要**: staff_idがUUID形式でない
  - **エラー処理の重要性**: 形式不正をDB到達前に弾く
- **入力値**: `{ "staff_id": "not-a-uuid", "role": "監督" }`
  - **不正な理由**: UUIDパースに失敗する文字列
  - **実際の発生シナリオ**: 手入力ミス、不正なクライアント
- **期待される結果**: HTTP 400, `error.code == "VALIDATION_ERROR"`（serdeデシリアライズ失敗 or parse段階で検出）
  - **エラーメッセージの内容**: staff_idの形式が不正である旨
  - **システムの安全性**: クエリ実行に到達しない
- **テストの目的**: UUID形式バリデーションの確認
  - **品質保証の観点**: 型不正の早期検出
- 🟡 信頼性レベル: staff-requirements.md 2.2・4.3「不正UUID → 400」（🟡）

### TC-E-08: DBエラー時に内部情報を漏らさない

- **テスト名**: DB障害時にApiErrorへ変換され内部詳細を返さない
  - **エラーケースの概要**: sqlx::Errorが発生（接続断・予期せぬSQLSTATE等）
  - **エラー処理の重要性**: DB内部情報の漏洩はセキュリティリスク
- **入力値**: （repository層で意図的にエラーを発生させる、または接続断を模す）
  - **不正な理由**: 予期しないDBエラーはクライアントへ詳細を返してはいけない
  - **実際の発生シナリオ**: DB一時障害、制約違反の予期せぬケース
- **期待される結果**: HTTP 500, `error.code == "INTERNAL_ERROR"`、レスポンスにSQLメッセージを含まない。詳細は`tracing::error!`でサーバーログのみ
  - **エラーメッセージの内容**: 汎用的な内部エラーメッセージのみ
  - **システムの安全性**: 内部情報の秘匿（item_repository.rs `db_error`パターン踏襲）
- **テストの目的**: DBエラー変換の安全性確認
  - **品質保証の観点**: 情報漏洩防止というセキュリティ要件の担保
- 🔵 信頼性レベル: note.md セクション2/6・staff-requirements.md 3 セキュリティ要件に明記

---

## 3. 境界値テストケース（最小値、最大値、null等）

### TC-B-01: name 255文字ちょうどで作成成功

- **テスト名**: nameが上限255文字で作成できる
  - **境界値の意味**: `VARCHAR(255)`の上限ちょうど
  - **境界値での動作保証**: 上限内なら正常に保存される
- **入力値**: `{ "name": "<255文字の文字列>" }`
  - **境界値選択の根拠**: NOT NULL列の最大長境界
  - **実際の使用場面**: 非常に長いスタッフ名・別名併記
- **期待される結果**: HTTP 201, `data.name`が255文字で保存される
  - **境界での正確性**: 切り詰めや拒否が起きない
  - **一貫した動作**: 254文字でも256文字超でも一貫した境界判定
- **テストの目的**: name最大長の正常境界確認
  - **堅牢性の確認**: 上限ぎりぎりで安定動作
- 🟡 信頼性レベル: database-schema.sql `VARCHAR(255)`から妥当推測（要件に明示の上限検証はnameは記載薄め）

### TC-B-02: role 100文字ちょうどで紐付け成功 / 101文字で400

- **テスト名**: roleが上限100文字で成功し、超過で400になる
  - **境界値の意味**: `VARCHAR(100)`かつ要件「role上限100文字」の境界
  - **境界値での動作保証**: 100=成功、101=VALIDATION_ERROR
- **入力値**: (a) `role`=100文字 → 201、(b) `role`=101文字 → 400
  - **境界値選択の根拠**: staff-requirements.md 3「role上限100文字」を直接検証
  - **実際の使用場面**: 長い役割名の入力
- **期待される結果**: (a) HTTP 201で保存、(b) HTTP 400 `VALIDATION_ERROR`
  - **境界での正確性**: ちょうど100は通し、101は弾く
  - **一貫した動作**: 境界の内外で判定が反転する
- **テストの目的**: role長さ制限の境界確認
  - **堅牢性の確認**: 上限超過の確実な拒否
- 🔵 信頼性レベル: staff-requirements.md 3 入力検証制約「role上限100文字」に明記

### TC-B-03: character_name 255文字ちょうどで成功 / 256文字で400

- **テスト名**: character_nameが上限255文字で成功し、超過で400になる
  - **境界値の意味**: `VARCHAR(255)`かつ要件「character_name上限255文字」の境界
  - **境界値での動作保証**: 255=成功、256=VALIDATION_ERROR
- **入力値**: (a) `character_name`=255文字 → 201、(b) `character_name`=256文字 → 400
  - **境界値選択の根拠**: staff-requirements.md 3「character_name上限255文字」を直接検証
  - **実際の使用場面**: 長いキャラクター名
- **期待される結果**: (a) HTTP 201で保存、(b) HTTP 400 `VALIDATION_ERROR`
  - **境界での正確性**: 255は通し、256は弾く
  - **一貫した動作**: 境界の内外で判定が反転する
- **テストの目的**: character_name長さ制限の境界確認
  - **堅牢性の確認**: 上限超過の確実な拒否
- 🔵 信頼性レベル: staff-requirements.md 3 入力検証制約「character_name上限255文字」に明記

### TC-B-04: optionalフィールドのnull/未指定の扱い

- **テスト名**: external_id/image_url/character_nameの未指定でnullとして扱われる
  - **境界値の意味**: optional値の「無」境界（未指定 vs 明示null）
  - **境界値での動作保証**: 未指定でもnull明示でも結果が同一
- **入力値**: (a) フィールド省略、(b) `{ "external_id": null, "image_url": null }`
  - **境界値選択の根拠**: NULLable列のnull/未指定の同値性確認
  - **実際の使用場面**: クライアントがフィールドを送らない/明示nullを送る両ケース
- **期待される結果**: いずれもHTTP 201、対象フィールドがnullで保存される
  - **境界での正確性**: 未指定とnullで挙動が一致
  - **一貫した動作**: serde `Option<T>`のデフォルト挙動と整合
- **テストの目的**: optionalフィールドのnull扱いの一貫性確認
  - **堅牢性の確認**: 欠落入力でもエラーにならない
- 🔵 信頼性レベル: staff-requirements.md 2.1・4.4 / note.md（NULLable）に基づく

### TC-B-05: カスケード削除（統合テスト）

- **テスト名**: staff削除時に関連item_staffがCASCADE削除される
  - **境界値の意味**: FK `ON DELETE CASCADE`の連鎖削除境界
  - **境界値での動作保証**: 親staff削除で子item_staffが自動消去される
- **入力値**: staff作成 → item_staff紐付け → staffをDBから削除
  - **境界値選択の根拠**: database-schema.sql `staff_id ... ON DELETE CASCADE`の実DB挙動確認
  - **実際の使用場面**: スタッフエンティティ削除時の関連データ整合
- **期待される結果**: staff削除後、当該staff_idを参照するitem_staffがすべて消えている
  - **境界での正確性**: 孤立した参照行が残らない
  - **一貫した動作**: DBレベルの参照整合性が保たれる
- **テストの目的**: CASCADE制約の実DB動作確認
  - **堅牢性の確認**: 参照整合性の自動維持
- 🟡 信頼性レベル: TASK-0020 統合テスト要件 / staff-requirements.md 4.3（🟡）

### TC-B-06: normalize系・空ボディ/不正JSON → 400

- **テスト名**: 空ボディや壊れたJSONで紐付け/作成すると400になる
  - **境界値の意味**: リクエストボディの最小・破損境界
  - **境界値での動作保証**: パース不能入力を安全に拒否
- **入力値**: (a) 空ボディ `` 、(b) 壊れたJSON `{ "name": ` 、(c) 必須キー欠落 `{}`
  - **境界値選択の根拠**: serdeデシリアライズの失敗境界
  - **実際の使用場面**: クライアント不具合、通信途中切断
- **期待される結果**: HTTP 400 `VALIDATION_ERROR`（または適切な4xx）、パニックせず安全に応答
  - **境界での正確性**: いずれもINSERTに到達しない
  - **一貫した動作**: 破損入力で一貫して400系
- **テストの目的**: 入力デシリアライズの堅牢性確認
  - **堅牢性の確認**: 不正入力でサーバーが落ちない
- 🟡 信頼性レベル: 一般的なAPIバリデーション方針からの妥当推測（要件に明示なし）

---

## 4. 開発言語・フレームワーク

- **プログラミング言語**: Rust (Edition 2024)
  - **言語選択の理由**: 既存mediavault-apiがRust実装（note.md 技術スタック）。型安全・所有権による堅牢性
  - **テストに適した機能**: `#[test]`標準テスト、`Result`/`Option`による明示的エラー、コンパイル時保証
- **テストフレームワーク**: Rust標準テスト（`#[test]` / 非同期は `#[tokio::test]`）、DB統合は `tests/integration_tests.rs`、sqlxはコンパイル時SQLチェック
  - **フレームワーク選択の理由**: note.md セクション5に明記（既存TASK-0009と同一構成）
  - **テスト実行環境**: `cargo test`。統合テストはテスト用DB（`.env.test`）に接続し、各テスト後にDELETEで清掃
- 🔵 信頼性レベル: note.md セクション1・5 に明記

### テスト分類と実行レイヤー

| 分類 | テストケース | レイヤー | DB依存 |
|---|---|---|---|
| models単体（バリデーション） | TC-N-06, TC-E-01, TC-E-06, TC-E-07, TC-B-02(判定), TC-B-03(判定) | `models/staff.rs` parse関数 | なし |
| handlers/repository統合 | TC-N-01〜05, TC-E-02〜05, TC-E-08, TC-B-01, TC-B-04, TC-B-06 | handlers→repo→DB | あり |
| 統合（CASCADE） | TC-B-05 | repo→実DB | あり |

---

## 5. テストケース実装時の日本語コメント指針（例）

### 例: TC-N-01 スタッフ作成（統合テスト・Rust）

```rust
#[tokio::test]
async fn test_create_staff_with_required_fields_only() {
    // 【テスト目的】: nameのみでスタッフ作成が成功し201でUUID付きStaffが返ることを確認
    // 【テスト内容】: POST /staff に { "name": "監督A" } を送信
    // 【期待される動作】: staffテーブルへINSERTされ、external_id/image_urlはnull
    // 🔵 信頼性レベル: TASK-0020 テストケース1 に明記

    // 【テストデータ準備】: クリーンなテストDBプールを用意（前テストの残骸なし）
    // 【初期条件設定】: staffテーブルに当該nameの行が存在しない状態
    let pool = setup_test_pool().await;

    // 【実際の処理実行】: create_staff_handler 相当のリクエストを実行
    // 【処理内容】: バリデーション→repository create_staff→201応答
    let resp = post_staff(&pool, json!({ "name": "監督A" })).await;

    // 【結果検証】: ステータスとレスポンスボディを検証
    // 【期待値確認】: 201かつ data.id がUUID、optionalフィールドはnull
    assert_eq!(resp.status(), 201); // 【検証項目】: 作成成功ステータス 🔵
    let body = resp.json().await;
    assert!(Uuid::parse_str(body["data"]["id"].as_str().unwrap()).is_ok()); // 【確認内容】: idがUUID形式 🔵
    assert!(body["data"]["external_id"].is_null()); // 【確認内容】: external_id未指定はnull 🔵
    assert_eq!(body["data"]["name"], "監督A"); // 【確認内容】: name保持 🔵

    // 【テスト後処理】: 作成したstaff行をDELETEで清掃し次テストへ影響させない
    cleanup_staff(&pool).await;
}
```

### 例: TC-E-01 name空文字バリデーション（models単体・Rust）

```rust
#[test]
fn test_parse_create_staff_request_rejects_empty_name() {
    // 【テスト目的】: 空nameがVALIDATION_ERRORで弾かれることを確認
    // 【テスト内容】: parse_create_staff_request に空文字nameを渡す
    // 【期待される動作】: Err(ApiError{ code: VALIDATION_ERROR }) が返る
    // 🔵 信頼性レベル: staff-requirements.md 2.1 に明記

    // 【テストデータ準備】: 空文字nameのリクエストDTOを構築（DB不要のpure関数テスト）
    let req = CreateStaffRequest { name: "".into(), external_id: None, image_url: None };

    // 【実際の処理実行】: バリデーション関数を直接呼び出す
    let result = parse_create_staff_request(req);

    // 【結果検証】: Errであり、エラーコードがVALIDATION_ERRORであること
    assert!(result.is_err()); // 【検証項目】: 空nameは拒否される 🔵
    assert_eq!(result.unwrap_err().code, ApiErrorCode::ValidationError); // 【確認内容】: 正しいエラーコード 🔵
}
```

---

## 6. 要件定義との対応関係

- **参照した機能概要**: staff-requirements.md 1（POST /staff, POST /items/:id/staff, DELETE /items/:id/staff/:item_staff_id の3エンドポイント）
- **参照した入力・出力仕様**: staff-requirements.md 2.1〜2.3（入力/出力/エラーコード）
- **参照した制約条件**: staff-requirements.md 3（DB制約、STAFF_NOT_FOUND新規追加、セキュリティ、長さ制限 role:100/character_name:255、name空不可）
- **参照した使用例**: staff-requirements.md 4.1〜4.4（基本パターン・正常系・エッジ/エラーケース・注意事項）
- **参照したタスク要件**: TASK-0020.md 完了条件5項目・単体テスト要件5ケース・統合テスト要件1ケース
- **参照した技術コンテキスト**: note.md（Rust/Axum/sqlx、レイヤード構成、parse_*パターン、db_error、テスト構成）

---

## 7. テストケース一覧サマリー

| カテゴリ | ケース数 | テストID |
|---|---|---|
| 正常系 | 6 | TC-N-01〜06 |
| 異常系 | 8 | TC-E-01〜08 |
| 境界値 | 6 | TC-B-01〜06 |
| **合計** | **20** | |

### 信頼性レベル分布

| 信頼性 | 件数 | 主な根拠 |
|---|---|---|
| 🔵 青信号 | 11 | TASK-0020完了条件・テストケース、staff-requirements.md 明記事項、note.md 技術スタック |
| 🟡 黄信号 | 9 | DELETE整合性チェック、CASCADE、UUID/長さ境界推測、空ボディ等の妥当推測 |
| 🔴 赤信号 | 0 | なし |

---

## 品質判定

| 評価項目 | 状態 |
|---|---|
| テストケース分類 | ✅ 正常系・異常系・境界値を網羅（20ケース） |
| 期待値定義 | ✅ 各ケースにHTTPステータス・エラーコード・保存値を明示 |
| 技術選択 | ✅ Rust + 標準テスト/tokio::test/sqlx統合に確定 |
| 実装可能性 | ✅ 既存items/tags CRUDパターンを踏襲可能 |
| 信頼性レベル | ✅ 🔵多数・🟡一部・🔴ゼロ |

**総合評価**: ✅ 高品質

---

## 次のステップ

次のお勧めステップ: `/tsumiki:tdd-red mediavault-backend TASK-0020` でRedフェーズ（失敗テスト作成）を開始します。
