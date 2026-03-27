# 668 - Impact Event

## 1. Overview
The "Impact Event" feature introduces a catastrophic, cross-layer mechanic where a massive asteroid or celestial body is detected on Layer 2, hurtling towards a colony on Layer 1. The player is given a countdown. If not deflected by a Layer 2 fleet, the object impacts Layer 1, causing a massive, unpreventable destruction event, altering the map and destroying any surface structures, leaving a crater. This fulfills the "The sky is falling. The ultimate deadline" fantasy.

## 2. Dependencies
- Cross-layer events (`simulation.rs` or integration systems)
- Layer 1 terrain modification (`TerrainGrid`, `GridPosition`)
- Layer 2 fleet movement/combat (for deflection, if implemented, or simply a generic countdown resource)
- Chronicle and Notifications (`ChronicleEvent`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::TerrainType;
    use crate::layer1::structure::Structure;
    use crate::shared::time::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.add_event::<ImpactWarningEvent>();
        app.add_event::<ImpactStrikeEvent>();
        app.add_systems(Update, (process_impact_countdown_system, process_impact_strike_system));
        app
    }

    #[test]
    fn test_impact_countdown_triggers_strike() {
        let mut app = setup_app();

        let impact = app.world_mut().spawn(IncomingImpact {
            target_pos: GridPosition { x: 20, y: 20 },
            radius: 5,
            ticks_remaining: 10,
        }).id();

        // Advance time 9 ticks
        for _ in 0..9 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        // Still exists
        assert!(app.world().get::<IncomingImpact>(impact).is_some());

        // Advance 10th tick
        app.world_mut().resource_mut::<SimulationTime>().tick += 1;
        app.update();

        // Impact should have struck and despawned
        assert!(app.world().get::<IncomingImpact>(impact).is_none());

        // Verify strike event
        let strike_events = app.world().get_resource::<Events<ImpactStrikeEvent>>().unwrap();
        let mut reader = strike_events.get_reader();
        let event = reader.read(strike_events).next().unwrap();
        assert_eq!(event.center, GridPosition { x: 20, y: 20 });
        assert_eq!(event.radius, 5);
    }

    #[test]
    fn test_impact_strike_destroys_structures_and_alters_terrain() {
        let mut app = setup_app();

        let center = GridPosition { x: 10, y: 10 };

        // Setup terrain and structure
        let terrain = app.world_mut().spawn((
            center,
            TerrainType::Grass,
        )).id();

        let structure = app.world_mut().spawn((
            center,
            Structure { integrity: 100.0, max_integrity: 100.0 },
        )).id();

        app.world_mut().send_event(ImpactStrikeEvent {
            center,
            radius: 2, // Will hit center
        });

        app.update();

        // Structure should be obliterated
        assert!(app.world().get_entity(structure).is_none());

        // Terrain should be converted to DeepRock/Crater
        let new_terrain = app.world().get::<TerrainType>(terrain).unwrap();
        assert_eq!(*new_terrain, TerrainType::DeepRock);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainType;
use crate::layer1::structure::Structure;
use crate::shared::time::SimulationTime;

#[derive(Component)]
pub struct IncomingImpact {
    pub target_pos: GridPosition,
    pub radius: i32,
    pub ticks_remaining: u32,
}

#[derive(Event)]
pub struct ImpactWarningEvent {
    pub target_pos: GridPosition,
    pub ticks_remaining: u32,
}

#[derive(Event)]
pub struct ImpactStrikeEvent {
    pub center: GridPosition,
    pub radius: i32,
}

pub fn process_impact_countdown_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut impacts: Query<(Entity, &mut IncomingImpact)>,
    mut strike_events: EventWriter<ImpactStrikeEvent>,
) {
    for (entity, mut impact) in impacts.iter_mut() {
        if impact.ticks_remaining > 0 {
            impact.ticks_remaining -= 1;
        }

        if impact.ticks_remaining == 0 {
            strike_events.send(ImpactStrikeEvent {
                center: impact.target_pos,
                radius: impact.radius,
            });
            commands.entity(entity).despawn();
        }
    }
}

pub fn process_impact_strike_system(
    mut strike_events: EventReader<ImpactStrikeEvent>,
    mut commands: Commands,
    structures: Query<(Entity, &GridPosition), With<Structure>>,
    mut terrains: Query<(&GridPosition, &mut TerrainType)>,
) {
    for ev in strike_events.read() {
        // Obliterate structures
        for (entity, pos) in structures.iter() {
            if pos.distance_chebyshev(&ev.center) <= ev.radius {
                commands.entity(entity).despawn();
            }
        }

        // Crater terrain
        for (pos, mut terrain) in terrains.iter_mut() {
            if pos.distance_chebyshev(&ev.center) <= ev.radius {
                *terrain = TerrainType::DeepRock;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Terrain Logic:** Add a distinct `Crater` terrain type or add a `Cratered` component instead of just making it `DeepRock` for better rendering context.
- **Deflection:** Hook the `IncomingImpact` up to a Layer 2 fleet system. If a fleet engages and destroys the incoming object in orbit, despawn it before the countdown hits 0.
- **Radius Scaling:** Make the destruction radius scale outward with a gradient of damage (total obliteration at center, structural damage at edges, fires spawned).
- **Chronicle Event:** Send an `AddChronicleEvent` documenting the exact tick the colony was hit.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Code coverage ≥ 85% for the new feature module.
- [ ] Countdown reaches 0 and fires an `ImpactStrikeEvent`.
- [ ] `ImpactStrikeEvent` destroys structures and alters terrain within its radius.

## 7. Technical Guidance

- **Placement:** Create a new module `src/layer2/disasters/impact.rs` or `src/layer1/environment/impact.rs`.
- **System Registration:** Add systems to the main update loop (likely `execution.rs` or similar for environment events).
- **Distance Calculation:** The `distance_chebyshev` method on `GridPosition` is standard in this codebase for radius checks. Ensure you import it if it's available, or implement a simple `max(dx.abs(), dy.abs())` check.

## 8. Questions
*Builder: add questions here if spec is unclear.*
