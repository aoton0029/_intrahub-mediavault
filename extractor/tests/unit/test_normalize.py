import pytest

from mediavault_extractor.normalize import normalize


@pytest.mark.parametrize(
    ("source", "expected"),
    [
        ("ＡＢＣ１２３", "ABC123"),
        ("a\r\nb\rc\nd", "a\nb\nc\nd"),
        ("a\x00b\x08c\td", "abc\td"),
        ("a\n\n\n\n\nb", "a\n\nb"),
        ("a    b", "a b"),
        ("これわテストです", "これわテストです"),
        ("サーバとサーバー", "サーバとサーバー"),
        ("", ""),
        ("   \n\n  ", ""),
    ],
)
def test_normalize(source: str, expected: str) -> None:
    assert normalize(source) == expected


def test_normalize_is_idempotent() -> None:
    source = "Ａ  Ｂ\r\n\r\n\r\n制御\x00文字\t保持"
    once = normalize(source)
    assert normalize(once) == once
