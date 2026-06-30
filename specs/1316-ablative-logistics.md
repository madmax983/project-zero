# 1316: Ablative Logistics

## 1. Overview
Supply drops from orbit are extremely fast but highly destructive. They are encased in "Ablative Foam" and fired from orbit, crashing into the `TerrainGrid`. The impact destroys existing structures and replaces the target tile with `AblativeFoam`. The foam must be mined away by Pops to access the packaged items inside.

This introduces a tension between the need for fast/cheap delivery and the risk of precision/safety failure.

## 2. Dependencies
- Layer 1 `TerrainGrid` (for terrain and structure modification)
- Layer 1 `Inventory` (for holding the delivered items)
- Layer 1 `MiningJob` or task system (for mining the foam)
- Layer 1 `ItemType` / `ResourceType`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{TerrainGrid, GridPosition, TerrainType};
    use crate::layer1::inventory::{Inventory, ItemType};

    #[test]
    fn test_ablative_drop_replaces_terrain() {
        // Arrange
        let mut app = App::new();
        let mut grid = TerrainGrid::new(10, 10);
        let drop_pos = GridPosition { x: 5, y: 5 };
        grid.set_terrain(drop_pos, TerrainType::Grass);
        app.insert_resource(grid);
        app.add_systems(Update, handle_ablative_drop_system);

        let drop_items = vec![ItemType::Rations, ItemType::MedicalSupplies];
        app.world_mut().send_event(AblativeDropEvent {
            target: drop_pos,
            contents: drop_items.clone(),
        });

        // Act
        app.update();

        // Assert
        let final_grid = app.world().resource::<TerrainGrid>();
        assert_eq!(final_grid.get_terrain(drop_pos), TerrainType::AblativeFoam);

        // Ensure a pod entity exists with the items
        let mut q_pods = app.world_mut().query::<(&GridPosition, &AblativePod)>();
        let mut found = false;
        for (pos, pod) in q_pods.iter(app.world()) {
            if *pos == drop_pos {
                assert_eq!(pod.contents, drop_items);
                found = true;
            }
        }
        assert!(found, "Ablative pod should be spawned at the drop location.");
    }

    #[test]
    fn test_mining_foam_yields_items() {
        // Arrange
        let mut app = App::new();
        let mut grid = TerrainGrid::new(10, 10);
        let drop_pos = GridPosition { x: 5, y: 5 };
        grid.set_terrain(drop_pos, TerrainType::AblativeFoam);
        app.insert_resource(grid);

        let pod_entity = app.world_mut().spawn((
            drop_pos,
            AblativePod {
                contents: vec![ItemType::Rations],
            }
        )).id();

        app.add_systems(Update, handle_foam_mining_system);

        // Simulate a completed mining action on this tile
        app.world_mut().send_event(FoamMinedEvent {
            target: drop_pos,
            miner: Entity::PLACEHOLDER,
        });

        // Act
        app.update();

        // Assert
        let final_grid = app.world().resource::<TerrainGrid>();
        // Terrain should revert or become rubble/crater. We'll use Dirt for now.
        assert_eq!(final_grid.get_terrain(drop_pos), TerrainType::Dirt);

        // Pod should be despawned
        assert!(app.world().get_entity(pod_entity).is_none());

        // Wait for inventory drop system or ensure the items are dropped at the location
        // (Simplified assertion: We assume FoamMinedEvent triggers an item drop)
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal definitions to pass the tests

#[derive(Event)]
pub struct AblativeDropEvent {
    pub target: GridPosition,
    pub contents: Vec<ItemType>,
}

#[derive(Component)]
pub struct AblativePod {
    pub contents: Vec<ItemType>,
}

#[derive(Event)]
pub struct FoamMinedEvent {
    pub target: GridPosition,
    pub miner: Entity,
}

// In the actual terrain enum:
// pub enum TerrainType { ... AblativeFoam, ... }

pub fn handle_ablative_drop_system(
    mut commands: Commands,
    mut drop_events: EventReader<AblativeDropEvent>,
    mut grid: ResMut<TerrainGrid>,
) {
    for event in drop_events.read() {
        grid.set_terrain(event.target, TerrainType::AblativeFoam);
        commands.spawn((
            event.target,
            AblativePod {
                contents: event.contents.clone(),
            },
        ));
    }
}

pub fn handle_foam_mining_system(
    mut commands: Commands,
    mut mined_events: EventReader<FoamMinedEvent>,
    mut grid: ResMut<TerrainGrid>,
    pod_query: Query<(Entity, &GridPosition, &AblativePod)>,
) {
    for event in mined_events.read() {
        grid.set_terrain(event.target, TerrainType::Dirt);
        for (entity, pos, _pod) in pod_query.iter() {
            if *pos == event.target {
                // Here we would drop the contents into the world's item system
                commands.entity(entity).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Structure Destruction:** Currently, the RED phase only checks terrain replacement. The system should also query and despawn or damage structures existing on that tile (e.g., Hospital).
- **Crater Mechanic:** Instead of turning into `Dirt` immediately, we might want it to leave a `Crater` terrain or require repair.
- **Safety Radius:** Consider adding a small area-of-effect (AoE) damage or stun to pops standing adjacent to the drop zone.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Ablative drop replaces terrain and spawns pod component
- [ ] Mining foam restores terrain and accesses the pod contents

## 7. Technical Guidance
- Integrate `TerrainType::AblativeFoam` into `src/layer1/terrain.rs`.
- The mining job should be a standard utility task. The AI should prioritize it if there's a need for the items inside.
- Ensure `AblativeDropEvent` handles out-of-bounds coordinates gracefully.

## 8. Questions
*Builder: add questions here if spec is unclear.*
