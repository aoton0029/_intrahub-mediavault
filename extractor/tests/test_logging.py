import json

import structlog

from mediavault_extractor.logging import configure_logging


def test_logging_masks_secrets_and_document_payloads(capsys) -> None:
    configure_logging()
    structlog.get_logger().info(
        "sample",
        internal_api_key="plain-secret",
        content="private body",
        extracted_text="private text",
        image_bytes="private image",
        extraction_id="safe-id",
    )

    output = capsys.readouterr().out
    event = json.loads(output)
    assert "plain-secret" not in output
    assert "private body" not in output
    assert "private text" not in output
    assert "private image" not in output
    assert event["internal_api_key"] == "[REDACTED]"
    assert event["extraction_id"] == "safe-id"
