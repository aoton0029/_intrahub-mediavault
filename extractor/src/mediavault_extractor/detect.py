from dataclasses import dataclass
from pathlib import Path

import puremagic

from mediavault_extractor.api_client import ExtractionErrorKind, FileType, PermanentError


@dataclass(frozen=True, slots=True)
class DetectedFormat:
    file_type: FileType
    mime_type: str
    extension_mismatch: bool


_IMAGE_EXTENSIONS = {
    ".avif",
    ".bmp",
    ".gif",
    ".heic",
    ".heif",
    ".jpeg",
    ".jpg",
    ".png",
    ".tif",
    ".tiff",
    ".webp",
}


def _file_type_from_mime(mime_type: str) -> FileType | None:
    if mime_type == "application/pdf":
        return FileType.PDF
    if mime_type.startswith("image/"):
        return FileType.IMAGE
    return None


def _file_type_from_extension(suffix: str) -> FileType | None:
    normalized = suffix.lower()
    if normalized == ".pdf":
        return FileType.PDF
    if normalized in _IMAGE_EXTENSIONS:
        return FileType.IMAGE
    return None


def detect_format(path: Path) -> DetectedFormat:
    """Detect a supported format by signature and compare it with the suffix."""
    try:
        matches = puremagic.magic_file(path)
    except (OSError, puremagic.PureError) as exc:
        raise PermanentError(
            ExtractionErrorKind.UNSUPPORTED_FORMAT,
            f"could not determine file format: {path.name}",
        ) from exc

    supported = next(
        (
            (match, file_type)
            for match in matches
            if (file_type := _file_type_from_mime(match.mime_type))
        ),
        None,
    )
    if supported is None:
        raise PermanentError(
            ExtractionErrorKind.UNSUPPORTED_FORMAT,
            f"unsupported file format: {path.name}",
        )

    match, signature_type = supported
    extension_type = _file_type_from_extension(path.suffix)
    return DetectedFormat(
        file_type=signature_type,
        mime_type=match.mime_type,
        extension_mismatch=bool(path.suffix) and extension_type is not signature_type,
    )
