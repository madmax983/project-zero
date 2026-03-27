# 669 - Urban Spelunking

## 1. Overview
The "Urban Spelunking" feature introduces ancient, buried infrastructure ("Ancient Conduits") on Layer 1. As the colony expands and mines underground, they uncover these conduits. Players can tap into them for free resources (Power, Water, Data). However, tapping them carries significant risk: the conduits are unstable and prone to "Surges" or "Leaks" of unknown, dangerous substances that can damage the colony's modern infrastructure or poison pops. This fulfills the "The city is built on the bones of giants" fantasy.

## 2. Dependencies
- Layer 1 mining and digging mechanics
- Energy/Water/Data resource grids
- GridPosition and Terrain features
- Event system for triggering "Surge" or "Leak" disasters

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::energy::PowerGrid;
    use crate::layer1::execution::mining::MineEvent;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<MineEvent>();
        app.add_event::<ConduitSurgeEvent>();
        app.insert_resource(PowerGrid { total_power: 0.0 });
        app.add_systems(Update, (
            process_mine_conduit_system,
            tap_conduit_power_system,
            process_conduit_surge_system,
        ));
        app
    }

    #[test]
    fn test_mining_reveals_ancient_conduit() {
        let mut app = setup_app();
        let entity = app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            BuriedConduit { resource_type: ConduitResource::Power, yield_amount: 100.0 },
        )).id();

        // Mine the tile containing the buried conduit
        app.world_mut().send_event(MineEvent { target: entity, amount: 10.0 });
        app.update();

        // Assert the conduit is now exposed
        assert!(app.world().get::<ExposedConduit>(entity).is_some());
    }

    #[test]
    fn test_exposed_conduit_provides_free_power_but_risks_surge() {
        let mut app = setup_app();
        let entity = app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            ExposedConduit { resource_type: ConduitResource::Power, yield_amount: 100.0 },
            ConduitRisk { surge_chance: 1.0 }, // 100% chance to surge for testing
        )).id();

        app.update();

        // The conduit should have provided power
        let grid = app.world().get_resource::<PowerGrid>().unwrap();
        assert_eq!(grid.total_power, 100.0);

        // And it should have caused a surge event due to 100% risk
        let surge_events = app.world().get_resource::<Events<ConduitSurgeEvent>>().unwrap();
        let mut reader = surge_events.get_reader();
        let ev = reader.read(surge_events).next().unwrap();
        assert_eq!(ev.source_entity, entity);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::map::GridPosition;
use crate::layer1::energy::PowerGrid;
use crate::layer1::execution::mining::MineEvent;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConduitResource {
    Power,
    Water,
    Data,
}

#[derive(Component)]
pub struct BuriedConduit {
    pub resource_type: ConduitResource,
    pub yield_amount: f32,
}

#[derive(Component)]
pub struct ExposedConduit {
    pub resource_type: ConduitResource,
    pub yield_amount: f32,
}

#[derive(Component)]
pub struct ConduitRisk {
    pub surge_chance: f32, // 0.0 to 1.0
}

#[derive(Event)]
pub struct ConduitSurgeEvent {
    pub source_entity: Entity,
    pub position: GridPosition,
    pub magnitude: f32,
}

pub fn process_mine_conduit_system(
    mut commands: Commands,
    mut mine_events: EventReader<MineEvent>,
    conduits: Query<(Entity, &BuriedConduit)>,
) {
    for ev in mine_events.read() {
        if let Ok((entity, buried)) = conduits.get(ev.target) {
            // Expose the conduit
            commands.entity(entity)
                .remove::<BuriedConduit>()
                .insert(ExposedConduit {
                    resource_type: buried.resource_type,
                    yield_amount: buried.yield_amount,
                })
                .insert(ConduitRisk { surge_chance: 0.01 }); // Base 1% risk per tick
        }
    }
}

pub fn tap_conduit_power_system(
    mut power_grid: ResMut<PowerGrid>,
    conduits: Query<&ExposedConduit>,
) {
    for conduit in conduits.iter() {
        if conduit.resource_type == ConduitResource::Power {
            power_grid.total_power += conduit.yield_amount;
        }
    }
}

pub fn process_conduit_surge_system(
    mut commands: Commands,
    mut surge_events: EventWriter<ConduitSurgeEvent>,
    conduits: Query<(Entity, &GridPosition, &ConduitRisk, &ExposedConduit)>,
) {
    let mut rng = rand::thread_rng();
    for (entity, pos, risk, conduit) in conduits.iter() {
        if rng.gen::<f32>() < risk.surge_chance {
            surge_events.send(ConduitSurgeEvent {
                source_entity: entity,
                position: *pos,
                magnitude: conduit.yield_amount * 2.0, // Surge is a massive spike
            });

            // Optionally, the conduit breaks after a surge
            commands.entity(entity).remove::<ExposedConduit>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Grid Integrations:** Integrate with `WaterGrid` and `DataNetwork` (if applicable) alongside `PowerGrid`.
- **Surge Consequences:** The `ConduitSurgeEvent` should be caught by an environmental hazard system that damages nearby buildings, scorches the terrain, or emits toxic gas (Leaks) using the fluid/atmosphere simulation.
- **Player Choice:** Make tapping the conduit a deliberate `Designation` action by a Pop rather than an automatic passive effect once exposed. It should require an "Ancient Tap" building.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Code coverage is ≥85%.
- [ ] Mining a `BuriedConduit` converts it to an `ExposedConduit`.
- [ ] `ExposedConduit` adds to the `PowerGrid` and periodically rolls for a `ConduitSurgeEvent`.

## 7. Technical Guidance

- **Location:** Place in `src/layer1/ruins/conduits.rs` or `src/layer1/infrastructure/ancient.rs`.
- **Map Generation:** In the map generator, scatter `BuriedConduit` entities underneath normal `Rock` or `DeepRock` terrain, forming linear paths if possible.
- **RNG Handling:** In tests, pass a fixed seed or bypass the RNG for deterministic verification, or use the 1.0 chance trick shown above.

## 8. Questions
*Builder: add questions here if spec is unclear.*
