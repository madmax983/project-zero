# 001: Project Scaffold and Bevy App Shell

## Overview

Set up the foundational Bevy application with proper project structure, dependencies, and a running window. This is the skeleton everything else builds on.

## Dependencies

None — this is the first task.

## Requirements

### Must Have
- `Cargo.toml` with Bevy 0.15+ dependency
- `src/main.rs` that opens a window titled "SCALE"
- Window size: 1280x720, resizable
- Background color: dark gray (`#1a1a1a`)
- Module structure matching DESIGN.md architecture
- Game state enum: `Loading`, `Playing`, `Paused`
- Basic app states wired up (even if they all show same screen)

### Must NOT Have
- Any gameplay logic
- Any rendering beyond background color
- Asset loading

## Technical Guidance

### Cargo.toml
```toml
[package]
name = "scale"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.15"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

### Directory Structure
```
src/
  main.rs
  lib.rs           # Re-exports, shared types
  layer1/
    mod.rs         # Just `// Colony simulation` comment for now
  layer2/
    mod.rs         # Just `// System simulation` comment for now
  layer3/
    mod.rs         # Just `// Galaxy simulation` comment for now
  ui/
    mod.rs         # Just `// UI systems` comment for now
```

### Game States
```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}
```

## Acceptance Criteria

- [ ] `cargo build` succeeds with no errors
- [ ] `cargo run` opens a window titled "SCALE"
- [ ] Window has dark gray background
- [ ] Module structure exists (even if empty)
- [ ] GameState enum is defined and registered
- [ ] Pressing Escape closes the window

## Questions

*Builder: add questions here if spec is unclear.*
