# mediavault-backend 受け入れ基準

**作成日**: 2026-06-21
**関連要件定義**: [requirements.md](requirements.md)
**関連ユーザストーリー**: [user-stories.md](user-stories.md)
**ヒアリング記録**: [interview-record.md](interview-record.md)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: PRD・tech-stack・ヒアリングを参考にした確実な基準
- 🟡 **黄信号**: PRD・tech-stack・ヒアリングから妥当な推測による基準
- 🔴 **赤信号**: PRD・tech-stack・ヒアリングにない推測による基準

---

## REQ-001/REQ-003: アイテムCRUD（手動追加含む） 🔵

**信頼性**: 🔵 *PRD「作品手動追加」「作品編集・削除」より*

### Given（前提条件）
- DBが起動済みで `items` テーブルが存在する

### When（実行条件）
- `POST /items` に media_type・title等を含むJSONを送信する

### Then（期待結果）
- `source=manual`、`external_id=NULL` の `items` レコードが作成され、IDが返却される

### テストケース

#### 正常系
- [ ] **TC-001-01**: 必須項目（media_type, title）のみで作成できる 🔵
  - **入力**: `{"media_type":"anime","title":"作品A"}`
  - **期待結果**: 201、レスポンスにUUID付きitemが返る
- [ ] **TC-001-02**: 既存アイテムをPATCHで部分更新できる 🔵
  - **入力**: `{"rating":4.5,"is_favorite":true}`
  - **期待結果**: 200、指定フィールドのみ更新される
- [ ] **TC-001-03**: アイテムをDELETEで削除できる 🔵
  - **期待結果**: 204、関連する item_tags/item_links等もカスケード削除される

#### 異常系
- [ ] **TC-001-E01**: media_type が enum値以外の場合は400 🔵
  - **信頼性**: 🔵 *NFR-102入力検証より*
- [ ] **TC-001-E02**: 存在しないIDへのPATCH/DELETEは404 🟡
  - **信頼性**: 🟡 *一般的なREST API挙動から推測*

#### 境界値
- [ ] **TC-001-B01**: titleが空文字の場合は400 🟡

---

## REQ-002: 外部API検索連携 🔵

**信頼性**: 🔵 *PRD「作品検索・追加（API）」より*

### Given（前提条件）
- 対象media_typeに必要な外部APIキーが登録済み（不要なAPI=Jikanは除く）

### When（実行条件）
- `GET /items/search?media_type=anime&q=タイトル` を呼ぶ

### Then（期待結果）
- media_typeに対応する外部API（Jikan等）を呼び、検索結果一覧を返す

### テストケース

#### 正常系
- [ ] **TC-002-01**: anime検索でJikan APIの結果が返る 🔵
- [ ] **TC-002-02**: movie/drama検索でTMDb APIの結果が返る 🔵
- [ ] **TC-002-03**: 検索結果から `POST /items/import` でitemを作成できる 🔵

#### 異常系
- [ ] **TC-002-E01**: 必要なAPIキーが未登録の場合は422でエラーメッセージを返す 🟡
  - **信頼性**: 🟡 *EDGE-001より妥当な推測*
- [ ] **TC-002-E02**: 外部APIタイムアウト時は502相当のエラーを返し、サーバーはクラッシュしない 🟡

---

## REQ-015/NFR-202: 外部APIキー管理 🔵

**信頼性**: 🔵 *ヒアリングQ5より*

### Given（前提条件）
- なし（初回登録）

### When（実行条件）
- `PUT /settings/api-keys/{provider}` にキー文字列を送信する

### Then（期待結果）
- DBにキーが保存（暗号化は本フェーズ対象外、平文保存で良い）され、以後の外部API呼び出しに使用される

### テストケース
- [ ] **TC-015-01**: TMDbキーを登録後、movie検索が成功する 🔵
- [ ] **TC-015-02**: 不正なproviderを指定すると400 🟡
- [ ] **TC-015-03**: キーを更新すると以後の呼び出しで新キーが使われる 🔵

---

## REQ-010/REQ-101/EDGE-101: シーズン・話数管理 🔵

**信頼性**: 🔵 *PRDデータモデルより*

### Given（前提条件）
- media_type=animeまたはdramaの親itemが存在する

### When（実行条件）
- `POST /items/{id}/groups` で group_type=season を作成し、`POST /groups/{id}/episodes` で話数を追加する

### Then（期待結果）
- season配下にepisodeが登録され、一覧表示APIで取得できる

### テストケース

#### 正常系
- [ ] **TC-010-01**: シーズン作成後、話数を複数登録できる 🔵
- [ ] **TC-010-02**: シーズン・話数一覧をorder/episode_number順に取得できる 🔵

#### 異常系
- [ ] **TC-010-E01**: group_type=volumeのグループに `item_episodes` を追加しようとすると400で拒否される 🔵
  - **信頼性**: 🔵 *EDGE-101より*

---

## REQ-011/REQ-102: 巻管理（漫画・小説） 🔵

**信頼性**: 🔵 *PRDデータモデルより*

### テストケース
- [ ] **TC-011-01**: group_type=volumeでitem_groupsを作成できる 🔵
- [ ] **TC-011-E01**: volume配下にitem_episodesを追加すると拒否される（TC-010-E01と同等） 🔵

---

## REQ-013: DLC関連付け 🔵

**信頼性**: 🔵 *PRDメディア別機能より*

### テストケース
- [ ] **TC-013-01**: `item_relations`にrelation_type=dlcでDLCアイテムを本体に紐付けできる 🔵
- [ ] **TC-013-02**: relation_type=referenceで他メディア作品への引用関連を作成できる 🔵

---

## REQ-016/REQ-017/EDGE-002: インポート機能 🔵

**信頼性**: 🔵 *PRD・ヒアリングQ3より*

### Given（前提条件）
- ブクログCSVファイルまたはSteam Web APIキー+SteamIDが用意されている

### When（実行条件）
- `POST /import/booklog`（multipart CSV）または `POST /import/steam`（steam_id指定）を呼ぶ

### Then（期待結果）
- 正常行はitemとして登録され、結果サマリー（成功数・失敗数・失敗理由）が返る

### テストケース

#### 正常系
- [ ] **TC-016-01**: ブクログCSVの正常データが全件インポートされる 🔵
- [ ] **TC-017-01**: Steamライブラリの所持ゲームが全件インポートされる（game_details.steam_appid紐付け） 🔵

#### 異常系
- [ ] **TC-016-E01**: CSV内の一部行が形式不正でも、正常行のみ取込まれ不正行はスキップされ理由がレスポンスに含まれる 🟡
  - **信頼性**: 🟡 *EDGE-002より*
- [ ] **TC-017-E01**: Steam APIキーが無効な場合401相当のエラーを返す 🟡

---

## REQ-018/REQ-403/NFR-101: 内部REST API認証 🔵

**信頼性**: 🔵 *PRD・tech-stackより*

### Given（前提条件）
- 内部APIキーが環境変数に設定済み

### When（実行条件）
- `Authorization: Bearer {key}` ヘッダー付きで内部APIエンドポイントを呼ぶ

### Then（期待結果）
- キーが一致する場合のみ処理を実行し、不一致・欠落時は401を返す

### テストケース
- [ ] **TC-018-01**: 正しいAPIキーでアイテム登録・検索ができる 🔵
- [ ] **TC-018-E01**: APIキーなしでアクセスすると401 🔵
- [ ] **TC-018-E02**: 誤ったAPIキーでアクセスすると401 🔵

---

## REQ-007/REQ-019/REQ-104: ファイル登録・アップロード 🔵

**信頼性**: 🔵 *PRD・ヒアリングQ2より*

### Given（前提条件）
- ファイルサーバーのパス（`/srv/files/pdf`または`/srv/media/photos`）がバインドマウント済み

### When（実行条件）
- (a) `POST /items/{id}/files` にpath・file_type・labelを指定する、または
- (b) `POST /items/{id}/files/upload` にmultipartでバイナリを送信する

### Then（期待結果）
- (a) 指定パスを `item_files.path` として登録する
- (b) サーバーがfile_typeに応じたディレクトリにファイルを配置し、配置後の相対パスを`item_files.path`に保存する

### テストケース

#### 正常系
- [ ] **TC-007-01**: 既存パス指定でitem_filesレコードが作成される 🔵
- [ ] **TC-019-01**: バイナリ直接アップロードでファイルが配置され、相対パスがDBに保存される 🔵

#### 異常系
- [ ] **TC-019-E01**: ファイルサーバーへの書き込みに失敗した場合、item_filesレコードは作成されない（ロールバック） 🟡
  - **信頼性**: 🟡 *EDGE-003より*

---

## REQ-020/REQ-103: Calibre-Web連携 🔵

**信頼性**: 🔵 *PRDより*

### テストケース
- [ ] **TC-020-01**: file_type=pdfのitem_filesに対し`calibre_book_id`をPATCHで更新できる 🔵
- [ ] **TC-020-02**: calibre_book_idが設定されたPDFは詳細APIレスポンスにCalibre-Web遷移用情報を含む 🟡

---

## 非機能要件テスト

### NFR-101/REQ-403: セキュリティ 🔵

- [ ] **TC-NFR-101-01**: 内部APIキー検証ミドルウェアが全内部エンドポイントに適用されている 🔵
- [ ] **TC-NFR-102-01**: 不正な型・欠落フィールドを含むリクエストは400で拒否される（Axum extractorによるバリデーション） 🔵

### NFR-001/NFR-002: パフォーマンス 🟡

- [ ] **TC-NFR-002-01**: 1000件規模のitemsに対する一覧・絞り込みAPIが1秒以内に応答する 🟡
  - **信頼性**: 🟡 *目標値はPRD/tech-stackに明記なし、妥当な推測*

---

## Edgeケーステスト

### EDGE-101: volumeへのepisode登録拒否 🔵

- [ ] **TC-EDGE-101-01**: group_type=volumeのグループIDに対し`POST /groups/{id}/episodes`を呼ぶと400 🔵

### EDGE-001: 外部APIキー未設定時の挙動 🟡

- [ ] **TC-EDGE-001-01**: APIキー未設定provider検索時、手動追加APIは引き続き利用可能であることを確認 🟡

---

## テストケースサマリー

### カテゴリ別件数

| カテゴリ | 正常系 | 異常系 | 境界値 | 合計 |
|---------|--------|--------|--------|------|
| 機能要件 | 16 | 11 | 1 | 28 |
| 非機能要件 | 1 | 1 | 0 | 2 |
| Edgeケース | 0 | 2 | 0 | 2 |
| **合計** | 17 | 14 | 1 | 32 |

### 信頼性レベル分布

- 🔵 青信号: 22件 (69%)
- 🟡 黄信号: 10件 (31%)
- 🔴 赤信号: 0件 (0%)

**品質評価**: 高品質（赤信号なし、黄信号は実装時に確定すべき細部のみ）

### 優先度別テストケース

- **Must Have**: 20件（アイテムCRUD・外部API検索・グループ/エピソード・インポート・内部API・APIキー管理）
- **Should Have**: 8件（ファイル管理・スタッフ・関連付け）
- **Could Have**: 4件（Calibre-Web連携・マイリスト）

---

## テスト実施計画

### Phase 1: コア機能（Must Have）
- REQ-001〜003, REQ-010〜011, REQ-015〜018, REQ-403
- 実施予定: 設計・実装完了後、初回リリース前

### Phase 2: 拡張機能（Should Have）
- REQ-004〜009, REQ-013, REQ-019
- 実施予定: Phase 1完了後

### Phase 3: 非機能・Edgeケース・Could Have
- NFR-001〜202, EDGE-001〜102, REQ-020, REQ-301/302
- 実施予定: Phase 2完了後、または次フェーズ
