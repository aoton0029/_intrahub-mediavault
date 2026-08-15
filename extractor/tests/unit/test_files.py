from pathlib import Path
from unittest.mock import patch

import pytest

from mediavault_extractor.api_client import ExtractionErrorKind, FileRef, PermanentError
from mediavault_extractor.config import ExtractorSettings, FileRefRoot
from mediavault_extractor.files import UnsafePathError, check_size_limit, resolve_file_ref


def _settings(tmp_path: Path, *, max_bytes: int = 100) -> ExtractorSettings:
    return ExtractorSettings(
        extractor_storage_root=tmp_path / "storage",
        extractor_library_root=tmp_path / "library",
        extractor_max_file_bytes=max_bytes,
    )


@pytest.mark.parametrize("root", [FileRefRoot.STORAGE, FileRefRoot.LIBRARY])
def test_resolves_each_allowed_root(tmp_path: Path, root: FileRefRoot) -> None:
    settings = _settings(tmp_path)
    expected_root = settings.allowed_root(root)

    assert (
        resolve_file_ref(FileRef(root, "files/a.pdf"), settings)
        == (expected_root / "files/a.pdf").resolve()
    )


@pytest.mark.parametrize("relative_path", ["../../etc/passwd", "a/../../b"])
def test_rejects_parent_segments(tmp_path: Path, relative_path: str) -> None:
    with pytest.raises(UnsafePathError):
        resolve_file_ref(FileRef(FileRefRoot.STORAGE, relative_path), _settings(tmp_path))


def test_rejects_absolute_path(tmp_path: Path) -> None:
    absolute = str((tmp_path / "outside").resolve())
    with pytest.raises(UnsafePathError):
        resolve_file_ref(FileRef(FileRefRoot.STORAGE, absolute), _settings(tmp_path))


def test_rejects_symlink_escape_without_opening_file(tmp_path: Path) -> None:
    settings = _settings(tmp_path)
    settings.extractor_storage_root.mkdir()
    outside = tmp_path / "outside.pdf"
    outside.write_bytes(b"%PDF-1.4\n")
    link = settings.extractor_storage_root / "link.pdf"
    try:
        link.symlink_to(outside)
    except OSError as exc:
        pytest.skip(f"symlinks are unavailable: {exc}")

    with patch("builtins.open") as mocked_open, pytest.raises(UnsafePathError):
        resolve_file_ref(FileRef(FileRefRoot.STORAGE, "link.pdf"), settings)
    mocked_open.assert_not_called()


def test_allows_symlink_that_stays_inside_root(tmp_path: Path) -> None:
    settings = _settings(tmp_path)
    settings.extractor_storage_root.mkdir()
    target = settings.extractor_storage_root / "target.pdf"
    target.write_bytes(b"%PDF-1.4\n")
    link = settings.extractor_storage_root / "link.pdf"
    try:
        link.symlink_to(target)
    except OSError as exc:
        pytest.skip(f"symlinks are unavailable: {exc}")

    assert resolve_file_ref(FileRef(FileRefRoot.STORAGE, "link.pdf"), settings) == target.resolve()


def test_rejects_file_over_size_limit(tmp_path: Path) -> None:
    path = tmp_path / "large.pdf"
    path.write_bytes(b"1234")

    with pytest.raises(PermanentError) as exc_info:
        check_size_limit(path, _settings(tmp_path, max_bytes=3))
    assert exc_info.value.kind is ExtractionErrorKind.SIZE_LIMIT_EXCEEDED
