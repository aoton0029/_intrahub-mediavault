from __future__ import annotations

import time
from pathlib import Path
from typing import Any

import httpx
import pytest

pytestmark = pytest.mark.e2e


def data(response: httpx.Response, expected: int = 200) -> dict[str, Any]:
    assert response.status_code == expected, response.text
    value = response.json()["data"]
    assert isinstance(value, dict)
    return value


def create_item(api: httpx.Client, title: str) -> str:
    return str(data(api.post("/items", json={"media_type": "anime", "title": title}), 201)["id"])


def upload(api: httpx.Client, item_id: str, path: Path) -> str:
    with path.open("rb") as source:
        response = api.post(
            f"/items/{item_id}/files/upload",
            files={"file": (path.name, source, "application/octet-stream")},
        )
    return str(data(response, 201)["id"])


def request_extraction(api: httpx.Client, item_id: str, file_id: str) -> dict[str, Any]:
    return data(api.post(f"/items/{item_id}/files/{file_id}/extraction"), 201)


def wait_for_state(
    api: httpx.Client, item_id: str, file_id: str, terminal: set[str], timeout: float = 180
) -> dict[str, Any]:
    deadline = time.monotonic() + timeout
    observed_progress: list[int] = []
    while time.monotonic() < deadline:
        extraction = data(api.get(f"/items/{item_id}/files/{file_id}/extraction"))
        observed_progress.append(int(extraction["progress_current"]))
        if extraction["state"] in terminal:
            extraction["observed_progress"] = observed_progress
            return extraction
        time.sleep(0.2)
    pytest.fail(f"extraction did not reach {terminal}; progress={observed_progress}")


def prepare(api: httpx.Client, fixtures_dir: Path, name: str) -> tuple[str, str]:
    item_id = create_item(api, f"TASK-0023 {name} {time.time_ns()}")
    return item_id, upload(api, item_id, fixtures_dir / name)


def test_text_layer_pdf_round_trip_and_reproducibility(
    api: httpx.Client, fixtures_dir: Path
) -> None:
    item_id, file_id = prepare(api, fixtures_dir, "text_layer.pdf")
    missing = api.get(f"/items/{item_id}/text", params={"file_id": file_id})
    assert missing.status_code == 422
    assert missing.json()["error"]["code"] == "TEXT_NOT_EXTRACTED"

    request_extraction(api, item_id, file_id)
    assert wait_for_state(api, item_id, file_id, {"succeeded"})["attempts"] == 1
    first = data(api.get(f"/items/{item_id}/text", params={"file_id": file_id}))
    assert first["chunk"]["label"].startswith("p.1")
    assert "MediaVault text layer fixture" in first["chunk"]["text"]
    assert first["extractor"]["method"] == "embedded_text"
    assert first["extractor"]["ocr"] is None

    request_extraction(api, item_id, file_id)
    assert wait_for_state(api, item_id, file_id, {"succeeded"})["attempts"] == 1
    second = data(api.get(f"/items/{item_id}/text", params={"file_id": file_id}))
    assert second["chunk"]["text"] == first["chunk"]["text"]
    assert second["extraction_version"] == first["extraction_version"]


@pytest.mark.parametrize(
    ("name", "kind"),
    [("corrupt.pdf", "corrupt_file"), ("fake.pdf", "unsupported_format")],
)
def test_permanent_failures(api: httpx.Client, fixtures_dir: Path, name: str, kind: str) -> None:
    item_id, file_id = prepare(api, fixtures_dir, name)
    request_extraction(api, item_id, file_id)
    extraction = wait_for_state(api, item_id, file_id, {"failed"})
    assert extraction["attempts"] == 1
    assert extraction["error"]["kind"] == kind
    assert extraction["error"]["retryable"] is False


@pytest.mark.slow
@pytest.mark.parametrize(
    ("name", "method", "embedded", "ocr_pages"),
    [("scanned.pdf", "ocr", 0, 3), ("mixed.pdf", "mixed", 2, 1), ("japanese.png", "ocr", 0, 1)],
)
def test_real_cpu_ocr(
    api: httpx.Client,
    fixtures_dir: Path,
    name: str,
    method: str,
    embedded: int,
    ocr_pages: int,
    record_property: Any,
) -> None:
    item_id, file_id = prepare(api, fixtures_dir, name)
    started = time.monotonic()
    request_extraction(api, item_id, file_id)
    extraction = wait_for_state(api, item_id, file_id, {"succeeded"}, timeout=600)
    duration = time.monotonic() - started
    text = data(api.get(f"/items/{item_id}/text", params={"file_id": file_id}))
    metadata = text["extractor"]
    assert metadata["method"] == method
    assert metadata["embedded_text_pages"] == embedded
    assert metadata["ocr_pages"] == ocr_pages
    assert metadata["ocr"]["device"] == "cpu"
    assert metadata["ocr"]["engine"] == "yomitoku"
    assert metadata["ocr"]["model"]
    assert text["chunk"]["text"].strip()
    record_property("ocr_total_seconds", round(duration, 3))
    record_property("ocr_seconds_per_page", round(duration / max(ocr_pages, 1), 3))
    if ocr_pages > 1:
        assert max(extraction["observed_progress"]) >= 1


@pytest.mark.slow
def test_running_extraction_can_be_cancelled(api: httpx.Client, fixtures_dir: Path) -> None:
    item_id, file_id = prepare(api, fixtures_dir, "scanned.pdf")
    request_extraction(api, item_id, file_id)
    wait_for_state(api, item_id, file_id, {"running"}, timeout=30)
    cancelled = data(api.post(f"/items/{item_id}/files/{file_id}/extraction/cancel"))
    assert cancelled["state"] in {"cancelling", "cancelled"}
    assert wait_for_state(api, item_id, file_id, {"cancelled"}, timeout=60)["state"] == "cancelled"
    assert api.get(f"/items/{item_id}/text", params={"file_id": file_id}).status_code == 422
