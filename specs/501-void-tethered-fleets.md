# 501 - Void-Tethered Fleets

## 1. Overview
Layer 2 ships powered directly by a Layer 1 Quantum Anchor offer immense firepower but tether your naval supremacy to local power grid stability. Instead of onboard reactors, players build a massive "Quantum Anchor" on Layer 1 that beams power to a specialized Layer 2 fleet. The fleet is devastatingly powerful and requires no fuel, but is restricted to the system. If the Anchor loses power or is destroyed, the entire fleet instantly shuts down in orbit, leaving it vulnerable to being picked apart by enemy forces.

## 2. Dependencies
- `158` Fleet Management
- `042` Energy System
- `159` Fleet Combat

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;
    // Assuming existence of these structures
    // use crate::layer1::energy::{PowerGrid, PowerConsumer};
    // use crate::layer2::fleet::{Fleet, FleetState};

    #[derive(Component)]
    struct QuantumAnchor {
        pub fleet_entity: Option<Entity>,
    }

    #[derive(Component)]
    struct PowerConsumer {
        pub powered: bool,
    }

    #[derive(Component)]
    struct Fleet {
        pub is_tethered: bool,
        pub active: bool,
    }

    fn check_tether_status_system(
        anchors: Query<(&QuantumAnchor, &PowerConsumer)>,
        mut fleets: Query<&mut Fleet>,
    ) {
        for (anchor, power) in anchors.iter() {
            if let Some(fleet_entity) = anchor.fleet_entity {
                if let Ok(mut fleet) = fleets.get_mut(fleet_entity) {
                    if fleet.is_tethered {
                        fleet.active = power.powered;
                    }
                }
            }
        }
    }

    #[test]
    fn test_tethered_fleet_active_when_anchor_powered() {
        let mut app = App::new();

        let fleet = app.world_mut().spawn(Fleet {
            is_tethered: true,
            active: false,
        }).id();

        app.world_mut().spawn((
            QuantumAnchor { fleet_entity: Some(fleet) },
            PowerConsumer { powered: true },
        ));

        app.add_systems(Update, check_tether_status_system);
        app.update();

        let fleet_data = app.world().get::<Fleet>(fleet).unwrap();
        assert!(fleet_data.active, "Tethered fleet should be active when the anchor is powered");
    }

    #[test]
    fn test_tethered_fleet_deactivates_when_anchor_loses_power() {
        let mut app = App::new();

        let fleet = app.world_mut().spawn(Fleet {
            is_tethered: true,
            active: true,
        }).id();

        app.world_mut().spawn((
            QuantumAnchor { fleet_entity: Some(fleet) },
            PowerConsumer { powered: false },
        ));

        app.add_systems(Update, check_tether_status_system);
        app.update();

        let fleet_data = app.world().get::<Fleet>(fleet).unwrap();
        assert!(!fleet_data.active, "Tethered fleet should deactivate when the anchor loses power");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct QuantumAnchor {
    pub fleet_entity: Option<Entity>,
}

// Assuming PowerConsumer is defined in the energy module
#[derive(Component)]
pub struct PowerConsumer {
    pub powered: bool,
    pub required_power: f32,
}

// Assuming Fleet is defined in the layer2 fleet module
#[derive(Component)]
pub struct Fleet {
    pub is_tethered: bool,
    pub active: bool,
    pub combat_power: f32, // Simplified metric
}

pub fn check_tether_status_system(
    anchors: Query<(&QuantumAnchor, &PowerConsumer)>,
    mut fleets: Query<&mut Fleet>,
) {
    for (anchor, power) in anchors.iter() {
        if let Some(fleet_entity) = anchor.fleet_entity {
            if let Ok(mut fleet) = fleets.get_mut(fleet_entity) {
                if fleet.is_tethered {
                    fleet.active = power.powered;
                    // If disabled, the fleet cannot attack or move
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Combat Resolution**: When a tethered fleet is `active == false`, its `combat_power` in combat calculations should effectively become 0, or it should automatically fail to evade incoming fire.
- **Resource Constraints**: The `QuantumAnchor` should have a massive `required_power` value, making it a significant drain on Layer 1 infrastructure.
- **System Boundaries**: Tethered fleets should be restricted from moving outside the system containing the `QuantumAnchor`. If a move order attempts to cross an interstellar boundary, it should be rejected.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Layer 2 `Fleet` entities linked to a `QuantumAnchor` sync their `active` state to the `powered` state of the anchor.
- [ ] The `check_tether_status_system` correctly evaluates power loss and updates the fleet state.

## 7. Technical Guidance
- `QuantumAnchor` needs a mechanism to target or bind to a specific `Fleet`. This could be handled during the construction phase of the fleet (e.g., "Build Tethered Dreadnought").
- This feature bridges Layer 1 and Layer 2. Be careful not to create circular dependencies between those modules. The system updating the fleet should probably run in the `Layer2SystemSet` but read from Layer 1 components.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
