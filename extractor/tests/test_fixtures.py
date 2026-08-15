from pathlib import Path

import puremagic
import pypdfium2 as pdfium  # type: ignore[import-untyped]

FIXTURES = Path(__file__).parent / "fixtures"


def test_required_e2e_fixtures_are_small_and_present() -> None:
    expected = {
        "text_layer.pdf",
        "scanned.pdf",
        "mixed.pdf",
        "japanese.png",
        "corrupt.pdf",
        "fake.pdf",
    }
    assert expected <= {path.name for path in FIXTURES.iterdir()}
    assert all((FIXTURES / name).stat().st_size < 100_000 for name in expected)


def test_valid_pdf_fixtures_have_expected_page_counts() -> None:
    expected = {"text_layer.pdf": 3, "scanned.pdf": 3, "mixed.pdf": 3}
    for name, pages in expected.items():
        document = pdfium.PdfDocument(FIXTURES / name)
        try:
            assert len(document) == pages
        finally:
            document.close()


def test_fake_pdf_has_png_signature() -> None:
    matches = puremagic.magic_file(FIXTURES / "fake.pdf")
    assert any(match.mime_type == "image/png" for match in matches)
