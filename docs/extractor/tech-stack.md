# MediaVault Extractor 技術スタック定義

## 🔧 生成情報
- **生成日**: 2026-08-14
- **生成ツール**: init-tech-stack
- **対象**: `mediavault-extractor`（Python worker）のみ。MediaVault全体のスタック定義ではない
- **前提文書**: [PRD.md](./PRD.md)
- **プロジェクトタイプ**: バックエンド常駐worker（ユーザー向けHTTP APIを持たない）
- **チーム規模**: 個人開発
- **開発期間**: MVP

## 🎯 プロジェクト要件サマリー
- **パフォーマンス**: 軽負荷・スループット非重視。同時実行1件が初期値（NFR-003）
- **セキュリティ**: 内部サービス認証（`INTERNAL_API_KEY`）＋ read-onlyマウント ＋ 許可ルート外の読み取り禁止（NFR-002）
- **技術スキル**: Python / Docker / 既存Rust APIとの連携
- **学習コスト許容度**: 既存スキル活用寄り、枯れた構成を優先
- **デプロイ先**: オンプレ homelab、Docker Compose
- **予算**: コスト最小化（自宅GPU・OSSのみ）

## 🚀 ランタイム・言語

- **言語**: Python 3.12
- **パッケージマネージャー**: uv（`uv.lock` をコミット）
- **配布形態**: 単一Dockerイメージ、常駐ポーリングプロセス

### 選択理由
- yomitoku / PyTorch / pypdfium2 のいずれも Python 3.12 で安定動作する。
- `uv.lock` による完全なバージョン固定が NFR-005（再現性）をそのまま満たす。
- Rust製 mediavault-api にPython依存を持ち込まないという PRD 2章の前提を、プロセス分離で実現する。

## ⚙️ worker 実行モデル

- **実行スタイル**: 同期ループ（asyncioを使わない）
- **HTTPクライアント**: httpx（`httpx.Client`、timeout明示）
- **リトライ**: tenacity（指数バックオフ、`retry_if_exception_type` で対象を限定）
- **並行制御**: heartbeat専用の daemon `threading.Thread` 1本のみ
- **キャンセル伝播**: `threading.Event`
- **ジョブ基盤**: 導入しない（Celery / RQ 等）

### 選択理由
- OCRはCPU/GPUバウンドで並列度1が初期値のため、asyncの利点が薄く複雑さだけが増える。
- ジョブと状態遷移の正本は MediaVault-api + PostgreSQL（PRD 8章）。worker側にジョブ基盤を置くと二重管理になる。
- heartbeat（lease延長・進捗更新・キャンセル取得）だけは抽出処理と独立して動く必要があるため、スレッド1本に限定して分離する。抽出本体はメインスレッドで同期実行し、ページ等の安全な区切りで `Event` を確認する（FR-008）。

### 実行ループの骨子

```text
loop:
  job = claim()                      # 排他取得、lease token 取得（FR-001）
  if job is None: sleep(poll_interval); continue
  start heartbeat thread(job, cancel_event)
  try:
      result = extract(job, cancel_event)   # ページ境界で cancel_event.is_set() を確認
      if cancel_event.is_set(): cancel(job); continue   # 成功確定してはならない
      complete(job, result, lease_token)
  except PermanentError as e: fail(job, e, retryable=False)
  except TransientError as e: fail(job, e, retryable=True)
  finally: stop heartbeat thread
```

## 📄 抽出ライブラリ

- **PDFテキスト抽出 / ページラスタライズ**: pypdfium2
- **形式判定**: puremagic（ファイルシグネチャ）＋ 拡張子の併用
- **画像処理**: Pillow
- **正規化**: 標準ライブラリ `unicodedata`（NFKC）＋ 自前の改行・空白・制御文字正規化

### 選択理由
- **ライセンス**: pypdfium2 は Apache-2.0 / BSD-3-Clause。高機能な PyMuPDF (fitz) は **AGPL-3.0** であり、将来コードを公開する余地を残すなら採用を避ける。
- pypdfium2 はページ単位のテキスト取得とOCR用画像レンダリングを1ライブラリで賄えるため、依存とイメージサイズが減る。
- puremagic は純Python実装で libmagic のネイティブ依存が不要。`python-magic` に比べDockerfileが単純になる。拡張子とシグネチャの不一致を明示的なエラーとして扱う（FR-003）。
- 正規化は決定的な変換のみに限る。LLMによる書き換えや校正は行わない（FR-005）。

### 検討したが採用しなかったもの
- **PyMuPDF (fitz)**: 速度・機能とも優秀だがAGPL-3.0。
- **pdfplumber / pdfminer.six**: レイアウト情報は豊富だが低速で、OCR用のページレンダリング機能を持たない。
- **EPUB対応（ebooklib 等）**: MVP対象外。FR-006 が章境界に言及しているため、境界データ構造は EPUB を後から足せる形で設計する。

## 🔤 OCR

- **境界**: `OcrEngine` Protocol（`ocr(image: PIL.Image) -> OcrResult`）
- **MVP実装**: yomitoku のみ
- **将来の差し替え候補**: ndlocr-lite / Tesseract
- **デバイス**: 環境変数 `EXTRACTOR_OCR_DEVICE=cpu|cuda`、**既定は `cpu`**

### 選択理由
- NFR-006 が要求する「OCRエンジンをmockした単体テスト」は、Protocol境界の存在そのものと等価。yomitoku の内部型を境界の外へ漏らさず、入力は `PIL.Image`、出力は `OcrResult(text, confidence)` に固定する。
- PRD 12章が yomitoku と ndlocr-lite を併記しているため、実装を1つに絞りつつ差し替え可能性を型で担保する。

### ⚠️ GPU に関する制約（重要）
`intrahub/services/vllm/compose.yaml` の vLLM サービスが NVIDIA GPU 1枚を
`--gpu-memory-utilization 0.90` で常時予約している。**GPUは利用可能だが常時空いてはいない。**

- MVPは `EXTRACTOR_OCR_DEVICE=cpu` を既定とし、vLLM と同居しても VRAM 競合で落ちない状態を保つ。
- GPUを使う場合は、compose に vllm と同形の
  `deploy.resources.reservations.devices: [{driver: nvidia, count: 1, capabilities: [gpu]}]`
  を追加したうえで、**vLLM側の `--gpu-memory-utilization` を下げる運用変更とセットで行う**。両モデルを同時に載せられる保証はない。
- CPUでのOCR処理時間が実用範囲かは実データでの計測が必要。PRDに処理時間の数値目標がないため、計測後にNFR-003の上限初期値を確定する。

## 🔐 ファイルアクセス

- **方式**: read-only 共有ボリューム（PRD 要調整事項3 の第一候補を採用）
- **マウント**: `${LIBRARY_SOURCE}:/library:ro`、`${MEDIAVAULT_STORAGE_SOURCE}:/srv/mediavault:ro`
- **パス解決**: 内部APIが返す参照を worker 側で `Path.resolve()` し、許可ルート配下であることを検証してから開く

### 設計方針
- 外部から渡された絶対パス・相対パスをそのまま開かない（FR-002）。
- `Path.resolve()` は symlink を展開するため、**展開後に** `is_relative_to(allowed_root)` で判定することでNFR-002のsymlink要件を満たす。判定前に開かない。
- DBドライバ（psycopg / asyncpg 等）を依存関係に一切含めない。これにより「DBへ直接接続しない」（PRD 5.1）が依存レベルで強制される。
- 内部APIからのストリーミング取得は採用しない。大容量PDFのランダムアクセス（ページ単位読み出し）に共有ボリュームの方が適するため。

## 🛠️ 設定・ログ・可観測性

- **設定**: pydantic-settings（環境変数を型付きで束ねる）
- **ログ**: structlog（JSON構造化出力）
- **ヘルスチェック**: プロセス生存とAPI到達性を別々のシグナルとして扱う（NFR-004）

### 主要な環境変数
| 変数 | 内容 |
|---|---|
| `MEDIAVAULT_API_BASE_URL` | 内部APIのベースURL |
| `INTERNAL_API_KEY` | 内部API認証キー |
| `EXTRACTOR_LIBRARY_ROOT` / `EXTRACTOR_STORAGE_ROOT` | 許可ルート |
| `EXTRACTOR_OCR_DEVICE` | `cpu`（既定）/ `cuda` |
| `EXTRACTOR_MAX_CONCURRENCY` | 既定 1（NFR-003） |
| `EXTRACTOR_MAX_FILE_BYTES` / `EXTRACTOR_MAX_PAGES` / `EXTRACTOR_JOB_TIMEOUT_SEC` | 上限値 |
| `EXTRACTOR_POLL_INTERVAL_SEC` / `EXTRACTOR_HEARTBEAT_INTERVAL_SEC` | ポーリング間隔 |

### ログ方針
- 記録する: ジョブID、ファイルID、処理形式、ページ数、処理時間、終了状態、使用抽出方式。
- 出力しない: `INTERNAL_API_KEY`、抽出本文、個人情報。マスキングは structlog の processor 1箇所に集約し、各所で書き分けない。

## 🧪 開発ツール

- **テスト**: pytest 8 + pytest-mock（`pytest-asyncio` は同期実装のため不要）
- **リンター/フォーマッター**: Ruff 0.8+（lint と format の両方）
- **型チェック**: mypy（`--strict`）
- **コンテナ**: Docker + Docker Compose v2（既存 `docker-compose.yml` にサービス追加）
- **E2Eテスト**: 対象外（UI・公開APIを持たないため Playwright は不要）

### テスト構成（NFR-006）
- 単体: `OcrEngine` を fake に差し替え、形式判定・正規化・境界計算・状態遷移を検証。
- 結合: 小さな実PDF（テキストレイヤーあり / なし各1）と実画像を fixture に置き、抽出パイプラインを通す。
- APIクライアント: httpx のトランスポートモックで claim / heartbeat / complete / fail / cancel の各分岐を検証。

## ☁️ インフラ・デプロイ

- **配置**: 既存 `docker-compose.yml` に `mediavault-extractor` サービスを追加
- **依存**: `mediavault-api`（healthy）
- **ネットワーク**: `mediavault-api` ネットワークのみに接続（`media-db` へは接続しない）
- **CI/CD**: 未整備。導入する場合は GitHub Actions で ruff / mypy / pytest を実行

### compose 追加イメージ
```yaml
  mediavault-extractor:
    build:
      context: ./extractor
      dockerfile: Dockerfile
    restart: unless-stopped
    environment:
      MEDIAVAULT_API_BASE_URL: http://mediavault-api:8080
      INTERNAL_API_KEY: ${INTERNAL_API_KEY:?INTERNAL_API_KEY is required}
      EXTRACTOR_LIBRARY_ROOT: /library
      EXTRACTOR_STORAGE_ROOT: /srv/mediavault
      EXTRACTOR_OCR_DEVICE: ${EXTRACTOR_OCR_DEVICE:-cpu}
      EXTRACTOR_MAX_CONCURRENCY: ${EXTRACTOR_MAX_CONCURRENCY:-1}
    volumes:
      - ${MEDIAVAULT_STORAGE_SOURCE:-mediavault-storage}:/srv/mediavault:ro
      - ${LIBRARY_SOURCE:-shares}:/library:ro
    depends_on:
      mediavault-api:
        condition: service_healthy
    security_opt:
      - no-new-privileges:true
    networks:
      - mediavault-api
```

## 🔒 セキュリティ
- **認証**: 内部APIへの全リクエストに `INTERNAL_API_KEY` を付与。worker は公開APIを一切提供しない
- **ファイル**: 共有ボリュームは read-only。許可ルート外（symlink経由を含む）は開かない
- **バリデーション**: 形式・サイズ・ページ数の上限を開く前に検証する
- **環境変数**: 機密は compose の `${...}` 経由で注入し、イメージへ焼き込まない
- **依存関係**: `uv.lock` 固定 ＋ 定期的な脆弱性チェック
- **ログ**: APIキー・本文・個人情報を出力しない

## 📊 品質基準
- **テストカバレッジ**: 抽出コアロジック 80%以上
- **コード品質**: Ruff（lint + format）エラーゼロ
- **型安全性**: mypy `--strict` を通す
- **再現性**: 同一ファイル＋同一 `extraction_version` から同等の本文が再生成できること

## 📁 推奨ディレクトリ構造

```
intrahub-mediavault/
├── backend/                       # 既存: Rust (mediavault-api, mediavault-mcp)
├── frontend/                      # 既存: React
├── extractor/                     # 新規: Python worker
│   ├── src/mediavault_extractor/
│   │   ├── __main__.py            # エントリポイント（常駐ループ）
│   │   ├── config.py              # pydantic-settings
│   │   ├── logging.py             # structlog 設定・マスキング
│   │   ├── api_client.py          # 内部APIクライアント（claim/heartbeat/complete/fail/cancel）
│   │   ├── heartbeat.py           # lease延長スレッド・キャンセル伝播
│   │   ├── files.py               # パス解決と許可ルート検証
│   │   ├── detect.py              # MIME/シグネチャによる形式判定
│   │   ├── extractors/
│   │   │   ├── base.py            # Extractor Protocol
│   │   │   ├── pdf.py             # pypdfium2
│   │   │   └── image.py
│   │   ├── ocr/
│   │   │   ├── base.py            # OcrEngine Protocol / OcrResult
│   │   │   └── yomitoku.py
│   │   ├── normalize.py           # FR-005
│   │   └── boundaries.py          # FR-006（ページ/章の文字範囲とlabel）
│   ├── tests/
│   │   ├── unit/
│   │   ├── integration/
│   │   └── fixtures/              # 小さな実PDF・画像
│   ├── pyproject.toml
│   ├── uv.lock
│   └── Dockerfile
├── docker-compose.yml             # 既存に mediavault-extractor を追加
└── docs/
    └── extractor/
        ├── PRD.md
        └── tech-stack.md          # このファイル
```

## 🚀 セットアップ手順

### 1. 開発環境準備
```bash
cd extractor
uv sync
```

### 2. 主要コマンド
```bash
uv run ruff check --fix .      # lint
uv run ruff format .           # format
uv run mypy src                # 型チェック
uv run pytest                  # テスト
uv run python -m mediavault_extractor   # ローカル起動
docker compose up -d --build mediavault-extractor
```

## ⚠️ このファイルで決めていないこと

以下は技術スタックではなく設計判断のため、後続の設計フェーズ（kairo-design）で確定する。

- PRD 要調整事項 4: 抽出結果をAPIへ一括送信するか、ページ単位で段階送信するか
- PRD 要調整事項 5: ページ・章境界情報の具体的なデータ構造
- PRD 要調整事項 2: 内部ルートのパス規約（`/internal/*` と `/api/v1/internal/*` の統一）
- OCRフォールバック判定の品質基準（FR-004 の「品質基準を満たさないページ」の閾値）
- CPUでのOCR処理時間の実測に基づく NFR-003 上限値の初期設定

## 🔄 更新履歴
- 2026-08-14: 初回生成（init-tech-stack）
