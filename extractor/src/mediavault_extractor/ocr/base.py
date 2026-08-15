from dataclasses import dataclass
from typing import Protocol

from PIL.Image import Image

from mediavault_extractor.api_client import OcrDeviceReport


@dataclass(frozen=True, slots=True)
class OcrResult:
    """Engine-independent OCR output."""

    text: str
    confidence: float | None


class OcrEngine(Protocol):
    """Boundary that keeps OCR-vendor-specific values out of extractors."""

    @property
    def engine_name(self) -> str: ...

    @property
    def model_id(self) -> str: ...

    @property
    def device(self) -> OcrDeviceReport: ...

    def ocr(self, image: Image) -> OcrResult: ...
