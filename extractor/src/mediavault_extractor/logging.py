from collections.abc import MutableMapping
from typing import Any

import structlog

_SENSITIVE_KEY_PARTS = ("internal_api_key", "content", "text", "image")
_REDACTED = "[REDACTED]"


def _mask_secrets(
    logger: Any, method_name: str, event_dict: MutableMapping[str, Any]
) -> MutableMapping[str, Any]:
    """Remove API credentials and document payloads in one central processor."""
    del logger, method_name
    for key in tuple(event_dict):
        if any(part in key.lower() for part in _SENSITIVE_KEY_PARTS):
            event_dict[key] = _REDACTED
    return event_dict


def configure_logging() -> None:
    """Configure one-line JSON logs for the worker process."""
    structlog.configure(
        processors=[
            structlog.contextvars.merge_contextvars,
            structlog.processors.add_log_level,
            structlog.processors.TimeStamper(fmt="iso", utc=True),
            _mask_secrets,
            structlog.processors.JSONRenderer(),
        ],
        wrapper_class=structlog.make_filtering_bound_logger(20),
        logger_factory=structlog.PrintLoggerFactory(),
        cache_logger_on_first_use=True,
    )
