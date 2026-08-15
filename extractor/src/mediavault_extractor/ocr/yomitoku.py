from __future__ import annotations

from collections.abc import Callable, Iterable
from typing import Protocol, cast

import numpy as np
from numpy.typing import NDArray
from PIL.Image import Image

from mediavault_extractor.api_client import OcrDeviceReport
from mediavault_extractor.config import OcrDeviceSetting
from mediavault_extractor.ocr.base import OcrResult


class OcrDeviceUnavailableError(Exception):
    """The configured OCR device cannot be used at worker startup."""


class _Word(Protocol):
    content: str
    rec_score: float


class _OcrOutput(Protocol):
    words: Iterable[_Word]


class _Analyzer(Protocol):
    def __call__(self, image: NDArray[np.uint8]) -> tuple[_OcrOutput, object]: ...


AnalyzerFactory = Callable[[dict[str, object], str, bool], _Analyzer]

_CPU_RECOGNIZER = "parseq-small"
_GPU_RECOGNIZER = "parseq-large-v4_1"
_DETECTOR = "dbnetv2_1"


def _cuda_is_available() -> bool:
    import torch

    return bool(torch.cuda.is_available())


def _verify_device_available(device: OcrDeviceSetting) -> None:
    if device is OcrDeviceSetting.CUDA and not _cuda_is_available():
        raise OcrDeviceUnavailableError(
            "EXTRACTOR_OCR_DEVICE=cuda was requested, but CUDA is unavailable"
        )


def _create_analyzer(configs: dict[str, object], device: str, visualize: bool) -> _Analyzer:
    # Keep the vendor import inside this implementation module.
    from yomitoku import OCR  # type: ignore[import-untyped]

    return cast(_Analyzer, OCR(configs=configs, device=device, visualize=visualize))


class YomitokuOcrEngine:
    """Stable worker-facing wrapper around yomitoku's OCR implementation."""

    def __init__(
        self,
        device: OcrDeviceSetting,
        *,
        analyzer_factory: AnalyzerFactory = _create_analyzer,
    ) -> None:
        _verify_device_available(device)
        self._configured_device = device
        recognizer = _CPU_RECOGNIZER if device is OcrDeviceSetting.CPU else _GPU_RECOGNIZER
        self._model_id = f"{_DETECTOR}+{recognizer}"
        configs: dict[str, object] = {
            "text_detector": {"model_name": _DETECTOR},
            "text_recognizer": {"model_name": recognizer},
        }
        self._analyzer = analyzer_factory(configs, device.value, False)

    @property
    def engine_name(self) -> str:
        return "yomitoku"

    @property
    def model_id(self) -> str:
        return self._model_id

    @property
    def device(self) -> OcrDeviceReport:
        if self._configured_device is OcrDeviceSetting.CUDA:
            return OcrDeviceReport.GPU
        return OcrDeviceReport.CPU

    def ocr(self, image: Image) -> OcrResult:
        # yomitoku expects an OpenCV-style BGR uint8 array, not a PIL image.
        rgb = np.asarray(image.convert("RGB"), dtype=np.uint8)
        bgr = np.ascontiguousarray(rgb[:, :, ::-1])
        output, _ = self._analyzer(bgr)
        words = tuple(output.words)
        text = "\n".join(word.content for word in words if word.content)
        confidence = sum(float(word.rec_score) for word in words) / len(words) if words else None
        return OcrResult(text=text, confidence=confidence)
