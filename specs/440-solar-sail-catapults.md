# 440: Solar Sail Catapults

## Overview

Massive catchers in orbit utilize solar flares to launch or receive cargo pods from other planets for free, skipping normal fuel costs. However, a sudden spike in solar weather can cause the pods to miss the catchers and strike the colony on Layer 1 as kinetic bombardments.

## Dependencies

- `039` Trade System
- `152` Orbital Stations
- `213` Solar Cycles

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<TradeShipArrivalEvent>();
        app.add_event::<SolarFlareEvent>();
        app.add_event::<KineticBombardmentEvent>();
        app.add_systems(Update, (solar_catapult_system, solar_flare_interference_system));
        app
    }

    #[test]
    fn test_catapult_receives_free_cargo() {
        let mut app = setup_test_app();

        let catapult = app.world.spawn(SolarCatapultCatcher { active: true }).id();
        app.world.insert_resource(ColonyResources::default());

        app.world.send_event(TradeShipArrivalEvent { cargo_value: 5000.0, fuel_cost: 0.0 });
        app.update();

        let resources = app.world.resource::<ColonyResources>();
        assert!(resources.food >= 5000.0, "Catapult should successfully receive cargo without fuel cost");
    }

    #[test]
    fn test_solar_flare_causes_miss() {
        let mut app = setup_test_app();

        let catapult = app.world.spawn(SolarCatapultCatcher { active: true }).id();

        // Spike in solar weather
        app.world.send_event(SolarFlareEvent { intensity: 10.0 });
        // Incoming pod
        app.world.send_event(TradeShipArrivalEvent { cargo_value: 5000.0, fuel_cost: 0.0 });

        app.update();

        let bombardment_events = app.world.resource::<Events<KineticBombardmentEvent>>();
        let mut reader = bombardment_events.get_reader();
        let events: Vec<_> = reader.read(bombardment_events).collect();

        assert!(!events.is_empty(), "A spike in solar weather should cause the pod to miss and become a kinetic bombardment");
    }

    #[test]
    fn test_missed_pod_destroys_colony_tiles() {
        let mut app = setup_test_app();

        // Assuming a grid tile, MUST have a Transform for the damage system to find it
        let tile = app.world.spawn((TerrainGridTile { integrity: 100.0 }, Transform::from_translation(Vec3::ZERO))).id();

        app.world.send_event(KineticBombardmentEvent { magnitude: 5000.0, target: Vec3::ZERO });

        app.update();

        let tile_component = app.world.get::<TerrainGridTile>(tile);
        assert!(tile_component.unwrap().integrity < 100.0, "Kinetic bombardment should severely damage the colony surface");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SolarCatapultCatcher {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub food: f32, // placeholder for generic cargo
}

#[derive(Event)]
pub struct TradeShipArrivalEvent {
    pub cargo_value: f32,
    pub fuel_cost: f32,
}

#[derive(Event)]
pub struct SolarFlareEvent {
    pub intensity: f32,
}

#[derive(Event)]
pub struct KineticBombardmentEvent {
    pub magnitude: f32,
    pub target: Vec3,
}

#[derive(Component)]
pub struct TerrainGridTile {
    pub integrity: f32,
}

pub fn solar_catapult_system(
    mut catchers: Query<&SolarCatapultCatcher>,
    mut trade_events: EventReader<TradeShipArrivalEvent>,
    mut resources: ResMut<ColonyResources>,
    mut bombard_events: EventWriter<KineticBombardmentEvent>,
    mut flare_events: EventReader<SolarFlareEvent>,
) {
    let mut flare_intensity = 0.0;
    for flare in flare_events.read() {
        flare_intensity = flare.intensity;
    }

    // Determine if any active catcher exists (simplification for events)
    let has_active_catcher = catchers.iter().any(|c| c.active);

    if has_active_catcher {
        for trade in trade_events.read() {
            if flare_intensity > 5.0 {
                // Miss!
                bombard_events.send(KineticBombardmentEvent {
                    magnitude: trade.cargo_value,
                    target: Vec3::ZERO, // Randomize in refactor
                });
            } else {
                // Success!
                resources.food += trade.cargo_value;
            }
        }
    }
}

pub fn solar_flare_interference_system(
    mut bombard_events: EventReader<KineticBombardmentEvent>,
    mut tiles: Query<(&mut TerrainGridTile, &Transform)>,
) {
    for event in bombard_events.read() {
        for (mut tile, transform) in tiles.iter_mut() {
            if transform.translation.distance(event.target) < 10.0 {
                tile.integrity -= event.magnitude;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Randomize the `target` vector for `KineticBombardmentEvent` so the pod crashes randomly across the colony grid.
- Integrate the `fuel_cost = 0.0` benefit directly with the Trade UI so the player actively selects "Catapult Launch" (free but risky) vs "Standard Launch" (safe but costs fuel).
- Display a severe warning to the player when a `SolarFlareEvent` occurs while a pod is in transit.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `SolarCatapultCatcher` successfully receives cargo.
- [ ] Active `SolarFlareEvent` causes incoming pods to miss.
- [ ] Missed pods emit a `KineticBombardmentEvent` that damages `TerrainGridTile`s on the surface.

## Technical Guidance

- Ensure `KineticBombardmentEvent` processing is tied into existing Layer 1 destruction logic, so buildings and Pops are killed, not just the base terrain.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
