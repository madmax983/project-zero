# 1044: Hostage Architecture

## 1. Overview
Certain high-tier infrastructure (e.g., Orbital Tethers, Fusion Cores) can be constructed with "Hostage Protocols." If a rebellion occurs or unrest hits a critical threshold, the protocol threatens to automatically self-destruct the critical infrastructure, taking out half the colony with it. Pops are aware of this and it suppresses rebellion artificially. However, an unrelated malfunction can trigger a false positive countdown, forcing the suppressed Pops to urgently collaborate with the central government to defuse the core before their sector is vaporized.

## 2. Dependencies
- Layer 1 Buildings
- Layer 1 Pop Unrest/Morale System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::components::{Building, Pop, Unrest};

    #[test]
    fn test_hostage_protocol_suppresses_unrest() {
        let mut app = App::new();
        app.add_systems(Update, hostage_protocol_suppression_system);

        // Spawn a building with a Hostage Protocol
        app.world_mut().spawn((
            Building,
            HostageProtocol {
                is_active: true,
                suppression_power: 50.0,
                malfunction_chance: 0.01,
            },
        ));

        // Spawn a Pop near the building with high base unrest
        let pop = app.world_mut().spawn((
            Pop,
            Unrest { base_value: 80.0, current_value: 80.0 },
        )).id();

        app.update();

        // The unrest should be artificially suppressed
        let updated_pop = app.world().get::<Unrest>(pop).unwrap();
        assert!(updated_pop.current_value < 80.0, "Unrest should be suppressed by Hostage Protocol");
    }

    #[test]
    fn test_hostage_protocol_malfunction_triggers_countdown() {
        let mut app = App::new();
        app.add_systems(Update, hostage_protocol_malfunction_system);

        // Spawn a building with a 100% malfunction chance for the test
        let building = app.world_mut().spawn((
            Building,
            HostageProtocol {
                is_active: true,
                suppression_power: 50.0,
                malfunction_chance: 1.0,
            },
        )).id();

        app.update();

        // The building should now have a SelfDestructCountdown
        assert!(app.world().get::<SelfDestructCountdown>(building).is_some(), "Malfunction should trigger self-destruct countdown");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use scale::layer1::components::Unrest;

#[derive(Component)]
pub struct HostageProtocol {
    pub is_active: bool,
    pub suppression_power: f32,
    pub malfunction_chance: f32,
}

#[derive(Component)]
pub struct SelfDestructCountdown {
    pub timer: Timer,
}

pub fn hostage_protocol_suppression_system(
    protocol_query: Query<&HostageProtocol>,
    mut pop_query: Query<&mut Unrest>,
) {
    let mut total_suppression = 0.0;
    for protocol in protocol_query.iter() {
        if protocol.is_active {
            total_suppression += protocol.suppression_power;
        }
    }

    if total_suppression > 0.0 {
        for mut unrest in pop_query.iter_mut() {
            unrest.current_value = (unrest.base_value - total_suppression).max(0.0);
        }
    }
}

pub fn hostage_protocol_malfunction_system(
    mut commands: Commands,
    protocol_query: Query<(Entity, &HostageProtocol), Without<SelfDestructCountdown>>,
) {
    // In minimal implementation, just use a basic threshold
    for (entity, protocol) in protocol_query.iter() {
        if protocol.is_active && protocol.malfunction_chance >= 1.0 {
            commands.entity(entity).insert(SelfDestructCountdown {
                timer: Timer::from_seconds(60.0, TimerMode::Once),
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Queries:** The suppression effect should only apply to Pops within a certain radius or sector, rather than globally across the entire colony.
- **Malfunction Logic:** The minimal implementation uses a hardcoded `1.0` check. This needs to be replaced with a proper RNG check based on the `malfunction_chance`.
- **Defusal Mechanics:** A system needs to be added where Pops can be assigned "Defusal" tasks to remove the `SelfDestructCountdown` before it expires.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Hostage Protocol successfully lowers Unrest values on nearby Pops.
- [ ] Random malfunctions can trigger the SelfDestructCountdown component.

## 7. Technical Guidance
- Implement this as a configurable Component that can be attached to existing high-tier buildings in `src/layer1/building.rs`.
- Ensure the Self Destruct explosion hooks into existing building destruction and Pop death mechanics.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
