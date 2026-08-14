"""MediaVault Extractor 型定義（Python worker 側）

作成日: 2026-08-14
関連設計: architecture.md, api-endpoints.md, dataflow.md

配置先（tech-stack.md §推奨ディレクトリ構造）:
    extractor/src/mediavault_extractor/config.py       … ExtractorSettings
    extractor/src/mediavault_extractor/api_client.py   … リクエスト/レスポンス dataclass
    extractor/src/mediavault_extractor/files.py        … FileRef, resolve_file_ref
    extractor/src/mediavault_extractor/ocr/base.py     … OcrEngine, OcrResult
    extractor/src/mediavault_extractor/extractors/base.py … Extractor, ExtractionOutcome
    extractor/src/mediavault_extractor/boundaries.py   … TextBoundary

本ファイルは設計ドキュメントであり、そのままは実行されない。
実装時は上記ファイルへ分割して配置する。

方針（tech-stack.md より）:
    * mypy --strict を通す。Any を使わない
    * 同期実装。async は使わない（heartbeat のみ threading.Thread）
    * OCR は Protocol 境界で抽象化し、yomitoku 固有型を境界外へ出さない
    * 設定は pydantic-settings、それ以外の値オブジェクトは frozen dataclass

信頼性レベル:
- 🔵 青信号: EARS要件定義書・設計文書・既存実装を参考にした確実な型定義
- 🟡 黄信号: EARS要件定義書・設計文書・既存実装から妥当な推測による型定義
- 🔴 赤信号: EARS要件定義書・設計文書・既存実装にない推測による型定義
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Protocol
from uuid import UUID

from PIL.Image import Image
from pydantic_settings import BaseSettings

# ========================================
# 列挙型
# ========================================


class ExtractionState(str, Enum):
    """抽出の状態。api の extraction_state ENUM と1対1で対応する。
    🔵 信頼性: database-schema.sql の extraction_state・REQ-201〜203 に直接対応
    """

    QUEUED = "queued"
    RUNNING = "running"
    CANCELLING = "cancelling"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    CANCELLED = "cancelled"


class FileRefRoot(str, Enum):
    """ファイル参照のルート種別。worker 側のマウント点へ引き当てる。
    🔵 信頼性: 設計ヒアリングQ4（要件定義フェーズ）・architecture.md D-3 に直接対応
    """

    STORAGE = "storage"  # 🔵 EXTRACTOR_STORAGE_ROOT（アップロード経路）
    LIBRARY = "library"  # 🔵 EXTRACTOR_LIBRARY_ROOT（リンク経路）


class FileType(str, Enum):
    """api の file_type ENUM。MVPで抽出対象になるのは PDF と IMAGE のみ。
    🔵 信頼性: 既存 backend/mediavault-api/src/models/item_file.rs:16 の FileType に直接対応
    """

    PDF = "pdf"
    IMAGE = "image"
    VIDEO = "video"
    AUDIO = "audio"
    ARCHIVE = "archive"
    OTHER = "other"


class OcrDeviceSetting(str, Enum):
    """OCR実行デバイスの**設定値**。外部へ報告する値（cpu/gpu）とは異なる。
    🔵 信頼性: PRD FR-011「環境変数 EXTRACTOR_OCR_DEVICE で cpu または cuda を選択」に直接対応
    """

    CPU = "cpu"
    CUDA = "cuda"


class OcrDeviceReport(str, Enum):
    """OCR実行方式の**報告値**。complete で api へ送る。
    🔵 信頼性: PRD FR-011「外部へ報告する実行方式はそれぞれ cpu、gpu とする」に直接対応
    """

    CPU = "cpu"
    GPU = "gpu"


class ExtractionMethod(str, Enum):
    """🟡 信頼性: REQ-043・architecture.md D-8（PDFの一部ページのみOCRを表現するため）"""

    EMBEDDED_TEXT = "embedded_text"
    OCR = "ocr"
    MIXED = "mixed"


class ExtractionErrorKind(str, Enum):
    """🟡 信頼性: PRD FR-009 の分類・api-endpoints.md §fail の想定値より"""

    UNSUPPORTED_FORMAT = "unsupported_format"
    CORRUPT_FILE = "corrupt_file"
    FILE_NOT_FOUND = "file_not_found"
    SIZE_LIMIT_EXCEEDED = "size_limit_exceeded"
    OCR_FAILED = "ocr_failed"
    API_UNREACHABLE = "api_unreachable"
    LEASE_EXPIRED = "lease_expired"
    INTERNAL = "internal"


# ========================================
# 設定（pydantic-settings）
# ========================================


class ExtractorSettings(BaseSettings):
    """環境変数を型付きで束ねる。
    🔵 信頼性: tech-stack.md §主要な環境変数に直接対応（一部の既定値は 🟡）
    """

    # --- 接続 ---
    mediavault_api_base_url: str  # 🔵 内部APIのベースURL
    internal_api_key: str  # 🔵 NFR-101

    # --- 許可ルート（read-only マウント） ---
    extractor_library_root: Path = Path("/library")  # 🔵 tech-stack.md
    extractor_storage_root: Path = Path("/srv/mediavault")  # 🔵 tech-stack.md

    # --- OCR ---
    extractor_ocr_device: OcrDeviceSetting = OcrDeviceSetting.CPU  # 🔵 REQ-113・FR-011（既定 cpu）
    # 🟡 architecture.md D-7。実データでのチューニングが前提のため既定値は暫定
    extractor_ocr_fallback_min_chars_per_page: int = 50

    # --- 実行制御 ---
    extractor_max_concurrency: int = 1  # 🔵 NFR-002・tech-stack.md
    extractor_poll_interval_sec: float = 5.0  # 🟡 NFR-004（値は暫定）
    extractor_heartbeat_interval_sec: float = 30.0  # 🟡 prep.md §確認事項（lease と併せて確定）
    extractor_lease_seconds: int = 300  # 🟡 同上
    extractor_job_timeout_sec: int = 3600  # 🟡 NFR-003（CPU OCR 実測後に確定）

    # --- 上限 ---
    extractor_max_file_bytes: int = 500 * 1024 * 1024  # 🟡 EDGE-106（値は暫定）
    extractor_max_pages: int = 2000  # 🟡 EDGE-106（値は暫定）

    def allowed_root(self, root: FileRefRoot) -> Path:
        """FileRefRoot から許可ルートを引く。
        🔵 信頼性: architecture.md D-3・REQ-403 に直接対応
        """
        if root is FileRefRoot.STORAGE:
            return self.extractor_storage_root
        return self.extractor_library_root


# ========================================
# ファイル参照と安全なパス解決
# ========================================


@dataclass(frozen=True, slots=True)
class FileRef:
    """api がマウントパスに依存しない形で返すファイル参照。
    🔵 信頼性: 設計ヒアリングQ4（要件定義フェーズ）・architecture.md D-3 に直接対応
    """

    root: FileRefRoot
    relative_path: str


class UnsafePathError(Exception):
    """許可ルート外を指す参照。再試行不可（PermanentError 扱い）。
    🔵 信頼性: REQ-402・REQ-403・NFR-103 に直接対応
    """


def resolve_file_ref(ref: FileRef, settings: ExtractorSettings) -> Path:
    """FileRef を検証済みの絶対パスへ解決する。

    【重要】判定は resolve() の**後**に行う。resolve() は symlink を展開するため、
    展開後に is_relative_to で判定しなければ symlink 経由の脱出を防げない。
    そして判定を通過するまでファイルを開かない。

    🔵 信頼性: REQ-402・REQ-403・NFR-103・tech-stack.md §ファイルアクセス・
               TC-NFR-103-01 / TC-NFR-103-02 に直接対応
    """
    allowed = settings.allowed_root(ref.root).resolve()

    # 【事前拒否】: 絶対パス・".." を含む相対パスは組み立て前に弾く 🔵 REQ-402
    candidate_rel = Path(ref.relative_path)
    if candidate_rel.is_absolute() or ".." in candidate_rel.parts:
        raise UnsafePathError(f"不正な相対パスです: {ref.relative_path}")

    # 【symlink 展開後に判定】: ここが NFR-103 の要点 🔵
    resolved = (allowed / candidate_rel).resolve()
    if not resolved.is_relative_to(allowed):
        raise UnsafePathError("許可ルート外を指す参照です")

    return resolved


# ========================================
# 抽出結果の構成要素
# ========================================


@dataclass(frozen=True, slots=True)
class TextBoundary:
    """ページ・章の境界。start は含む / end は含まない（half-open）。
    いずれも**文字**オフセット（バイトではない）。
    🔵 信頼性: REQ-042・REQ-068・architecture.md D-5 に直接対応
    """

    start: int
    end: int
    label: str  # 例: "p.1" / "第3章"


@dataclass(frozen=True, slots=True)
class OcrMetadata:
    """🔵 信頼性: REQ-043・PRD FR-007 に直接対応"""

    engine: str  # 例: "yomitoku"
    device: OcrDeviceReport
    model: str


@dataclass(frozen=True, slots=True)
class ExtractorMetadata:
    """🟡 信頼性: REQ-043・architecture.md D-8（記録粒度）より"""

    method: ExtractionMethod
    embedded_text_pages: int
    ocr_pages: int
    ocr: OcrMetadata | None  # OCRを一度も使わなければ None 🔵 FR-007


@dataclass(frozen=True, slots=True)
class ExtractionOutcome:
    """抽出パイプラインの出力。complete リクエストの素になる。
    🔵 信頼性: REQ-065・FR-007 に直接対応
    """

    content: str  # 正規化済み（REQ-063）
    boundaries: tuple[TextBoundary, ...]  # 🔵 REQ-068
    extraction_version: str  # 🔵 REQ-104
    extractor: ExtractorMetadata  # 🔵 REQ-043


# ========================================
# OCR 境界（Protocol）
# ========================================


@dataclass(frozen=True, slots=True)
class OcrResult:
    """OcrEngine の戻り値。yomitoku 固有型をこの境界の外へ出さない。
    🔵 信頼性: REQ-069・tech-stack.md §OCR「入力は PIL.Image、出力は OcrResult に固定」に直接対応
    """

    text: str
    confidence: float | None


class OcrEngine(Protocol):
    """OCRエンジンの差し替え境界。

    MVP実装は yomitoku のみ。将来 ndlocr-lite / Tesseract へ差し替える際も、
    この Protocol より外側のコード（pdf.py / image.py / normalize.py / api_client.py）は
    変更しない。単体テストではこの Protocol を満たす fake に差し替える。

    🔵 信頼性: REQ-069・PRD §12・tech-stack.md §OCR・TC-060-07 に直接対応
    """

    @property
    def engine_name(self) -> str:
        """complete で報告するエンジン識別子。🔵 REQ-043"""
        ...

    @property
    def model_id(self) -> str:
        """complete で報告するモデル識別子。🔵 REQ-043"""
        ...

    @property
    def device(self) -> OcrDeviceReport:
        """complete で報告する実行方式。起動時に確定し変わらない。🔵 REQ-411"""
        ...

    def ocr(self, image: Image) -> OcrResult:
        """1枚の画像からテキストを抽出する。🔵 tech-stack.md §OCR"""
        ...


class OcrDeviceUnavailableError(Exception):
    """cuda 指定時に CUDA GPU を利用できない。

    起動時に送出し、抽出を claim する前にプロセスを終了させる。
    yomitoku の暗黙CPUフォールバックには依存しない。

    🔵 信頼性: REQ-113・REQ-412・PRD §8.9・TC-NFR-301-03 に直接対応
    """


# ========================================
# 抽出器境界（Protocol）
# ========================================


class CancelledError(Exception):
    """キャンセル要求を検知して中断した。complete を送ってはならない。
    🔵 信頼性: REQ-207・PRD FR-008 に直接対応
    """


class TransientError(Exception):
    """一時的な失敗。fail(retryable=True) として報告する。
    🔵 信頼性: REQ-109・PRD FR-009・tech-stack.md 実行ループに直接対応
    """

    kind: ExtractionErrorKind = ExtractionErrorKind.API_UNREACHABLE


class PermanentError(Exception):
    """恒久的な失敗。fail(retryable=False) として報告し、無限再試行しない。
    🔵 信頼性: REQ-110・PRD FR-009・tech-stack.md 実行ループに直接対応
    """

    def __init__(self, kind: ExtractionErrorKind, message: str) -> None:
        super().__init__(message)
        self.kind = kind


class ProgressReporter(Protocol):
    """抽出器が進捗を報告し、キャンセルを確認するための境界。

    実装は heartbeat スレッドと threading.Event を共有する。抽出器側は
    HTTPやスレッドを知らずに済む。

    🔵 信頼性: REQ-066・REQ-207・tech-stack.md §worker 実行モデルに直接対応
    """

    def report(self, current: int, total: int) -> None:
        """ページ境界で進捗を報告する。🔵 REQ-023・REQ-066"""
        ...

    def is_cancelled(self) -> bool:
        """キャンセル要求の有無。ページ境界でのみ確認する。🔵 REQ-066・REQ-207"""
        ...


class Extractor(Protocol):
    """形式ごとの抽出器（pdf.py / image.py）。
    🔵 信頼性: tech-stack.md §推奨ディレクトリ構造 extractors/base.py に直接対応
    """

    def extract(
        self,
        path: Path,
        ocr: OcrEngine,
        progress: ProgressReporter,
    ) -> ExtractionOutcome:
        """検証済みパスからテキストを抽出する。

        Raises:
            CancelledError: ページ境界でキャンセルを検知した（REQ-207）
            PermanentError: 破損・未対応形式・上限超過（REQ-110）
            TransientError: 一時的なI/O失敗（REQ-109）
        """
        ...


# ========================================
# 形式判定
# ========================================


@dataclass(frozen=True, slots=True)
class DetectedFormat:
    """MIME/シグネチャによる形式判定の結果。
    🔵 信頼性: REQ-062・PRD FR-003・tech-stack.md §形式判定に直接対応
    """

    file_type: FileType
    mime_type: str
    """拡張子とシグネチャが食い違っていたか。True なら PermanentError とする
    🔵 EDGE-005・tech-stack.md「拡張子とシグネチャの不一致を明示的なエラーとして扱う」"""
    extension_mismatch: bool


# ========================================
# 内部APIクライアント
# ========================================


@dataclass(frozen=True, slots=True)
class ClaimedExtraction:
    """claim レスポンス。
    🔵 信頼性: REQ-021・REQ-022・api-endpoints.md §claim に直接対応
    """

    extraction_id: UUID
    item_file_id: UUID
    item_id: UUID
    file_type: FileType
    size_bytes: int
    attempts: int
    lease_token: UUID
    lease_expires_at: datetime
    file_ref: FileRef


@dataclass(frozen=True, slots=True)
class HeartbeatResult:
    """heartbeat レスポンス。
    🔵 信頼性: REQ-023・REQ-202・api-endpoints.md §heartbeat に直接対応
    """

    state: ExtractionState
    cancel_requested: bool
    lease_expires_at: datetime


@dataclass(frozen=True, slots=True)
class ExtractionFailure:
    """fail リクエストの error 本体。
    🔵 信頼性: REQ-026・api-endpoints.md §fail に直接対応
    """

    kind: ExtractionErrorKind
    message: str
    retryable: bool


class ExtractorApiClient(Protocol):
    """内部APIクライアントの境界。

    テストでは httpx のトランスポートモックで差し替える
    （tech-stack.md §テスト構成「claim / heartbeat / complete / fail / cancel の各分岐を検証」）。

    すべてのリクエストに INTERNAL_API_KEY を付与する（NFR-101）。
    一時的な通信失敗は tenacity の指数バックオフでリトライし、
    それでも失敗した場合のみ TransientError を送出する（REQ-109）。

    🔵 信頼性: REQ-020〜027・api-endpoints.md §内部API に直接対応
    """

    def claim(self, worker_id: str, lease_seconds: int) -> ClaimedExtraction | None:
        """実行可能な抽出を1件取得する。なければ None。🔵 REQ-020"""
        ...

    def heartbeat(
        self,
        extraction_id: UUID,
        lease_token: UUID,
        progress_current: int | None,
        progress_total: int | None,
        lease_seconds: int | None,
    ) -> HeartbeatResult:
        """lease延長・進捗更新・キャンセル要求取得を1リクエストで行う。🔵 REQ-023"""
        ...

    def complete(
        self,
        extraction_id: UUID,
        lease_token: UUID,
        outcome: ExtractionOutcome,
        extracted_at: datetime,
    ) -> None:
        """抽出結果を一括送信して succeeded へ遷移させる。🔵 REQ-024・REQ-067"""
        ...

    def fail(
        self,
        extraction_id: UUID,
        lease_token: UUID,
        failure: ExtractionFailure,
    ) -> None:
        """構造化エラーを報告する。🔵 REQ-026"""
        ...

    def cancelled(self, extraction_id: UUID, lease_token: UUID) -> None:
        """キャンセル確認を報告して cancelled へ遷移させる。🔵 REQ-027"""
        ...


class InvalidLeaseTokenError(Exception):
    """complete/fail/cancelled が 409 INVALID_LEASE_TOKEN で拒否された。

    lease 失効後に別 worker が再claimしている。**再試行してはならない。**
    ログに記録してループの次の周回へ進む。

    🔵 信頼性: REQ-407・EDGE-002・TC-024-E01・TC-EDGE-008-01 に直接対応
    """


# ========================================
# 実行ループ（骨子）
# ========================================


@dataclass(slots=True)
class WorkerContext:
    """1回の抽出試行のコンテキスト。
    🟡 信頼性: tech-stack.md §実行ループの骨子から妥当な推測
    """

    claimed: ClaimedExtraction
    resolved_path: Path
    ocr: OcrEngine
    progress: ProgressReporter
    started_at: datetime
    pages_done: int = 0
    boundaries: list[TextBoundary] = field(default_factory=list)


def needs_ocr(
    page_text: str,
    page_area_pt2: float,
    min_chars_per_page: int,
    a4_area_pt2: float = 595.0 * 842.0,
) -> bool:
    """このページをOCRへフォールバックすべきか判定する。

    「テキストが存在しない」だけでなく「品質基準を満たさない」（FR-004）を扱うため、
    ページ面積あたりの文字数で判定する。文字数0のみを条件にすると、文字化けPDFや
    透かしテキストだけのスキャンPDFを取りこぼす。

    閾値は EXTRACTOR_OCR_FALLBACK_MIN_CHARS_PER_PAGE で調整でき、実データで
    チューニングできる。

    🔵 信頼性: REQ-106・PRD FR-004・設計ヒアリングQ4・architecture.md D-7 に直接対応
              （既定値 50 そのものは 🟡）
    """
    if not page_text.strip():
        return True
    normalized = len(page_text) * (a4_area_pt2 / page_area_pt2)
    return normalized < min_chars_per_page


# 実行ループの骨子（tech-stack.md §実行ループの骨子を本設計で具体化したもの）
# 🔵 信頼性: dataflow.md §フロー2・tech-stack.md に直接対応
#
# def run_loop(client: ExtractorApiClient, settings: ExtractorSettings, ocr: OcrEngine) -> None:
#     while not shutdown_requested():
#         claimed = client.claim(worker_id, settings.extractor_lease_seconds)
#         if claimed is None:
#             sleep(settings.extractor_poll_interval_sec)
#             continue
#
#         cancel_event = threading.Event()
#         hb = start_heartbeat_thread(client, claimed, cancel_event, settings)
#         try:
#             path = resolve_file_ref(claimed.file_ref, settings)   # REQ-403（開く前に検証）
#             check_size_limits(path, claimed.size_bytes, settings)  # EDGE-106（開く前）
#             fmt = detect_format(path)                              # REQ-062・EDGE-005
#             extractor = select_extractor(fmt)                      # REQ-062
#
#             outcome = extractor.extract(path, ocr, progress)       # ページ境界で cancel 確認
#
#             if cancel_event.is_set():                              # REQ-207（二重の担保）
#                 client.cancelled(claimed.extraction_id, claimed.lease_token)
#             else:
#                 client.complete(claimed.extraction_id, claimed.lease_token,
#                                 outcome, now())
#         except CancelledError:
#             client.cancelled(claimed.extraction_id, claimed.lease_token)
#         except PermanentError as e:
#             client.fail(..., ExtractionFailure(e.kind, str(e), retryable=False))
#         except TransientError as e:
#             client.fail(..., ExtractionFailure(e.kind, str(e), retryable=True))
#         except InvalidLeaseTokenError:
#             log.warning("lease失効。別workerが再claim済み")   # 再試行しない（EDGE-002）
#         finally:
#             hb.stop()

# ========================================
# 信頼性レベルサマリー
# ========================================
# 型定義・関数・Protocol 38件の内訳:
# - 🔵 青信号: 29件 (76%)
# - 🟡 黄信号: 9件 (24%)
# - 🔴 赤信号: 0件 (0%)
#
# 品質評価: ✅ 高品質
#
# 🟡 の内訳: ExtractorSettings の未確定な既定値6件（ポーリング間隔・heartbeat 間隔・
# lease 期間・タイムアウト・ファイルサイズ上限・ページ数上限。いずれも prep.md §確認事項
# または NFR-003 の実測待ち）、ExtractionMethod / ExtractionErrorKind の値域、WorkerContext。
