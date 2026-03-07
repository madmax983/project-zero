# 425: Stellar Wind Surfing

## 1. Overview
The "Stellar Wind Surfing" feature allows players to harness "Solar Flare" events on the System Map (Layer 2) to rapidly travel between nodes without spending fuel. By equipping ships with "Solar Sails", fleets can catch the cosmic wave for an instant arrival. However, this high-risk maneuver subjects the ship and crew to intense radiation if not adequately shielded, forcing a strategic trade-off between speed, cost, and safety.

## 2. Dependencies
- `src/layer2/fleet.rs`: Fleet movement and fuel consumption mechanics.
- `src/layer2/events.rs`: Solar Flare events mapping.
- `src/layer1/health.rs`: Applying radiation damage or traits to pops/crews.
- `src/shared/math.rs`: Distance and pathing calculations.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, ShipConfig, SolarSail};
    use crate::layer2::events::SolarFlareEvent;
    use crate::layer1::health::RadiationExposure;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<SolarFlareEvent>>();
        world
    }

    #[test]
    fn test_solar_sail_instant_travel() {
        let mut world = setup_world();

        let fleet = world.spawn((
            Fleet {
                current_node: 1,
                destination_node: 5,
                fuel: 100.0,
            },
            SolarSail { efficiency: 1.0 },
            ShipConfig { shielding: 50.0 },
            RadiationExposure { level: 0.0 },
        )).id();

        // Trigger a solar flare affecting the route
        world.resource_mut::<Events<SolarFlareEvent>>().send(SolarFlareEvent {
            origin: 1,
            target: 5,
            intensity: 100.0,
        });

        // Run movement system
        update_fleet_movement_system(&mut world);

        let updated_fleet = world.get::<Fleet>(fleet).unwrap();

        // Instant travel => destination reached
        assert_eq!(updated_fleet.current_node, 5);
        // Free travel => no fuel consumed
        assert_eq!(updated_fleet.fuel, 100.0);
    }

    #[test]
    fn test_solar_sail_radiation_damage() {
        let mut world = setup_world();

        let fleet = world.spawn((
            Fleet {
                current_node: 1,
                destination_node: 2,
                fuel: 50.0,
            },
            SolarSail { efficiency: 1.0 },
            ShipConfig { shielding: 10.0 }, // Low shielding
            RadiationExposure { level: 0.0 },
        )).id();

        world.resource_mut::<Events<SolarFlareEvent>>().send(SolarFlareEvent {
            origin: 1,
            target: 2,
            intensity: 100.0, // High intensity
        });

        update_fleet_movement_system(&mut world);

        let exposure = world.get::<RadiationExposure>(fleet).unwrap();
        // Radiation = intensity - shielding = 90.0
        assert_eq!(exposure.level, 90.0, "Crew should suffer heavy radiation due to low shielding");
    }

    #[test]
    fn test_normal_travel_consumes_fuel() {
        let mut world = setup_world();

        let fleet = world.spawn((
            Fleet {
                current_node: 1,
                destination_node: 2,
                fuel: 50.0,
            },
            ShipConfig { shielding: 50.0 },
            RadiationExposure { level: 0.0 },
        )).id(); // No SolarSail

        // No flare event

        update_fleet_movement_system(&mut world);

        let updated_fleet = world.get::<Fleet>(fleet).unwrap();
        assert!(updated_fleet.fuel < 50.0, "Normal movement must consume fuel");
        assert_ne!(updated_fleet.current_node, 2, "Normal movement is not instant");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer2::fleet::{Fleet, ShipConfig, SolarSail};
use crate::layer2::events::SolarFlareEvent;
use crate::layer1::health::RadiationExposure;

pub fn update_fleet_movement_system(world: &mut World) {
    let mut flare_events = world.resource_mut::<Events<SolarFlareEvent>>();
    let mut current_flares = Vec::new();
    for event in flare_events.drain() {
        current_flares.push(event);
    }

    let mut query = world.query::<(
        Entity,
        &mut Fleet,
        Option<&SolarSail>,
        &ShipConfig,
        &mut RadiationExposure
    )>();

    for (_entity, mut fleet, sail_opt, config, mut radiation) in query.iter_mut(world) {
        if fleet.current_node == fleet.destination_node {
            continue;
        }

        let mut used_flare = false;

        if let Some(sail) = sail_opt {
            for flare in &current_flares {
                if flare.origin == fleet.current_node && flare.target == fleet.destination_node {
                    // Instant travel
                    fleet.current_node = fleet.destination_node;
                    used_flare = true;

                    // Apply radiation
                    let damage = flare.intensity - config.shielding;
                    if damage > 0.0 {
                        radiation.level += damage * sail.efficiency;
                    }
                    break;
                }
            }
        }

        if !used_flare {
            // Normal slow movement logic placeholder
            fleet.fuel -= 1.0;
            // In a real implementation, progress towards destination is updated based on speed
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Reading:** Refactor `update_fleet_movement_system` to use an `EventReader<SolarFlareEvent>` instead of draining the event queue so other systems can also react to the flare.
- **Components:** Extract `SolarSail` into an optional component so standard fleets don't need to carry empty data.
- **Radiation Scaling:** The radiation damage formula `flare.intensity - config.shielding` is basic; consider changing it to a percentage mitigation (e.g., `intensity * (1.0 - shielding_percentage)`) for better late-game scaling.

## 6. Acceptance Criteria (Testable!)
- [ ] RED phase tests compile and pass.
- [ ] Solar sailing allows instant movement to the destination node.
- [ ] Solar sailing consumes zero fuel.
- [ ] Radiation exposure is applied correctly based on flare intensity minus shielding.
- [ ] Normal movement behavior remains unaffected.
- [ ] Code has >=85% test coverage.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- **Scheduling:** Ensure `update_fleet_movement_system` runs after `SolarFlareEvent` emission.
- **UI Indication:** Consider adding a `SolarSailingStatus` component to flag the fleet for UI feedback (e.g., "Riding Flare").
- **Edge Cases:** Handle fleets trying to sail *against* a solar flare (it shouldn't work, or it should destroy them).

## 8. Questions
*Builder: add questions here if spec is unclear.*
