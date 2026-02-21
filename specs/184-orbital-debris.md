# 184: Orbital Debris

## Overview

As the colony expands into space (Layer 2), every launch, destroyed ship, and orbital battle leaves behind debris. This "space junk" accumulates in orbit, increasing the risk of future launches failing and damaging ships stationed in orbit.

This mechanic introduces a "Kessler Syndrome" risk, forcing players to balance rapid expansion with orbital hygiene.

## Dependencies

- `094` — System View Architecture (for `SystemBody` entities)
- `105` — Launch Logistics (for `LaunchEvent`)
- `159` — Fleet Combat Resolution (for `ShipDestroyedEvent`)

## RED Phase: Tests First

Write these tests in `src/layer2/debris_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::system::{SystemBody, OrbitalDebris};
    use crate::layer2::events::{LaunchEvent, ShipDestroyedEvent};
    use crate::layer2::fleet::{Fleet, InOrbit, FleetHealth};
    use crate::layer2::debris::{debris_accumulation_system, debris_attrition_system, DebrisConfig};

    #[test]
    fn test_debris_accumulation_on_launch() {
        let mut world = World::new();
        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 0.0 })).id();

        // Trigger Launch Event
        world.send_event(LaunchEvent { planet, success: true });

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(debris_accumulation_system);
        schedule.run(&mut world);

        // Verify Debris Increase
        let debris = world.get::<OrbitalDebris>(planet).unwrap();
        assert!(debris.amount > 0.0);
    }

    #[test]
    fn test_debris_accumulation_on_ship_destruction() {
        let mut world = World::new();
        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 0.0 })).id();

        // Trigger Destruction Event
        world.send_event(ShipDestroyedEvent { planet, ship_class: "Frigate".to_string() });

        let mut schedule = Schedule::default();
        schedule.add_systems(debris_accumulation_system);
        schedule.run(&mut world);

        let debris = world.get::<OrbitalDebris>(planet).unwrap();
        // Should be higher than launch debris
        assert!(debris.amount >= 0.1);
    }

    #[test]
    fn test_orbit_attrition_damage() {
        let mut world = World::new();
        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 0.5 })).id(); // High debris

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetHealth { current: 100.0, max: 100.0 }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(debris_attrition_system);
        schedule.run(&mut world);

        let health = world.get::<FleetHealth>(fleet).unwrap();
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_launch_failure_risk_from_debris() {
        // This requires mocking RNG or checking probability calculation function directly
        let risk = crate::layer2::debris::calculate_launch_risk(0.8); // 80% debris
        assert!(risk > 0.5); // High risk

        let risk_low = crate::layer2::debris::calculate_launch_risk(0.0);
        assert_eq!(risk_low, 0.0);
    }

    #[test]
    fn test_debris_cleanup_action() {
        let mut world = World::new();
        let planet = world.spawn((SystemBody, OrbitalDebris { amount: 0.5 })).id();

        // Assume a Cleanup Action system exists or simulate it
        crate::layer2::debris::perform_cleanup(&mut world, planet, 0.2);

        let debris = world.get::<OrbitalDebris>(planet).unwrap();
        assert!((debris.amount - 0.3).abs() < 0.001);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Component

In `src/layer2/debris.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct OrbitalDebris {
    pub amount: f32, // 0.0 to 1.0 (or higher for extreme danger)
}
```

### 2. Implement Accumulation System

```rust
use crate::layer2::events::{LaunchEvent, ShipDestroyedEvent};

pub fn debris_accumulation_system(
    mut events_launch: EventReader<LaunchEvent>,
    mut events_destroy: EventReader<ShipDestroyedEvent>,
    mut query: Query<&mut OrbitalDebris>,
) {
    for event in events_launch.read() {
        if let Ok(mut debris) = query.get_mut(event.planet) {
            debris.amount += 0.05; // Tunable constant
        }
    }

    for event in events_destroy.read() {
        if let Ok(mut debris) = query.get_mut(event.planet) {
            debris.amount += 0.20; // Much higher
        }
    }
}
```

### 3. Implement Attrition System

```rust
use crate::layer2::fleet::{Fleet, InOrbit, FleetHealth};

pub fn debris_attrition_system(
    planet_query: Query<&OrbitalDebris>,
    mut fleet_query: Query<(&InOrbit, &mut FleetHealth), With<Fleet>>,
) {
    for (orbit, mut health) in fleet_query.iter_mut() {
        if let Ok(debris) = planet_query.get(orbit.parent) {
            if debris.amount > 0.1 {
                // Damage based on debris amount
                let damage = debris.amount * 5.0;
                health.current = (health.current - damage).max(0.0);
            }
        }
    }
}
```

### 4. Helper Functions

```rust
pub fn calculate_launch_risk(debris_amount: f32) -> f32 {
    // Sigmoid or linear scaling
    (debris_amount * 0.8).min(0.9)
}

pub fn perform_cleanup(world: &mut World, planet: Entity, amount: f32) {
    if let Some(mut debris) = world.get_mut::<OrbitalDebris>(planet) {
        debris.amount = (debris.amount - amount).max(0.0);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Decay**: Add a system that slowly reduces debris over time (atmospheric drag), especially for low-orbit debris.
- **Visuals**: Render debris density on the System Map (Layer 2) as a particle cloud or ring opacity.
- **Tech**: Unlock "Shielded Launch Thrusters" to reduce launch risk.
- **Events**: Ensure `LaunchEvent` is emitted by Layer 1 launch logic.

## Acceptance Criteria

- [ ] `OrbitalDebris` component added to planets.
- [ ] Debris increases on Launch and Ship Destruction.
- [ ] Ships in orbit take damage from high debris.
- [ ] Launch risk calculation scales with debris.
- [ ] Cleanup mechanism exists and works.
- [ ] Tests pass.

## Questions

- *Builder: Should debris eventually form a ring system if it gets high enough?* (Visual flavor)
- *Builder: Does debris affect incoming trade ships?* (Yes, they should take damage too)
