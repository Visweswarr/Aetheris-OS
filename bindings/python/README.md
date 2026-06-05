# Polymera Python SDK

`polymera` is the trusted Python automation SDK for Polymera AI Dev OS.

```python
import polymera

plan = polymera.plan("scan files then summarize results")
echo = polymera.call_tool("echo", message="hello")
```

The SDK expects the PyO3 extension from `services/python_runtime` to provide
`polymera_native`. Untrusted user scripts should not use this native boundary;
they belong in the future Pyodide/WASM Component Model path.
