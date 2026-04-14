# 1020: Spatial Compression

## 1. Overview
High-tech "Pocket Dimension" rooms allow immense space saving on Layer 1. They occupy a single tile on the macro map but contain an entire sub-grid (e.g., 10x10) inside. However, maintaining this dimensional fold requires constant Energy and imposes Sanity/Stress costs on inhabitants. If external power fails, the pocket collapses, explosively ejecting all contents (Pops, items, structures) onto the single exterior tile, causing massive damage and chaos.

## 2. Dependencies
- Layer 1 `TerrainGrid` (Sub-grid handling).
- Layer 1 `Power` system.
- Layer 1 `Pop` entity (Sanity/Needs).
- Layer 1 `Damage`/`Physics` (Ejection mechanics).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::power::{PowerGrid, PowerNode};
    use crate::layer1::terrain::GridPosition;
    use crate::layer1::health::{Health, DamageEvent};
    use crate::layer1::pop::Pop;

    #[test]
    fn test_power_failure_collapses_pocket_dimension() {
        let mut app = App::new();
        app.add_event::<PocketCollapseEvent>();
        app.add_systems(Update, monitor_pocket_power_system);

        let pocket = app.world_mut().spawn((
            PocketDimension { is_stable: true, external_position: GridPosition { x: 5, y: 5, z: 0 } },
            PowerNode { current_power: 0, required_power: 100 }, // Underpowered
        )).id();

        app.update();

        let collapse_events = app.world().resource::<Events<PocketCollapseEvent>>();
        let mut reader = collapse_events.get_reader();
        let mut found = false;
        for event in reader.read(collapse_events) {
            if event.pocket == pocket {
                found = true;
            }
        }

        assert!(found, "An underpowered pocket dimension should trigger a collapse event.");
        let dim = app.world().get::<PocketDimension>(pocket).unwrap();
        assert!(!dim.is_stable, "Pocket dimension should be marked unstable.");
    }

    #[test]
    fn test_collapse_ejects_and_damages_contents() {
        let mut app = App::new();
        app.add_event::<PocketCollapseEvent>();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, process_pocket_collapse_system);

        let pocket = app.world_mut().spawn((
            PocketDimension { is_stable: false, external_position: GridPosition { x: 5, y: 5, z: 0 } },
        )).id();

        // Spawn a pop "inside" the pocket (using a parent/child or specific coordinate plane)
        let trapped_pop = app.world_mut().spawn((
            Pop,
            InsidePocket { pocket },
            GridPosition { x: 1, y: 1, z: -99 }, // Internal coordinates
        )).id();

        app.world_mut().resource_mut::<Events<PocketCollapseEvent>>().send(PocketCollapseEvent { pocket });

        app.update();

        // Check ejection position
        let pop_pos = app.world().get::<GridPosition>(trapped_pop).unwrap();
        assert_eq!(pop_pos.x, 5, "Ejected pop should match external pocket X coordinate.");
        assert_eq!(pop_pos.y, 5, "Ejected pop should match external pocket Y coordinate.");
        assert_eq!(pop_pos.z, 0, "Ejected pop should match external pocket Z coordinate.");

        // Check damage
        let damage_events = app.world().resource::<Events<DamageEvent>>();
        let mut reader = damage_events.get_reader();
        let mut found_damage = false;
        for event in reader.read(damage_events) {
            if event.target == trapped_pop {
                found_damage = true;
            }
        }
        assert!(found_damage, "Pops ejected from a collapsing pocket dimension should take damage.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/spatial_compression.rs
use bevy::prelude::*;
use crate::layer1::power::PowerNode;
use crate::layer1::terrain::GridPosition;
use crate::layer1::health::DamageEvent;

#[derive(Component)]
pub struct PocketDimension {
    pub is_stable: bool,
    pub external_position: GridPosition,
}

#[derive(Component)]
pub struct InsidePocket {
    pub pocket: Entity,
}

#[derive(Event)]
pub struct PocketCollapseEvent {
    pub pocket: Entity,
}

pub fn monitor_pocket_power_system(
    mut query: Query<(Entity, &mut PocketDimension, &PowerNode)>,
    mut collapse_events: EventWriter<PocketCollapseEvent>,
) {
    for (entity, mut dim, power) in query.iter_mut() {
        if dim.is_stable && power.current_power < power.required_power {
            dim.is_stable = false;
            collapse_events.send(PocketCollapseEvent { pocket: entity });
        }
    }
}

pub fn process_pocket_collapse_system(
    mut collapse_events: EventReader<PocketCollapseEvent>,
    dim_query: Query<&PocketDimension>,
    mut contents_query: Query<(Entity, &InsidePocket, &mut GridPosition)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for event in collapse_events.read() {
        if let Ok(dim) = dim_query.get(event.pocket) {
            for (ent, inside, mut pos) in contents_query.iter_mut() {
                if inside.pocket == event.pocket {
                    // Eject to external position
                    *pos = dim.external_position.clone();

                    // Apply ejection trauma
                    damage_events.send(DamageEvent {
                        target: ent,
                        amount: 50.0,
                        source: "Dimensional Collapse".to_string(),
                    });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Sub-Grid Architecture:** The MVP uses a component `InsidePocket` and negative Z coordinates. The actual implementation should ideally instantiate a secondary `TerrainGrid` that links to the primary grid tile.
- **Sanity Drain:** Implement the Sanity/Stress cost for living in compressed space (a slow drain over time).
- **Collision Handling:** When 500 Pops are ejected onto a single tile, the physics/movement engine needs a way to handle extreme over-stacking (e.g., pushing them to adjacent tiles or applying crush damage).

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_power_failure_collapses_pocket_dimension` passes.
- [ ] Test `test_collapse_ejects_and_damages_contents` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Removing `InsidePocket` from the entities during ejection is necessary to clean up state, which can be done via `Commands` in the collapse system.
- Ensure the `PowerNode` is correctly hooked into the colony's overarching `PowerGrid`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
