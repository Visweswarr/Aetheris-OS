"""Polymera Python SDK.

Trusted automation entry point for Polymera AI Dev OS. The native module is
optional at import time so docs and tests can run before the PyO3 extension is
built.
"""

from .client import Service, call_tool, memory, plan
from .events import on_event, registered_handlers

__all__ = [
    "Service",
    "call_tool",
    "memory",
    "on_event",
    "plan",
    "registered_handlers",
]
