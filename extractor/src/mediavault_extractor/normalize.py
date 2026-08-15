import re
import unicodedata

_INLINE_SPACES = re.compile(r" {2,}")
_EXCESS_NEWLINES = re.compile(r"\n{3,}")


def normalize(text: str) -> str:
    """Apply deterministic, meaning-preserving normalization to extracted text."""
    normalized = text.replace("\r\n", "\n").replace("\r", "\n")
    normalized = unicodedata.normalize("NFKC", normalized)
    normalized = "".join(
        character
        for character in normalized
        if character in "\t\n" or unicodedata.category(character) != "Cc"
    )
    normalized = _INLINE_SPACES.sub(" ", normalized)
    normalized = _EXCESS_NEWLINES.sub("\n\n", normalized)
    return "" if not normalized.strip() else normalized
