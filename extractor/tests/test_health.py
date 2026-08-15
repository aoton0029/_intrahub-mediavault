import httpx

from mediavault_extractor.api_client import OcrDeviceReport
from mediavault_extractor.health import check_api_reachable, health_status
from mediavault_extractor.ocr.base import OcrResult


class _FakeOcr:
    engine_name = "fake"
    model_id = "test"
    device = OcrDeviceReport.CPU

    def ocr(self, image: object) -> OcrResult:
        return OcrResult("", None)


def test_api_reachable(monkeypatch) -> None:
    def fake_get(url: str, timeout: float) -> httpx.Response:
        assert url == "http://api:8080/api/v1/health"
        assert timeout == 5.0
        return httpx.Response(200, request=httpx.Request("GET", url))

    monkeypatch.setattr(httpx, "get", fake_get)
    assert check_api_reachable("http://api:8080") is True


def test_api_unreachable(monkeypatch) -> None:
    def fake_get(url: str, timeout: float) -> httpx.Response:
        del timeout
        raise httpx.ConnectError("unreachable", request=httpx.Request("GET", url))

    monkeypatch.setattr(httpx, "get", fake_get)
    assert check_api_reachable("http://api:8080") is False


def test_health_status_keeps_signals_distinct(monkeypatch) -> None:
    monkeypatch.setattr("mediavault_extractor.health.check_api_reachable", lambda _: False)

    status = health_status("http://api")

    assert status.process_alive is True
    assert status.api_reachable is False
    assert status.ocr_backend_ready is False


def test_health_status_reports_initialized_ocr(monkeypatch) -> None:
    monkeypatch.setattr("mediavault_extractor.health.check_api_reachable", lambda _: True)

    status = health_status("http://api", _FakeOcr())  # type: ignore[arg-type]

    assert status.api_reachable is True
    assert status.ocr_backend_ready is True
