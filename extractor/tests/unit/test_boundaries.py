from mediavault_extractor.boundaries import build_boundaries


def test_build_boundaries_returns_content_and_contiguous_boundaries() -> None:
    boundaries, content = build_boundaries((("one", "p.1"), ("二", "p.2"), ("xyz", "p.3")))

    assert content == "one二xyz"
    assert [(item.start, item.end, item.label) for item in boundaries] == [
        (0, 3, "p.1"),
        (3, 4, "p.2"),
        (4, 7, "p.3"),
    ]
    assert boundaries[-1].end == len(content)


def test_build_boundaries_keeps_empty_segments() -> None:
    boundaries, content = build_boundaries((("first", "第1章"), ("", "第2章"), ("last", "第3章")))

    assert content == "firstlast"
    assert (boundaries[1].start, boundaries[1].end, boundaries[1].label) == (5, 5, "第2章")
    assert boundaries[1].end == boundaries[2].start


def test_build_boundaries_accepts_no_segments() -> None:
    assert build_boundaries(()) == ((), "")
