# backend-frontend-integration データフロー図

**作成日**: 2026-07-02
**関連アーキテクチャ**: [architecture.md](architecture.md)
**関連要件定義**: [requirements.md](../../spec/backend-frontend-integration/requirements.md)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: EARS要件定義書・設計文書・ユーザヒアリングを参考にした確実なフロー
- 🟡 **黄信号**: EARS要件定義書・設計文書・ユーザヒアリングから妥当な推測によるフロー
- 🔴 **赤信号**: EARS要件定義書・設計文書・ユーザヒアリングにない推測によるフロー

---

## システム全体のデータフロー 🔵

**信頼性**: 🔵 *PRD 提案アーキテクチャ・要件定義REQ-007/008/009より*

```mermaid
flowchart TD
    A[ブラウザ] -->|"http://localhost:80"| B[frontend: nginx]
    B -->|"/api/* → proxy_pass"| C[backend: Axum :8080]
    B -->|"それ以外 → try_files $uri /index.html"| B
    C -->|"sqlx"| D[(db: PostgreSQL :5432)]

    D --> C
    C --> B
    B --> A
```

## 起動シーケンス（REQ-001, REQ-101） 🔵

**信頼性**: 🔵 *要件定義REQ-101・既存backend/docker-compose.ymlのdepends_on設定より*

**関連要件**: REQ-001, REQ-101, REQ-002, REQ-003, REQ-004

```mermaid
sequenceDiagram
    participant Dev as 開発者
    participant Compose as docker compose
    participant DB as db (Postgres)
    participant BE as backend (Axum)
    participant FE as frontend (nginx)

    Dev->>Compose: docker compose up -d
    Compose->>DB: db コンテナ起動
    DB->>DB: pg_isready ヘルスチェック
    DB-->>Compose: healthy
    Compose->>BE: backend コンテナ起動（db healthy 待ち）
    BE->>DB: マイグレーション/接続確認（既存挙動）
    Compose->>FE: frontend コンテナ起動（nginx: 静的配信開始）
    FE-->>Dev: http://localhost:80 で待受開始
```

**詳細ステップ**:
1. `docker compose up` 実行後、`db` サービスがまず起動しヘルスチェック（`pg_isready`）を開始する
2. `db` のヘルスチェックが成功（`service_healthy`）するまで `backend` は起動を待機する（REQ-101）
3. `backend` は既存の `backend/Dockerfile` によりビルド・起動される（REQ-002）
4. `frontend` は `db`/`backend` の起動条件に依存せず、nginxによる静的配信を独立して開始できる（ビルド成果物は事前にDockerfileでビルド済みのため）

## API疎通フロー（REQ-007, REQ-008, REQ-009, REQ-102） 🔵

**信頼性**: 🔵 *PRD nginx設定例・ヒアリングQ3, Q4より*

**関連要件**: REQ-007, REQ-008, REQ-009, REQ-102

```mermaid
sequenceDiagram
    participant U as ユーザー（ブラウザ）
    participant N as frontend: nginx
    participant B as backend: Axum

    U->>N: GET http://localhost/
    N-->>U: index.html + 静的アセット（try_files）
    U->>N: fetch('/api/v1/items') （相対パス、REQ-009）
    N->>B: proxy_pass http://backend:8080/api/v1/items
    B-->>N: JSON レスポンス
    N-->>U: JSON レスポンス（同一オリジン、CORS不要）
```

**詳細ステップ**:
1. `frontend/src/api/client.ts` の `BASE_URL` は相対パス `/api/v1` をデフォルト値とする（REQ-009）ため、ブラウザは常に現在アクセス中のオリジン（`http://localhost`）宛にリクエストを発行する
2. nginxは `/api/` 宛のリクエストのみを `http://backend:8080/api/` へ `proxy_pass` する（REQ-007）
3. `/api/` 以外（SPAのルート等）は `try_files $uri /index.html` によりReact Routerへ処理を委譲する（REQ-008）
4. ブラウザ視点では常に同一オリジン通信となるため、CORS設定は不要（REQ-102）

## ネットワーク分離フロー（REQ-201, REQ-401, REQ-402） 🔵

**信頼性**: 🔵 *ヒアリングQ5・PRD ポート方針より*

**関連要件**: REQ-201, REQ-202, REQ-401, REQ-402

```mermaid
flowchart LR
    subgraph Host[ホストマシン]
        Dev[開発者]
    end
    subgraph Compose[docker-compose 内部network]
        FE2[frontend :80]
        BE2[backend :8080]
        DB2[(db :5432)]
    end

    Dev -->|"公開: 80:80"| FE2
    FE2 -.->|"内部network通信のみ"| BE2
    BE2 -.->|"内部network通信のみ"| DB2
    Dev -.->|"❌ 直接接続不可（ports:未設定）"| BE2
    Dev -.->|"❌ 直接接続不可（ports:未設定）"| DB2
```

**詳細ステップ**:
1. 統合用 `docker-compose.yml` において `backend`・`db` サービスには `ports:` を設定しない（REQ-201, REQ-401）
2. `frontend` のみホストの80番ポートに公開する（REQ-202）
3. 既存の `backend/docker-compose.yml`（backend単体起動用、ポート公開あり）はこの分離方針の対象外であり、変更しない（REQ-402）

## エラーハンドリングフロー 🟡

**信頼性**: 🟡 *EDGE-001, EDGE-002・nginxリバースプロキシの一般的挙動からの妥当な推測*

```mermaid
flowchart TD
    A[db ヘルスチェック失敗継続] --> B[backend 起動待機し続ける<br/>EDGE-001]
    C[backend 停止中に /api/ アクセス] --> D[nginx が 502 Bad Gateway 等を返却<br/>EDGE-002]
    E[frontend ホストポート80が使用中] --> F[docker compose up が<br/>ポートバインドエラーで失敗<br/>EDGE-101]
```

## 状態管理フロー

### フロントエンド状態管理 🔵

**信頼性**: 🔵 *frontend/CLAUDE.md・既存実装（TanStack Query）より、本設計による変更なし*

TanStack Queryによるサーバー状態管理は本結合作業による変更を受けない。`apiClient` のリクエスト先URLのみが相対パス化される（REQ-009）。

```mermaid
stateDiagram-v2
    [*] --> 初期状態
    初期状態 --> ローディング: apiClient呼び出し（相対パス /api/v1/...）
    ローディング --> 成功: nginx経由でbackendから応答
    ローディング --> エラー: ネットワークエラー/502等
    成功 --> ローディング: 再取得
    エラー --> ローディング: リトライ
```

### バックエンド・DB状態管理 🔵

**信頼性**: 🔵 *既存backend/docker-compose.ymlのdepends_on・healthcheck設定より、本設計による変更なし*

`backend`・`db` 間の状態管理（マイグレーション、コネクションプール等）は既存実装のまま変更しない。

## データ整合性の保証 🔵

**信頼性**: 🔵 *REQ-002, REQ-003より、既存実装をそのまま利用するため新規の整合性設計は不要*

- **トランザクション管理**: 既存backend実装（sqlx）のまま変更しない
- **マイグレーション**: 既存のマイグレーション機構をそのまま利用し、統合用composeにおいてもスキーマ変更は行わない

## 関連文書

- **アーキテクチャ**: [architecture.md](architecture.md)
- **ヒアリング記録**: [design-interview.md](design-interview.md)
- **要件定義**: [requirements.md](../../spec/backend-frontend-integration/requirements.md)

## 信頼性レベルサマリー

- 🔵 青信号: 9件 (82%)
- 🟡 黄信号: 2件 (18%)
- 🔴 赤信号: 0件 (0%)

**品質評価**: 高品質
