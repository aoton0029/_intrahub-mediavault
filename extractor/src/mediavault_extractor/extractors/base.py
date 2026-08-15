from pathlib import Path
from typing import Protocol

from mediavault_extractor.api_client import ExtractionOutcome
from mediavault_extractor.ocr.base import OcrEngine


class CancelledError(Exception):
    """Extraction was cancelled at a safe content boundary."""


class ProgressReporter(Protocol):
    def report(self, current: int, total: int) -> None: ...

    def is_cancelled(self) -> bool: ...


class Extractor(Protocol):
    def extract(
        self, path: Path, ocr: OcrEngine, progress: ProgressReporter
    ) -> ExtractionOutcome: ...
