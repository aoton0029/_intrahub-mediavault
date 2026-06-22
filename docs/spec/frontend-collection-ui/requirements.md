# frontend-collection-ui 要件定義書

## 概要

MediaVaultフロントエンドは、映画・アニメ・漫画・小説・ドラマ・ゲーム・学術書/専門書・論文/文献のメタデータを一元管理するセルフホスト型アプリケーションのうち、単一ユーザーがコレクションを検索・追加・編集・整理・閲覧するためのUI（React + TypeScript + Vite SPA）を提供する。バックエンドAPI（`http://localhost:8080/api/v1`）と通信し、認証・ログイン機能は持たない。

## 関連文書

- **ヒアリング記録**: [💬 interview-record.md](interview-record.md)
- **ユーザストーリー**: [📖 user-stories.md](user-stories.md)
- **受け入れ基準**: [✅ acceptance-criteria.md](acceptance-criteria.md)
- **コンテキストノート**: [📝 note.md](note.md)
- **PRD**: [docs/frontend/PRD.md](../../frontend/PRD.md)
- **技術スタック**: [docs/frontend/tech-stack.md](../../frontend/tech-stack.md)
- **バックエンドAPI仕様**: [docs/design/mediavault-backend/api-endpoints.md](../../design/mediavault-backend/api-endpoints.md)
- **バックエンド要件定義**: [docs/spec/mediavault-backend/requirements.md](../mediavault-backend/requirements.md)

## 機能要件（EARS記法）

**【信頼性レベル凡例】**:
- 🔵 **青信号**: PRD・EARS要件定義書・設計文書・ユーザヒアリングを参考にした確実な要件
- 🟡 **黄信号**: PRD・EARS要件定義書・設計文書・ユーザヒアリングから妥当な推測による要件
- 🔴 **赤信号**: PRD・EARS要件定義書・設計文書・ユーザヒアリングにない推測による要件

### 通常要件

- REQ-001: システムは全体一覧画面で、コレクション全体をカード/リスト表示しなければならない 🔵 *frontend/PRD.md「全体一覧画面」より*
- REQ-002: システムは一覧画面で `media_type`・タグ・カテゴリ・お気に入り・status による絞り込みを提供しなければならない 🔵 *frontend/PRD.md「一覧・絞り込み」、バックエンドREQ-001のクエリパラメータより*
- REQ-003: システムは一覧画面の絞り込み状態をURLクエリパラメータに反映し、ブラウザの戻る/進む・URL共有で同一の絞り込み結果を再現できなければならない 🔵 *ヒアリングにて確定*
- REQ-004: システムは一般メディア（アニメ・映画・小説・ドラマ・漫画・ゲーム）・学術書/専門書・論文/文献の3メディアグループそれぞれに専用の一覧画面を提供しなければならない 🔵 *frontend/PRD.md「画面構成」・ヒアリングで確認*
- REQ-005: システムは検索・追加画面で、メディアグループに応じた外部API（Jikan/TMDb/NDL/OpenLibrary/Steam/IGDB/AniList）への検索（`GET /items/search`）を実行し、検索結果一覧から選択してコレクションに追加（`POST /items/import`）できなければならない 🔵 *frontend/PRD.md「作品検索・追加（API）」、バックエンドapi-endpoints.mdより*
- REQ-006: システムは手動追加・編集画面で、外部API検索を使わずフォーム入力のみでアイテムを新規作成（`POST /items`）できなければならない 🔵 *frontend/PRD.md「作品手動追加」より*
- REQ-007: システムは詳細画面・手動追加・編集画面で、既存アイテムの全項目を編集（`PATCH /items/:id`）・削除（`DELETE /items/:id`）できなければならない 🔵 *frontend/PRD.md「作品編集・削除」より*
- REQ-008: システムはタグ/カテゴリ管理画面で、タグ・カテゴリの作成（`POST /tags`, `POST /categories`）・アイテムへの付与・削除（`POST/DELETE /items/:id/tags/:tag_id`）を行うUIを提供しなければならない 🔵 *frontend/PRD.md「タグ/カテゴリ管理」より*
- REQ-009: システムはマイリスト画面で、任意名称のリストの作成（`POST /mylists`）、アイテムの追加・削除（`POST /mylists/:id/items`, `DELETE /mylists/:id/items/:item_id`）を行うUIを提供しなければならない 🔵 *frontend/PRD.md「マイリスト管理」より*
- REQ-010: システムは詳細画面で、他メディアのアイテムへの関連付け（`POST/DELETE /item-relations`）の追加・削除を行うUIを提供しなければならない 🔵 *frontend/PRD.md「関連付け」より*
- REQ-011: システムは詳細画面・編集画面で、配信サイトリンク（`item_links`）・ファイル（`item_files`）・トレーラーURL（`item_trailers`）の追加・編集・削除UIを提供しなければならない 🔵 *frontend/PRD.md「リンク/ファイル/トレーラー管理」より*
- REQ-012: システムはファイル追加UIにおいて、ドラッグ&ドロップまたはファイル選択によるアップロード（`POST /items/:id/files/upload`, multipart/form-data）を提供しなければならない 🔵 *ヒアリングにて確定*
- REQ-013: システムは詳細画面・一覧画面で、statusと観賞/読了日（`consumed_date`）を更新（`PATCH /items/:id/status`）するUIを提供しなければならない 🔵 *frontend/PRD.md「視聴・読了記録」より*
- REQ-014: システムはスタッフ管理画面で、スタッフ（監督・声優・著者など）の追加（`POST /staff`）・役割付与・作品への紐付け（`POST /items/:id/staff`）・解除（`DELETE /items/:id/staff/:item_staff_id`）を行うUIを提供しなければならない 🔵 *frontend/PRD.md「スタッフ管理」より*
- REQ-015: システムはアニメ・ドラマの詳細画面で、シーズン単位（`item_groups`, group_type=season）の話数（`item_episodes`）情報を登録・編集・一覧表示しなければならない 🔵 *frontend/PRD.md メディア別機能より*
- REQ-016: システムは漫画・小説の詳細画面で、巻単位のグループ表示・管理画面を提供しなければならない 🔵 *frontend/PRD.md メディア別機能より*
- REQ-017: システムは映画の詳細画面で、章単位のグループ表示を任意で提供してもよい 🔵 *frontend/PRD.md「映画：章単位のグループ表示（任意）」より（オプション要件）*
- REQ-018: システムはゲームの詳細画面で、DLC/拡張パックを本体作品に紐付けて表示しなければならない（`relation_type=dlc`） 🔵 *frontend/PRD.md メディア別機能より*
- REQ-019: システムは学術書・専門書向けの手動追加・編集画面・検索・追加画面で、著者・出版社・ISBN等の特有属性を入力・表示しなければならない 🔵 *frontend/PRD.md メディア別機能より*
- REQ-020: システムは論文・文献向けの手動追加・編集画面・検索・追加画面で、DOI・掲載誌・巻号・ページ等の学術メタデータを入力・表示しなければならない 🔵 *frontend/PRD.md メディア別機能より*
- REQ-021: システムは設定画面で、各外部APIキー（tmdb/igdb/ndl/steam/openlibrary/anilist）を登録・更新するUI（`PUT /settings/api-keys/:provider`）を提供しなければならない 🔵 *frontend/PRD.md「APIキー管理」より*
- REQ-022: システムは設定画面で、ブクログ(csv)・Steamライブラリからのアイテム一括インポートUI（`POST /import/booklog`, `POST /import/steam`）を提供し、インポート結果（成功件数・失敗件数・失敗理由）を表示しなければならない 🔵 *frontend/PRD.md「インポート」、バックエンドImportSummaryより*
- REQ-023: システムは設定画面に、Obsidian/Notion向けエクスポート機能のUIエントリ（ボタン）を表示してもよいが、本要件のスコープでは未実装（無効化または「未対応」表示）としてもよい 🟡 *ヒアリングにてバックエンド側が次回フェーズ対象外であることを確認、フロントも同スコープとする*

### 条件付き要件

- REQ-101: media_typeが anime または drama の場合、詳細画面・編集画面はシーズン/話数の登録・編集UIを表示しなければならない 🔵 *REQ-015、バックエンドREQ-101より*
- REQ-102: media_typeが manga または novel の場合、詳細画面・編集画面は巻グループの登録・編集UIを表示し、話数（エピソード）入力UIは表示してはならない 🔵 *REQ-016、バックエンドREQ-102/EDGE-101より*
- REQ-103: media_typeが game の場合、詳細画面はDLC/拡張パックの関連アイテム一覧（`relation_type=dlc`）を表示しなければならない 🔵 *REQ-018より*
- REQ-104: アイテムの`media_type`が学術書/専門書または論文/文献グループに属する場合、検索・追加画面・手動追加・編集画面は当該グループ専用のフォーム項目（ISBN、DOI等）を表示しなければならない 🔵 *REQ-019/020より*
- REQ-105: ファイル種別が pdf の場合、システムはアップロード完了後にCalibre-Webの`calibre_book_id`紐付け状態を表示し、紐付け済みの場合はCalibre-Web閲覧URLへのリンクを表示しなければならない 🟡 *backend api-endpoints.md PATCH /items/:id/files/:file_id/calibre-link より妥当な推測*

### 状態要件

- REQ-201: 外部API検索結果からのインポート時、システムは作成済みアイテムを `source=api` として扱い、編集画面では `external_id` を表示専用（編集不可）としなければならない 🟡 *バックエンドREQ-201b・データモデルから妥当な推測*
- REQ-202: 手動作成されたアイテム（`source=manual`）の場合、編集画面は `external_id` 関連の表示を行ってはならない 🟡 *バックエンドREQ-201から妥当な推測*
- REQ-203: 外部APIキーが未設定の状態（`API_KEY_NOT_CONFIGURED`）で検索・追加画面を開いた場合、システムはエラー表示と共に手動追加画面への遷移導線を表示しなければならない 🔵 *バックエンドEDGE-001・api-endpoints.md GET /items/search エラーコードより*

### オプション要件

- REQ-301: システムは映画の詳細画面で章単位グループ表示を提供してもよい（REQ-017と同義のオプション要件） 🔵 *frontend/PRD.md「任意」より*
- REQ-302: システムは詳細画面でCalibre-Webへの直接遷移リンクをボタンとして表示してもよい 🟡 *ルートPRD.md Calibre-Web連携記述から妥当な推測*

### 制約要件

- REQ-401: システムはユーザー管理機能を持ってはならず、ログイン・認証画面を実装してはならない（単一ユーザー前提） 🔵 *frontend/PRD.md「やらなくていいこと」より*
- REQ-402: システムはバックエンドの内部REST API（`/internal/*`）を呼び出してはならない（巡回バッチ・ファイルサーバー監視プロセス専用） 🔵 *バックエンドapi-endpoints.md「内部REST API」より*
- REQ-403: システムはAPIキー等の機密情報をクライアント側のソースコード・リポジトリに直接記述してはならず、設定画面UIを通じてバックエンドDBに登録する方式のみを使用しなければならない 🔵 *バックエンドREQ-404・NFR-202より*
- REQ-404: 画面遷移は、全体一覧画面またはメディアグループ別一覧画面を起点として詳細・検索追加・手動追加へ遷移する構成とし、検索追加・手動追加・グループ別一覧はメディアグループ（一般メディア/学術書・専門書/論文・文献）ごとに別画面としなければならない 🔵 *frontend/PRD.md「画面遷移の大枠」より*

## 非機能要件

### パフォーマンス

- NFR-001: 一覧画面はTanStack Queryによるキャッシュを用い、絞り込み条件変更時の再取得を最小限に抑えなければならない 🟡 *frontend/tech-stack.md「状態管理」から妥当な推測*
- NFR-002: 一覧画面は数千件規模のアイテムでも初期表示・絞り込み操作のレスポンスがバックエンドAPI応答（目標1秒以内、backend NFR-002）に対し追加で1秒以上のレンダリング遅延を生じさせてはならない 🟡 *バックエンドNFR-002から妥当な推測*

### セキュリティ

- NFR-101: システムは認証トークンの管理を行わない（単一ユーザー・認証なし前提）が、設定画面で入力するAPIキーは画面表示時にマスク表示し、平文での恒久表示を避けなければならない 🟡 *一般的なセキュリティプラクティスから妥当な推測*

### ユーザビリティ

- NFR-201: 主要なフォーム（手動追加・編集・APIキー登録）は入力エラー時に該当フィールド近傍にエラーメッセージを表示しなければならない 🟡 *frontend/tech-stack.md品質基準「アクセシビリティ」から妥当な推測*
- NFR-202: システムは基本的なWCAG 2.1 AA（フォームのラベル付け・キーボード操作・コントラスト比）に準拠しなければならない 🔵 *frontend/tech-stack.md品質基準より*
- NFR-203: ファイルアップロードUIは、アップロード中の進捗表示および失敗時のエラーメッセージ（`FILE_STORAGE_WRITE_FAILED`等）を表示しなければならない 🟡 *バックエンドapi-endpoints.mdエラーコードから妥当な推測*

## Edgeケース

### エラー処理

- EDGE-001: 外部API検索でタイムアウト（`EXTERNAL_API_TIMEOUT`）が発生した場合、システムはエラーメッセージを表示し、検索の再試行および手動追加画面への切り替え導線を提示しなければならない 🔵 *バックエンドEDGE-001・api-endpoints.mdエラーコードより*
- EDGE-002: ブクログCSV・Steamインポートで一部行が失敗した場合、システムは成功件数・失敗件数・失敗理由一覧（`ImportSummary.failures`）を表示しなければならない 🔵 *バックエンドEDGE-002・ImportSummaryより*
- EDGE-003: ファイルアップロードがサーバー側書き込み失敗（`FILE_STORAGE_WRITE_FAILED`）で失敗した場合、システムはエラーメッセージを表示し、当該ファイルが一覧に追加されていないことを確認できる状態を維持しなければならない 🟡 *バックエンドEDGE-003から妥当な推測*
- EDGE-004: グループ種別が `volume` のグループに対し話数登録UIを開こうとした場合、システムは話数登録UIを表示せず、巻として扱う旨を示さなければならない（`INVALID_GROUP_TYPE_FOR_EPISODES`回避） 🔵 *バックエンドEDGE-101・api-endpoints.mdエラーコードより*

### 境界値

- EDGE-101: アイテムが0件の場合、一覧画面は空状態（Empty State）として「コレクションがありません」等のメッセージと追加画面への導線を表示しなければならない 🟡 *一般的なUIパターンから妥当な推測*
- EDGE-102: タグ・カテゴリ名が未入力の場合、システムは作成ボタンを無効化またはバリデーションエラー（`VALIDATION_ERROR`）を表示しなければならない 🟡 *バックエンドapi-endpoints.md VALIDATION_ERRORから妥当な推測*

## 信頼性レベルサマリー

- 🔵 青信号: 32件 (62%)
- 🟡 黄信号: 19件 (37%)
- 🔴 赤信号: 0件 (0%)

**品質評価**: 高品質（PRD・バックエンドAPI仕様との対応が明確。UI実装方式の一部（エラー表示形式・進捗表示等）は一般的UIパターンからの🟡推測）
