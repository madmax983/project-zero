# 6. WASM Browser Support via Ratzilla

Date: 2025-02-05

## Status

Accepted

## Context

SCALE is a TUI-based 4X colony sim using ratatui + crossterm. The terminal-only architecture made automated UI testing difficult: the main rendering pipeline (~57% coverage) was essentially untestable without a real terminal. All rendering functions (6 UI modules, ~1400 lines) were unreachable by unit tests.

Two approaches were considered for improving testability:
1. **Headless mode with snapshot testing** — Mock the terminal backend and compare text buffers
2. **Browser rendering with E2E tests** — Compile to WASM, render in a browser, test with Playwright

## Decision

We adopted **Ratzilla** (the official WASM bridge for ratatui) to enable browser rendering, paired with **Playwright** for E2E testing.

### Architecture

```
┌─────────────────┐     ┌──────────────────┐
│  src/main.rs     │     │ src/bin/wasm_app  │
│  (crossterm)     │     │ (ratzilla DOM)    │
└────────┬────────┘     └────────┬──────────┘
         │                       │
         ▼                       ▼
┌────────────────────────────────────────────┐
│          Platform Abstraction               │
│  GameKeyEvent / GameMouseEvent              │
│  (src/platform/)                            │
└────────────────────┬───────────────────────┘
                     │
                     ▼
┌────────────────────────────────────────────┐
│          Shared Game Logic                  │
│  setup_world() │ run_simulation_tick()      │
│  render()      │ InputRouter                │
└────────────────────────────────────────────┘
```

**Feature flags** control the backend:
- `native` (default): crossterm terminal backend
- `wasm`: ratzilla DOM backend for browser rendering

### Crossterm Surface Area

Only 3 files originally imported crossterm types. After refactoring:
- `src/main.rs` — translates crossterm events at the boundary only
- `src/platform/native.rs` — `TryFrom` implementations
- `src/platform/wasm.rs` — `TryFrom` implementations

All shared code (InputRouter, selection, simulation, rendering) uses platform-agnostic `GameKeyEvent` / `GameMouseEvent` types.

## Consequences

**Positive:**
- E2E tests can drive the actual rendered UI via Playwright against the WASM build
- Zero changes to existing ratatui widget code — all rendering is backend-agnostic
- Ratzilla's DomBackend renders `<span>` elements inside `<pre>`, enabling text-based assertions
- The same game logic runs identically in terminal and browser
- Opens the door for a web demo / playable prototype

**Negative:**
- WASM builds are slower than native (Trunk build time ~60-120s)
- `Instant::now()` unavailable in WASM — simulation uses frame counting instead
- `Rc<RefCell<T>>` pattern required for shared state in WASM (no threads)
- `getrandom` crate with `wasm_js` feature needed for `rand::thread_rng()` in browser

**Trade-offs:**
- Visual regression tests depend on font rendering, which varies across CI environments
- E2E tests add Node.js / Playwright as dev dependencies
- Two binary targets to maintain (native + WASM), though they share >95% of code
