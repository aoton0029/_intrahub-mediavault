# MediaVault Extractor 全文抽出 準備タスク（ユーザー作業）

> **仕様**: [requirements.md](requirements.md)
> **生成日**: 2026-08-14

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・設計文書・ユーザヒアリングで明確に必要と判明したタスク
- 🟡 **黄信号**: 要件定義書・設計文書から妥当に推測されるタスク
- 🔴 **赤信号**: 推測による予防的タスク（実装時に不要と判明する可能性あり）

外部SaaSのAPIキー取得やドメイン・証明書の準備は本要件では不要である（すべてhomelab内で完結し、OSSのみを使用する）。

---

## 必須（実装開始前に完了が必要）

以下が完了していないと、実装フェーズでブロッカーになる。

- [ ] **`INTERNAL_API_KEY` の払い出しと共有** 🔵 *[requirements.md](requirements.md) REQ-028・NFR-101・tech-stack.md より*
  - mediavault-api / mediavault-mcp / mediavault-extractor の3サービスが同一の値を参照する必要がある
  - 十分な長さのランダム値を生成し、`.env` へ設定する（イメージへ焼き込まない）
  - 既に mediavault-api / mcp で運用中の値がある場合は、それを extractor へ共有すればよい
  - 関連要件: REQ-028, NFR-101

- [ ] **共有ボリュームの read-only マウント構成の確定** 🔵 *ヒアリングQ2・REQ-404・tech-stack.md §インフラより*
  - Extractor が読む必要のある実体は2系統ある
    - 実データ領域（リンク経路。`/srv/anime`・`/srv/live-action`・`/srv/manga` 等）→ `/library:ro`
    - MediaVault専用領域（アップロード経路。`STORAGE_ROOT`）→ `/srv/mediavault:ro`
  - `.env` に `LIBRARY_SOURCE` と `MEDIAVAULT_STORAGE_SOURCE` を設定する
  - **確認事項**: 実データ領域が複数のマウントポイントに分かれている場合、`/library` 1本にまとめられるか、複数マウントが必要かを確定する（内部APIが返すファイル参照の形式に影響する）
  - 関連要件: REQ-022, REQ-404, NFR-102

- [ ] **`item_files.path` の実データ分布の確認** 🔵 *item-files.md §2つの登録経路・[note.md](note.md) §6-1 より*
  - リンク経路（絶対パス）とアップロード経路（相対パス）の両方が実際に登録されているかを確認する
  - リンク経路のパスがどのディレクトリを指しているかを洗い出す（許可ルートの設計に直結する）
  - 旧レイアウト（`STORAGE_ROOT` 直下の file_type 別サブディレクトリ）に残っているファイルの有無を確認する
  - 関連要件: REQ-022, REQ-403

- [ ] **抽出対象となるファイルの実データサンプルの用意** 🔵 *NFR-003・tech-stack.md §テスト構成・§GPU制約より*
  - テキストレイヤーありPDF / なし（スキャン）PDF / 画像 の各1件以上
  - 処理時間計測用に、実運用で最大クラスのページ数を持つPDFを1件
  - 小サイズのものは `extractor/tests/fixtures/` へ置くテスト用 fixture としても使う
  - 関連要件: NFR-003, NFR-601, TC-NFR-003-01

---

## 推奨（実装中に用意できればOK）

実装は開始できるが、該当機能の実装前までに準備する。

- [ ] **yomitoku のモデル取得と配布方式の決定** 🔵 *PRD §12・REQ-069 より*
  - 初回実行時にダウンロードさせるか、イメージへ同梱するか、ボリュームへ事前配置するかを決める
  - オフライン環境や再起動のたびの再ダウンロードを避けたい場合はボリューム配置が無難
  - CPU実行では軽量モデルを使うため、軽量モデルと通常モデルの両方の入手を確認する
  - 必要になるフェーズ: Phase 3（抽出処理本体）
  - 関連要件: REQ-069, NFR-301

- [ ] **CPU実行時のOCR処理時間の実測** 🔵 *PRD FR-011「実データで評価する」・NFR-003 より*
  - 軽量モデルと通常モデルで、1ページあたりの処理時間と精度を比較する
  - 計測結果に基づき `EXTRACTOR_JOB_TIMEOUT_SEC` の初期値を確定する（要件では固定値を定めていない）
  - 必要になるフェーズ: Phase 5（非機能検証）
  - 関連要件: NFR-003, TC-NFR-003-01

- [ ] **docker-compose への `mediavault-extractor` サービス追加** 🔵 *tech-stack.md §インフラ・デプロイ より*
  - tech-stack.md にひな型あり。`depends_on: mediavault-api (healthy)`、`networks: mediavault-api` のみ、`media-db` へは接続しない
  - `security_opt: no-new-privileges:true` を付ける
  - 必要になるフェーズ: Phase 3
  - 関連要件: NFR-106, NFR-201

- [ ] **`extractor/` ディレクトリの初期化（uv プロジェクト）** 🟡 *tech-stack.md §推奨ディレクトリ構造より*
  - 現状 `extractor/` は空ディレクトリ
  - `uv init` → `pyproject.toml` / `uv.lock` / `Dockerfile` の作成
  - 必要になるフェーズ: Phase 3
  - 関連要件: NFR-602

---

## 確認事項（判断が必要）

実装方針に影響するため、早めの判断が推奨される。

- [ ] **既存 `/internal/*` パスの移行方法** 🔵 *ヒアリングQ3・REQ-029・[interview-record.md](interview-record.md) §残課題7 より*
  - 選択肢A: 即時に `/api/v1/internal/*` へ切り替え、旧パスを削除する（利用者が mediavault-mcp のみなら安全）
  - 選択肢B: 旧パスを暫定 alias として残し、後で削除する
  - 判断材料: `/internal/*` を叩いている外部スクリプト・バッチが他にあるか
  - 関連要件: REQ-029

- [ ] **`max_attempts` の既定値** 🟡 *REQ-111・REQ-112・[interview-record.md](interview-record.md) §残課題4 より*
  - 大きすぎると壊れたファイルで無駄なCPUを消費し、小さすぎると一時障害で諦めてしまう
  - CPU OCRの処理時間が長いことを踏まえると 3 前後が妥当と思われる
  - 関連要件: REQ-111, REQ-112

- [ ] **lease 期間と heartbeat 間隔の具体値** 🟡 *REQ-021・REQ-023・[interview-record.md](interview-record.md) §残課題5 より*
  - lease 期間は「1ページのOCR処理時間 × 安全係数」を下回ってはならない（処理中に失効すると二重実行になる）
  - CPU OCRの実測（上記「CPU実行時のOCR処理時間の実測」）の後に確定するのが確実
  - 関連要件: REQ-021, REQ-023, REQ-118

- [ ] **抽出本文・エラーの保存サイズ上限値** 🟡 *REQ-408・EDGE-009・[interview-record.md](interview-record.md) §残課題3 より*
  - 蔵書のうち最大クラスの文書の文字数を確認したうえで決める
  - 上限を超えたときの扱い（complete 拒否 / 切り詰めて保存）も併せて決める。要件では「拒否」を前提としている
  - 関連要件: REQ-408, EDGE-009

- [ ] **OCRフォールバック判定の品質基準** 🟡 *PRD FR-004・tech-stack.md §決めていないこと・[interview-record.md](interview-record.md) §残課題1 より*
  - 「テキストが存在しない、または品質基準を満たさないページ」の閾値（例: 抽出文字数がNページ面積比で極端に少ない）
  - 実データのサンプルを見て決める必要がある
  - 関連要件: REQ-106

- [ ] **MVP の抽出対象形式に `archive`（cbz / cbr）を含めるか** 🟡 *REQ-410・[user-stories.md](user-stories.md) ストーリー1.3 備考より*
  - 現在の要件は pdf / image のみを対象とし、archive は `UNSUPPORTED_FILE_TYPE` としている
  - 蔵書に電子コミック（cbz）が多い場合は、MVP に含める価値があるかもしれない
  - 関連要件: REQ-410

- [ ] **GPU（`cuda`）運用を行うかどうか** 🔵 *PRD NFR-003・§8.9・NFR-302 より*
  - 行う場合、vLLM の `--gpu-memory-utilization 0.90` を下げる運用変更が必要（`intrahub/services/vllm/compose.yaml`）
  - vLLM を止めてよいタイミングがあるか、常時稼働が必要かで方針が変わる
  - MVP は CPU 既定で進められるため、判断を Phase 5 まで先送りしてもブロッカーにはならない
  - 関連要件: NFR-301, NFR-302, TC-NFR-302-01

---

## サマリー

| 優先度 | 件数 | 🔵 | 🟡 | 🔴 |
|--------|------|-----|-----|-----|
| 必須 | 4 | 4 | 0 | 0 |
| 推奨 | 4 | 3 | 1 | 0 |
| 確認事項 | 7 | 2 | 5 | 0 |
| **合計** | **15** | **9** | **6** | **0** |

**外部サービス契約・APIキー取得・DNS・証明書・法務確認は不要**（homelab内で完結、OSSのみ）。

## 関連文書

- **要件定義書**: [requirements.md](requirements.md)
- **ヒアリング記録**: [interview-record.md](interview-record.md)
- **ユーザストーリー**: [user-stories.md](user-stories.md)
- **受け入れ基準**: [acceptance-criteria.md](acceptance-criteria.md)
- **コンテキストノート**: [note.md](note.md)
