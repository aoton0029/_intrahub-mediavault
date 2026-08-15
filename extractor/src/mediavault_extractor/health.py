from dataclasses import dataclass

import httpx

from mediavault_extractor.ocr.base import OcrEngine


@dataclass(frozen=True, slots=True)
class HealthStatus:
    """Independent worker health signals."""

    process_alive: bool
    api_reachable: bool
    ocr_backend_ready: bool | None = None


def check_api_reachable(base_url: str, timeout_sec: float = 5.0) -> bool:
    """Return whether the MediaVault API health endpoint is reachable and healthy."""
    url = f"{base_url.rstrip('/')}/api/v1/health"
    try:
        response = httpx.get(url, timeout=timeout_sec)
        response.raise_for_status()
    except httpx.HTTPError:
        return False
    return True


def health_status(base_url: str, ocr_engine: OcrEngine | None = None) -> HealthStatus:
    """Collect independent process, API, and initialized OCR backend signals."""
    return HealthStatus(
        process_alive=True,
        api_reachable=check_api_reachable(base_url),
        ocr_backend_ready=ocr_engine is not None,
    )
