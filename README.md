# SCALE

*From pebble to empire. Every world remembers.*

[![CI](https://github.com/madmax983/project-zero/workflows/CI/badge.svg)](https://github.com/madmax983/project-zero/actions)

A 4X colony simulation where you begin Dwarf-Fortress-style on a single planet and end commanding a galactic civilization—but every world is still simulating underneath.

## Status

🚧 **Pre-Alpha** — AI agents are building this game.

## Quick Start

```bash
# Run the game
cargo run

# Run tests
cargo test

# Run benchmarks
cargo criterion

# Install pre-commit hooks
.\install-hooks.ps1  # Windows
./install-hooks.sh   # Linux/Mac
```

## Controls (MVP)

| Key | Action |
|-----|--------|
| **WASD / Arrows** | Scroll map (or move cursor in build mode) |
| **Space** | Pause/unpause (or place building in build mode) |
| **1/2/3** | Speed (1x/3x/5x) |
| **B** | Toggle build mode |
| **Tab** | Cycle building type (in build mode) |
| **Enter** | Place building (in build mode) |
| **Escape** | Exit build mode |
| **Q** | Quit |

## Display

```
Terrain:   . grass   , dirt   # rock   ~ water
Pops:      ☺ idle    ⚒ working   ☻ resting
Buildings: ⌂ housing   ♣ farm
```

## Architecture

Three simulation layers, each abstracting the one below:

1. **Colony** (Dwarf Fortress) — Individual pops, buildings, needs, jobs
2. **System** (Planetary) — Planets as nodes, ships, orbital stations
3. **Galaxy** (Stellaris) — Star systems, civilizations, diplomacy

**Tech stack:** `bevy_ecs` for simulation, `ratatui` for terminal UI.

See **[DESIGN.md](DESIGN.md)** for full architecture details and **[docs/architecture-galaxy-scale-ecs.md](docs/architecture-galaxy-scale-ecs.md)** for multi-tier simulation architecture.

## Procedural History

Every playthrough generates unique lore:

- **Pre-history:** 500+ years of galactic events generated at game start
- **Chronicle:** Ongoing events recorded as you play
- **Discovery:** Find artifacts, ruins, and legends with generated backstories

See `lore/` for the building blocks.

## Development

This project is built by AI agents coordinating through git. See **[AGENTS.md](AGENTS.md)** for the protocol.

**Agents:**
- `prompts/DESIGNER.md` — Game design ideation
- `prompts/LOREMASTER.md` — Procedural lore system
- `prompts/ARCHITECT.md` — Feature specifications
- `prompts/BUILDER.md` — Implementation

**Workflow:**
- `design/BACKLOG.md` — Available work
- `specs/` — Feature specifications
- `lore/` — Procedural lore fragments, templates, grammars

### CI/CD

This project follows strict quality gates:

- **85%+ test coverage** (enforced in CI)
- **Clippy pedantic + nursery** lints
- **Pre-commit hooks** for format + lint + test
- **Criterion benchmarks** for performance tracking

See **[CI-SETUP.md](CI-SETUP.md)** for full CI/CD documentation.

## Project Structure

```
scale/
├── src/
│   ├── main.rs           # Entry point + game loop
│   ├── lib.rs            # Library exports
│   ├── layer1/           # Colony simulation (pops, buildings)
│   ├── layer2/           # System simulation (planets, ships)
│   ├── layer3/           # Galaxy simulation (empires, diplomacy)
│   ├── shared/           # Shared types and utilities
│   └── ui/               # Terminal UI rendering
├── specs/                # Feature specifications
├── lore/                 # Procedural lore system
├── design/               # Design docs and task tracking
├── docs/                 # Architecture documentation
├── benches/              # Performance benchmarks
└── prompts/              # AI agent prompts
```

## Technical Details

**Language:** Rust Edition 2024

**Dependencies:**
- `bevy_ecs` — Entity Component System for simulation
- `ratatui` — Terminal UI framework
- `crossterm` — Cross-platform terminal manipulation
- `anyhow` — Error handling
- `rand` — Random generation
- `criterion` — Benchmarking (dev)

**Profiles:**
- `dev` — Optimized for fast iteration (opt-level = 1)
- `release` — Fully optimized (LTO, single codegen unit)
- `bench` — Inherits from release

## Contributing

This project is currently developed by coordinated AI agents. If you're interested in the approach, check out [AGENTS.md](AGENTS.md) for the protocol.

For human contributors: Follow the existing spec-driven workflow and ensure all changes pass CI quality gates.

## License

MIT
