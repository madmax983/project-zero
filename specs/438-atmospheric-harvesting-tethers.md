# 438: Atmospheric Harvesting Tethers

## Overview

Construct massive tethers that reach from Layer 1 into the upper atmosphere (Layer 2) to harvest rare gases. They are highly efficient but obstruct Layer 2 traffic and are vulnerable to Layer 1 extreme weather, which can snap them, causing catastrophic kinetic damage to the colony below.

## Dependencies

- `207` Atmospheric Processors
- `079` Weather Events
- `152` Orbital Stations

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<WeatherEvent>();
        app.add_event::<KineticDamageEvent>();
        app.add_systems(Update, (tether_harvest_system, tether_weather_damage_system));
        app
    }

    #[test]
    fn test_tether_harvests_gases() {
        let mut app = setup_test_app();

        let tether = app.world.spawn(AtmosphericTether { efficiency: 10.0, integrity: 100.0 }).id();
        app.world.insert_resource(ColonyResources::default());

        app.update();

        let resources = app.world.resource::<ColonyResources>();
        assert!(resources.rare_gases > 0.0, "Tether should harvest rare gases over time");
    }

    #[test]
    fn test_hurricane_snaps_tether() {
        let mut app = setup_test_app();

        let tether = app.world.spawn(AtmosphericTether { efficiency: 10.0, integrity: 10.0 }).id();

        // Spawn severe hurricane weather
        app.world.send_event(WeatherEvent { type_: WeatherType::Hurricane, severity: 5.0 });

        app.update();

        let tether_component = app.world.get::<AtmosphericTether>(tether);
        assert!(tether_component.is_none(), "Tether should be destroyed by extreme weather");
    }

    #[test]
    fn test_snapped_tether_causes_kinetic_damage() {
        let mut app = setup_test_app();

        let tether = app.world.spawn(AtmosphericTether { efficiency: 10.0, integrity: 10.0 }).id();
        app.world.send_event(WeatherEvent { type_: WeatherType::Hurricane, severity: 5.0 });

        app.update();

        let damage_events = app.world.resource::<Events<KineticDamageEvent>>();
        let mut reader = damage_events.get_reader();
        let events: Vec<_> = reader.read(damage_events).collect();

        assert!(!events.is_empty(), "A snapped tether should emit a KineticDamageEvent");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct AtmosphericTether {
    pub efficiency: f32,
    pub integrity: f32, // 0 to 100
}

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub rare_gases: f32,
}

#[derive(Event)]
pub struct WeatherEvent {
    pub type_: WeatherType,
    pub severity: f32,
}

#[derive(PartialEq)]
pub enum WeatherType {
    Clear,
    Hurricane,
}

#[derive(Event)]
pub struct KineticDamageEvent {
    pub magnitude: f32,
}

pub fn tether_harvest_system(
    mut tethers: Query<&AtmosphericTether>,
    mut resources: ResMut<ColonyResources>,
    time: Res<Time>,
) {
    for tether in tethers.iter() {
        if tether.integrity > 0.0 {
            resources.rare_gases += tether.efficiency * time.delta_seconds();
        }
    }
}

pub fn tether_weather_damage_system(
    mut commands: Commands,
    mut tethers: Query<(Entity, &mut AtmosphericTether)>,
    mut weather_events: EventReader<WeatherEvent>,
    mut damage_events: EventWriter<KineticDamageEvent>,
) {
    for event in weather_events.read() {
        if event.type_ == WeatherType::Hurricane {
            for (entity, mut tether) in tethers.iter_mut() {
                tether.integrity -= event.severity * 10.0;

                if tether.integrity <= 0.0 {
                    // Tether snaps
                    damage_events.send(KineticDamageEvent { magnitude: 100.0 });
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Implement spatial damage for the `KineticDamageEvent` so it destroys buildings/Pops directly under or near the tether's base.
- Add a Layer 2 obstruction mechanic where trade ships have a small chance to collide with the tether, sparking a `DiplomaticIncidentEvent`.
- Provide player notifications/warnings when a tether's integrity is getting low, allowing them to retract or reinforce it.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Tether passively harvests `rare_gases` for the colony.
- [ ] Severe weather (e.g. `Hurricane`) reduces tether integrity.
- [ ] When integrity reaches 0, tether despawns and emits `KineticDamageEvent`.

## Technical Guidance

- For the `KineticDamageEvent`, pass along the `Transform` of the tether's base on Layer 1 so that the resulting damage system can compute blast radii or line-of-effect destruction on the colony grid.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
