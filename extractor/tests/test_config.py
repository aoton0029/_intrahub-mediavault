from pathlib import Path

from mediavault_extractor.config import ExtractorSettings, FileRefRoot, OcrDeviceSetting


def test_settings_defaults(monkeypatch) -> None:
    monkeypatch.delenv("EXTRACTOR_OCR_DEVICE", raising=False)
    monkeypatch.delenv("EXTRACTOR_MAX_CONCURRENCY", raising=False)

    settings = ExtractorSettings()

    assert settings.extractor_ocr_device is OcrDeviceSetting.CPU
    assert settings.extractor_max_concurrency == 1


def test_allowed_root_maps_storage_and_library() -> None:
    settings = ExtractorSettings(
        extractor_storage_root=Path("/storage-test"),
        extractor_library_root=Path("/library-test"),
    )

    assert settings.allowed_root(FileRefRoot.STORAGE) == Path("/storage-test")
    assert settings.allowed_root(FileRefRoot.LIBRARY) == Path("/library-test")
