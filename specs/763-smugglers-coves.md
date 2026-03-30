# 763 - Smuggler's Coves

## 1. Overview
**Layer:** 1
**Fantasy:** The map is big, and you can't watch every canyon.
**Mechanic:** If Colony Authority is low or Corruption is high, "Smuggler" ships will land in unobserved areas of the map (fog of war / far from buildings) to set up temporary shops. They sell illicit goods but drain credits and spread vice.
**Emergence:** You notice your miners are addicted to "Red Sand". You follow them into the woods and find a hidden landing pad operating right under your nose.
**Tension:** Patrol the wilderness (expensive) or tolerate the free market (social decay)?

## 2. Dependencies
- Layer 1 Map & Fog of War visibility (`VisibilityGrid` or similar tracking of observed tiles)
- Layer 1 Economy and Trading (ability to purchase/sell goods and pop tracking of credits/inventory)
- Layer 1 Authority / Corruption stats (`ColonyAuthority` resource)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{TerrainGrid, GridPosition};
    use crate::layer1::economy::TradeAction;
    use crate::shared::time::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.insert_resource(ColonyAuthority { level: 20.0 }); // Low authority
        app.insert_resource(TerrainGrid::new(100, 100)); // 100x100 map

        // Add minimal required systems
        app.add_systems(Update, spawn_smugglers_cove_system);
        app.add_systems(Update, process_smuggler_decay_system);
        app
    }

    #[test]
    fn test_smuggler_cove_spawns_in_low_authority() {
        let mut app = setup_app();

        // Force deterministic spawn by setting rng probability check to 1.0
        // in test environment, or run enough ticks to guarantee it.
        // For simplicity in this test, we run for 1000 ticks to make failure statistically impossible.
        for _ in 0..1000 {
            app.update();
        }

        // Verify a smuggler's cove has spawned
        let mut cove_query = app.world_mut().query::<&SmugglersCove>();
        assert!(
            cove_query.iter(app.world()).count() > 0,
            "A smuggler's cove should spawn when authority is low"
        );
    }

    #[test]
    fn test_smuggler_cove_does_not_spawn_in_high_authority() {
        let mut app = setup_app();

        // High authority should prevent smuggler coves from spawning
        app.world_mut().resource_mut::<ColonyAuthority>().level = 90.0;

        for _ in 0..100 {
            app.update();
        }

        let mut cove_query = app.world_mut().query::<&SmugglersCove>();
        assert_eq!(
            cove_query.iter(app.world()).count(),
            0,
            "A smuggler's cove should not spawn when authority is high"
        );
    }

    #[test]
    fn test_smuggler_cove_decays_over_time() {
        let mut app = setup_app();

        // Manually spawn a cove
        let cove_entity = app.world_mut().spawn((
            SmugglersCove { lifespan: 5 },
            GridPosition { x: 50, y: 50 }
        )).id();

        // Advance time and check lifespan
        for _ in 0..6 {
            app.update();
        }

        // The cove should be despawned after its lifespan expires
        assert!(app.world().get_entity(cove_entity).is_none(), "Cove should despawn after lifespan");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::map::{TerrainGrid, GridPosition};
use crate::shared::time::SimulationTime;

#[derive(Resource, Default)]
pub struct ColonyAuthority {
    pub level: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct SmugglersCove {
    pub lifespan: u32,
}

pub fn spawn_smugglers_cove_system(
    mut commands: Commands,
    authority: Res<ColonyAuthority>,
    grid: Res<TerrainGrid>,
) {
    // Only spawn if authority is low
    if authority.level > 30.0 {
        return;
    }

    let mut rng = rand::thread_rng();

    // Small chance to spawn a cove each tick
    if rng.gen::<f32>() < 0.02 {
        // Pick a random unobserved location (simplified logic)
        let x = rng.gen_range(0..grid.width());
        let y = rng.gen_range(0..grid.height());

        commands.spawn((
            SmugglersCove { lifespan: 100 }, // Cove lasts for 100 ticks
            GridPosition { x, y }
        ));
    }
}

pub fn process_smuggler_decay_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SmugglersCove)>,
) {
    for (entity, mut cove) in query.iter_mut() {
        cove.lifespan = cove.lifespan.saturating_sub(1);
        if cove.lifespan == 0 {
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spawn Conditions:** The random position selection needs to query `VisibilityGrid` to ensure the smuggler's cove only spawns in unobserved (Fog of War) tiles.
- **Inventory/Economy:** `SmugglersCove` needs an inventory system attached so that Pops can pathfind to it and execute `TradeAction`s for illicit goods (e.g. Red Sand).
- **Social Impact:** When a Pop buys from a Smuggler's Cove, it should trigger a `ViceEvent` or alter the Pop's utility weights to increase future desires for illicit goods.
- **Patrols:** Add an interaction where if a Guard or Sheriff pathfinds near a `SmugglersCove`, the cove is immediately despawned (busted) and `ColonyAuthority` goes up slightly.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Code coverage is ≥85% for the new module.
- [ ] Smuggler coves only spawn when `ColonyAuthority` is below a certain threshold.
- [ ] Coves despawn naturally over time.
- [ ] Integration: Pops pathfind to coves to fulfill needs if illicit goods are available.

## 7. Technical Guidance
- **Integration:** The `spawn_smugglers_cove_system` should be added to the `Economy` schedule in Layer 1, while `process_smuggler_decay_system` fits in `Observation` or `Cleanup`.
- **Visibility:** To find an unobserved tile, randomly sample points and check `VisibilityGrid::is_observed(x, y)`. Cap the number of random checks per tick (e.g., 10 attempts) to prevent performance hits when the map is fully explored.
- **Lore Hooks:** Emit an `AddChronicleEvent` when a cove is busted by security forces.

## 8. Questions
*Builder: add questions here if spec is unclear.*
