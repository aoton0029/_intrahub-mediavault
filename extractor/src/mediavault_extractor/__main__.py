from __future__ import annotations

import signal
import socket
import threading
import time
from collections.abc import Callable
from datetime import UTC, datetime
from pathlib import Path
from time import monotonic
from typing import Protocol
from uuid import UUID

import structlog

from mediavault_extractor.api_client import (
    ClaimedExtraction,
    ExtractionErrorKind,
    ExtractionFailure,
    ExtractionOutcome,
    FileRef,
    FileType,
    HttpExtractorApiClient,
    InvalidLeaseTokenError,
    PermanentError,
    TransientError,
)
from mediavault_extractor.config import ExtractorSettings
from mediavault_extractor.detect import DetectedFormat, detect_format
from mediavault_extractor.extractors import ImageExtractor, PdfExtractor
from mediavault_extractor.extractors.base import CancelledError, Extractor
from mediavault_extractor.files import UnsafePathError, check_size_limit, resolve_file_ref
from mediavault_extractor.heartbeat import HeartbeatThread
from mediavault_extractor.logging import configure_logging
from mediavault_extractor.ocr.base import OcrEngine
from mediavault_extractor.ocr.yomitoku import YomitokuOcrEngine


class ExtractorApiClient(Protocol):
    def claim(self, worker_id: str, lease_seconds: int) -> ClaimedExtraction | None: ...
    def heartbeat(
        self,
        extraction_id: UUID,
        lease_token: UUID,
        progress_current: int | None,
        progress_total: int | None,
        lease_seconds: int | None,
    ) -> object: ...
    def complete(
        self,
        extraction_id: UUID,
        lease_token: UUID,
        outcome: ExtractionOutcome,
        extracted_at: datetime,
    ) -> None: ...
    def fail(self, extraction_id: UUID, lease_token: UUID, failure: ExtractionFailure) -> None: ...
    def cancelled(self, extraction_id: UUID, lease_token: UUID) -> None: ...


class SharedProgress:
    def __init__(self, cancel_event: threading.Event) -> None:
        self._cancel_event = cancel_event
        self._lock = threading.Lock()
        self._current: int | None = None
        self._total: int | None = None

    def report(self, current: int, total: int) -> None:
        with self._lock:
            self._current, self._total = current, total

    def is_cancelled(self) -> bool:
        return self._cancel_event.is_set()

    def snapshot(self) -> tuple[int | None, int | None]:
        with self._lock:
            return self._current, self._total


HeartbeatFactory = Callable[
    [ExtractorApiClient, ClaimedExtraction, SharedProgress, threading.Event, ExtractorSettings],
    HeartbeatThread,
]


def _start_heartbeat(
    client: ExtractorApiClient,
    claimed: ClaimedExtraction,
    progress: SharedProgress,
    cancel_event: threading.Event,
    settings: ExtractorSettings,
) -> HeartbeatThread:
    return HeartbeatThread(
        client,
        claimed.extraction_id,
        claimed.lease_token,
        progress,
        cancel_event,
        settings.extractor_lease_seconds,
        settings.extractor_heartbeat_interval_sec,
    ).start()


def select_extractor(fmt: DetectedFormat, settings: ExtractorSettings) -> Extractor:
    if fmt.extension_mismatch:
        raise PermanentError(
            ExtractionErrorKind.UNSUPPORTED_FORMAT,
            "file extension does not match its detected format",
        )
    if fmt.file_type is FileType.PDF:
        return PdfExtractor(settings)
    if fmt.file_type is FileType.IMAGE:
        return ImageExtractor()
    raise PermanentError(ExtractionErrorKind.UNSUPPORTED_FORMAT, "unsupported file format")


def run_loop(
    client: ExtractorApiClient,
    settings: ExtractorSettings,
    ocr: OcrEngine,
    *,
    shutdown_event: threading.Event | None = None,
    worker_id: str | None = None,
    sleep: Callable[[float], None] = time.sleep,
    heartbeat_factory: HeartbeatFactory = _start_heartbeat,
    path_resolver: Callable[[FileRef, ExtractorSettings], Path] = resolve_file_ref,
    format_detector: Callable[[Path], DetectedFormat] = detect_format,
    extractor_selector: Callable[[DetectedFormat, ExtractorSettings], Extractor] = select_extractor,
) -> None:
    shutdown = shutdown_event or threading.Event()
    identity = worker_id or socket.gethostname()
    log = structlog.get_logger(__name__)

    while not shutdown.is_set():
        try:
            claimed = client.claim(identity, settings.extractor_lease_seconds)
        except TransientError as exc:
            log.warning("claim_failed", error=str(exc))
            sleep(settings.extractor_poll_interval_sec)
            continue
        if claimed is None:
            sleep(settings.extractor_poll_interval_sec)
            continue

        cancel_event = threading.Event()
        progress = SharedProgress(cancel_event)
        heartbeat = heartbeat_factory(client, claimed, progress, cancel_event, settings)
        started = monotonic()
        file_type = claimed.file_type.value
        outcome_name = "failed"
        pages = 0
        try:
            path = path_resolver(claimed.file_ref, settings)
            check_size_limit(path, settings)
            fmt = format_detector(path)
            file_type = fmt.file_type.value
            extractor = extractor_selector(fmt, settings)
            outcome = extractor.extract(path, ocr, progress)
            pages = len(outcome.boundaries)
            if cancel_event.is_set():
                client.cancelled(claimed.extraction_id, claimed.lease_token)
                outcome_name = "cancelled"
            else:
                client.complete(
                    claimed.extraction_id,
                    claimed.lease_token,
                    outcome,
                    datetime.now(UTC).replace(tzinfo=None),
                )
                outcome_name = "succeeded"
        except CancelledError:
            client.cancelled(claimed.extraction_id, claimed.lease_token)
            outcome_name = "cancelled"
        except (UnsafePathError, FileNotFoundError) as exc:
            client.fail(
                claimed.extraction_id,
                claimed.lease_token,
                ExtractionFailure(ExtractionErrorKind.FILE_NOT_FOUND, str(exc), False),
            )
        except PermanentError as exc:
            client.fail(
                claimed.extraction_id,
                claimed.lease_token,
                ExtractionFailure(exc.kind, str(exc), False),
            )
        except TransientError as exc:
            client.fail(
                claimed.extraction_id,
                claimed.lease_token,
                ExtractionFailure(exc.kind, str(exc), True),
            )
        except InvalidLeaseTokenError:
            log.warning("lease_lost", extraction_id=str(claimed.extraction_id))
            outcome_name = "lease_lost"
        finally:
            heartbeat.stop()
            log.info(
                "extraction_finished",
                extraction_id=str(claimed.extraction_id),
                item_file_id=str(claimed.item_file_id),
                file_type=file_type,
                pages=pages,
                duration_sec=round(monotonic() - started, 3),
                outcome=outcome_name,
                ocr_device=ocr.device.value,
                ocr_engine=ocr.engine_name,
                ocr_model=ocr.model_id,
            )


def main() -> None:
    settings = ExtractorSettings()
    configure_logging()
    if settings.extractor_max_concurrency != 1:
        raise ValueError("EXTRACTOR_MAX_CONCURRENCY must be 1")
    # Device validation and model loading must finish before the first claim.
    ocr = YomitokuOcrEngine(settings.extractor_ocr_device)
    shutdown = threading.Event()

    def request_shutdown(_signum: int, _frame: object) -> None:
        shutdown.set()

    signal.signal(signal.SIGTERM, request_shutdown)
    signal.signal(signal.SIGINT, request_shutdown)
    with HttpExtractorApiClient(
        str(settings.mediavault_api_base_url), settings.internal_api_key, timeout=30.0
    ) as client:
        run_loop(client, settings, ocr, shutdown_event=shutdown)


if __name__ == "__main__":
    main()
