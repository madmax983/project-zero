# SCALE

*From pebble to empire. Every world remembers.*

[![CI](https://github.com/madmax983/scale/workflows/CI/badge.svg)](https://github.com/madmax983/scale/actions)

A 4X colony simulation where you begin Dwarf-Fortress-style on a single planet and end commanding a galactic civilization—but every world is still simulating underneath.

## Status

**Pre-Alpha** — AI agents are building this game.

## Quick Start

### Native (Terminal)

```bash
cargo run --features native

cargo test --features native

cargo criterion
```

### Browser (WASM)

```bash
# Install prerequisites
rustup target add wasm32-unknown-unknown
cargo install --locked trunk

# Serve locally at http://localhost:8080
trunk serve
```

### Pre-commit Hooks

```bash
.\install-hooks.ps1  # Windows
./install-hooks.sh   # Linux/Mac
```

## Usage as a Library

To use SCALE's procedural generation (e.g. `NarrativeGenerator`) in your own Rust code:

```rust
use scale::shared::narrative::{NarrativeContext, NarrativeGenerator};

fn main() -> anyhow::Result<()> {
    // 1. Initialize Generator (loads embedded lore by default)
    let generator = NarrativeGenerator::from_embedded();

    // Or load from a directory (must contain TEMPLATES.md and FRAGMENTS.md)
    // let mut generator = NarrativeGenerator::default();
    // generator.load_from_files("./lore")?;

    // 2. Prepare Context
    let mut context = NarrativeContext::default();
    context.insert("CIV_NAME", "Terran Dominion");
    context.insert("ORIGIN_STAR", "Sol Prime");
    context.insert("YEAR", "2150");

    // 3. Generate Story
    let story = generator.generate("CIVILIZATION_RISE", &context)?;
    println!("{}", story);

    Ok(())
}
```

See `examples/story_demo.rs` for a complete example.

## Controls

| Key | Action |
|-----|--------|
| **WASD / Arrows** | Scroll map (or move cursor in build mode) |
| **Space** | Pause/unpause |
| **1/2/3** | Speed (1x/3x/5x) |
| **B** | Toggle build mode |
| **M** | Toggle mine designation mode |
| **X** | Toggle demolish designation mode |
| **Tab** | Cycle building type (in build mode) |
| **Enter** | Place building / confirm |
| **L** | Toggle chronicle overlay |
| **Escape** | Exit current mode / close overlay |
| **Q** | Quit |
| **Mouse click** | Select entity or tile |

## Display

```
Terrain:   , grass   . dirt   # rock   ~ water   ^ tree
Pops:      @ idle/moving
Buildings: H housing   F farm   S stockpile   T tavern
```

## Architecture

Three simulation layers, each abstracting the one below:

1. **Colony** (Dwarf Fortress) — Individual pops, buildings, needs, jobs
2. **System** (Planetary) — Planets as nodes, ships, orbital stations
3. **Galaxy** (Stellaris) — Star systems, civilizations, diplomacy

**Tech stack:** Rust, `bevy_ecs` for simulation, `ratatui` for UI, `ratzilla` for browser support.

Runs natively in a terminal via crossterm, or in any browser via WASM + Ratzilla's DOM backend. Both targets share identical game logic — only the input translation and render backend differ.

See **[DESIGN.md](DESIGN.md)** for full architecture details and **[docs/adr/](docs/adr/)** for architecture decision records.

## Procedural History

Every playthrough generates unique lore:

- **Pre-history:** 500+ years of galactic events generated at game start
- **Chronicle:** Ongoing events recorded as you play
- **Discovery:** Find artifacts, ruins, and legends with generated backstories

See `lore/` for the building blocks.

## Project Structure

```
scale/
├── src/
│   ├── main.rs           # Native entry point (crossterm)
│   ├── lib.rs            # Library exports
│   ├── bin/
│   │   ├── wasm_app.rs   # WASM entry point (ratzilla)
│   │   └── headless.rs   # Headless simulation runner
│   ├── platform/         # Input abstraction (native ↔ wasm)
│   ├── setup.rs          # Shared world initialization
│   ├── simulation.rs     # Shared simulation tick
│   ├── layer1/           # Colony simulation (pops, buildings, needs, AI)
│   ├── shared/           # Input routing, selection, time, narrative
│   └── ui/               # Terminal UI rendering (backend-agnostic)
├── web/                  # Trunk HTML entry point for WASM
├── e2e/                  # Playwright E2E browser tests
├── specs/                # Feature specifications
├── lore/                 # Procedural lore system
├── design/               # Design docs and task tracking
├── docs/                 # Architecture documentation + ADRs
├── benches/              # Performance benchmarks
└── prompts/              # AI agent prompts
```

## Technical Details

**Language:** Rust Edition 2024

**Dependencies:**
- `bevy_ecs` — Entity Component System for simulation
- `ratatui` — Terminal UI framework (backend-agnostic)
- `crossterm` — Native terminal backend (optional, `native` feature)
- `ratzilla` — Browser WASM backend (optional, `wasm` feature)
- `anyhow` — Error handling
- `rand` — Random generation
- `criterion` — Benchmarking (dev)

**Feature flags:**
- `native` (default) — Terminal mode via crossterm
- `wasm` — Browser mode via ratzilla
- `nova` — Enables experimental features (e.g. Ghosts)

**Profiles:**
- `dev` — Optimized for fast iteration (opt-level = 1)
- `release` — Fully optimized (LTO, single codegen unit)
- `bench` — Inherits from release

## CI/CD

- **Format** + **Clippy** (pedantic + nursery) + **Tests** + **Coverage** (native)
- **WASM Build** via Trunk
- **E2E Tests** via Playwright against the WASM build
- **85%+ test coverage** enforced

See **[CI-SETUP.md](CI-SETUP.md)** for full CI/CD documentation.

## Development

This project is built by AI agents coordinating through git. See **[AGENTS.md](AGENTS.md)** for the protocol.

See **[docs/guides/EXTENDING.md](docs/guides/EXTENDING.md)** for a guide on adding new buildings.

## Contributing

Follow the existing spec-driven workflow and ensure all changes pass CI quality gates.

## License

MIT
