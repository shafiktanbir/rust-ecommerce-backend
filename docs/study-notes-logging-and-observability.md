# Backend Engineering Study Notes: Logging, Observability & Performance Profiling

## 1. Rust Command Anatomy: `RUST_LOG=info cargo run --release`

```bash
RUST_LOG=info    cargo run    --release
│                │            │
│                │            └─ Compilation Profile (LLVM Optimization Flags)
│                └────────────── Cargo Build & Execute Pipeline
└────────────────────────────── Environment Variable Injection
```

### Component Breakdown:

1. **`RUST_LOG=info` (Environment Variable Injection)**
   - In Unix/Linux, prefixing a command with `VAR=value` passes that variable **only** to the launched process.
   - The Rust `tracing-subscriber` engine reads `RUST_LOG` at startup to determine which logs to output.
   - **Performance impact**: Setting `RUST_LOG=debug` logs raw SQL queries for every request, which CPU-throttles the API server during stress tests. `RUST_LOG=info` keeps console I/O fast.

2. **`cargo run` (Build & Run)**
   - Automatically checks source changes in `src/`, compiles the binary, and executes it.

3. **`--release` (Optimization Profile)**
   - **Debug Mode (`cargo run`)**: `opt-level = 0`, debug assertions enabled, function inlining disabled. Used for fast local compiles during feature dev (~10x–50x slower runtime).
   - **Release Mode (`cargo run --release`)**: `opt-level = 3`, Link-Time Optimization (`lto = true`), `codegen-units = 1`. Generates zero-cost machine code for production and load testing.

---

## 2. Log Level Hierarchy (Severity vs Volume)

```
High Severity / Low Volume (Always On in Production)
  ▲
  │   1. ERROR   ── System/Request broken, alert on-call engineer
  │   2. WARN    ── Unexpected event, but system recovered safely
  │   3. INFO    ── Production operational milestones (Server start, Order created)
  │   4. DEBUG   ── Developer diagnostics (SQL queries, payload parameters)
  │   5. TRACE   ── Microscopic execution steps (Async polling, TCP byte buffers)
  ▼
Low Severity / Huge Volume (Only for targeted deep debugging)
```

---

## 3. Parity: Rust (`tracing`) vs. Node.js (`pino`)

Both Rust and Node.js (`pino`) implement the exact same logging hierarchy. Pino assigns numeric values under the hood for fast JSON output:

| Log Level | Node.js Pino Level | Pino Numeric Value | Rust (`tracing`) | Production Policy |
| :--- | :--- | :--- | :--- | :--- |
| **FATAL** | `pino.fatal()` | `60` | `panic!` / `error!` | Process crashing / Fatal exit |
| **ERROR** | `pino.error()` | `50` | `tracing::error!` | Always On (Triggers Alerts) |
| **WARN** | `pino.warn()` | `40` | `tracing::warn!` | Always On |
| **INFO** | `pino.info()` | `30` | `tracing::info!` | **Production Default** |
| **DEBUG** | `pino.debug()` | `20` | `tracing::debug!` | Staging / Local Dev Only |
| **TRACE** | `pino.trace()` | `10` | `tracing::trace!` | Isolated Module Debugging Only |

---

## 4. What is `TRACE` & When is it Needed?

`TRACE` records the absolute lowest-level execution footprint of an application.

### Real-World Use Cases for `TRACE`:
1. **Async Runtime Deadlocks / Task Starvation**:
   - Inspecting whether a Tokio async task or Node.js event loop tick was spawned, yielded, or un-parked by Linux `epoll`.
2. **Raw Socket & DB Binary Protocol Wire Frames**:
   - Inspecting raw TCP byte buffers and PostgreSQL frontend/backend protocol frames (`Parse`, `Bind`, `Execute`).
3. **Complex Multi-Step Algorithms & State Machines**:
   - Logging every single loop iteration and intermediate variable state during complex price/tax calculations.
4. **Targeted Module "Heisenbug" Isolation**:
   - Never turn on global `TRACE`. Target **only the failing module** while keeping the rest of the app at `INFO`:
     ```bash
     RUST_LOG=info,ecommerce_lab::services::order_service=trace cargo run --release
     ```
