from collections.abc import Sequence

from mediavault_extractor.api_client import TextBoundary


def build_boundaries(
    segments: Sequence[tuple[str, str]],
) -> tuple[tuple[TextBoundary, ...], str]:
    """Build half-open boundaries and content from ordered text/label segments."""
    boundaries: list[TextBoundary] = []
    parts: list[str] = []
    cursor = 0
    for segment, label in segments:
        end = cursor + len(segment)
        boundaries.append(TextBoundary(cursor, end, label))
        parts.append(segment)
        cursor = end
    return tuple(boundaries), "".join(parts)
