from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any

try:
    import polymera_native  # type: ignore
except Exception:  # pragma: no cover - native module is built separately.
    polymera_native = None


class PolymeraNativeUnavailable(RuntimeError):
    pass


def _native() -> Any:
    if polymera_native is None:
        raise PolymeraNativeUnavailable(
            "polymera_native is not built; build services/python_runtime first"
        )
    return polymera_native


@dataclass(frozen=True)
class Service:
    name: str

    def call(self, method: str, **parameters: Any) -> dict[str, Any]:
        payload = json.dumps({"method": method, "parameters": parameters}).encode()
        response = _native().send_message(self.name, payload)
        return json.loads(response.decode() or "{}")


def plan(goal: str, *, session_id: str | None = None, user_id: str | None = None) -> dict[str, Any]:
    envelope = _native().create_plan(goal, session_id, user_id)
    return json.loads(envelope.decode())


def call_tool(tool_name: str, **parameters: Any) -> dict[str, Any]:
    envelope = _native().call_tool(tool_name, json.dumps(parameters))
    return json.loads(envelope.decode())


def memory(key: str, value: Any | None = None) -> dict[str, Any]:
    method = "memory.put" if value is not None else "memory.get"
    return call_tool(method, key=key, value=value)
