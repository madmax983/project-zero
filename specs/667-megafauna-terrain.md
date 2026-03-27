# 667 - Megafauna Terrain

## 1. Overview
The "Megafauna Terrain" feature implements living, colossal creatures that masquerade as landscape features. This feature realizes the "The landscape is alive. That hill is breathing" fantasy from the design ideas. Players may unwittingly build on top of these dormant titans, or attempt to mine them. Damaging or mining them causes them to awaken, stand up, destroy any structures built upon them, and potentially move or attack, turning a prime defensive location into a massive threat.

## 2. Dependencies
- `GridPosition` and Map querying (Layer 1 Map)
- `TerrainType` and map rendering
- `Health` and damage systems
- `Structure` and building destruction mechanics

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::TerrainType;
    use crate::layer1::health::Health;
    use crate::layer1::structure::Structure;
    use crate::layer1::execution::mining::MineEvent;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<MineEvent>();
        app.add_event::<AwakenTitanEvent>();
        app.add_systems(Update, (process_mining_titan_system, awaken_titan_system));
        app
    }

    #[test]
    fn test_mining_dormant_titan_triggers_awakening() {
        let mut app = setup_app();

        let titan_entity = app.world_mut().spawn((
            GridPosition { x: 10, y: 10 },
            TerrainType::Rock,
            MegafaunaTerrain {
                is_dormant: true,
                base_health: 1000.0,
            },
        )).id();

        // Simulate a pop mining the titan terrain
        app.world_mut().send_event(MineEvent {
            target: titan_entity,
            amount: 10.0,
        });

        app.update();

        // Assert an awaken event was fired or the titan state changed
        let titan = app.world().get::<MegafaunaTerrain>(titan_entity).unwrap();
        assert!(!titan.is_dormant, "Mining a dormant titan should wake it up");
    }

    #[test]
    fn test_titan_awakening_destroys_structures_on_top() {
        let mut app = setup_app();

        let pos = GridPosition { x: 15, y: 15 };

        let titan_entity = app.world_mut().spawn((
            pos,
            TerrainType::Rock,
            MegafaunaTerrain {
                is_dormant: true,
                base_health: 1000.0,
            },
        )).id();

        let building_entity = app.world_mut().spawn((
            pos,
            Structure { integrity: 100.0, max_integrity: 100.0 },
        )).id();

        // Wake up the titan
        app.world_mut().send_event(AwakenTitanEvent {
            entity: titan_entity,
        });

        app.update();

        // The structure should be destroyed (despawned or integrity 0)
        assert!(app.world().get_entity(building_entity).is_none(), "Buildings on top of an awakened titan must be destroyed");
    }

    #[test]
    fn test_awakened_titan_changes_terrain_type() {
        let mut app = setup_app();

        let titan_entity = app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            TerrainType::Rock,
            MegafaunaTerrain {
                is_dormant: true,
                base_health: 1000.0,
            },
        )).id();

        app.world_mut().send_event(AwakenTitanEvent {
            entity: titan_entity,
        });

        app.update();

        let terrain = app.world().get::<TerrainType>(titan_entity).unwrap();
        assert_eq!(*terrain, TerrainType::DeepRock, "Awakened titan changes its physical terrain representation or becomes a distinct entity");
        // Note: Implementation may choose to convert it to a Fauna entity instead.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainType;
use crate::layer1::execution::mining::MineEvent;
use crate::layer1::structure::Structure;

#[derive(Component)]
pub struct MegafaunaTerrain {
    pub is_dormant: bool,
    pub base_health: f32,
}

#[derive(Event)]
pub struct AwakenTitanEvent {
    pub entity: Entity,
}

pub fn process_mining_titan_system(
    mut mine_events: EventReader<MineEvent>,
    mut titans: Query<&mut MegafaunaTerrain>,
    mut awaken_events: EventWriter<AwakenTitanEvent>,
) {
    for ev in mine_events.read() {
        if let Ok(mut titan) = titans.get_mut(ev.target) {
            if titan.is_dormant {
                titan.is_dormant = false;
                awaken_events.send(AwakenTitanEvent { entity: ev.target });
            }
        }
    }
}

pub fn awaken_titan_system(
    mut awaken_events: EventReader<AwakenTitanEvent>,
    mut commands: Commands,
    titans: Query<&GridPosition, With<MegafaunaTerrain>>,
    structures: Query<(Entity, &GridPosition), With<Structure>>,
) {
    for ev in awaken_events.read() {
        if let Ok(titan_pos) = titans.get(ev.entity) {
            // Destroy any structures built on the titan
            for (struct_entity, struct_pos) in structures.iter() {
                if struct_pos == titan_pos {
                    commands.entity(struct_entity).despawn();
                }
            }

            // Alter the terrain type or convert to fauna
            commands.entity(ev.entity).insert(crate::layer1::terrain::TerrainType::DeepRock);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Event Integration:** Hook `AwakenTitanEvent` into the Chronicle system so the awakening is recorded in the colony's history ("A mountain stood up today...").
- **Visuals and UI:** Ensure the player receives a severe notification when a titan awakens, as this is a massive colony-threatening event.
- **Titan Movement:** Refactor the titan into a mobile entity (using standard Fauna pathfinding but ignoring many obstacles) after it awakens.
- **Damage over Time:** If a building isn't instantly destroyed, it could take massive structural damage over a few ticks to allow Pops a split-second to flee.

## 6. Acceptance Criteria

- [ ] All tests in the RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes without errors.
- [ ] Test coverage is ≥85% for the new module.
- [ ] Mining a MegafaunaTerrain entity triggers the `AwakenTitanEvent`.
- [ ] The `awaken_titan_system` destroys all `Structure` components sharing the same `GridPosition`.

## 7. Technical Guidance

- **Placement:** Create `src/layer1/nature/megafauna_terrain.rs`.
- **Integration:** Register the `MegafaunaTerrain` systems in `src/layer1/systems/environment.rs` or observation schedule. You might need to intercept normal mining logic so that mining a titan yields less ore before it wakes up, or uses a special interaction.
- **Titan Spawn:** Add logic to world generation to occasionally spawn these instead of standard mountains/forests, keeping them indistinguishable via `TerrainType` to the player until acted upon.

## 8. Questions
*Builder: add questions here if spec is unclear.*
