from __future__ import annotations

import threading
from types import SimpleNamespace
from uuid import uuid4

from mediavault_extractor.api_client import TransientError
from mediavault_extractor.heartbeat import MAX_HEARTBEAT_FAILURES, HeartbeatThread


class Progress:
    def snapshot(self) -> tuple[int, int]:
        return 2, 5


def test_heartbeat_propagates_cancellation_and_stops() -> None:
    called = threading.Event()

    class Client:
        def heartbeat(self, *args: object) -> object:
            assert args[2:4] == (2, 5)
            called.set()
            return SimpleNamespace(cancel_requested=True)

    cancel = threading.Event()
    heartbeat = HeartbeatThread(Client(), uuid4(), uuid4(), Progress(), cancel, 300, 60).start()
    assert called.wait(1)
    assert cancel.wait(1)
    heartbeat.stop()


def test_heartbeat_cancels_after_consecutive_transient_failures() -> None:
    attempts = 0

    class Client:
        def heartbeat(self, *args: object) -> object:
            nonlocal attempts
            attempts += 1
            raise TransientError("offline")

    cancel = threading.Event()
    heartbeat = HeartbeatThread(Client(), uuid4(), uuid4(), Progress(), cancel, 300, 0).start()
    assert cancel.wait(1)
    heartbeat.stop()
    assert attempts == MAX_HEARTBEAT_FAILURES
