from __future__ import annotations

from collections.abc import Callable
from typing import Any

_HANDLERS: dict[str, list[Callable[[dict[str, Any]], Any]]] = {}


def on_event(topic: str) -> Callable[[Callable[[dict[str, Any]], Any]], Callable[[dict[str, Any]], Any]]:
    def decorator(func: Callable[[dict[str, Any]], Any]) -> Callable[[dict[str, Any]], Any]:
        _HANDLERS.setdefault(topic, []).append(func)
        return func

    return decorator


def registered_handlers() -> dict[str, int]:
    return {topic: len(handlers) for topic, handlers in _HANDLERS.items()}
