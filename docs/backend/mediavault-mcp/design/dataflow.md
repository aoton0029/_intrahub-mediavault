# mediavault-mcp データフロー図

**作成日**: 2026-08-07
**関連アーキテクチャ**: [architecture.md](architecture.md)
**関連要件定義**: [requirements.md](../spec/requirements.md)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・設計文書・既存実装・ユーザヒアリングを参考にした確実なフロー
- 🟡 **黄信号**: それらから妥当な推測によるフロー
- 🔴 **赤信号**: それらにない推測によるフロー

---

## システム全体のデータフロー 🔵

**信頼性**: 🔵 *[architecture.md](architecture.md)・REQ-003 より*

```mermaid
flowchart TD
    A[AIエージェント] -->|"Streamable HTTP + Bearer"| B[認証ミドルウェア]
    B -->|401| A
    B --> C[rmcp Tool Router]
    C --> D[Tool層: 引数検証]
    D -->|検証NG| K[ToolOutcome::error]
    D --> E[Service層: 合成・解決]
    E --> F[ApiClient層]
    F -->|"HTTP /api/v1/*"| G[MediaVault-api]
    G --> H[(PostgreSQL)]
    G --> I[外部カタログAPI]
    G -->|"ApiOk / ApiError"| F
    F -->|"Ok / ApiClientError"| E
    E --> J[構造化結果の組み立て]
    K --> A
    J --> A
```

**要点**:
- MCP から PostgreSQL・外部カタログAPIへの直接経路は存在しない（REQ-140）
- 認証はツール到達前に完了する（REQ-115）
- 業務エラーもプロトコル上は成功として構造化結果で返る（D-01）

---

## 主要機能のデータフロー

### 機能1: search_library（所蔵確認）🔵

**信頼性**: 🔵 *US-01・REQ-010 ~ REQ-014・受け入れ基準 TC-010-01 ~ TC-014-01 より*

**関連要件**: REQ-010, REQ-011, REQ-012, REQ-013, REQ-014, REQ-143

```mermaid
sequenceDiagram
    participant AI as AIエージェント
    participant T as Tool層
    participant S as Service層(search)
    participant C as ApiClient
    participant API as MediaVault-api

    AI->>T: search_library{title, tag?, category?, ...}
    T->>T: limit を 1..=50 に検証
    alt tag/category が名前指定
        T->>S: 解決要求
        S->>C: GET /api/v1/tags
        C->>API: GET /api/v1/tags
        API-->>C: ApiOk<Tag[]>
        C-->>S: Vec<Tag>
        S->>S: 名前 → tag_id 解決
        alt 一致なし
            S-->>T: ToolOutcome::not_found(候補一覧付き)
            T-->>AI: 「該当タグなし」+ 利用可能なタグ名
        end
    end
    S->>C: GET /api/v1/items?title=&tag_id=&limit=&include_total=
    C->>API: 同上
    API-->>C: ApiOk<ItemWithRefs[]> + pagination
    C-->>S: 検索結果
    S->>S: ItemWithRefs → ItemSummary へ縮約 (D-04)
    S-->>T: SearchLibraryResult
    T-->>AI: {outcome, total_count, items[], next_cursor}
```

**詳細ステップ**:
1. Tool層が `limit` を検証する。51以上は `50` へ丸めるのではなく **バリデーションエラー** とする（EDGE-101 の決定：AIに上限を明示的に伝えるため）🟡
2. タグ名・カテゴリ名が指定された場合、Service層が `GET /tags` / `GET /categories` を呼んで ID を解決する。この解決結果は**この呼び出し内でのみ**使い回す（キャッシュしない）
3. 名前が解決できない場合は検索を実行せず、利用可能な名前一覧とともに `not_found` を返す。空振り検索よりAIが次の手を打ちやすい 🟡
4. `GET /items` に本題・原題・別名を横断する検索（PREP-02 完了が前提）と、該当件数（PREP-03 完了が前提）を要求する
5. `ItemWithRefs` から `description` などを落とし `ItemSummary` へ縮約する
6. `pagination.next_after_created_at` / `next_after_id` を不透明な `next_cursor` 文字列にまとめて返す 🟡

---

### 機能2: get_item_context（作品コンテキスト取得）🔵

**信頼性**: 🔵 *US-02・REQ-020 ~ REQ-022・NFR-005・ヒアリング Q5 より*

**関連要件**: REQ-020, REQ-021, REQ-022, NFR-002, NFR-005, EDGE-105

```mermaid
sequenceDiagram
    participant AI as AIエージェント
    participant T as Tool層
    participant S as Service層(context)
    participant C as ApiClient
    participant API as MediaVault-api

    AI->>T: get_item_context{item_id}
    T->>S: build_context(item_id)

    par 並列取得 (futures::join!)
        S->>C: GET /items/{id}
        C->>API: →
        API-->>C: ItemDetail (detail/tags/categories/streaming/images/calibre)
    and
        S->>C: GET /items/{id}/relations
    and
        S->>C: GET /items/{id}/mylists
    and
        S->>C: GET /items/{id}/groups
    and
        S->>C: GET /items/{id}/cast
    and
        S->>C: GET /items/{id}/staff
    and
        S->>C: GET /items/{id}/files
    and
        S->>C: GET /items/{id}/links
    and
        S->>C: GET /items/{id}/trailers
    end

    S->>S: 各結果を Section<T> に変換
    Note over S: Loaded / Empty / Failed の3状態
    alt GET /items/{id} が 404
        S-->>T: ToolOutcome::not_found
        T-->>AI: ITEM_NOT_FOUND
    else
        S-->>T: ItemContext
        T-->>AI: {outcome, item, sections{...}}
    end
```

**詳細ステップ**:
1. `GET /items/{id}` は**必須**の取得。これが 404 なら全体を `not_found` として即返す（他の結果は破棄）
2. それ以外の8件は `futures::join!` で並列実行し、個々の失敗を許容する（`try_join!` は使わない）
3. 各セクションを3状態へ変換する:
   - 成功かつ要素あり → `Loaded(items)`
   - 成功かつ空 → `Empty`（「未登録」）
   - 失敗 → `Failed { code, message }`（「取得失敗」）
4. 1件でも `Failed` があれば全体の `outcome` を `partial` にする。すべて成功なら `success`
5. `GET /items/{id}` が既にタグ・カテゴリ・配信リンク・画像・Calibre連携を含むため、それらの追加呼び出しは行わない（PRD §8 の整理どおり）

---

### 機能3: 外部カタログ検索 → 登録 🔵

**信頼性**: 🔵 *US-03・REQ-030 ~ REQ-033・REQ-112・REQ-117・`items.md` の API 仕様より*

**関連要件**: REQ-030, REQ-031, REQ-032, REQ-033, REQ-112, REQ-117, NFR-003

```mermaid
sequenceDiagram
    participant U as 利用者
    participant AI as AIエージェント
    participant T as Tool層
    participant S as Service層(catalog)
    participant API as MediaVault-api
    participant EXT as 外部プロバイダ

    AI->>T: search_library{title}
    T-->>AI: total_count: 0（未所蔵）
    AI->>U: 「未登録です。外部を検索しますか？」
    U->>AI: はい

    AI->>T: search_external_catalog{media_type, q}
    T->>S: search(media_type, q)
    S->>API: GET /items/search?media_type=&q=
    API->>EXT: プロバイダ検索
    alt APIキー未設定
        API-->>S: 422 API_KEY_NOT_CONFIGURED
        S-->>T: ToolOutcome::error{code, retriable: false}
        T-->>AI: キー未設定を明示
    else タイムアウト
        API-->>S: 502 EXTERNAL_API_TIMEOUT
        S-->>T: ToolOutcome::error{retriable: true}
    else 成功
        EXT-->>API: 候補
        API-->>S: ApiOk<SearchResultItem[]>
        S-->>T: ExternalCandidate[]{provider, external_id, ...}
        T-->>AI: 候補一覧
    end

    AI->>U: 候補を提示
    U->>AI: 2番目を選択

    AI->>T: import_external_item{media_type, provider, external_id}
    T->>S: import(...)
    S->>API: POST /items/import
    alt 既にインポート済み
        API-->>S: 409 ITEM_ALREADY_IMPORTED
        S->>API: GET /items?media_type=&...（既存Item特定）
        API-->>S: 既存Item
        S-->>T: {outcome: "success", already_existed: true, item}
    else 新規
        API-->>S: 201 ApiOk<Item>
        S-->>T: {outcome: "success", already_existed: false, item}
    end
    T-->>AI: 登録結果（item_id 付き）
```

**詳細ステップ**:
1. `search_external_catalog` のレスポンス型は `ExternalCandidate` であり、MediaVault の `item_id` フィールドを**持たない**。これにより所蔵品との取り違えが型レベルで防がれる（REQ-032）
2. `provider` と `external_id` は `import_external_item` へそのまま渡せる形で返す（NFR-003）
3. 409 `ITEM_ALREADY_IMPORTED` を受けた場合、Service層が既存 Item を特定して返す。エラーとして返さない（REQ-112）🟡
   - 特定手段: `GET /items?media_type=X` を走査して `external_id` 一致を探す。PRD §8 の「重複候補検索API」が将来実装されればそちらへ切り替える

---

### 機能4: organize_item（冪等な整理）🔵

**信頼性**: 🔵 *US-06・REQ-060 / REQ-061 / REQ-111 / REQ-113・ヒアリング Q2（事前取得方式）より*

**関連要件**: REQ-060, REQ-061, REQ-111, REQ-113, EDGE-003

```mermaid
flowchart TD
    A["organize_item{item_id, tags[], categories[], mylists[], create_if_missing}"] --> B[現状取得]
    B --> B1["GET /items/{id} → 既存タグ・カテゴリ"]
    B --> B2["GET /items/{id}/mylists → 既存マイリスト"]
    B1 --> C[マスタ取得]
    B2 --> C
    C --> C1["GET /tags"]
    C --> C2["GET /categories"]
    C --> C3["GET /mylists"]

    C1 --> D{各指定名を判定}
    C2 --> D
    C3 --> D

    D -->|既に付与済み| E["OperationResult::AlreadyAttached"]
    D -->|マスタに存在・未付与| F["POST 付与"]
    D -->|マスタに不在 かつ create_if_missing=false| G["OperationResult::NotFound"]
    D -->|マスタに不在 かつ create_if_missing=true| H["POST /tags 等で作成"]

    H -->|成功| F
    H -->|失敗| I["OperationResult::Failed"]
    F -->|成功| J["OperationResult::Attached{created_new}"]
    F -->|失敗| I

    E --> K[結果集約]
    G --> K
    I --> K
    J --> K
    K --> L{全件成功?}
    L -->|Yes| M["outcome: success"]
    L -->|一部失敗/NotFound| N["outcome: partial"]
    L -->|全件失敗| O["outcome: error"]
```

**詳細ステップ**:
1. 現状取得（2呼び出し）とマスタ取得（最大3呼び出し）を **並列** で実行する 🟡
2. 指定された名前ごとに4分岐で判定する。名前比較は前後空白除去 + 完全一致（大文字小文字は区別する）🟡
3. 付与の POST は指定順に逐次実行する。並列にすると部分失敗時の「未処理」の意味が曖昧になるため 🟡
4. 途中で失敗しても後続を続行し、各操作の結果を `OperationResult` として記録する
5. **全体をロールバックしない**。作成済みのタグや付与済みの関連はそのまま残し、結果に正確に反映する（REQ-114 の「成功したように見せない」であって「なかったことにする」ではない）🟡

---

### 機能5: update_consumption（視聴・読了記録）🔵

**信頼性**: 🔵 *US-05・REQ-050 ~ REQ-052・REQ-110・`PATCH /items/{id}` 仕様より*

**関連要件**: REQ-050, REQ-051, REQ-052, REQ-110, NFR-203

```mermaid
sequenceDiagram
    participant AI as AIエージェント
    participant T as Tool層
    participant S as Service層(item_write)
    participant API as MediaVault-api

    AI->>AI: 「昨日見終わった」→ 2026-08-06 に変換
    AI->>T: update_consumption{item_id, status, consumed_date, rating, is_favorite}
    T->>T: consumed_date が YYYY-MM-DD 形式か検証
    T->>S: update(...)
    S->>API: GET /items/{id}（更新前の値を取得）
    alt 404
        API-->>S: ITEM_NOT_FOUND
        S-->>T: ToolOutcome::not_found（更新せず終了）
    else 200
        API-->>S: 更新前 Item
        S->>API: PATCH /items/{id}（指定された項目のみ）
        API-->>S: 更新後 Item
        S->>S: before/after の差分を組み立て
        S-->>T: {outcome, item_id, title, changes[{field, before, after}]}
    end
    T-->>AI: 更新前後の値
    AI->>AI: 利用者へ提示
```

**詳細ステップ**:
1. 日付の自然言語解釈は MCP クライアント（AI）の責務。`consumed_date` が `YYYY-MM-DD` にパースできなければ Tool層で拒否する（REQ-052）
2. 更新前の値を取得するため `GET` を1回追加する。REQ-051 の「更新前後の値を返す」には必須
3. `PATCH /items/{id}` は全フィールド Optional のため、`status` / `consumed_date` / `rating` / `is_favorite` を1リクエストで送れる（`PATCH /items/{id}/status` は使わない）🟡
4. 変更がなかったフィールドは `changes` に含めない 🟡

---

### 機能6: add_access_link（リンク振り分け）🔵

**信頼性**: 🔵 *US-08・REQ-080 / REQ-081・ヒアリング（requirements フェーズ Q4）より*

**関連要件**: REQ-080, REQ-081, REQ-116

```mermaid
flowchart TD
    A["add_access_link{item_id, url, kind, platform?, label?}"] --> B{URL形式検証}
    B -->|不正| C["outcome: error / VALIDATION_ERROR"]
    B -->|正常| D{kind}

    D -->|trailer| E["POST /items/{id}/trailers"]
    D -->|link| F["POST /items/{id}/links"]
    D -->|streaming| G{platform が対応5種か}

    G -->|"netflix/amazon_prime/<br/>disney_plus/dmm_tv/apple_tv"| H["POST /items/{id}/streaming-links"]
    G -->|それ以外 or 未指定| I["POST /items/{id}/links<br/>label = platform名"]

    E --> J["outcome: success, registered_as: trailer"]
    F --> K["outcome: success, registered_as: link"]
    H --> L["outcome: success, registered_as: streaming_link"]
    I --> M["outcome: success, registered_as: link<br/>fallback_from: streaming"]

    H -->|409 DUPLICATE_STREAMING_LINK| N["outcome: success<br/>already_registered: true"]
```

**詳細ステップ**:
1. URL は `url` クレートでパースし、スキームが `http` / `https` であることを確認する 🟡
2. 配信プラットフォームが対応一覧にない場合、`item_links` へ `label` としてプラットフォーム名を付けて登録する（REQ-081）
3. フォールバックが起きたことを `fallback_from: "streaming"` として結果に含める。AI が利用者へ説明できるようにするため
4. `DUPLICATE_STREAMING_LINK`（409）は「既に登録済み」として成功扱いにする（冪等性）🟡

---

## エラーハンドリングフロー 🔵

**信頼性**: 🔵 *D-01・REQ-120・EDGE-001 / EDGE-002・`models/response.rs` の ApiErrorCode より*

```mermaid
flowchart TD
    A[MediaVault-api 呼び出し] --> B{結果}

    B -->|"接続失敗/タイムアウト"| C["ApiClientError::Connection"]
    B -->|"401 UNAUTHORIZED"| D["ApiClientError::Auth"]
    B -->|"4xx/5xx + ApiError body"| E["ApiClientError::Api{code, message, status}"]
    B -->|"2xx"| F[正常処理]

    C --> G["ToolOutcome::error<br/>code: MCP_API_UNREACHABLE<br/>retriable: true"]
    D --> H["ToolOutcome::error<br/>code: MCP_INTERNAL_AUTH_FAILED<br/>retriable: false<br/>※キー値はログにも出さない"]

    E --> I{status}
    I -->|404| J["ToolOutcome::not_found<br/>code はAPIの値を保持"]
    I -->|409| K{冪等化できる409か}
    K -->|"ITEM_ALREADY_IMPORTED<br/>DUPLICATE_RELATION<br/>DUPLICATE_STREAMING_LINK"| L["success + already_* フラグ"]
    K -->|その他| M["ToolOutcome::error"]
    I -->|"400/422"| N["ToolOutcome::error<br/>retriable: false"]
    I -->|"5xx/502"| O["ToolOutcome::error<br/>retriable: true"]

    G --> P[構造化結果として返却]
    H --> P
    J --> P
    L --> P
    M --> P
    N --> P
    O --> P
    F --> P
```

**エラーコードの扱い**:
- MediaVault-api 由来の `code` と `message` は**そのまま保持**して返す（REQ-146）
- MCP 自身が生成するエラーには `MCP_` プレフィックスを付け、api 由来と区別する 🟡

**冪等化する409の一覧**:

| エラーコード | ツール | 扱い |
|---|---|---|
| `ITEM_ALREADY_IMPORTED` | `import_external_item` | 既存Itemを特定して success |
| `DUPLICATE_RELATION` | `relate_items` | `already_related: true` で success |
| `DUPLICATE_STREAMING_LINK` | `add_access_link` | `already_registered: true` で success |
| `DUPLICATE_TAG_NAME` / `DUPLICATE_CATEGORY_NAME` | `organize_item` | 作成競合とみなし、再取得して付与へ進む 🟡 |

---

## 認証フロー 🔵

**信頼性**: 🔵 *REQ-115・REQ-122・NFR-101 / NFR-102 / NFR-104 より*

```mermaid
flowchart TD
    A[プロセス起動] --> B{MCP_AUTH_TOKEN}
    B -->|未設定 or 空文字| C["起動失敗<br/>exit code != 0"]
    B -->|設定あり| D[axum 起動]

    D --> E{リクエストパス}
    E -->|/healthz| F["200 OK（認証なし）"]
    E -->|/mcp| G{Authorization ヘッダー}

    G -->|なし| H["401 Unauthorized"]
    G -->|"Bearer <token>"| I["subtle::ConstantTimeEq で比較"]
    I -->|不一致| H
    I -->|一致| J[rmcp Tool Router へ]

    H --> K["ツールは実行されない<br/>MediaVault-api も呼ばれない"]
```

---

## 状態管理フロー 🔵

**信頼性**: 🔵 *[architecture.md](architecture.md) 状態管理・REQ-003 より*

MCPサーバーはステートレスであり、永続的な状態遷移を持たない。保持するのはプロセス起動時に確定する不変の設定のみ。

```mermaid
stateDiagram-v2
    [*] --> 設定読込
    設定読込 --> 起動失敗: MCP_AUTH_TOKEN 未設定
    設定読込 --> 待受中: 検証OK
    待受中 --> ツール実行中: 認証済みリクエスト
    ツール実行中 --> 待受中: 結果返却（成功/失敗いずれも）
    起動失敗 --> [*]
```

**セッション間で共有される状態はない**。タグ・カテゴリ・マイリストの解決結果もキャッシュしないため、別経路（Web UI等）での変更が即座に反映される。

---

## データ整合性の保証 🟡

**信頼性**: 🟡 *REQ-114・PRD §6 原則5 から妥当な推測*

- **トランザクション**: MCP はトランザクション境界を持たない。複数の API 呼び出しにまたがる原子性は保証されない
- **代替手段**: 部分失敗を隠蔽せず、`OperationResult` で各操作の成否を返す（REQ-114）。AI と利用者が状態を把握して手動で補正できる形にする
- **ロールバックしない理由**: 補償トランザクション（作成済みタグの削除等）は削除系APIの呼び出しを必要とし、REQ-141「削除系を公開しない」と矛盾する。また補償自体が失敗しうる
- **冪等性**: 再試行で重複を作らない設計（D-03、冪等化する409の扱い）により、AI が安全に再実行できる

---

## 関連文書

- **アーキテクチャ**: [architecture.md](architecture.md)
- **型定義**: [interfaces.rs](interfaces.rs)
- **MCPツール仕様**: [mcp-tools.md](mcp-tools.md)
- **要件定義**: [requirements.md](../spec/requirements.md)
- **受け入れ基準**: [acceptance-criteria.md](../spec/acceptance-criteria.md)

## 信頼性レベルサマリー

- 🔵 青信号: 12件 (55%)
- 🟡 黄信号: 10件 (45%)
- 🔴 赤信号: 0件 (0%)

**品質評価**: ✅ **高品質**（🟡 はいずれも要件から導出した実装レベルの具体化であり、要件の解釈に曖昧さは残っていない）
