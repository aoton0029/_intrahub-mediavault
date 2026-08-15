import time

import structlog

from mediavault_extractor.config import ExtractorSettings
from mediavault_extractor.health import health_status
from mediavault_extractor.logging import configure_logging
from mediavault_extractor.ocr.yomitoku import YomitokuOcrEngine


def main() -> None:
    """Run the foundation worker until the extraction loop is added in later tasks."""
    settings = ExtractorSettings()
    configure_logging()
    # Device validation and model loading happen before the future claim loop starts.
    ocr_engine = YomitokuOcrEngine(settings.extractor_ocr_device)
    log = structlog.get_logger()
    log.info(
        "extractor_started",
        ocr_device=ocr_engine.device.value,
        ocr_engine=ocr_engine.engine_name,
        ocr_model=ocr_engine.model_id,
    )

    while True:
        status = health_status(str(settings.mediavault_api_base_url), ocr_engine)
        log_method = log.info if status.api_reachable else log.warning
        log_method(
            "health_checked",
            process_alive=status.process_alive,
            api_reachable=status.api_reachable,
            ocr_backend_ready=status.ocr_backend_ready,
        )
        time.sleep(settings.extractor_poll_interval_sec)


if __name__ == "__main__":
    main()
