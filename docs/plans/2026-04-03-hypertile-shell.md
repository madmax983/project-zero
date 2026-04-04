# Hypertile Shell Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace SCALE's hard-coded root UI layout with a `ratatui-hypertile-extras` shell that owns panes, workspaces, and a unified command palette.

**Architecture:** Introduce a shell layer under `src/ui/` that wraps existing renderers as hypertile plugins. Extend the platform input model to preserve modifiers and route shell-level events through the runtime before forwarding focused events to gameplay plugins. Persist workspace/layout state independently from simulation state and fall back cleanly to a default workspace if persisted shell state is invalid.

**Tech Stack:** Rust, `ratatui` 0.30, `ratatui-hypertile` 0.3, `ratatui-hypertile-extras` 0.3, `crossterm`, `ratzilla`, `serde`, `serde_json`, `bevy_ecs`

---

### Task 1: Add Shell Scaffolding And Modifier-Aware Input

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/platform/input.rs`
- Modify: `src/platform/native.rs`
- Modify: `src/platform/wasm.rs`
- Create: `src/ui/shell/mod.rs`
- Create: `src/ui/shell/config.rs`
- Test: `src/platform/input.rs`
- Test: `src/ui/shell/config.rs`

**Step 1: Write the failing tests**

Add tests that prove:

```rust
#[test]
fn ctrl_k_preserves_modifiers() {
    let ev = GameKeyEvent::new(GameKeyCode::Char('k')).with_ctrl();
    assert!(ev.modifiers.ctrl);
}

#[test]
fn shell_config_round_trips_default_workspace() {
    let cfg = ShellConfig::default();
    let json = serde_json::to_string(&cfg).unwrap();
    let decoded: ShellConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.startup_workspace, "Colony Ops");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test platform::input shell_config_round_trips_default_workspace -- --nocapture`

Expected: FAIL because modifiers and shell config do not exist yet.

**Step 3: Write minimal implementation**

- Add `ratatui-hypertile` and `ratatui-hypertile-extras` to `Cargo.toml`.
- Extend `GameKeyEvent` with modifier state.
- Update native and WASM platform adapters to preserve modifiers.
- Add `ShellConfig` with:
  - `startup_workspace`
  - `theme`
  - `recent_commands`
  - workspace/layout persistence payload placeholder

**Step 4: Run tests to verify they pass**

Run: `cargo test platform::input shell_config_round_trips_default_workspace -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/platform/input.rs src/platform/native.rs src/platform/wasm.rs src/ui/shell/mod.rs src/ui/shell/config.rs
git commit -m "feat(ui): scaffold hypertile shell state"
```

### Task 2: Wrap Existing Renderers As Pane Plugins

**Files:**
- Create: `src/ui/shell/plugins/mod.rs`
- Create: `src/ui/shell/plugins/colony_map.rs`
- Create: `src/ui/shell/plugins/system_map.rs`
- Create: `src/ui/shell/plugins/inspector.rs`
- Create: `src/ui/shell/plugins/status.rs`
- Create: `src/ui/shell/plugins/chronicle.rs`
- Create: `src/ui/shell/plugins/tech.rs`
- Modify: `src/ui/mod.rs`
- Test: `src/ui/shell/plugins/*.rs`

**Step 1: Write the failing tests**

Add `ratatui::TestBackend` tests that prove plugins render into arbitrary rectangles:

```rust
#[test]
fn status_plugin_renders_inside_small_rect() {
    let rect = Rect::new(0, 0, 24, 3);
    let buffer = render_status_plugin_for_test(rect);
    assert!(buffer_contains(&buffer, "Food"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test status_plugin_renders_inside_small_rect -- --nocapture`

Expected: FAIL because plugin wrappers do not exist.

**Step 3: Write minimal implementation**

- Define one plugin type per pane.
- Keep old renderer internals where possible:
  - colony map plugin delegates to `src/ui/map.rs`
  - system map plugin delegates to `src/layer2/render.rs`
  - inspector/status/chronicle/tech plugins delegate to existing renderer functions
- Move any fixed-root assumptions out of the wrappers rather than rewriting the renderers wholesale.

**Step 4: Run tests to verify they pass**

Run: `cargo test ui::shell::plugins -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/mod.rs src/ui/shell/plugins
git commit -m "feat(ui): add hypertile pane plugins"
```

### Task 3: Bootstrap Runtime, Registry, And Default Workspaces

**Files:**
- Create: `src/ui/shell/runtime.rs`
- Create: `src/ui/shell/commands.rs`
- Modify: `src/setup.rs`
- Modify: `src/ui/shell/mod.rs`
- Test: `src/ui/shell/runtime.rs`

**Step 1: Write the failing tests**

Add tests that prove the shell boots curated workspaces and palette-visible actions:

```rust
#[test]
fn default_shell_contains_curated_workspaces() {
    let shell = build_default_shell_for_test();
    assert!(shell.has_workspace("Colony Ops"));
    assert!(shell.has_workspace("System Survey"));
    assert!(shell.has_workspace("Director"));
}

#[test]
fn command_registry_exposes_pause_and_open_chronicle() {
    let registry = build_command_registry_for_test();
    assert!(registry.contains("Pause Simulation"));
    assert!(registry.contains("Open Chronicle"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test default_shell_contains_curated_workspaces command_registry_exposes_pause_and_open_chronicle -- --nocapture`

Expected: FAIL because runtime/bootstrap code does not exist.

**Step 3: Write minimal implementation**

- Build a shell resource that owns:
  - plugin registry
  - runtime
  - persisted config
  - command registry
- Register three workspaces:
  - `Colony Ops`
  - `System Survey`
  - `Director`
- Register unified commands for shell actions and gameplay actions.

**Step 4: Run tests to verify they pass**

Run: `cargo test ui::shell::runtime -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/setup.rs src/ui/shell/mod.rs src/ui/shell/runtime.rs src/ui/shell/commands.rs
git commit -m "feat(ui): bootstrap hypertile workspaces"
```

### Task 4: Integrate The Shell Into Native And WASM Entry Points

**Files:**
- Modify: `src/main.rs`
- Modify: `src/bin/wasm_app.rs`
- Modify: `src/shared/input.rs`
- Modify: `src/ui/mod.rs`
- Test: `src/shared/input.rs`

**Step 1: Write the failing tests**

Add tests that prove shell-owned actions do not leak into plugin-owned handlers:

```rust
#[test]
fn ctrl_k_opens_palette_without_toggling_gameplay_state() {
    let mut world = setup_shell_world_for_test();
    route_input(&mut world, GameKeyEvent::ctrl_char('k'));
    assert!(shell_palette_is_open(&world));
    assert_eq!(*world.resource::<GameState>(), GameState::Running);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test ctrl_k_opens_palette_without_toggling_gameplay_state -- --nocapture`

Expected: FAIL because the shell does not intercept top-level input yet.

**Step 3: Write minimal implementation**

- Native loop:
  - translate terminal events with modifiers,
  - let shell/runtime process shell events first,
  - forward remaining events to focused plugin/game handlers.
- WASM loop:
  - mirror the same routing using ratzilla event translation.
- Replace the hard-coded root `ui::render()` path with shell rendering.

**Step 4: Run tests to verify they pass**

Run: `cargo test shared::input -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/main.rs src/bin/wasm_app.rs src/shared/input.rs src/ui/mod.rs
git commit -m "feat(ui): route shell input through hypertile runtime"
```

### Task 5: Remove Overlay-Specific UI State And Promote Panes To First-Class Views

**Files:**
- Modify: `src/shared/input.rs`
- Modify: `src/ui/state.rs`
- Modify: `src/ui/chronicle.rs`
- Modify: `src/ui/tech.rs`
- Modify: `src/setup.rs`
- Test: `src/shared/input.rs`

**Step 1: Write the failing tests**

Add regression tests that prove chronicle/tech are pane-managed rather than overlay-managed:

```rust
#[test]
fn open_chronicle_command_creates_or_focuses_chronicle_pane() {
    let mut world = setup_shell_world_for_test();
    run_command(&mut world, "Open Chronicle");
    assert!(workspace_contains_pane(&world, "chronicle"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test open_chronicle_command_creates_or_focuses_chronicle_pane -- --nocapture`

Expected: FAIL because chronicle and tech are still overlay-driven.

**Step 3: Write minimal implementation**

- Remove or shrink `ChronicleUiState` / `TechUiState` responsibilities.
- Route old overlay hotkeys to shell commands.
- Keep renderers pure and let the shell decide existence/focus.

**Step 4: Run tests to verify they pass**

Run: `cargo test chronicle tech -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/shared/input.rs src/ui/state.rs src/ui/chronicle.rs src/ui/tech.rs src/setup.rs
git commit -m "refactor(ui): promote overlays into hypertile panes"
```

### Task 6: Persist Layouts, Update Architecture Docs, And Verify End-To-End

**Files:**
- Modify: `src/ui/shell/config.rs`
- Create: `docs/adr/048-hypertile-shell.md`
- Modify: `docs/architecture.md`
- Modify: `docs/plans/2026-04-03-hypertile-shell-design.md`
- Test: `tests/integration/ui_hypertile_shell.rs`

**Step 1: Write the failing tests**

Add integration tests for fallback and persistence:

```rust
#[test]
fn invalid_shell_layout_falls_back_to_colony_ops() {
    let shell = load_shell_config_for_test("{\"broken\":true}");
    assert_eq!(shell.current_workspace_name(), "Colony Ops");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test invalid_shell_layout_falls_back_to_colony_ops -- --nocapture`

Expected: FAIL because persistence fallback is incomplete.

**Step 3: Write minimal implementation**

- Persist shell/workspace state with version-tolerant serde structures.
- Fall back to default workspaces on invalid config.
- Write ADR for the shell migration.
- Update architecture diagrams to show shell -> plugins -> simulation flow.

**Step 4: Run verification**

Run:

```bash
cargo fmt --all
cargo test -j 1 --workspace
cargo check -j 1
```

Expected:

- `cargo fmt --all` exits cleanly
- tests pass
- workspace compiles

**Step 5: Commit**

```bash
git add src/ui/shell/config.rs tests/integration/ui_hypertile_shell.rs docs/adr/048-hypertile-shell.md docs/architecture.md
git commit -m "feat(ui): persist hypertile shell workspaces"
```

### Completion Checklist

- `Cargo.toml` contains shell dependencies only once.
- `Ctrl+K` works in native and WASM builds.
- `Colony Ops`, `System Survey`, and `Director` boot successfully.
- Chronicle and tech are pane-backed, not overlay-backed.
- Shell config corruption falls back to `Colony Ops`.
- ADR and architecture diagram are updated in the same change.
- Verification commands were run and recorded.
