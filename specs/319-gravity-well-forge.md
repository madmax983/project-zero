# 319: The Gravity Well Forge

## 1. Overview

Players can construct specialized "Deep Forges" in the lower atmosphere of Gas Giants (Layer 2) to forge advanced "Hyper-Alloys" using extreme pressure. Operating these forges is incredibly dangerous; any maintenance failure or structural damage causes the forge (and its crew) to be instantly crushed by gravity. This mechanic creates a tension between accessing endgame materials and the constant, catastrophic risk of operating in extreme environments.

## 2. Dependencies

- `024` Metal Industry (for standard alloy mechanics)
- `152` Orbital Stations (for construction in Layer 2)
- `112` Maintenance Debt (for calculating catastrophic failure chance)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_deep_forge_production() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_deep_forges);

        let forge = app.world_mut().spawn((
            DeepForge {
                production_rate: 10.0,
                base_crush_chance: 0.01,
                is_active: true,
            },
            MaintenanceLevel { current: 100.0 }, // Perfect maintenance
        )).id();

        app.world_mut().insert_resource(HyperAlloyStockpile { amount: 0.0 });

        // Act
        app.update();

        // Assert
        let stockpile = app.world().get_resource::<HyperAlloyStockpile>().unwrap();
        assert_eq!(stockpile.amount, 10.0, "Active forge should produce hyper-alloys");

        let forge_exists = app.world().get::<DeepForge>(forge).is_some();
        assert!(forge_exists, "Perfectly maintained forge should survive");
    }

    #[test]
    fn test_deep_forge_crush_failure() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_deep_forges);

        let forge = app.world_mut().spawn((
            DeepForge {
                production_rate: 10.0,
                base_crush_chance: 1.0, // Guaranteed failure
                is_active: true,
            },
            MaintenanceLevel { current: 0.0 }, // Zero maintenance
            Crew { count: 50 },
        )).id();

        // Act
        app.update();

        // Assert
        let forge_exists = app.world().get::<DeepForge>(forge).is_some();
        assert!(!forge_exists, "Poorly maintained forge should be crushed");

        // Crew should be killed, triggering a chronicle event (implied in process)
        let crush_events = app.world_mut().query::<&ForgeCrushEvent>().iter(&app.world()).count();
        assert!(crush_events > 0, "A crush event should be spawned");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct DeepForge {
    pub production_rate: f32,
    pub base_crush_chance: f32,
    pub is_active: bool,
}

#[derive(Component)]
pub struct MaintenanceLevel {
    pub current: f32,
}

#[derive(Component)]
pub struct Crew {
    pub count: u32,
}

#[derive(Resource)]
pub struct HyperAlloyStockpile {
    pub amount: f32,
}

#[derive(Component)]
pub struct ForgeCrushEvent {
    pub location: Entity,
    pub casualties: u32,
}

pub fn process_deep_forges(
    mut commands: Commands,
    mut forge_query: Query<(Entity, &DeepForge, &MaintenanceLevel, Option<&Crew>)>,
    mut stockpile: ResMut<HyperAlloyStockpile>,
) {
    for (entity, forge, maintenance, crew) in forge_query.iter_mut() {
        if !forge.is_active { continue; }

        // Calculate failure chance inversely proportional to maintenance
        let crush_risk = forge.base_crush_chance * (1.0 - (maintenance.current / 100.0));

        if rand::random::<f32>() < crush_risk {
            // Catastrophic failure
            let casualties = if let Some(c) = crew { c.count } else { 0 };
            commands.spawn(ForgeCrushEvent {
                location: entity,
                casualties,
            });
            commands.entity(entity).despawn();
        } else {
            // Success
            stockpile.amount += forge.production_rate;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **RNG:** Use a deterministic `GlobalRng` resource instead of `rand::random`.
- **Event Bus:** `ForgeCrushEvent` should be sent via Bevy's `EventWriter` rather than spawned as an entity, so other systems (like the Chronicle or UI) can subscribe to it.
- **Resource Definition:** Ensure `HyperAlloyStockpile` integrates correctly with the global `ColonyResources` struct rather than being an isolated resource.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Deep Forges produce hyper-alloys when active and well-maintained.
- [ ] Low maintenance increases the chance of a catastrophic crush event.
- [ ] A crush event despawns the forge and registers casualties.

## 7. Technical Guidance

- Integrate the `DeepForge` building logic into `src/layer2/stations.rs`.
- The `ForgeCrushEvent` should be hooked up to `AddChronicleEvent` in `src/layer1/integration.rs` to ensure the loss is recorded in the colony's history.

## 8. Questions

*Builder: add questions here if spec is unclear.*
