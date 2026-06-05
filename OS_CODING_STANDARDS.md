# Polymera OS Coding Standards

To maintain a robust and bug-free operating system environment, please adhere to the following best practices across all components:

## 1. Concurrency & Cross-Thread Flags
- **Always use Atomics:** When communicating state (such as a run loop flag `m_running`) across threads, you must use atomic primitives (e.g., `std::atomic<bool>` in C++, `AtomicBool` in Rust).
- Plain boolean variables can be optimized by the compiler in loops (such as `while(m_running)`), which may lead to infinite loops if modified externally.

## 2. State Commits & Error Handling (Storage/WAL)
- **Ensure Completion Before Early Return:** When exiting a function early via an `Ok(())` or `return`, verify that all transitional states are committed.
- For example, if a deduplicated write occurs, the initial Write-Ahead Log (`WAL`) entry must still be committed before returning; otherwise, crash consistency is compromised.

## 3. Resource Lifecycle & Context Switching
- **Handle Dead Objects Gracefully:** The kernel scheduler or any component that iterates over stored resources must gracefully handle situations where a resource has gracefully self-terminated.
- Never tightly couple the success of switching to a `next_task` on the persistence of the `current_task`, as `current_task` might have removed itself from storage (e.g., `exit()` syscall).

## 4. Access Control & Capabilities
- **Flexible Token Schemas:** Capability tokens must be able to satisfy multiple requirements. If a token represents multiple permissions inside a single string, parse it accurately (e.g. comma-separated parsing).
- Avoid logical flaws like `required.all(|r| token_string == r)`, which mathematically forces failure for requests needing multiple permissions simultaneously.
