from collections.abc import Iterable
from dataclasses import dataclass
from typing import cast

import numpy as np
import pytest
from PIL import Image
from pydantic import ValidationError

from mediavault_extractor.api_client import OcrDeviceReport
from mediavault_extractor.config import ExtractorSettings, OcrDeviceSetting
from mediavault_extractor.ocr.base import OcrEngine, OcrResult
from mediavault_extractor.ocr.yomitoku import OcrDeviceUnavailableError, YomitokuOcrEngine


class FakeOcrEngine:
    engine_name = "fake"
    model_id = "test-model"
    device = OcrDeviceReport.CPU

    def ocr(self, image: Image.Image) -> OcrResult:
        return OcrResult("fake text", 1.0)


def _accepts_engine(engine: OcrEngine) -> OcrResult:
    return engine.ocr(Image.new("RGB", (1, 1)))


@dataclass
class _FakeWord:
    content: str
    rec_score: float


@dataclass
class _FakeOutput:
    words: Iterable[_FakeWord]


class _FakeAnalyzer:
    def __init__(self) -> None:
        self.received: np.ndarray | None = None

    def __call__(self, image: np.ndarray) -> tuple[_FakeOutput, object]:
        self.received = image
        return _FakeOutput([_FakeWord("日本語", 0.8), _FakeWord("文字", 1.0)]), None


def _engine(device: OcrDeviceSetting) -> tuple[YomitokuOcrEngine, _FakeAnalyzer]:
    analyzer = _FakeAnalyzer()
    engine = YomitokuOcrEngine(device, analyzer_factory=lambda _config, _device, _vis: analyzer)
    return engine, analyzer


def test_fake_satisfies_protocol_boundary() -> None:
    assert _accepts_engine(FakeOcrEngine()).text == "fake text"


def test_cpu_device_and_lightweight_model() -> None:
    engine, _ = _engine(OcrDeviceSetting.CPU)
    assert engine.device is OcrDeviceReport.CPU
    assert "parseq-small" in engine.model_id
    assert engine.engine_name == "yomitoku"


def test_cuda_reports_gpu(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("mediavault_extractor.ocr.yomitoku._cuda_is_available", lambda: True)
    engine, _ = _engine(OcrDeviceSetting.CUDA)
    assert engine.device is OcrDeviceReport.GPU
    assert engine.device.value == "gpu"


def test_cuda_unavailable_fails_before_analyzer_creation(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    created = False

    def factory(_config: dict[str, object], _device: str, _visualize: bool) -> _FakeAnalyzer:
        nonlocal created
        created = True
        return _FakeAnalyzer()

    monkeypatch.setattr("mediavault_extractor.ocr.yomitoku._cuda_is_available", lambda: False)
    with pytest.raises(OcrDeviceUnavailableError):
        YomitokuOcrEngine(OcrDeviceSetting.CUDA, analyzer_factory=factory)
    assert created is False


def test_unknown_device_is_rejected(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("EXTRACTOR_OCR_DEVICE", "metal")
    with pytest.raises(ValidationError):
        ExtractorSettings()


def test_default_device_is_cpu(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("EXTRACTOR_OCR_DEVICE", raising=False)
    settings = ExtractorSettings()
    engine, _ = _engine(settings.extractor_ocr_device)
    assert engine.device is OcrDeviceReport.CPU


def test_device_cannot_be_assigned() -> None:
    engine, _ = _engine(OcrDeviceSetting.CPU)
    with pytest.raises(AttributeError):
        engine.device = cast(OcrDeviceReport, OcrDeviceReport.GPU)  # type: ignore[misc]


def test_vendor_result_is_converted_and_image_is_bgr() -> None:
    engine, analyzer = _engine(OcrDeviceSetting.CPU)
    result = engine.ocr(Image.new("RGB", (1, 1), (10, 20, 30)))
    assert result == OcrResult("日本語\n文字", 0.9)
    assert analyzer.received is not None
    assert analyzer.received[0, 0].tolist() == [30, 20, 10]
