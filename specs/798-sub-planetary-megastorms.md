# 798: Sub-Planetary Megastorms

## Overview

On worlds with highly volatile crusts or immense subterranean gas pockets, "Megastorms" can originate from deep within the earth rather than the atmosphere. These storms blast toxic gases, heat, and sonic pressure *upwards* through mine shafts and caverns, destroying lower levels before venting to the surface. The safety and stable temperatures of subterranean building must be balanced against the terrifying risk of vertical weather events that turn deepest bunkers into pressure cookers.

## Dependencies

- `012` — Tile targeting and structural health (must exist for storms to damage buildings)
- `103` — Weather event framework (must exist for spawning events)

## RED Phase: Tests First

```rust
// tests/layer1/sub_planetary_megastorms.rs

use scale::layer1::weather::{WeatherSystem, SubPlanetaryMegastormEvent};
use scale::layer1::structure::StructureHealth;
use scale::layer1::terrain::TilePosition;
use bevy::prelude::*;

#[test]
fn test_megastorm_damages_subterranean_structures() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, scale::layer1::weather::process_sub_planetary_megastorms);

    let structure_entity = app.world.spawn((
        TilePosition { x: 10, y: 10, z: -5 }, // z < 0 indicates subterranean
        StructureHealth { current: 100, max: 100 }
    )).id();

    app.world.insert_resource(Events::<SubPlanetaryMegastormEvent>::default());

    // Act
    app.world.resource_mut::<Events<SubPlanetaryMegastormEvent>>().send(
        SubPlanetaryMegastormEvent {
            origin: TilePosition { x: 10, y: 10, z: -10 },
            intensity: 20,
            radius: 5,
        }
    );
    app.update();

    // Assert
    let health = app.world.get::<StructureHealth>(structure_entity).unwrap();
    assert!(health.current < 100, "Structure should take damage from the upward blast");
}

#[test]
fn test_megastorm_vents_to_surface() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, scale::layer1::weather::process_sub_planetary_megastorms);

    let surface_structure = app.world.spawn((
        TilePosition { x: 10, y: 10, z: 0 }, // Surface level
        StructureHealth { current: 100, max: 100 }
    )).id();

    app.world.insert_resource(Events::<SubPlanetaryMegastormEvent>::default());

    // Act
    app.world.resource_mut::<Events<SubPlanetaryMegastormEvent>>().send(
        SubPlanetaryMegastormEvent {
            origin: TilePosition { x: 10, y: 10, z: -10 },
            intensity: 20,
            radius: 5,
        }
    );
    app.update();

    // Assert
    let health = app.world.get::<StructureHealth>(surface_structure).unwrap();
    assert!(health.current < 100, "Venting megastorms should also damage the immediate surface tile");
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/weather.rs

use bevy::prelude::*;
use crate::layer1::structure::StructureHealth;
use crate::layer1::terrain::TilePosition;

#[derive(Event, Debug)]
pub struct SubPlanetaryMegastormEvent {
    pub origin: TilePosition,
    pub intensity: u32,
    pub radius: i32,
}

pub fn process_sub_planetary_megastorms(
    mut events: EventReader<SubPlanetaryMegastormEvent>,
    mut structures: Query<(&TilePosition, &mut StructureHealth)>,
) {
    for event in events.read() {
        for (pos, mut health) in structures.iter_mut() {
            // Check if the structure is vertically aligned with the origin (venting upwards)
            // and within the radius
            let dx = pos.x - event.origin.x;
            let dy = pos.y - event.origin.y;
            let distance_sq = dx * dx + dy * dy;

            // The storm vents upwards, so z must be >= origin.z
            if pos.z >= event.origin.z && distance_sq <= event.radius * event.radius {
                let damage = event.intensity;
                health.current = health.current.saturating_sub(damage);
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Iterating over all structures for every event is slow. We should use a spatial hash or spatial query system to only check structures in the storm's upward column.
- **Atmospheric Pressure**: The storm could alter local atmospheric pressure grids, creating temporary toxic zones instead of just dealing flat damage.
- **Visuals**: Trigger particle effects or visual warnings on the UI layer shortly before the storm erupts.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Sub-planetary megastorms deal structural damage to tiles situated vertically above the origin point.

## Technical Guidance

- Integrate the `process_sub_planetary_megastorms` system into the `WeatherSystem` set.
- Ensure the event is registered using `app.add_event::<SubPlanetaryMegastormEvent>()`.
- You may need to create or modify a random event spawner to occasionally dispatch these events based on planetary conditions.

## Questions
*Builder: add questions here if spec is unclear.*
