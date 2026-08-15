from __future__ import annotations

from pathlib import Path

import pytest
from PIL import Image

from mediavault_extractor.api_client import (
    ExtractionErrorKind,
    ExtractionMethod,
    OcrDeviceReport,
    PermanentError,
)
from mediavault_extractor.config import ExtractorSettings
from mediavault_extractor.extractors.base import CancelledError
from mediavault_extractor.extractors.pdf import A4_AREA_PT2, PdfExtractor, needs_ocr
from mediavault_extractor.ocr.base import OcrResult


class FakeTextPage:
    def __init__(self, text: str) -> None:
        self.text = text

    def get_text_bounded(self) -> str:
        return self.text

    def close(self) -> None:
        pass


class FakeBitmap:
    def to_pil(self) -> Image.Image:
        return Image.new("RGB", (1, 1))

    def close(self) -> None:
        pass


class FakePage:
    def __init__(self, text: str, size: tuple[float, float] = (595, 842)) -> None:
        self.text = text
        self.size = size
        self.rendered = False

    def get_textpage(self) -> FakeTextPage:
        return FakeTextPage(self.text)

    def get_size(self) -> tuple[float, float]:
        return self.size

    def render(self, *, scale: float) -> FakeBitmap:
        assert scale > 0
        self.rendered = True
        return FakeBitmap()

    def close(self) -> None:
        pass


class FakeDocument:
    def __init__(self, pages: list[FakePage]) -> None:
        self.pages = pages
        self.closed = False

    def __len__(self) -> int:
        return len(self.pages)

    def __getitem__(self, index: int) -> FakePage:
        return self.pages[index]

    def close(self) -> None:
        self.closed = True


class FakeOcr:
    engine_name = "fake"
    model_id = "fake-v1"
    device = OcrDeviceReport.CPU

    def __init__(self, texts: list[str] | None = None, *, fail: bool = False) -> None:
        self.texts = iter(texts or [])
        self.calls = 0
        self.fail = fail

    def ocr(self, image: Image.Image) -> OcrResult:
        self.calls += 1
        if self.fail:
            raise RuntimeError("ocr failed")
        return OcrResult(next(self.texts), 1.0)


class FakeProgress:
    def __init__(self, cancel_on_check: int | None = None) -> None:
        self.reports: list[tuple[int, int]] = []
        self.checks = 0
        self.cancel_on_check = cancel_on_check

    def is_cancelled(self) -> bool:
        self.checks += 1
        return self.checks == self.cancel_on_check

    def report(self, current: int, total: int) -> None:
        self.reports.append((current, total))


def make_extractor(document: FakeDocument, *, max_pages: int = 2000) -> PdfExtractor:
    settings = ExtractorSettings(
        extractor_max_pages=max_pages,
        extractor_ocr_fallback_min_chars_per_page=50,
    )
    return PdfExtractor(settings, document_factory=lambda _path: document)


def test_embedded_pages_do_not_use_ocr() -> None:
    document = FakeDocument([FakePage("x" * 50), FakePage("y" * 50)])
    ocr = FakeOcr()
    outcome = make_extractor(document).extract(Path("a.pdf"), ocr, FakeProgress())
    assert ocr.calls == 0
    assert outcome.extractor.method is ExtractionMethod.EMBEDDED_TEXT
    assert outcome.extractor.ocr is None


def test_only_sparse_pages_use_ocr_and_build_contiguous_boundaries() -> None:
    document = FakeDocument([FakePage("x" * 50), FakePage(""), FakePage("z" * 50)])
    ocr = FakeOcr(["scanned"])
    progress = FakeProgress()
    outcome = make_extractor(document).extract(Path("a.pdf"), ocr, progress)
    assert ocr.calls == 1
    assert outcome.extractor.method is ExtractionMethod.MIXED
    assert (outcome.extractor.embedded_text_pages, outcome.extractor.ocr_pages) == (2, 1)
    assert [boundary.label for boundary in outcome.boundaries] == ["p.1", "p.2", "p.3"]
    assert all(
        left.end == right.start
        for left, right in zip(outcome.boundaries, outcome.boundaries[1:], strict=False)
    )
    assert outcome.boundaries[-1].end == len(outcome.content)
    assert progress.reports == [(1, 3), (2, 3), (3, 3)]


@pytest.mark.parametrize(
    ("text", "expected"), [("", True), ("   ", True), ("x" * 50, False), ("x" * 49, True)]
)
def test_needs_ocr_boundaries(text: str, expected: bool) -> None:
    assert needs_ocr(text, A4_AREA_PT2, 50) is expected


def test_small_page_normalization_avoids_ocr() -> None:
    page = FakePage("12345", (10, 10))
    ocr = FakeOcr()
    make_extractor(FakeDocument([page])).extract(Path("a.pdf"), ocr, FakeProgress())
    assert ocr.calls == 0


def test_cancellation_happens_before_next_page() -> None:
    pages = [FakePage("x" * 50) for _ in range(4)]
    progress = FakeProgress(cancel_on_check=4)
    with pytest.raises(CancelledError):
        make_extractor(FakeDocument(pages)).extract(Path("a.pdf"), FakeOcr(), progress)
    assert progress.reports == [(1, 4), (2, 4), (3, 4)]


def test_page_limit_is_checked_before_page_access() -> None:
    document = FakeDocument([FakePage("x" * 50), FakePage("y" * 50)])
    with pytest.raises(PermanentError) as exc_info:
        make_extractor(document, max_pages=1).extract(Path("a.pdf"), FakeOcr(), FakeProgress())
    assert exc_info.value.kind is ExtractionErrorKind.SIZE_LIMIT_EXCEEDED
    assert document.closed


def test_ocr_failure_returns_no_partial_result() -> None:
    with pytest.raises(PermanentError) as exc_info:
        make_extractor(FakeDocument([FakePage("")])).extract(
            Path("a.pdf"), FakeOcr(fail=True), FakeProgress()
        )
    assert exc_info.value.kind is ExtractionErrorKind.OCR_FAILED


def test_corrupt_pdf_is_mapped_to_permanent_error() -> None:
    settings = ExtractorSettings()

    def fail(_path: Path) -> FakeDocument:
        raise ValueError("broken")

    with pytest.raises(PermanentError) as exc_info:
        PdfExtractor(settings, document_factory=fail).extract(
            Path("bad.pdf"), FakeOcr(), FakeProgress()
        )
    assert exc_info.value.kind is ExtractionErrorKind.CORRUPT_FILE
