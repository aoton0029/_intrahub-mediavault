from __future__ import annotations

import threading
from datetime import UTC, datetime
from pathlib import Path
from uuid import uuid4

import pytest

from mediavault_extractor.__main__ import run_loop
from mediavault_extractor.api_client import (
    ClaimedExtraction,
    ExtractionErrorKind,
    ExtractionMethod,
    ExtractionOutcome,
    ExtractorMetadata,
    FileRef,
    FileType,
    InvalidLeaseTokenError,
    OcrDeviceReport,
    PermanentError,
    TransientError,
)
from mediavault_extractor.config import ExtractorSettings, FileRefRoot
from mediavault_extractor.detect import DetectedFormat
from mediavault_extractor.extractors.base import CancelledError


class Ocr:
    engine_name = "fake"
    model_id = "fake-v1"
    device = OcrDeviceReport.CPU

    def ocr(self, image: object) -> object:
        raise AssertionError("not used")


class Heartbeat:
    def __init__(self) -> None:
        self.stopped = False

    def stop(self) -> None:
        self.stopped = True


def claimed() -> ClaimedExtraction:
    return ClaimedExtraction(
        uuid4(),
        uuid4(),
        uuid4(),
        FileType.PDF,
        1,
        1,
        uuid4(),
        datetime.now(UTC),
        FileRef(FileRefRoot.STORAGE, "a.pdf"),
    )


OUTCOME = ExtractionOutcome(
    "text", (), "v1", ExtractorMetadata(ExtractionMethod.EMBEDDED_TEXT, 1, 0, None)
)


class Client:
    def __init__(self, jobs: list[ClaimedExtraction | None]) -> None:
        self.jobs = jobs
        self.calls: list[tuple[str, object]] = []

    def claim(self, worker_id: str, lease_seconds: int) -> ClaimedExtraction | None:
        self.calls.append(("claim", worker_id))
        return self.jobs.pop(0)

    def complete(
        self,
        extraction_id: object,
        lease_token: object,
        outcome: ExtractionOutcome,
        extracted_at: datetime,
    ) -> None:
        self.calls.append(("complete", outcome))

    def fail(self, extraction_id: object, lease_token: object, failure: object) -> None:
        self.calls.append(("fail", failure))

    def cancelled(self, extraction_id: object, lease_token: object) -> None:
        self.calls.append(("cancelled", extraction_id))

    def heartbeat(self, *args: object) -> object:
        raise AssertionError("fake heartbeat factory is used")


def execute(
    tmp_path: Path, extractor: object, *, client: Client | None = None, cancel: bool = False
) -> tuple[Client, Heartbeat]:
    job = claimed()
    api = client or Client([job, None])
    shutdown = threading.Event()
    heartbeat = Heartbeat()
    source = tmp_path / "a.pdf"
    source.write_bytes(b"x")

    def sleep(_seconds: float) -> None:
        shutdown.set()

    def heartbeat_factory(*args: object) -> Heartbeat:
        if cancel:
            progress = args[2]
            progress._cancel_event.set()  # type: ignore[attr-defined]
        return heartbeat

    run_loop(
        api,
        ExtractorSettings(extractor_storage_root=tmp_path),
        Ocr(),
        shutdown_event=shutdown,
        worker_id="worker-1",
        sleep=sleep,
        heartbeat_factory=heartbeat_factory,
        format_detector=lambda _path: DetectedFormat(FileType.PDF, "application/pdf", False),
        extractor_selector=lambda _fmt, _settings: extractor,
    )  # type: ignore[arg-type]
    return api, heartbeat


def test_claim_extract_complete_and_heartbeat_cleanup(tmp_path: Path) -> None:
    class Extractor:
        def extract(self, *args: object) -> ExtractionOutcome:
            return OUTCOME

    client, heartbeat = execute(tmp_path, Extractor())
    assert [name for name, _ in client.calls] == ["claim", "complete", "claim"]
    assert heartbeat.stopped


def test_cancel_before_complete_sends_cancelled(tmp_path: Path) -> None:
    class Extractor:
        def extract(self, *args: object) -> ExtractionOutcome:
            return OUTCOME

    client, _ = execute(tmp_path, Extractor(), cancel=True)
    assert "cancelled" in [name for name, _ in client.calls]
    assert "complete" not in [name for name, _ in client.calls]


@pytest.mark.parametrize(
    ("error", "retryable"),
    [
        (PermanentError(ExtractionErrorKind.CORRUPT_FILE, "bad"), False),
        (TransientError("offline"), True),
    ],
)
def test_errors_are_classified(tmp_path: Path, error: Exception, retryable: bool) -> None:
    class Extractor:
        def extract(self, *args: object) -> ExtractionOutcome:
            raise error

    client, _ = execute(tmp_path, Extractor())
    failure = next(value for name, value in client.calls if name == "fail")
    assert failure.retryable is retryable  # type: ignore[attr-defined]


def test_cancelled_error_does_not_fail(tmp_path: Path) -> None:
    class Extractor:
        def extract(self, *args: object) -> ExtractionOutcome:
            raise CancelledError

    client, _ = execute(tmp_path, Extractor())
    assert "cancelled" in [name for name, _ in client.calls]
    assert "fail" not in [name for name, _ in client.calls]


def test_invalid_lease_on_complete_continues_without_fail(tmp_path: Path) -> None:
    first, second = claimed(), claimed()

    class LeaseClient(Client):
        def complete(self, *args: object) -> None:
            self.calls.append(("complete", args[2]))
            if len([call for call in self.calls if call[0] == "complete"]) == 1:
                raise InvalidLeaseTokenError

    class Extractor:
        def extract(self, *args: object) -> ExtractionOutcome:
            return OUTCOME

    client = LeaseClient([first, second, None])
    client, _ = execute(tmp_path, Extractor(), client=client)
    assert len([call for call in client.calls if call[0] == "complete"]) == 2
    assert "fail" not in [name for name, _ in client.calls]
