# Stellar Engineering

## 1. Overview
We have outgrown planets. This late-game feature introduces Megaprojects that permanently alter the Layer 3 System map. Examples include "Starlifting" (extracting massive amounts of minerals from a star) or "Planet Cracking" (destroying a planetary body for its core resources). These actions yield immense short-term resources but permanently destroy habitats and generate hazardous debris fields that block trade routes.

## 2. Dependencies
- `094-system-view` (Layer 2 System Map Nodes)
- `157-ship-classes` (Megaproject constructor ships)
- `184-orbital-debris` (Debris generation from cracked planets)
- `212-planetary-core-tap` (Endgame resources)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_planet_cracker_destroys_planet_and_yields_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let planet_entity = app.world.spawn((
            SystemNode { name: "Tau Ceti Prime".to_string() },
            SystemResource { core_metals: 10000 },
        )).id();

        let mut inventory = ColonyResources::default();
        app.world.insert_resource(inventory);

        // Act: Execute Planet Cracking
        app.world.send_event(PlanetCrackedEvent { target: planet_entity });

        app.add_systems(Update, process_planet_cracker_system);
        app.update();

        // Assert: Planet is despawned, resources added, debris spawned
        assert!(app.world.get_entity(planet_entity).is_none(), "Planet should be destroyed");
        let res = app.world.resource::<ColonyResources>();
        assert!(res.metals >= 10000, "Core metals should be transferred to colony");
    }

    #[test]
    fn test_planet_cracker_spawns_impassable_debris() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let planet_entity = app.world.spawn((
            SystemNode { name: "Dead Rock".to_string() },
            Position2D { x: 50.0, y: 50.0 }, // Layer 2 pos
            SystemResource { core_metals: 5000 },
        )).id();

        app.world.send_event(PlanetCrackedEvent { target: planet_entity });

        app.add_systems(Update, process_planet_cracker_system);
        app.update();

        // Find debris field
        let mut debris_found = false;
        let mut debris_query = app.world.query::<(&DebrisField, &Position2D)>();
        for (debris, pos) in debris_query.iter(&app.world) {
            if pos.x == 50.0 && pos.y == 50.0 {
                assert!(debris.density > 90.0, "Debris field should be extremely dense and impassable");
                debris_found = true;
            }
        }
        assert!(debris_found, "A debris field must spawn where the planet was");
    }

    #[test]
    fn test_starlifting_reduces_insolation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let star_entity = app.world.spawn((
            SystemNode { name: "Sol".to_string() },
            StarStats { insolation: 100.0, mass: 1000.0 },
        )).id();

        // Act: Starlift
        app.world.send_event(StarliftingEvent { target: star_entity, amount: 100 });

        app.add_systems(Update, process_starlifting_system);
        app.update();

        // Assert: Insolation drops
        let stats = app.world.get::<StarStats>(star_entity).unwrap();
        assert!(stats.insolation < 100.0, "Starlifting should cool the star");
        assert!(stats.mass < 1000.0, "Starlifting should reduce mass");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct PlanetCrackedEvent {
    pub target: Entity,
}

#[derive(Event)]
pub struct StarliftingEvent {
    pub target: Entity,
    pub amount: u32,
}

// System to handle the destruction of a planet
pub fn process_planet_cracker_system(
    mut commands: Commands,
    mut events: EventReader<PlanetCrackedEvent>,
    query: Query<(&Position2D, &SystemResource)>,
    mut resources: ResMut<ColonyResources>,
) {
    for ev in events.read() {
        if let Ok((pos, node_res)) = query.get(ev.target) {
            // Transfer massive wealth
            resources.metals += node_res.core_metals;

            // Spawn impassable debris
            commands.spawn((
                DebrisField { density: 100.0 },
                Position2D { x: pos.x, y: pos.y },
            ));

            // Destroy the world
            commands.entity(ev.target).despawn_recursive();
        }
    }
}

// System to handle draining mass/energy from a star
pub fn process_starlifting_system(
    mut events: EventReader<StarliftingEvent>,
    mut query: Query<&mut StarStats>,
    mut resources: ResMut<ColonyResources>,
) {
    for ev in events.read() {
        if let Ok(mut stats) = query.get_mut(ev.target) {
            // Convert star mass directly into advanced materials/energy
            stats.mass -= ev.amount as f32 * 0.1;
            stats.insolation -= ev.amount as f32 * 0.05; // Plunges planets into ice ages

            resources.energy += ev.amount * 1000;
            resources.exotic_matter += ev.amount * 10;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor 1:** `process_planet_cracker_system` should trigger a global `DiplomaticIncidentEvent` since neighboring empires will be horrified by you destroying celestial bodies.
- **Refactor 2:** `process_starlifting_system` immediately lowers `insolation`. The `Layer 1` climate/weather simulation needs to subscribe to `StarStats` changes to instantly begin freezing planets in the system.
- **Refactor 3:** Create a unified `MegastructureConstruction` pipeline instead of raw events, requiring thousands of in-game days to build the cracker/starlifter around the target before the event fires.

## 6. Acceptance Criteria
- [ ] `cargo test` passes all RED phase tests with 0 failures.
- [ ] `cargo clippy -- -D warnings` returns 0 warnings.
- [ ] Test coverage hits at least 85% for `src/layer3/stellar_engineering.rs`.
- [ ] `PlanetCrackedEvent` successfully despawns the target node, grants its `core_metals` to the colony, and spawns a 100% density `DebrisField` at its exact coordinates.
- [ ] `StarliftingEvent` grants massive amounts of energy and exotic matter while permanently reducing the star's `insolation` and `mass`.

## 7. Technical Guidance
- Ensure that destroying a SystemNode doesn't leave dangling references in `Fleet` pathfinding routes. Fleets currently traveling to a cracked planet need their orders reset or they will fly into the debris field and die.
- The `DebrisField` spawned should use the existing system from `184-orbital-debris`, just scaled to an impassable extreme.
- Add `exotic_matter` to `ColonyResources` if it doesn't already exist.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
- **Builder:** The spec describes `Stellar Engineering` to be added in `src/layer3/stellar_engineering.rs`. However, the required components like `SystemNode`, `SystemResource`, `Position2D`, `DebrisField`, `StarStats`, etc., do not seem to exist in the codebase. Also, `ColonyResources` is in `layer2`, not `layer3`, and it does not have `core_metals`, `energy`, or `exotic_matter`. Since these dependencies (e.g. `094-system-view`, `157-ship-classes`, `184-orbital-debris`, `212-planetary-core-tap`) appear to be missing or incomplete, should I mock these dependencies, implement them first, or pick another task?
