from __future__ import annotations

import threading
from typing import Protocol
from uuid import UUID

import structlog

from mediavault_extractor.api_client import InvalidLeaseTokenError, TransientError

MAX_HEARTBEAT_FAILURES = 3


class HeartbeatClient(Protocol):
    def heartbeat(
        self,
        extraction_id: UUID,
        lease_token: UUID,
        progress_current: int | None,
        progress_total: int | None,
        lease_seconds: int | None,
    ) -> object: ...


class ProgressSnapshot(Protocol):
    def snapshot(self) -> tuple[int | None, int | None]: ...


class HeartbeatThread:
    """Extend one extraction lease and propagate cancellation to its worker."""

    def __init__(
        self,
        client: HeartbeatClient,
        extraction_id: UUID,
        lease_token: UUID,
        progress: ProgressSnapshot,
        cancel_event: threading.Event,
        lease_seconds: int,
        interval: float,
    ) -> None:
        self._client = client
        self._extraction_id = extraction_id
        self._lease_token = lease_token
        self._progress = progress
        self._cancel_event = cancel_event
        self._lease_seconds = lease_seconds
        self._interval = interval
        self._stop = threading.Event()
        self._thread = threading.Thread(
            target=self._run, name=f"heartbeat-{extraction_id}", daemon=True
        )
        self._log = structlog.get_logger(__name__)

    def start(self) -> HeartbeatThread:
        self._thread.start()
        return self

    def stop(self) -> None:
        self._stop.set()
        if self._thread is not threading.current_thread():
            self._thread.join()

    def _run(self) -> None:
        consecutive_failures = 0
        while not self._stop.is_set():
            current, total = self._progress.snapshot()
            try:
                result = self._client.heartbeat(
                    self._extraction_id, self._lease_token, current, total, self._lease_seconds
                )
                consecutive_failures = 0
                if bool(getattr(result, "cancel_requested", False)):
                    self._cancel_event.set()
            except InvalidLeaseTokenError:
                self._log.warning("heartbeat_lease_lost", extraction_id=str(self._extraction_id))
                self._cancel_event.set()
                return
            except TransientError:
                consecutive_failures += 1
                self._log.warning(
                    "heartbeat_failed",
                    extraction_id=str(self._extraction_id),
                    consecutive_failures=consecutive_failures,
                )
                if consecutive_failures >= MAX_HEARTBEAT_FAILURES:
                    self._cancel_event.set()
                    return
            if self._stop.wait(self._interval):
                return
