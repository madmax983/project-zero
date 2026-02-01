# SCALE

*From pebble to empire. Every world remembers.*

## Vision

A 4X grand strategy game where you begin as a single struggling colony on one planet, micromanaging survival Dwarf-Fortress-style, and end commanding a galactic civilization—but every planet you ever colonized is still simulating underneath. Drill down from galactic war to check why your agricultural world's grain output dropped. Find a fistfight in the granary.

## Core Design Principles

1. **Emergence over scripting** — Simple rules, complex outcomes. No hand-authored story beats.
2. **Scale is a camera, not a mode** — The simulation doesn't change when you zoom out. You just see less detail.
3. **Every problem has a face** — Abstract resource shortages trace back to specific pops, buildings, decisions.
4. **Losing is interesting** — Colony collapse, planetary disasters, and galactic defeat should generate stories, not frustration.
5. **Modular isolation** — Each system (buildings, pops, ships, diplomacy) is a self-contained module with clean interfaces.

---

## Architecture: Three Layers

### Layer 1: Colony (Dwarf Fortress scale)

The foundational simulation. Everything else is abstraction over this.

**Map:** 2D grid. Tiles have terrain type, elevation, resources underground.

**Pops:** Individual colonists with:
- Needs: hunger, rest, shelter, morale
- Skills: mining, farming, construction, research, combat
- Traits: (extensible) hardworking, lazy, genius, aggressive, etc.
- State: working, idle, sleeping, dead

**Buildings:** Placed on tiles. Types include:
- Housing (shelter need)
- Farms (food production)
- Mines (raw resources)
- Factories (resource transformation)
- Research labs (tech points)
- Spaceport (required for Layer 2)

**Jobs:** Buildings create jobs. Pops claim jobs based on skill + availability.

**Resources:** 
- Basic: food, ore, energy, materials
- Advanced: components, fuel, research points
- Chains: ore → materials → components

**Time:** Discrete ticks. One tick = one "day." Needs decay, production accumulates.

**Events:** Random occurrences that create drama:
- Natural: earthquake, drought, bountiful harvest
- Social: strike, romance, feud, crime
- Discovery: ancient ruins, resource vein, anomaly

**Interface contract (up to Layer 2):**
```
ColonyState {
  population: u32,
  morale: f32,        // 0.0 - 1.0
  output: ResourceBundle,
  input_needs: ResourceBundle,
  can_build_ships: bool,
  anomalies: Vec<Anomaly>,
}
```

---

### Layer 2: System (Stellaris planetary scale)

Unlocked when you build your first spaceport and launch a ship.

**Map:** Star system. Central star, orbital slots, planets/moons as nodes.

**Planets:** Each planet IS a Layer 1 simulation, but viewed abstractly as:
- Type: terran, barren, gas giant, toxic, etc.
- Habitability: how hard to colonize
- Resources: what's available to mine
- Colony: Option<ColonyState>

**Ships:** Units that move between orbital bodies.
- Colony ships (consumes pops, founds new colony)
- Freighters (move resources between colonies)
- Military (defense, eventual offense)

**Stations:** Orbital buildings.
- Mining stations (extract from uninhabitable bodies)
- Defense platforms
- Shipyards

**Resources:** Aggregate of all colony outputs + station outputs.

**Time:** 1 Layer 2 tick = 10 Layer 1 ticks. Colonies simulate in batches.

**Interface contract (up to Layer 3):**
```
SystemState {
  colonies: Vec<ColonyState>,
  ships: Vec<Ship>,
  total_output: ResourceBundle,
  military_power: u32,
  anomalies: Vec<Anomaly>,
}
```

---

### Layer 3: Galaxy (Grand strategy scale)

Unlocked when you research interstellar travel and launch to another star.

**Map:** Procedural galaxy. Stars as nodes, hyperlanes as edges.

**Systems:** Each system IS a Layer 2 simulation, abstracted as:
- Star type: affects habitability of planets
- Planets: count and types
- Owner: you, other civ, none
- SystemState if owned by you

**Civilizations:** Other spacefaring species.
- Traits: aggressive, peaceful, traders, isolationist, etc.
- Relations: diplomacy state with you
- AI: acts on SystemState-level abstractions

**Fleets:** Groups of military ships that move between systems.

**Diplomacy:** 
- States: peace, war, alliance, trade agreement
- Actions: declare war, propose treaty, trade resources

**Victory/Loss:**
- Conquest: control X% of galaxy
- Federation: unite Y civs peacefully
- Collapse: lose all colonies

---

## Technical Constraints (for LLM contributors)

### Stack

- **bevy_ecs** — ECS for simulation logic (not full Bevy engine)
- **ratatui** — Terminal UI rendering
- **crossterm** — Terminal backend, input handling

Why this combo:
- Bevy ECS gives us ergonomic queries, schedules, and systems
- Ratatui gives us simple, fast rendering with no asset pipeline
- Terminal aesthetic matches DF authenticity
- LLMs reason better about "char at row,col" than "sprite at pixel"
- Simulation logic is completely decoupled from rendering

### Code organization

```
src/
  main.rs           # App setup, main loop
  app.rs            # Bevy World + Schedule setup
  input.rs          # Crossterm event → ECS event translation
  layer1/           # Colony simulation (pure ECS, no rendering)
    mod.rs
    pop.rs
    building.rs
    terrain.rs
    needs.rs
    job.rs
    resource.rs
    event.rs
  layer2/           # System simulation  
    mod.rs
    planet.rs
    ship.rs
    station.rs
  layer3/           # Galaxy simulation
    mod.rs
    system.rs
    civ.rs
    diplomacy.rs
  ui/               # Ratatui rendering (reads from World)
    mod.rs
    colony_view.rs
    system_view.rs
    galaxy_view.rs
    widgets/        # Reusable UI components
  shared/           # Cross-layer types
    mod.rs
    resources.rs
    events.rs
```

### Rendering Philosophy

The simulation runs in Bevy ECS. The UI reads from the World and draws to terminal.

```rust
// Main loop pseudocode
loop {
    // 1. Handle input (crossterm → ECS events)
    handle_input(&mut world);
    
    // 2. Run simulation (bevy_ecs schedules)
    schedule.run(&mut world);
    
    // 3. Render (ratatui reads from world)
    terminal.draw(|frame| {
        render_colony(frame, &world);
    })?;
}
```

### Display Characters

```
Terrain:
  . grass     , dirt      # rock      ~ water

Pops:
  ☺ healthy   ☻ working   ⚉ resting  † dead

Buildings:
  ⌂ housing   ♣ farm      ⛏ mine     ⚙ factory

UI:
  ─│┌┐└┘├┤┬┴┼  box drawing
  ▓▒░              shading
  ►◄▲▼            arrows
```

### PR guidelines

Each PR should:
1. Touch only ONE layer unless absolutely necessary
2. Add ONE feature (new building, new ship type, new event, etc.)
3. Include at minimum a doc comment explaining the feature
4. Not break existing interface contracts
5. Compile and run (CI will check)

Good PR examples:
- "Add greenhouse building to Layer 1"
- "Add pirate raid event to Layer 2"
- "Add trade agreement diplomacy option to Layer 3"

Bad PR examples:
- "Refactor everything"
- "Add full combat system" (too big, break into pieces)
- "Change ResourceBundle format" (breaks interfaces)

---

## MVP Milestone (v0.1)

Layer 1 only. A single colony that can:
- [ ] Generate a terrain map
- [ ] Spawn initial pops
- [ ] Build: housing, farm, mine, factory
- [ ] Pops claim jobs and work
- [ ] Resources accumulate and get consumed
- [ ] Pops die if needs unmet
- [ ] One random event type
- [ ] Win condition: reach 50 population
- [ ] Lose condition: all pops dead

No Layer 2 or 3. No aliens. No ships. Just survive and grow.

---

*This document is the canonical reference. When in doubt, add emergence.*