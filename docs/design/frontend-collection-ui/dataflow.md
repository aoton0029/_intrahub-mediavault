# frontend-collection-ui データフロー図

**作成日**: 2026-06-22
**関連アーキテクチャ**: [architecture.md](architecture.md)
**関連要件定義**: [requirements.md](../../spec/frontend-collection-ui/requirements.md)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: EARS要件定義書・設計文書・ユーザヒアリングを参考にした確実なフロー
- 🟡 **黄信号**: EARS要件定義書・設計文書・ユーザヒアリングから妥当な推測によるフロー
- 🔴 **赤信号**: EARS要件定義書・設計文書・ユーザヒアリングにない推測によるフロー

---

## システム全体のデータフロー 🔵

**信頼性**: 🔵 *requirements.md・architecture.mdより*

```mermaid
flowchart TD
    A[ユーザー] --> B[pages/]
    B --> C[features/*]
    C --> D[api/ TanStack Query hooks]
    D --> E[apiClient fetch]
    E --> F[(MediaVault Backend\n/api/v1)]
    F --> E
    E --> D
    D --> C
    C --> B
    B --> A
```

## 主要機能のデータフロー

### 機能1: 全体一覧の閲覧・絞り込み 🔵

**信頼性**: 🔵 *ユーザーストーリー1.1/1.2・REQ-001/002/003・TC-001系より*

```mermaid
sequenceDiagram
    participant U as ユーザー
    participant P as HomePage
    participant F as FilterBar
    participant R as React Router(useSearchParams)
    participant Q as useItemsQuery
    participant A as Backend GET /items

    U->>P: 画面を開く
    P->>R: 現在のクエリパラメータ取得
    R-->>P: media_type/tag_id/favorite/status/page
    P->>Q: queryKey=['items', filters]で実行
    Q->>A: GET /items?media_type=...&page=...&limit=20
    A-->>Q: items[] + pagination
    Q-->>P: data
    P-->>U: カード一覧表示
    U->>F: フィルタ条件を変更
    F->>R: setSearchParams(新条件)
    R->>P: 再レンダリング（queryKey変化）
    P->>Q: 新queryKeyで再実行（キャッシュ確認→必要なら再取得）
```

**詳細ステップ**:
1. 画面マウント時に`useSearchParams`からフィルタ条件を読み出し、TanStack Queryの`queryKey`に含める
2. フィルタUI変更時は`setSearchParams`でURLを更新し、それをトリガーにクエリが再実行される（REQ-003: URL同期）
3. 0件時は`EmptyState`コンポーネントを表示し「追加画面への導線」を出す（EDGE-101）

### 機能2: 外部API検索からの追加 🔵

**信頼性**: 🔵 *ユーザーストーリー2.1/2.2・REQ-005/203・TC-005系より*

```mermaid
sequenceDiagram
    participant U as ユーザー
    participant S as SearchAddPage
    participant Q as useExternalSearchQuery
    participant M as useImportItemMutation
    participant A as Backend

    U->>S: 検索語を入力し検索実行
    S->>Q: GET /items/search?media_type=...&q=...
    Q->>A: リクエスト
    alt API_KEY_NOT_CONFIGURED (422)
        A-->>Q: エラー
        Q-->>S: エラー状態
        S-->>U: 「APIキー未設定」+手動追加への導線表示
    else EXTERNAL_API_TIMEOUT (502)
        A-->>Q: エラー
        Q-->>S: エラー状態
        S-->>U: エラーメッセージ+再試行ボタン
    else 成功
        A-->>Q: 検索結果一覧
        Q-->>S: data
        S-->>U: 結果カード一覧表示
        U->>S: 1件選択し「追加」
        S->>M: POST /items/import
        M->>A: リクエスト
        A-->>M: 201 作成済みitem
        M-->>S: 成功
        S-->>U: toast表示 + 詳細画面へ遷移
    end
```

**備考**: エラーコード分岐（`API_KEY_NOT_CONFIGURED`/`EXTERNAL_API_TIMEOUT`）はbackend api-endpoints.mdのエラーコードに対応（🔵 EDGE-001）。

### 機能3: 手動追加・編集 🔵

**信頼性**: 🔵 *ユーザーストーリー2.3/3.1・REQ-006/007・TC-006系より*

```mermaid
sequenceDiagram
    participant U as ユーザー
    participant F as ItemFormPage (react-hook-form)
    participant Z as zod schema
    participant M as useCreateItem/useUpdateItemMutation
    participant A as Backend

    U->>F: 必須項目を入力し保存
    F->>Z: クライアント側バリデーション
    alt バリデーションNG
        Z-->>F: エラー
        F-->>U: フィールド近傍にエラー表示（NFR-201）
    else バリデーションOK
        F->>M: mutate(formData)
        M->>A: POST /items または PATCH /items/:id
        alt VALIDATION_ERROR (400) / ITEM_NOT_FOUND (404)
            A-->>M: エラー
            M-->>F: エラー
            F-->>U: toast+フィールドエラー表示 or 一覧へリダイレクト
        else 成功
            A-->>M: 201/200 item
            M-->>F: 成功
            F-->>U: 詳細画面へ遷移 + toast
        end
    end
```

### 機能4: シーズン/巻管理（メディア別構成） 🔵

**信頼性**: 🔵 *ユーザーストーリー4.1/4.2・REQ-015/016/101/102・EDGE-004より*

```mermaid
flowchart TD
    A[詳細画面 media_type判定] -->|anime/drama| B[GroupSection: group_type=season]
    A -->|manga/novel| C[GroupSection: group_type=volume]
    B --> D[話数登録UI表示可]
    C --> E[話数登録UI非表示\n巻として扱う旨を表示]
    D --> F[POST /groups/:group_id/episodes]
    C -.->|誤操作防止| G["INVALID_GROUP_TYPE_FOR_EPISODES回避\n（UIでボタン自体を出さない）"]
```

**信頼性**: 🔵 *EDGE-004：UI側でvolumeグループには話数登録ボタンを表示しないことでバックエンドのエラーを未然に防止する設計*

### 機能5: 一括インポート（ブクログ/Steam） 🔵

**信頼性**: 🔵 *ユーザーストーリー5.2・REQ-022・EDGE-002より*

```mermaid
sequenceDiagram
    participant U as ユーザー
    participant S as SettingsPage(Importタブ)
    participant M as useImportBooklog/SteamMutation
    participant A as Backend

    U->>S: CSVファイル選択 or Steam ID入力し実行
    S->>M: mutate(file or steamId)
    M->>A: POST /import/booklog (multipart) または /import/steam
    A-->>M: ImportSummary{success_count, failure_count, failures[]}
    M-->>S: data
    S-->>U: 成功件数・失敗件数・失敗理由一覧を表示（EDGE-002）
```

## データ処理パターン

### 同期処理 🔵

**信頼性**: 🔵 *アーキテクチャ設計より*

一覧取得・詳細取得・フォーム保存・タグ/カテゴリ操作はすべてユーザー操作に対する同期的なリクエスト/レスポンスで処理する（ポーリング・WebSocket等の非同期通知機構は対象外）。

### 非同期処理（UI上の進捗表示） 🟡

**信頼性**: 🟡 *NFR-203から妥当な推測*

ファイルアップロード（`POST /items/:id/files/upload`）・CSVインポートはリクエスト自体は同期だが、UI側で進捗インジケータ（アップロード中/インポート中）を表示する。

### バッチ処理 🔴

**信頼性**: 🔴 *フロントエンドにバッチ処理要件なし*

フロントエンド側でのバッチ処理は行わない（巡回バッチはバックエンドの内部API管轄）。

## エラーハンドリングフロー 🟡

**信頼性**: 🟡 *backend api-endpoints.mdエラーコード・NFR-201/203から妥当な推測*

```mermaid
flowchart TD
    A[APIエラー受信] --> B{エラーコード}
    B -->|VALIDATION_ERROR 400| C[フィールド近傍にエラー表示]
    B -->|*_NOT_FOUND 404| D[一覧へリダイレクト+エラートースト]
    B -->|API_KEY_NOT_CONFIGURED 422| E[手動追加導線付きエラー表示]
    B -->|EXTERNAL_API_TIMEOUT 502| F[再試行ボタン付きエラー表示]
    B -->|FILE_STORAGE_WRITE_FAILED 500| G[エラートースト+ファイル一覧へ反映しない]
    B -->|その他| H[汎用エラートースト]
```

## 状態管理フロー

### フロントエンド状態管理 🔵

**信頼性**: 🔵 *architecture.md・tech-stack.mdより*

```mermaid
stateDiagram-v2
    [*] --> idle
    idle --> loading: クエリ/ミューテーション開始
    loading --> success: データ取得・更新成功
    loading --> error: エラー応答
    success --> loading: 再取得・再送信
    error --> loading: 再試行
```

サーバー状態（一覧・詳細・検索結果等）はTanStack Queryの`status`（`pending`/`success`/`error`）で管理し、フォーム入力状態はreact-hook-formの`formState`で管理する。

## データ整合性の保証 🟡

**信頼性**: 🟡 *NFR-002・既存実装パターンから妥当な推測*

- **キャッシュ無効化**: 作成・更新・削除ミューテーション成功時に、関連する一覧クエリ（`['items', ...]`）を`invalidateQueries`で無効化し再取得する
- **楽観的更新**: お気に入りトグル・status更新等の軽量操作のみ楽観的更新を採用し、失敗時はロールバックする（🟡 一般的なTanStack Queryパターンから推測。要件に明記なし）
- **整合性チェック**: グループ種別（season/volume/chapter）に応じたUI制御で、バックエンドのドメイン制約（EDGE-004）と矛盾しない操作のみを許可する

## 関連文書

- **アーキテクチャ**: [architecture.md](architecture.md)
- **型定義**: [interfaces.ts](interfaces.ts)
- **ヒアリング記録**: [design-interview.md](design-interview.md)

## 信頼性レベルサマリー

- 🔵 青信号: 12件 (60%)
- 🟡 黄信号: 6件 (30%)
- 🔴 赤信号: 2件 (10%)

**品質評価**: 高品質（主要フローはユーザーストーリー・受け入れ基準・バックエンドエラーコードと対応。キャッシュ無効化・楽観的更新の具体策はPRD未記載のため🟡推測）
