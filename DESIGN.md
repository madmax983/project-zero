# SCALE

*From pebble to empire. Every world remembers.*

A 4X colony simulation where you begin Dwarf-Fortress-style on a single planet and end commanding a galactic civilization—but every world is still simulating underneath.

## Status

🚧 **Pre-Alpha** — AI agents are building this game.

## Quickstart

```bash
cargo run
```

## Architecture

Three simulation layers, each abstracting the one below:

1. **Colony** (Dwarf Fortress) — Individual pops, buildings, needs, jobs
2. **System** (Planetary) — Planets as nodes, ships, orbital stations  
3. **Galaxy** (Stellaris) — Star systems, civilizations, diplomacy

**Tech stack:** bevy_ecs for simulation, ratatui for terminal UI.

See [DESIGN.md](DESIGN.md) for full architecture.

## Development

This project is built by AI agents coordinating through git. See [AGENTS.md](AGENTS.md) for the protocol.

**Agents:**
- `prompts/DESIGNER.md` — Game design ideation
- `prompts/LOREMASTER.md` — Procedural lore system
- `prompts/ARCHITECT.md` — Feature specifications
- `prompts/BUILDER.md` — Implementation

**Workflow:**
- `design/BACKLOG.md` — Available work
- `specs/` — Feature specifications
- `lore/` — Procedural lore fragments, templates, grammars

## Procedural History

Every playthrough generates unique lore:
- **Pre-history:** 500+ years of galactic events generated at game start
- **Chronicle:** Ongoing events recorded as you play
- **Discovery:** Find artifacts, ruins, and legends with generated backstories

See `lore/` for the building blocks.

## Controls (MVP)

- **WASD / Arrows** — Scroll map (or move cursor in build mode)
- **Space** — Pause/unpause (or place building in build mode)
- **1/2/3** — Speed (1x/3x/5x)
- **B** — Toggle build mode
- **Tab** — Cycle building type (in build mode)
- **Enter** — Place building (in build mode)
- **Escape** — Exit build mode
- **Q** — Quit

## Display

```
Terrain:  . grass   , dirt   # rock   ~ water
Pops:     ☺ idle    ⚒ working   ☻ resting
Buildings: ⌂ housing   ♣ farm
```

## License

MIT
