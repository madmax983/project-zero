# Design & Architecture

## Architecture

Three simulation layers, each abstracting the one below:

1. **Colony** (Dwarf Fortress) — Individual pops, buildings, needs, jobs
2. **System** (Planetary) — Planets as nodes, ships, orbital stations  
3. **Galaxy** (Stellaris) — Star systems, civilizations, diplomacy

**Tech stack:** bevy_ecs for simulation, ratatui for terminal UI.

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
