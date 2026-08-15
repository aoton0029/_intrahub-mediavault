from enum import StrEnum
from pathlib import Path

from pydantic import HttpUrl
from pydantic_settings import BaseSettings, SettingsConfigDict


class FileRefRoot(StrEnum):
    """Root identifiers accepted from the internal API."""

    STORAGE = "storage"
    LIBRARY = "library"


class OcrDeviceSetting(StrEnum):
    """OCR device selected at worker startup."""

    CPU = "cpu"
    CUDA = "cuda"


class ExtractorSettings(BaseSettings):
    """Type-safe worker configuration populated from environment variables."""

    model_config = SettingsConfigDict(extra="ignore")

    mediavault_api_base_url: HttpUrl = HttpUrl("http://localhost:8080")
    internal_api_key: str = ""

    extractor_library_root: Path = Path("/library")
    extractor_storage_root: Path = Path("/srv/mediavault")

    extractor_ocr_device: OcrDeviceSetting = OcrDeviceSetting.CPU
    # Provisional: tune using real data; see prep.md, confirmation items.
    extractor_ocr_fallback_min_chars_per_page: int = 50

    extractor_max_concurrency: int = 1
    # Provisional operational values; finalize together with lease/OCR measurements.
    extractor_poll_interval_sec: float = 5.0
    extractor_heartbeat_interval_sec: float = 30.0
    extractor_lease_seconds: int = 300
    extractor_job_timeout_sec: int = 3600

    # Provisional safety limits; revise after inspecting the library distribution.
    extractor_max_file_bytes: int = 500 * 1024 * 1024
    extractor_max_pages: int = 2000

    def allowed_root(self, root: FileRefRoot) -> Path:
        """Map an API root identifier to its read-only mount."""
        if root is FileRefRoot.STORAGE:
            return self.extractor_storage_root
        return self.extractor_library_root
