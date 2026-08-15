from pathlib import Path

import pytest
from PIL import Image

from mediavault_extractor.api_client import (
    ExtractionErrorKind,
    ExtractionMethod,
    OcrDeviceReport,
    PermanentError,
)
from mediavault_extractor.extractors.base import CancelledError
from mediavault_extractor.extractors.image import ImageExtractor
from mediavault_extractor.ocr.base import OcrResult


class FakeOcr:
    engine_name = "fake"
    model_id = "fake-v1"
    device = OcrDeviceReport.CPU

    def __init__(self, text: str = "Ａ  Ｂ") -> None:
        self.text = text
        self.calls = 0

    def ocr(self, image: Image.Image) -> OcrResult:
        self.calls += 1
        assert image.size == (2, 3)
        return OcrResult(self.text, 1.0)


class FakeProgress:
    def __init__(self, *, cancelled: bool = False) -> None:
        self.cancelled = cancelled
        self.reports: list[tuple[int, int]] = []

    def is_cancelled(self) -> bool:
        return self.cancelled

    def report(self, current: int, total: int) -> None:
        self.reports.append((current, total))


def test_image_extractor_uses_ocr_and_returns_one_page(tmp_path: Path) -> None:
    path = tmp_path / "image.png"
    Image.new("RGB", (2, 3)).save(path)
    ocr = FakeOcr()
    progress = FakeProgress()

    outcome = ImageExtractor().extract(path, ocr, progress)

    assert ocr.calls == 1
    assert outcome.content == "A B"
    assert [(item.start, item.end, item.label) for item in outcome.boundaries] == [(0, 3, "p.1")]
    assert outcome.extraction_version == "image-v1"
    assert outcome.extractor.method is ExtractionMethod.OCR
    assert (outcome.extractor.embedded_text_pages, outcome.extractor.ocr_pages) == (0, 1)
    assert outcome.extractor.ocr is not None
    assert outcome.extractor.ocr.engine == "fake"
    assert outcome.extractor.ocr.device is OcrDeviceReport.CPU
    assert outcome.extractor.ocr.model == "fake-v1"
    assert progress.reports == [(1, 1)]


def test_image_extractor_honours_cancellation_before_opening_or_ocr(tmp_path: Path) -> None:
    ocr = FakeOcr()
    with pytest.raises(CancelledError):
        ImageExtractor().extract(tmp_path / "missing.png", ocr, FakeProgress(cancelled=True))
    assert ocr.calls == 0


def test_corrupt_image_is_a_permanent_error(tmp_path: Path) -> None:
    path = tmp_path / "broken.png"
    path.write_bytes(b"not an image")

    with pytest.raises(PermanentError) as exc_info:
        ImageExtractor().extract(path, FakeOcr(), FakeProgress())

    assert exc_info.value.kind is ExtractionErrorKind.CORRUPT_FILE
