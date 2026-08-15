from pathlib import Path

import pytest

from mediavault_extractor.api_client import ExtractionErrorKind, FileType, PermanentError
from mediavault_extractor.detect import detect_format

PNG = b"\x89PNG\r\n\x1a\n" + b"\x00" * 32
JPEG = b"\xff\xd8\xff\xe0\x00\x10JFIF\x00" + b"\x00" * 32
PDF = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\n%%EOF\n"
MP4 = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom"


@pytest.mark.parametrize(
    ("name", "content", "file_type"),
    [
        ("document.pdf", PDF, FileType.PDF),
        ("image.png", PNG, FileType.IMAGE),
        ("image.jpg", JPEG, FileType.IMAGE),
    ],
)
def test_detects_supported_signatures(
    tmp_path: Path, name: str, content: bytes, file_type: FileType
) -> None:
    path = tmp_path / name
    path.write_bytes(content)

    detected = detect_format(path)

    assert detected.file_type is file_type
    assert detected.extension_mismatch is False


def test_reports_extension_mismatch(tmp_path: Path) -> None:
    path = tmp_path / "image.pdf"
    path.write_bytes(PNG)

    detected = detect_format(path)

    assert detected.file_type is FileType.IMAGE
    assert detected.extension_mismatch is True


def test_extensionless_supported_file_is_not_a_mismatch(tmp_path: Path) -> None:
    path = tmp_path / "document"
    path.write_bytes(PDF)

    detected = detect_format(path)

    assert detected.file_type is FileType.PDF
    assert detected.extension_mismatch is False


def test_rejects_unsupported_signature(tmp_path: Path) -> None:
    path = tmp_path / "video.mp4"
    path.write_bytes(MP4)

    with pytest.raises(PermanentError) as exc_info:
        detect_format(path)
    assert exc_info.value.kind is ExtractionErrorKind.UNSUPPORTED_FORMAT
