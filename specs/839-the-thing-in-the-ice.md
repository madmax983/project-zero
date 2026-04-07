# The "Thing" in the Ice

## 1. Overview
**Layer:** 1
**Fantasy:** We dug too deep, and we found something that was sleeping.
**Mechanic:** Mining ice or frozen terrain has a chance to spawn a "Frozen Block". Thawing it (Heat) releases *something* (Ancient Pop, Precursor Robot, or Horrific Beast). You don't know until you thaw it.
**Emergence:** You thaw a block hoping for an Ancient Scientist. It's a "Phase-Spider". It eats the intern.
**Tension:** Curiosity (Potential reward) vs. Safety.

## 2. Dependencies
- Mining and Terrain generation systems.
- Temperature/Heat mechanics.
- Spawning tables/Entity generation logic.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct TerrainCell { is_frozen: bool }

    #[derive(Component)]
    struct FrozenBlock { thaw_progress: f32 }

    #[derive(Component)]
    struct HeatSource { temperature: f32 }

    #[test]
    fn test_mining_frozen_terrain_spawns_frozen_block() {
        let mut app = App::new();
        app.add_systems(Update, process_mining_event);

        let terrain = app.world_mut().spawn(TerrainCell { is_frozen: true }).id();

        app.world_mut().resource_mut::<Events<MineIntent>>().send(MineIntent { target: terrain });
        app.update();

        let mut found_block = false;
        for _ in app.world_mut().query::<&FrozenBlock>().iter(app.world()) {
            found_block = true;
        }
        assert!(found_block, "Mining frozen terrain should yield a frozen block.");
    }

    #[test]
    fn test_heat_thaws_frozen_block() {
        let mut app = App::new();
        app.add_systems(Update, process_thawing);

        let block = app.world_mut().spawn((FrozenBlock { thaw_progress: 0.0 }, Transform::default())).id();
        app.world_mut().spawn((HeatSource { temperature: 50.0 }, Transform::default()));

        app.update();

        let thawed_block = app.world().get::<FrozenBlock>(block).unwrap();
        assert!(thawed_block.thaw_progress > 0.0, "Heat sources should increase thaw progress.");
    }

    #[test]
    fn test_fully_thawed_block_releases_entity() {
        let mut app = App::new();
        app.add_systems(Update, process_thawing);

        let block = app.world_mut().spawn((FrozenBlock { thaw_progress: 99.0 }, Transform::default())).id();
        app.world_mut().spawn((HeatSource { temperature: 50.0 }, Transform::default()));

        app.update();

        assert!(app.world().get::<FrozenBlock>(block).is_none(), "Fully thawed block should despawn.");
        let mut spawned_creature = false;
        for _ in app.world_mut().query::<&ReleasedEntityMarker>().iter(app.world()) {
            spawned_creature = true;
        }
        assert!(spawned_creature, "Despawning block should spawn a new entity.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct MineIntent {
    pub target: Entity,
}

#[derive(Component)]
pub struct ReleasedEntityMarker;

pub fn process_mining_event(
    mut commands: Commands,
    mut intents: EventReader<MineIntent>,
    terrain: Query<&TerrainCell>,
) {
    for intent in intents.read() {
        if let Ok(cell) = terrain.get(intent.target) {
            if cell.is_frozen {
                commands.spawn(FrozenBlock { thaw_progress: 0.0 });
                // Note: Real implementation needs random chance, not 100%.
            }
        }
    }
}

pub fn process_thawing(
    mut commands: Commands,
    mut blocks: Query<(Entity, &mut FrozenBlock, &Transform)>,
    heat_sources: Query<(&HeatSource, &Transform)>,
) {
    for (entity, mut block, block_transform) in blocks.iter_mut() {
        for (heat, heat_transform) in heat_sources.iter() {
            let distance = block_transform.translation.distance(heat_transform.translation);
            if distance < 5.0 {
                block.thaw_progress += heat.temperature * 0.1;

                if block.thaw_progress >= 100.0 {
                    commands.entity(entity).despawn();
                    commands.spawn(ReleasedEntityMarker); // Spawn "The Thing"
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spawn Chances:** Mining shouldn't guarantee a block. Use a `rand` roll (e.g., 5% chance per block mined).
- **Loot Tables:** Releasing a generic `ReleasedEntityMarker` is insufficient. The block should roll on a weighted loot table upon despawn to yield either an ancient resource cache, a neutral Precursor Robot, or a hostile Phase-Spider.
- **Spatial Grid Optimization:** Calculating distance between all blocks and all heat sources per frame is inefficient. Use the game's spatial partitioning map to look up local ambient temperature instead.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Frozen blocks spawn probabilistically when mining frozen terrain cells.
- [ ] Ambient heat sources accumulate thaw progress over time.
- [ ] Reaching 100% thaw progress despawns the block and spawns a random entity from a loot table.

## 7. Technical Guidance
- Link the thawing rate to the `Temperature` map already implemented in Layer 1. If ambient temperature exceeds freezing (0°C), increment progress based on delta.
- Add an `Unidentified` tag to the frozen block UI so the player can't inspect the contents until it's too late.

## 8. Questions
*Builder: add questions here if spec is unclear.*
