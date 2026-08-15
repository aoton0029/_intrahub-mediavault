from __future__ import annotations

import os
from pathlib import Path

import httpx
import pytest


@pytest.fixture(scope="session")
def api() -> httpx.Client:
    if os.getenv("MEDIAVAULT_E2E") != "1":
        pytest.skip("set MEDIAVAULT_E2E=1 and start the compose E2E stack")
    base_url = os.getenv("MEDIAVAULT_E2E_API_URL", "http://127.0.0.1:18080/api/v1")
    with httpx.Client(base_url=base_url, timeout=30) as client:
        response = client.get("/health")
        response.raise_for_status()
        yield client


@pytest.fixture(scope="session")
def fixtures_dir() -> Path:
    return Path(__file__).parents[1] / "fixtures"
