# Hypertile Shell Design

Date: 2026-04-03
Status: Validated

## Summary

SCALE should stop treating layout as a fixed rendering detail and instead adopt `ratatui-hypertile-extras` as the actual application shell. The shell owns panes, workspaces, command palette, layout mode, and pane focus. The game becomes a set of focused plugins rendered inside that shell.

This is intentionally not "add auto-tiling to the old UI." The old root layout in `src/ui/mod.rs` hard-codes map, info panel, status bar, chronicle, and tech tree into one orchestration path. That made sense when the interface was a single screen. It becomes brittle once the user should be able to rearrange panes, promote overlays into normal views, and navigate the game through a command palette.

The hypertile runtime exposes the right concepts for this pivot: a runtime shell, workspaces, plugin registration, palette/runtime actions, and explicit input modes. Those concepts should become the top-level UI model for SCALE.

## Goals

- Replace the fixed `ratatui::Layout` root UI with a hypertile shell.
- Make the colony map a normal pane, not a protected center viewport.
- Ship opinionated default workspaces instead of an empty tiling canvas.
- Use one unified command palette for both shell commands and gameplay commands.
- Persist workspace/layout/theme state independently from simulation state.
- Preserve existing rendering modules where possible by wrapping them in pane plugins.

## Non-Goals

- Rewriting every renderer from scratch.
- Replacing Bevy ECS or simulation scheduling.
- Turning the headless CLI into a hypertile app.
- Perfecting every palette command in the first pass.

## Shell Architecture

The shell should sit above all current UI modules.

Native and WASM entry points should:

1. collect terminal/web input,
2. translate it into shell events or focused-plugin events,
3. run simulation as they do today,
4. ask the hypertile runtime to render the current workspace.

The root renderer no longer computes fixed rectangles. Instead, pane plugins render into runtime-assigned areas. Existing modules survive behind adapters:

- `src/ui/map.rs` becomes the colony-map pane renderer.
- `src/layer2/render.rs` becomes the system-map pane renderer.
- `src/ui/inspector.rs` becomes the inspector pane renderer.
- `src/ui/status.rs` becomes a compact telemetry pane instead of a hard-coded footer.
- `src/ui/chronicle.rs` and `src/ui/tech.rs` become normal panes instead of overlay state machines.

This keeps the migration incremental: replace orchestration first, then retire overlay-specific assumptions.

## Workspaces And Panes

The shell should ship with curated workspaces.

### `Colony Ops`

The default gameplay workspace. Large colony map, adjacent inspector, compact resource/status pane, and a log/chronicle stack available without leaving the workspace.

### `System Survey`

The same shell model, but the main pane is the system renderer. Inspector and telemetry remain visible so the user can still reason about selection and consequences without bouncing between "modes."

### `Director`

An operator desk for command-heavy play: chronicle, tech tree, logs, selected-entity details, and simulation controls. The map is available, but it is no longer the UI's sovereign center.

Pane boundaries should match real surfaces:

- `ColonyMapPane`
- `SystemMapPane`
- `InspectorPane`
- `StatusPane`
- `ChroniclePane`
- `TechPane`
- optionally `CommandResultPane` or `NotificationsPane` if palette actions need persistent feedback

Every visible surface should fit the same mental model: panes can be split, tabbed, moved, maximized, restored, and reopened from the palette.

## Input And Command Model

The current `InputContextStack` in `src/shared/input.rs` mixes shell concerns with gameplay concerns. That should be split.

### Shell-owned actions

- `Ctrl+K` palette
- layout mode toggle
- workspace switching
- pane focus/move/split/close/maximize
- reset workspace/layout

### Plugin-owned actions

- map movement
- selection
- build/designation workflows
- in-pane scrolling/navigation
- tech-specific actions when the tech pane is focused

The command palette should be the canonical action registry, not just a pane launcher.

Palette commands should include shell actions:

- open pane
- split with pane
- switch workspace
- reset current workspace
- toggle layout mode

and game actions:

- pause/resume
- set sim speed
- enter build mode
- enter mine/chop/demolish mode
- open tech
- open chronicle
- center on selection
- focus alerts or selected entity

Hotkeys can remain as accelerators, but the palette becomes the discoverable, cohesive control surface.

## Input Representation Changes

The current platform-agnostic input types do not preserve modifier keys. That is insufficient for a real shell because `Ctrl+K` and similar chords are first-class actions.

The platform input layer should therefore grow modifier-aware key events so both native (`crossterm`) and WASM (`ratzilla`) can participate in the same shell model. This is a structural prerequisite, not an implementation detail.

## State And Persistence

Shell state should live separately from simulation state.

Persisted shell state should include:

- available workspaces
- pane trees / tabs
- focused pane
- preferred theme or palette
- recently used commands

Corrupt or stale persisted state must not block boot. If deserialization fails or references unknown pane types, SCALE should log the issue and fall back to the default `Colony Ops` workspace.

Existing resources like `UiState`, `ChronicleUiState`, and `TechUiState` should shrink or disappear as pane existence/focus becomes runtime-managed.

## Migration Strategy

The migration should happen in slices:

1. add shell state/config and modifier-aware input,
2. add plugin wrappers around existing renderers,
3. boot curated workspaces through the hypertile runtime,
4. route shell input and palette actions through the runtime,
5. remove overlay-specific orchestration from `src/ui/mod.rs` and `src/shared/input.rs`,
6. update ADRs and architecture docs.

This keeps rendering logic reusable and prevents the classic "rewrite the terminal, discover nothing works, perish in rounded borders" outcome.

## Implementation Status

Implemented on 2026-04-03:

- shell state and modifier-aware input plumbing,
- pane plugins for colony map, system map, inspector, status, chronicle, and tech,
- curated workspaces bootstrapped through hypertile,
- `Ctrl+K` unified command palette with shell and gameplay actions,
- chronicle/tech promoted to pane-managed views instead of overlay-open gating,
- persisted shell layout snapshots with fallback to `Colony Ops` when saved state is invalid,
- ADR and architecture map updates for the shell model.

Known limitation:

- WASM-target verification is still blocked by the repository's existing `getrandom 0.3.2` wasm backend configuration issue, so native verification is complete but `wasm32-unknown-unknown` remains externally blocked.

## Failure Handling

- Bad layout state falls back to `Colony Ops`.
- Unknown pane IDs become placeholder error panes instead of crashing the frame.
- Palette command failures should write to log/notification surfaces.
- Missing plugin registration should fail loudly in tests and visibly in development.

## Testing Strategy

Proof should come from multiple levels:

- unit tests for shell config serialization and default workspace bootstrapping,
- unit tests for modifier-aware input translation,
- `ratatui::TestBackend` tests for pane plugin rendering under arbitrary rectangles,
- command dispatch tests for unified palette actions,
- integration tests covering workspace startup, pane focus changes, and palette-driven state transitions,
- compile checks for both native and WASM entry points.

## Architectural Consequences

Positive:

- one cohesive shell model instead of bespoke layout plus overlays,
- real discoverability through the palette,
- user-controlled layout without hard-coded panel ownership,
- clearer separation between shell state and simulation state.

Negative:

- input model changes are unavoidable,
- plugin wrappers add some boilerplate,
- native and WASM event translation need to stay aligned,
- more UI state is now persistent and must be versioned carefully.
