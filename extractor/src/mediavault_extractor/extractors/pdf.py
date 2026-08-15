from __future__ import annotations

from collections.abc import Callable
from pathlib import Path
from typing import Any

import pypdfium2 as pdfium  # type: ignore[import-untyped]

from mediavault_extractor.api_client import (
    ExtractionErrorKind,
    ExtractionMethod,
    ExtractionOutcome,
    ExtractorMetadata,
    OcrMetadata,
    PermanentError,
    TextBoundary,
)
from mediavault_extractor.config import ExtractorSettings
from mediavault_extractor.extractors.base import CancelledError, ProgressReporter
from mediavault_extractor.ocr.base import OcrEngine

A4_AREA_PT2 = 595.0 * 842.0
OCR_RENDER_SCALE = 2.0
EXTRACTION_VERSION = "pdf-v1"


def needs_ocr(
    page_text: str,
    page_area_pt2: float,
    min_chars_per_page: int,
    a4_area_pt2: float = A4_AREA_PT2,
) -> bool:
    """Return whether embedded text is too sparse to be useful."""
    if not page_text.strip():
        return True
    if page_area_pt2 <= 0:
        return True
    normalized_chars = len(page_text) * (a4_area_pt2 / page_area_pt2)
    return normalized_chars < min_chars_per_page


def _normalize_page(text: str) -> str:
    # TASK-0021 owns the full normalization policy. Keeping this as a named,
    # deterministic boundary lets that implementation replace it without moving
    # page-boundary calculation out of this extractor.
    return text.replace("\r\n", "\n").replace("\r", "\n")


class PdfExtractor:
    def __init__(
        self,
        settings: ExtractorSettings,
        *,
        document_factory: Callable[[Path], Any] | None = None,
        normalize: Callable[[str], str] = _normalize_page,
    ) -> None:
        self._settings = settings
        self._document_factory = document_factory or pdfium.PdfDocument
        self._normalize = normalize

    def extract(
        self, path: Path, ocr: OcrEngine, progress: ProgressReporter
    ) -> ExtractionOutcome:
        try:
            document = self._document_factory(path)
        except Exception as exc:
            raise PermanentError(
                ExtractionErrorKind.CORRUPT_FILE, f"PDFを開けません: {path.name}"
            ) from exc

        try:
            total_pages = len(document)
            if total_pages > self._settings.extractor_max_pages:
                raise PermanentError(
                    ExtractionErrorKind.SIZE_LIMIT_EXCEEDED,
                    f"PDFのページ数が上限を超えています: {total_pages}",
                )

            parts: list[str] = []
            boundaries: list[TextBoundary] = []
            cursor = 0
            embedded_pages = 0
            ocr_pages = 0

            for page_index in range(total_pages):
                if progress.is_cancelled():
                    raise CancelledError()

                page = document[page_index]
                try:
                    text = self._embedded_text(page)
                    width, height = page.get_size()
                    if needs_ocr(
                        text,
                        float(width) * float(height),
                        self._settings.extractor_ocr_fallback_min_chars_per_page,
                    ):
                        text = self._ocr_page(page, ocr)
                        ocr_pages += 1
                    else:
                        embedded_pages += 1
                finally:
                    _close(page)

                text = self._normalize(text)
                parts.append(text)
                end = cursor + len(text)
                boundaries.append(TextBoundary(cursor, end, f"p.{page_index + 1}"))
                cursor = end
                progress.report(page_index + 1, total_pages)

            method = (
                ExtractionMethod.MIXED
                if ocr_pages and embedded_pages
                else ExtractionMethod.OCR
                if ocr_pages
                else ExtractionMethod.EMBEDDED_TEXT
            )
            ocr_metadata = (
                OcrMetadata(ocr.engine_name, ocr.device, ocr.model_id) if ocr_pages else None
            )
            return ExtractionOutcome(
                content="".join(parts),
                boundaries=tuple(boundaries),
                extraction_version=EXTRACTION_VERSION,
                extractor=ExtractorMetadata(method, embedded_pages, ocr_pages, ocr_metadata),
            )
        except (CancelledError, PermanentError):
            raise
        except Exception as exc:
            raise PermanentError(
                ExtractionErrorKind.CORRUPT_FILE, f"PDFを読み取れません: {path.name}"
            ) from exc
        finally:
            _close(document)

    @staticmethod
    def _embedded_text(page: Any) -> str:
        text_page = page.get_textpage()
        try:
            return str(text_page.get_text_bounded())
        finally:
            _close(text_page)

    @staticmethod
    def _ocr_page(page: Any, ocr: OcrEngine) -> str:
        bitmap = None
        image = None
        try:
            bitmap = page.render(scale=OCR_RENDER_SCALE)
            image = bitmap.to_pil()
            return ocr.ocr(image).text
        except Exception as exc:
            raise PermanentError(
                ExtractionErrorKind.OCR_FAILED, "PDFページのOCRに失敗しました"
            ) from exc
        finally:
            if image is not None:
                image.close()
            if bitmap is not None:
                _close(bitmap)


def _close(resource: Any) -> None:
    close = getattr(resource, "close", None)
    if close is not None:
        close()
