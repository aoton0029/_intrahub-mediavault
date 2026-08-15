from pathlib import Path

from PIL import Image, UnidentifiedImageError

from mediavault_extractor.api_client import (
    ExtractionErrorKind,
    ExtractionMethod,
    ExtractionOutcome,
    ExtractorMetadata,
    OcrMetadata,
    PermanentError,
)
from mediavault_extractor.boundaries import build_boundaries
from mediavault_extractor.extractors.base import CancelledError, ProgressReporter
from mediavault_extractor.normalize import normalize
from mediavault_extractor.ocr.base import OcrEngine

EXTRACTION_VERSION = "image-v1"


class ImageExtractor:
    def extract(
        self, path: Path, ocr: OcrEngine, progress: ProgressReporter
    ) -> ExtractionOutcome:
        if progress.is_cancelled():
            raise CancelledError()

        try:
            with Image.open(path) as image:
                image.load()
                try:
                    text = ocr.ocr(image).text
                except Exception as exc:
                    raise PermanentError(
                        ExtractionErrorKind.OCR_FAILED, "画像のOCRに失敗しました"
                    ) from exc
        except (UnidentifiedImageError, OSError) as exc:
            raise PermanentError(
                ExtractionErrorKind.CORRUPT_FILE, f"画像を開けません: {path.name}"
            ) from exc
        except PermanentError:
            raise

        text = normalize(text)
        boundaries, content = build_boundaries(((text, "p.1"),))
        progress.report(1, 1)
        return ExtractionOutcome(
            content=content,
            boundaries=boundaries,
            extraction_version=EXTRACTION_VERSION,
            extractor=ExtractorMetadata(
                method=ExtractionMethod.OCR,
                embedded_text_pages=0,
                ocr_pages=1,
                ocr=OcrMetadata(ocr.engine_name, ocr.device, ocr.model_id),
            ),
        )
