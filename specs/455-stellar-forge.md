# 455: The Stellar Forge

## 1. Overview

Players can construct a specialized orbital forge incredibly close to the system's star. It produces rare "Stellar Alloys" without fuel but operates under massive thermal stress. If not constantly supplied with coolant shipped from Layer 1, the forge melts down, creating a deadly solar flare that damages Layer 1 colonies.

**Fantasy:** Forging advanced materials using the raw heat of a close-orbiting star, risking the entire facility for the ultimate resource.
**Emergence:** You forget a coolant shipment due to a pirate raid. The forge melts down, and the resulting solar flare fries your colony's power grid just as the pirates land.
**Tension:** Free access to endgame materials vs. an incredibly fragile logistical chain that can cause a localized apocalypse if broken.

## 2. Dependencies

- `004` Basic Building (Building component)
- `018` Mining and Resources (Resource/Item systems)
- `023` Refining Industry (Crafting logic)
- `152` Orbital Stations (Orbital construction)

## 3. RED Phase: Tests First

```rust
// tests/integration/stellar_forge_tests.rs

use bevy::prelude::*;
use scale::layer2::orbital_stations::{OrbitalStation, OrbitalStationType};
use scale::layer1::items::{ItemType, Inventory};
use scale::layer2::stellar_forge::{
    StellarForge, ThermalStress, SolarFlareEvent,
    stellar_forge_production_system, thermal_stress_management_system
};

#[test]
fn test_stellar_forge_produces_without_fuel_when_cooled() {
    let mut app = App::new();
    app.add_systems(Update, stellar_forge_production_system);

    // Arrange: Forge with coolant
    let mut inventory = Inventory::new();
    inventory.add(ItemType::RawMass, 50); // Raw material
    inventory.add(ItemType::Coolant, 20); // Required to prevent stress/allow production

    let forge_entity = app.world_mut().spawn((
        OrbitalStation { station_type: OrbitalStationType::StellarForge },
        StellarForge { production_rate: 1.0, active: true },
        ThermalStress { current_stress: 0.0, max_stress: 100.0, stress_rate: 5.0 },
        inventory,
    )).id();

    // Act
    app.update();

    // Assert: RawMass converted to StellarAlloy, Coolant consumed
    let inventory_after = app.world().get::<Inventory>(forge_entity).unwrap();
    assert!(inventory_after.has_item(ItemType::StellarAlloy));
    assert!(inventory_after.get_count(ItemType::Coolant) < 20);
}

#[test]
fn test_stellar_forge_accumulates_stress_without_coolant() {
    let mut app = App::new();
    app.add_systems(Update, thermal_stress_management_system);

    // Arrange: Forge without coolant
    let forge_entity = app.world_mut().spawn((
        OrbitalStation { station_type: OrbitalStationType::StellarForge },
        StellarForge { production_rate: 1.0, active: true },
        ThermalStress { current_stress: 0.0, max_stress: 100.0, stress_rate: 10.0 },
        Inventory::new(), // No Coolant
    )).id();

    // Act
    app.update();

    // Assert: Stress increased
    let stress_after = app.world().get::<ThermalStress>(forge_entity).unwrap();
    assert!(stress_after.current_stress > 0.0, "Thermal stress should increase when no coolant is available");
}

#[test]
fn test_stellar_forge_meltdown_causes_solar_flare() {
    let mut app = App::new();
    app.add_event::<SolarFlareEvent>();
    app.add_systems(Update, thermal_stress_management_system);

    // Arrange: Forge at max stress
    let forge_entity = app.world_mut().spawn((
        OrbitalStation { station_type: OrbitalStationType::StellarForge },
        StellarForge { production_rate: 1.0, active: true },
        ThermalStress { current_stress: 100.0, max_stress: 100.0, stress_rate: 10.0 },
        Inventory::new(),
    )).id();

    // Act
    app.update();

    // Assert: SolarFlareEvent fired
    let events = app.world().resource::<Events<SolarFlareEvent>>();
    let mut reader = events.get_reader();
    let flare_events: Vec<_> = reader.read(events).collect();

    assert_eq!(flare_events.len(), 1, "SolarFlareEvent should trigger on meltdown");
    assert_eq!(flare_events[0].source_entity, forge_entity);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/stellar_forge.rs

use bevy::prelude::*;
use crate::layer1::items::{ItemType, Inventory};
use crate::layer2::orbital_stations::{OrbitalStation, OrbitalStationType};

#[derive(Component, Default)]
pub struct StellarForge {
    pub production_rate: f32,
    pub active: bool,
}

#[derive(Component, Default)]
pub struct ThermalStress {
    pub current_stress: f32,
    pub max_stress: f32,
    pub stress_rate: f32,
}

#[derive(Event)]
pub struct SolarFlareEvent {
    pub source_entity: Entity,
    pub severity: f32,
}

pub fn stellar_forge_production_system(
    mut query: Query<(&StellarForge, &mut Inventory)>,
) {
    for (forge, mut inventory) in query.iter_mut() {
        if forge.active {
            // Simplified recipe: 10 RawMass + 1 Coolant = 1 StellarAlloy
            if inventory.get_count(ItemType::RawMass) >= 10 && inventory.get_count(ItemType::Coolant) >= 1 {
                inventory.remove(ItemType::RawMass, 10);
                inventory.remove(ItemType::Coolant, 1);
                inventory.add(ItemType::StellarAlloy, 1);
            }
        }
    }
}

pub fn thermal_stress_management_system(
    mut commands: Commands,
    mut query: Query<(Entity, &StellarForge, &mut ThermalStress, &Inventory)>,
    mut flare_events: EventWriter<SolarFlareEvent>,
) {
    for (entity, forge, mut stress, inventory) in query.iter_mut() {
        if forge.active {
            // If we have coolant, we reduce stress. Otherwise, stress increases.
            if inventory.get_count(ItemType::Coolant) > 0 {
                // Coolant consumed during production handles cooling, but if not producing, maybe passive cooling?
                // For minimal, just keep stress at 0 if coolant present.
                stress.current_stress = (stress.current_stress - stress.stress_rate).max(0.0);
            } else {
                stress.current_stress += stress.stress_rate;
            }

            if stress.current_stress >= stress.max_stress {
                // Meltdown!
                flare_events.send(SolarFlareEvent {
                    source_entity: entity,
                    severity: stress.current_stress, // Or some defined scale
                });

                // Destroy the forge to prevent endless flares
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

// Add these systems and events to the app setup
```

## 5. REFACTOR Phase: Quality & Design

- **Recipe Integration:** Use the standard refining recipe system instead of hardcoding `10 RawMass + 1 Coolant` to create `StellarAlloy`.
- **System Layer Integration:** Propagate the `SolarFlareEvent` from Layer 2 to Layer 1, causing massive damage to power grids, communication arrays, and surface temperature.
- **Lore Events:** Emit a major chronicle event documenting the construction of the Stellar Forge and an existential threat event upon a meltdown.
- **Coolant Drain Rate:** Separate the coolant consumption for production from passive coolant usage required just to keep the forge active and cool.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes without warnings.
- [ ] Test coverage ≥85% for `src/layer2/stellar_forge.rs`.
- [ ] Stellar Forge produces `StellarAlloy` when supplied with `RawMass` and `Coolant`.
- [ ] Thermal stress increases rapidly when `Coolant` is missing.
- [ ] A `SolarFlareEvent` triggers when thermal stress reaches its maximum, destroying the forge.

## 7. Technical Guidance

- Create `src/layer2/stellar_forge.rs`.
- Ensure `OrbitalStationType::StellarForge` is added to `src/layer2/orbital_stations.rs`.
- Register the `SolarFlareEvent` in the Bevy app setup.
- Add `stellar_forge_production_system` and `thermal_stress_management_system` to the appropriate execution schedule.

## 8. Questions

*Builder: add questions here if spec is unclear.*
