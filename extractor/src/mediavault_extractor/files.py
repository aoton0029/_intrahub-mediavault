from pathlib import Path

from mediavault_extractor.api_client import (
    ExtractionErrorKind,
    FileRef,
    PermanentError,
)
from mediavault_extractor.config import ExtractorSettings


class UnsafePathError(Exception):
    """A file reference escaped the read-only roots available to the worker."""


def resolve_file_ref(ref: FileRef, settings: ExtractorSettings) -> Path:
    """Resolve an API file reference without opening the referenced file."""
    allowed = settings.allowed_root(ref.root).resolve()

    candidate_rel = Path(ref.relative_path)
    if candidate_rel.is_absolute() or ".." in candidate_rel.parts:
        raise UnsafePathError(f"unsafe relative path: {ref.relative_path}")

    # resolve() expands symlinks. The containment check must happen afterwards.
    resolved = (allowed / candidate_rel).resolve()
    if not resolved.is_relative_to(allowed):
        raise UnsafePathError("file reference points outside its allowed root")

    return resolved


def check_size_limit(path: Path, settings: ExtractorSettings) -> None:
    """Reject oversized input using metadata, before a caller opens the file."""
    if path.stat().st_size > settings.extractor_max_file_bytes:
        raise PermanentError(
            ExtractionErrorKind.SIZE_LIMIT_EXCEEDED,
            f"file exceeds the {settings.extractor_max_file_bytes}-byte limit",
        )
