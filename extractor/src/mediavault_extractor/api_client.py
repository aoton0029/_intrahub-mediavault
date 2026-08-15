from __future__ import annotations

from collections.abc import Callable, Mapping
from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from time import monotonic
from typing import cast
from uuid import UUID

import httpx
import structlog
from tenacity import RetryCallState, Retrying, retry_if_exception_type, stop_after_attempt
from tenacity.wait import wait_exponential

from mediavault_extractor.config import FileRefRoot


class FileType(StrEnum):
    PDF = "pdf"
    IMAGE = "image"
    VIDEO = "video"
    AUDIO = "audio"
    ARCHIVE = "archive"
    OTHER = "other"


class ExtractionState(StrEnum):
    QUEUED = "queued"
    RUNNING = "running"
    CANCELLING = "cancelling"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    CANCELLED = "cancelled"


class OcrDeviceReport(StrEnum):
    CPU = "cpu"
    GPU = "gpu"


class ExtractionMethod(StrEnum):
    EMBEDDED_TEXT = "embedded_text"
    OCR = "ocr"
    MIXED = "mixed"


class ExtractionErrorKind(StrEnum):
    UNSUPPORTED_FORMAT = "unsupported_format"
    CORRUPT_FILE = "corrupt_file"
    FILE_NOT_FOUND = "file_not_found"
    SIZE_LIMIT_EXCEEDED = "size_limit_exceeded"
    OCR_FAILED = "ocr_failed"
    API_UNREACHABLE = "api_unreachable"
    LEASE_EXPIRED = "lease_expired"
    INTERNAL = "internal"


@dataclass(frozen=True, slots=True)
class FileRef:
    root: FileRefRoot
    relative_path: str


@dataclass(frozen=True, slots=True)
class ClaimedExtraction:
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
    state: ExtractionState
    cancel_requested: bool
    lease_expires_at: datetime


@dataclass(frozen=True, slots=True)
class TextBoundary:
    start: int
    end: int
    label: str


@dataclass(frozen=True, slots=True)
class OcrMetadata:
    engine: str
    device: OcrDeviceReport
    model: str


@dataclass(frozen=True, slots=True)
class ExtractorMetadata:
    method: ExtractionMethod
    embedded_text_pages: int
    ocr_pages: int
    ocr: OcrMetadata | None


@dataclass(frozen=True, slots=True)
class ExtractionOutcome:
    content: str
    boundaries: tuple[TextBoundary, ...]
    extraction_version: str
    extractor: ExtractorMetadata


@dataclass(frozen=True, slots=True)
class ExtractionFailure:
    kind: ExtractionErrorKind
    message: str
    retryable: bool


class InvalidLeaseTokenError(Exception):
    """The lease is no longer owned by this worker and must not be retried."""


class TransientError(Exception):
    kind = ExtractionErrorKind.API_UNREACHABLE


class PermanentError(Exception):
    def __init__(self, kind: ExtractionErrorKind, message: str) -> None:
        super().__init__(message)
        self.kind = kind


class _ServerError(Exception):
    pass


WaitFunction = Callable[[RetryCallState], float]


class HttpExtractorApiClient:
    """Synchronous client for the worker-only extraction API."""

    def __init__(
        self,
        base_url: str,
        api_key: str,
        timeout: float,
        *,
        transport: httpx.BaseTransport | None = None,
        retry_wait: WaitFunction | None = None,
    ) -> None:
        internal_base = f"{base_url.rstrip('/')}/api/v1/internal"
        self._client = httpx.Client(
            base_url=internal_base,
            headers={"Authorization": api_key},
            timeout=timeout,
            transport=transport,
        )
        self._retry_wait = retry_wait or cast(WaitFunction, wait_exponential(multiplier=1, max=30))
        self._log = structlog.get_logger(__name__)

    def __enter__(self) -> HttpExtractorApiClient:
        return self

    def __exit__(self, exc_type: object, exc_value: object, traceback: object) -> None:
        self.close()

    def close(self) -> None:
        self._client.close()

    def claim(self, worker_id: str, lease_seconds: int) -> ClaimedExtraction | None:
        response = self._request(
            "POST", "/extractions/claim", {"worker_id": worker_id, "lease_seconds": lease_seconds}
        )
        data = _response_data(response)
        if data is None:
            return None
        return _parse_claimed(_mapping(data))

    def heartbeat(
        self,
        extraction_id: UUID,
        lease_token: UUID,
        progress_current: int | None,
        progress_total: int | None,
        lease_seconds: int | None,
    ) -> HeartbeatResult:
        payload: dict[str, object] = {
            "lease_token": str(lease_token),
            "progress_current": progress_current,
            "progress_total": progress_total,
            "lease_seconds": lease_seconds,
        }
        response = self._request(
            "POST", f"/extractions/{extraction_id}/heartbeat", payload, extraction_id
        )
        data = _mapping(_response_data(response))
        return HeartbeatResult(
            state=ExtractionState(_string(data, "state")),
            cancel_requested=_boolean(data, "cancel_requested"),
            lease_expires_at=datetime.fromisoformat(_string(data, "lease_expires_at")),
        )

    def complete(
        self,
        extraction_id: UUID,
        lease_token: UUID,
        outcome: ExtractionOutcome,
        extracted_at: datetime,
    ) -> None:
        ocr = outcome.extractor.ocr
        payload: dict[str, object] = {
            "lease_token": str(lease_token),
            "content": outcome.content,
            "boundaries": [
                {"start": item.start, "end": item.end, "label": item.label}
                for item in outcome.boundaries
            ],
            "extraction_version": outcome.extraction_version,
            "extracted_at": extracted_at.isoformat(),
            "extractor": {
                "method": outcome.extractor.method.value,
                "embedded_text_pages": outcome.extractor.embedded_text_pages,
                "ocr_pages": outcome.extractor.ocr_pages,
                "ocr": None
                if ocr is None
                else {"engine": ocr.engine, "device": ocr.device.value, "model": ocr.model},
            },
        }
        self._request("POST", f"/extractions/{extraction_id}/complete", payload, extraction_id)

    def fail(self, extraction_id: UUID, lease_token: UUID, failure: ExtractionFailure) -> None:
        payload: dict[str, object] = {
            "lease_token": str(lease_token),
            "error": {
                "kind": failure.kind.value,
                "message": failure.message,
                "retryable": failure.retryable,
            },
        }
        self._request("POST", f"/extractions/{extraction_id}/fail", payload, extraction_id)

    def cancelled(self, extraction_id: UUID, lease_token: UUID) -> None:
        self._request(
            "POST",
            f"/extractions/{extraction_id}/cancelled",
            {"lease_token": str(lease_token)},
            extraction_id,
        )

    def _request(
        self,
        method: str,
        path: str,
        json: Mapping[str, object],
        extraction_id: UUID | None = None,
    ) -> httpx.Response:
        operation = path.rsplit("/", maxsplit=1)[-1]
        started = monotonic()
        try:
            for attempt in Retrying(
                retry=retry_if_exception_type((httpx.TransportError, _ServerError)),
                wait=self._retry_wait,
                stop=stop_after_attempt(5),
                reraise=True,
            ):
                with attempt:
                    response = self._client.request(method, path, json=json)
                    if response.status_code >= 500:
                        raise _ServerError(f"internal API returned {response.status_code}")
                    self._raise_for_client_error(response)
                    self._log.info(
                        "internal_api_request",
                        extraction_id=str(extraction_id) if extraction_id else None,
                        operation=operation,
                        duration_ms=round((monotonic() - started) * 1000, 2),
                        result="success",
                        attempt=attempt.retry_state.attempt_number,
                    )
                    return response
        except (httpx.TransportError, _ServerError) as exc:
            self._log.warning(
                "internal_api_request",
                extraction_id=str(extraction_id) if extraction_id else None,
                operation=operation,
                duration_ms=round((monotonic() - started) * 1000, 2),
                result="retry_exhausted",
            )
            raise TransientError(f"internal API is unreachable: {exc}") from exc
        raise AssertionError("retry loop completed without a response")

    @staticmethod
    def _raise_for_client_error(response: httpx.Response) -> None:
        if response.status_code < 400:
            return
        code = _error_code(response)
        if response.status_code == 409 and code == "INVALID_LEASE_TOKEN":
            raise InvalidLeaseTokenError("lease token is invalid or expired")
        message = f"internal API returned {response.status_code} ({code or 'unknown error'})"
        raise PermanentError(ExtractionErrorKind.INTERNAL, message)


def _json_object(response: httpx.Response) -> Mapping[str, object]:
    value = cast(object, response.json())
    return _mapping(value)


def _response_data(response: httpx.Response) -> object:
    return _json_object(response).get("data")


def _error_code(response: httpx.Response) -> str | None:
    try:
        error = _json_object(response).get("error")
    except (ValueError, TypeError):
        return None
    if not isinstance(error, Mapping):
        return None
    code = error.get("code")
    return code if isinstance(code, str) else None


def _mapping(value: object) -> Mapping[str, object]:
    if not isinstance(value, Mapping):
        raise PermanentError(ExtractionErrorKind.INTERNAL, "internal API returned invalid JSON")
    return cast(Mapping[str, object], value)


def _string(data: Mapping[str, object], key: str) -> str:
    value = data.get(key)
    if not isinstance(value, str):
        raise PermanentError(ExtractionErrorKind.INTERNAL, f"response field {key} is invalid")
    return value


def _integer(data: Mapping[str, object], key: str) -> int:
    value = data.get(key)
    if not isinstance(value, int) or isinstance(value, bool):
        raise PermanentError(ExtractionErrorKind.INTERNAL, f"response field {key} is invalid")
    return value


def _boolean(data: Mapping[str, object], key: str) -> bool:
    value = data.get(key)
    if not isinstance(value, bool):
        raise PermanentError(ExtractionErrorKind.INTERNAL, f"response field {key} is invalid")
    return value


def _parse_claimed(data: Mapping[str, object]) -> ClaimedExtraction:
    file_ref = _mapping(data.get("file_ref"))
    return ClaimedExtraction(
        extraction_id=UUID(_string(data, "extraction_id")),
        item_file_id=UUID(_string(data, "item_file_id")),
        item_id=UUID(_string(data, "item_id")),
        file_type=FileType(_string(data, "file_type")),
        size_bytes=_integer(data, "size_bytes"),
        attempts=_integer(data, "attempts"),
        lease_token=UUID(_string(data, "lease_token")),
        lease_expires_at=datetime.fromisoformat(_string(data, "lease_expires_at")),
        file_ref=FileRef(
            root=FileRefRoot(_string(file_ref, "root")),
            relative_path=_string(file_ref, "relative_path"),
        ),
    )
