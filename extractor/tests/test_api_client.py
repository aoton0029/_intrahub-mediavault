from datetime import datetime
from uuid import UUID

import httpx
import pytest

from mediavault_extractor.api_client import (
    ExtractionErrorKind,
    ExtractionFailure,
    ExtractionMethod,
    ExtractionOutcome,
    ExtractorMetadata,
    FileRefRoot,
    HttpExtractorApiClient,
    InvalidLeaseTokenError,
    OcrDeviceReport,
    OcrMetadata,
    PermanentError,
    TextBoundary,
    TransientError,
)

EXTRACTION_ID = UUID("e1a2b3c4-0000-0000-0000-000000000001")
LEASE_TOKEN = UUID("9f8e7d6c-0000-0000-0000-00000000000a")


def _client(handler: httpx.MockTransport) -> HttpExtractorApiClient:
    return HttpExtractorApiClient(
        "http://api:8080", "secret", 5.0, transport=handler, retry_wait=lambda _: 0.0
    )


def _ok(data: object = None) -> httpx.Response:
    return httpx.Response(200, json={"success": True, "data": data})


def test_claim_success_and_authorization() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path == "/api/v1/internal/extractions/claim"
        assert request.headers["Authorization"] == "secret"
        return _ok(
            {
                "extraction_id": str(EXTRACTION_ID),
                "item_file_id": "f1a2b3c4-1e3e-4c9a-9c3e-2f6b1a2a0002",
                "item_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
                "file_type": "pdf",
                "size_bytes": 123,
                "attempts": 1,
                "lease_token": str(LEASE_TOKEN),
                "lease_expires_at": "2026-08-14T09:35:00",
                "file_ref": {"root": "storage", "relative_path": "item/file.pdf"},
            }
        )

    with _client(httpx.MockTransport(handler)) as client:
        claimed = client.claim("worker-1", 300)
    assert claimed is not None
    assert claimed.file_ref.root is FileRefRoot.STORAGE


def test_claim_none() -> None:
    with _client(httpx.MockTransport(lambda _: _ok())) as client:
        assert client.claim("worker-1", 300) is None


def test_retries_server_errors_until_success() -> None:
    calls = 0

    def handler(_: httpx.Request) -> httpx.Response:
        nonlocal calls
        calls += 1
        return httpx.Response(503) if calls < 3 else _ok()

    with _client(httpx.MockTransport(handler)) as client:
        assert client.claim("worker-1", 300) is None
    assert calls == 3


@pytest.mark.parametrize(
    ("status", "code", "exception"),
    [(409, "INVALID_LEASE_TOKEN", InvalidLeaseTokenError), (401, "UNAUTHORIZED", PermanentError)],
)
def test_non_retryable_errors(status: int, code: str, exception: type[Exception]) -> None:
    calls = 0

    def handler(_: httpx.Request) -> httpx.Response:
        nonlocal calls
        calls += 1
        return httpx.Response(status, json={"error": {"code": code}})

    with _client(httpx.MockTransport(handler)) as client, pytest.raises(exception):
        client.cancelled(EXTRACTION_ID, LEASE_TOKEN)
    assert calls == 1


def test_transport_error_becomes_transient_after_five_attempts() -> None:
    calls = 0

    def handler(request: httpx.Request) -> httpx.Response:
        nonlocal calls
        calls += 1
        raise httpx.ConnectError("offline", request=request)

    with _client(httpx.MockTransport(handler)) as client, pytest.raises(TransientError) as exc_info:
        client.claim("worker-1", 300)
    assert exc_info.value.kind is ExtractionErrorKind.API_UNREACHABLE
    assert calls == 5


def test_heartbeat_cancel_requested() -> None:
    with _client(
        httpx.MockTransport(
            lambda _: _ok(
                {
                    "state": "cancelling",
                    "cancel_requested": True,
                    "lease_expires_at": "2026-08-14T09:40:00",
                }
            )
        )
    ) as client:
        result = client.heartbeat(EXTRACTION_ID, LEASE_TOKEN, 5, 10, 300)
    assert result.cancel_requested is True


def test_complete_payload_reports_gpu() -> None:
    captured: dict[str, object] = {}

    def handler(request: httpx.Request) -> httpx.Response:
        captured.update(request.read() and __import__("json").loads(request.content))
        assert request.headers["Authorization"] == "secret"
        return _ok({})

    outcome = ExtractionOutcome(
        content="text",
        boundaries=(TextBoundary(0, 4, "p.1"),),
        extraction_version="pdf-v1",
        extractor=ExtractorMetadata(
            ExtractionMethod.MIXED, 1, 1, OcrMetadata("yomitoku", OcrDeviceReport.GPU, "v1")
        ),
    )
    with _client(httpx.MockTransport(handler)) as client:
        client.complete(EXTRACTION_ID, LEASE_TOKEN, outcome, datetime(2026, 8, 14, 9, 45))
    assert captured["content"] == "text"
    assert captured["boundaries"] == [{"start": 0, "end": 4, "label": "p.1"}]
    extractor = captured["extractor"]
    assert isinstance(extractor, dict)
    assert extractor["ocr"] == {"engine": "yomitoku", "device": "gpu", "model": "v1"}


def test_complete_payload_has_null_ocr() -> None:
    captured: dict[str, object] = {}

    def handler(request: httpx.Request) -> httpx.Response:
        captured.update(__import__("json").loads(request.content))
        return _ok({})

    outcome = ExtractionOutcome(
        content="text",
        boundaries=(),
        extraction_version="image-v1",
        extractor=ExtractorMetadata(ExtractionMethod.EMBEDDED_TEXT, 1, 0, None),
    )
    with _client(httpx.MockTransport(handler)) as client:
        client.complete(EXTRACTION_ID, LEASE_TOKEN, outcome, datetime(2026, 8, 14))
    extractor = captured["extractor"]
    assert isinstance(extractor, dict)
    assert extractor["ocr"] is None


def test_fail_payload() -> None:
    captured: dict[str, object] = {}

    def handler(request: httpx.Request) -> httpx.Response:
        captured.update(__import__("json").loads(request.content))
        return _ok({})

    failure = ExtractionFailure(ExtractionErrorKind.OCR_FAILED, "failed", False)
    with _client(httpx.MockTransport(handler)) as client:
        client.fail(EXTRACTION_ID, LEASE_TOKEN, failure)
    assert captured["error"] == {
        "kind": "ocr_failed",
        "message": "failed",
        "retryable": False,
    }
