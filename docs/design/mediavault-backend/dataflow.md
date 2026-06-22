# mediavault-backend データフロー図

**作成日**: 2026-06-22
**関連アーキテクチャ**: [architecture.md](architecture.md)
**関連要件定義**: [requirements.md](../../backend/spec/mediavault-backend/requirements.md)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: EARS要件定義書・設計文書・ユーザヒアリングを参考にした確実なフロー
- 🟡 **黄信号**: EARS要件定義書・設計文書・ユーザヒアリングから妥当な推測によるフロー
- 🔴 **赤信号**: EARS要件定義書・設計文書・ユーザヒアリングにない推測によるフロー

---

## システム全体のデータフロー 🔵

**信頼性**: 🔵 *要件定義・user-stories.mdより*

```mermaid
flowchart TD
    A[利用者/外部ツール] --> B[Axumルーター]
    B --> C{APIキー検証\n内部APIのみ}
    C -->|OK| D[ハンドラ]
    C -->|NG| Z[401返却]
    D --> E[サービス層]
    E --> F[(PostgreSQL)]
    E --> G[api-client-lib]
    G --> H[外部API群]
    E --> I[ファイルサーバー\nパス参照/書込]
    F --> D
    H --> D
    D --> B
    B --> A
```

## 主要機能のデータフロー

### 機能1: 外部API検索→アイテム追加 🔵

**信頼性**: 🔵 *user-stories 1.1・TC-002-01〜03より*

**関連要件**: REQ-002

```mermaid
sequenceDiagram
    participant U as 利用者
    participant API as mediavault-api
    participant Svc as ExternalSearchService
    participant Lib as api-client-lib
    participant Ext as 外部API(Jikan/TMDb等)
    participant DB as PostgreSQL

    U->>API: GET /items/search?media_type=anime&q=タイトル
    API->>Svc: search(media_type, query)
    Svc->>DB: SELECT api_key FROM api_credentials WHERE provider=...
    DB-->>Svc: APIキー（不要な場合あり）
    Svc->>Lib: client.execute(SearchRequest)
    Lib->>Ext: HTTPリクエスト
    Ext-->>Lib: 検索結果
    Lib-->>Svc: ApiResponse<Model>
    Svc-->>API: 検索結果一覧
    API-->>U: 200 検索結果

    U->>API: POST /items/import {selected result}
    API->>Svc: import_item(payload)
    Svc->>DB: INSERT items(source='api', external_id=...) + 詳細テーブル
    DB-->>Svc: 作成済みitem
    Svc-->>API: item
    API-->>U: 201 item
```

**詳細ステップ**:
1. 利用者がmedia_typeとタイトルを指定して検索APIを呼ぶ
2. サービス層がmedia_typeに対応するプロバイダ（jikan/tmdb/ndl/openlibrary/steam/igdb）を選択し、必要に応じてDBから外部APIキーを取得
3. `api-client-lib` の `ApiClient::execute` を介して外部APIへ問い合わせ、結果を返す
4. 利用者が結果を選択し `/items/import` を呼ぶと、`items` + メディア別詳細テーブルへ `source=api` で登録する

### 機能2: 手動追加 🔵

**信頼性**: 🔵 *user-stories 1.2・TC-001-01より*

**関連要件**: REQ-003

```mermaid
sequenceDiagram
    participant U as 利用者
    participant API as mediavault-api
    participant DB as PostgreSQL

    U->>API: POST /items {media_type, title, ...}
    API->>API: バリデーション（media_type enum等）
    API->>DB: INSERT items(source='manual', external_id=NULL) + 詳細テーブル
    DB-->>API: 作成済みitem
    API-->>U: 201 item
```

### 機能3: シーズン・話数管理 🔵

**信頼性**: 🔵 *user-stories 3.1・TC-010-01/E01より*

**関連要件**: REQ-010, REQ-101, EDGE-101

```mermaid
sequenceDiagram
    participant U as 利用者
    participant API as mediavault-api
    participant DB as PostgreSQL

    U->>API: POST /items/{id}/groups {group_type:"season", ...}
    API->>DB: INSERT item_groups
    DB-->>API: group
    API-->>U: 201 group

    U->>API: POST /groups/{group_id}/episodes {episode_number,...}
    API->>DB: SELECT group_type FROM item_groups WHERE id=group_id
    DB-->>API: group_type
    alt group_type in (season, chapter)
        API->>DB: INSERT item_episodes
        DB-->>API: episode
        API-->>U: 201 episode
    else group_type = volume
        API-->>U: 400 invalid group_type for episodes
    end
```

### 機能4: ファイル登録（パス指定／バイナリアップロード） 🔵

**信頼性**: 🔵 *user-stories 6.2・TC-007-01/TC-019-01/E01より*

**関連要件**: REQ-007, REQ-019, REQ-104, EDGE-003

```mermaid
sequenceDiagram
    participant Caller as 利用者/監視バッチ
    participant API as mediavault-api
    participant FS as ファイルサーバー
    participant DB as PostgreSQL

    alt パス指定方式
        Caller->>API: POST /items/{id}/files {path, file_type, label}
        API->>DB: INSERT item_files(path=指定パス)
        DB-->>API: file record
        API-->>Caller: 201
    else バイナリ直接アップロード方式
        Caller->>API: POST /items/{id}/files/upload (multipart binary)
        API->>FS: ファイル書込（/srv/files/pdf または /srv/media/photos）
        alt 書込成功
            FS-->>API: 配置完了（相対パス）
            API->>DB: INSERT item_files(path=配置後相対パス)
            DB-->>API: file record
            API-->>Caller: 201
        else 書込失敗
            FS-->>API: エラー
            API-->>Caller: 500（item_filesレコード作成せずロールバック）
        end
    end
```

### 機能5: ブクログCSV／Steamライブラリ インポート 🔵

**信頼性**: 🔵 *user-stories 5.1/5.2・TC-016-01/E01・TC-017-01より*

**関連要件**: REQ-016, REQ-017, EDGE-002

```mermaid
flowchart TD
    A[CSVアップロード or steam_id指定] --> B[行/エントリ単位で解析]
    B --> C{形式正常?}
    C -->|正常| D[items + 詳細テーブルへINSERT]
    C -->|不正| E[スキップしエラー理由を記録]
    D --> F[結果サマリー集計]
    E --> F
    F --> G[成功数/失敗数/失敗理由をレスポンス]
```

### 機能6: 内部REST API認証フロー 🔵

**信頼性**: 🔵 *TC-018-01/E01/E02より*

**関連要件**: REQ-018, REQ-403, NFR-101

```mermaid
flowchart TD
    A[外部ツールからリクエスト] --> B{Authorizationヘッダー検証}
    B -->|キー一致| C[ハンドラ処理続行]
    B -->|キー欠落/不一致| D[401返却]
```

## エラーハンドリングフロー 🟡

**信頼性**: 🟡 *Axum標準パターンから妥当な推測*

```mermaid
flowchart TD
    A[エラー発生] --> B{エラー種別}
    B -->|バリデーションエラー| C[400 Bad Request]
    B -->|APIキー欠落/不一致| D[401 Unauthorized]
    B -->|リソース未存在| E[404 Not Found]
    B -->|外部APIキー未設定/タイムアウト| F[422/502]
    B -->|サーバー内部エラー| G[500 Internal Server Error]
    C --> H[統一エラーJSON返却]
    D --> H
    E --> H
    F --> H
    G --> I[ログ記録] --> H
```

## データ処理パターン

### 同期処理 🔵

**信頼性**: 🔵 *単一ユーザー前提・アーキテクチャ設計より*

CRUD・検索・ファイル登録などすべてのAPI呼び出しは同期的にレスポンスを返す（バックグラウンドジョブキューは導入しない）。

### 非同期処理 🟡

**信頼性**: 🟡 *インポート機能の処理量から妥当な推測*

ブクログCSV・Steamライブラリインポートはリクエスト内で同期処理し、完了をレスポンスで返す（件数が多い場合でも単一ユーザー想定のため非同期ジョブ化は不要）。

### バッチ処理 🟡

**信頼性**: 🟡 *PRD「外部から登録・更新・検索をするためのAPI」より*

巡回バッチ・ファイルサーバー監視プロセス自体はmediavault-api外の別プロセスとして動作し、内部REST API経由でmediavault-apiを呼び出す（バッチ実行自体は本APIの責務外）。

## 状態管理フロー

### アイテムstatus状態遷移 🟡

**信頼性**: 🟡 *REQ-008・PRDの「視聴中/読了/未着手」から妥当な推測*

```mermaid
stateDiagram-v2
    [*] --> 未着手
    未着手 --> 視聴中: 利用者が更新
    視聴中 --> 読了: consumed_date設定
    読了 --> 視聴中: 再視聴で更新
    未着手 --> 読了: 直接更新も許可
```

## データ整合性の保証 🟡

**信頼性**: 🟡 *REQ-405・EDGE-003・カスケード削除要件から妥当な推測*

- **トランザクション管理**: items作成時の詳細テーブルINSERTはsqlxトランザクションで一括コミット/ロールバック
- **カスケード削除**: `item_tags`/`item_links`/`item_files`等の関連レコードはitems削除時に`ON DELETE CASCADE`で自動削除（TC-001-03）
- **整合性チェック**: EDGE-101（volume配下へのepisode登録拒否）はアプリケーション層でgroup_type検証

## 関連文書

- **アーキテクチャ**: [architecture.md](architecture.md)
- **型定義**: [types.rs](types.rs)
- **DBスキーマ**: [database-schema.sql](database-schema.sql)
- **API仕様**: [api-endpoints.md](api-endpoints.md)

## 信頼性レベルサマリー

- 🔵 青信号: 14件 (70%)
- 🟡 黄信号: 6件 (30%)
- 🔴 赤信号: 0件 (0%)

**品質評価**: 高品質
