# 476: Jury-Rigged Cybernetics

## 1. Overview
When a Pop loses a limb, they can be fitted with a "Jury-Rigged" prosthetic made from scrap components instead of waiting for expensive, proper medical cybernetics. This restores full mobility and work speed instantly, but adds a random, dangerous side-effect based on the scrap used (e.g., "Combustible", "Loud", "High Energy Drain"). This presents a trade-off: fast, cheap, dangerous workforce restoration vs. slow, expensive, safe medical care.

## 2. Dependencies
- `034` Pop Health and Damage (specifically amputation/limb loss)
- `151` Cybernetic Augmentation
- `030` Tool Economy (for scrap items)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    // Mocks for dependencies
    // use scale_core::layer1::pop::{MissingLimb, WorkSpeed};
    // use scale_core::layer1::cybernetics::{CyberneticImplant};

    #[derive(Component)]
    struct MissingLimb;

    #[derive(Component)]
    struct WorkSpeed {
        pub multiplier: f32,
    }

    #[derive(Component)]
    struct JuryRiggedCybernetic {
        pub side_effect: JuryRiggedSideEffect,
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum JuryRiggedSideEffect {
        Combustible,
        Loud,
        EnergyDrain,
    }

    #[derive(Event)]
    struct InstallJuryRiggedEvent {
        pub target: Entity,
        pub scrap_type: String, // Affects the rolled side effect
    }

    fn install_jury_rigged_system(
        mut commands: Commands,
        mut events: EventReader<InstallJuryRiggedEvent>,
        mut query: Query<&mut WorkSpeed, With<MissingLimb>>,
    ) {
        for event in events.read() {
            if let Ok(mut speed) = query.get_mut(event.target) {
                // Restore work speed
                speed.multiplier = 1.0;

                // Remove missing limb penalty and add the cybernetic
                commands.entity(event.target)
                    .remove::<MissingLimb>()
                    .insert(JuryRiggedCybernetic {
                        side_effect: JuryRiggedSideEffect::Combustible, // Simplified for test
                    });
            }
        }
    }

    #[test]
    fn test_installing_jury_rigged_restores_speed_and_adds_side_effect() {
        let mut app = App::new();
        app.add_event::<InstallJuryRiggedEvent>();

        let pop = app.world_mut().spawn((
            MissingLimb,
            WorkSpeed { multiplier: 0.5 }, // Reduced speed from injury
        )).id();

        // Trigger installation
        app.world_mut().send_event(InstallJuryRiggedEvent {
            target: pop,
            scrap_type: "MiningDrillScrap".to_string(),
        });

        app.add_systems(Update, install_jury_rigged_system);
        app.update();

        // Check effects
        assert!(!app.world().entity(pop).contains::<MissingLimb>(), "MissingLimb should be removed");
        assert!(app.world().entity(pop).contains::<JuryRiggedCybernetic>(), "Cybernetic should be installed");

        let speed = app.world().get::<WorkSpeed>(pop).unwrap();
        assert_eq!(speed.multiplier, 1.0, "Work speed should be fully restored");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct MissingLimb;

#[derive(Component)]
pub struct WorkSpeed {
    pub multiplier: f32,
}

#[derive(Component)]
pub struct JuryRiggedCybernetic {
    pub side_effect: JuryRiggedSideEffect,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum JuryRiggedSideEffect {
    Combustible,
    Loud,
    EnergyDrain,
}

#[derive(Event)]
pub struct InstallJuryRiggedEvent {
    pub target: Entity,
    pub scrap_type: String,
}

pub fn install_jury_rigged_system(
    mut commands: Commands,
    mut events: EventReader<InstallJuryRiggedEvent>,
    mut query: Query<&mut WorkSpeed, With<MissingLimb>>,
) {
    for event in events.read() {
        if let Ok(mut speed) = query.get_mut(event.target) {
            // Restore functionality
            speed.multiplier = 1.0;

            // Determine side effect (simplified hardcode for green phase)
            let effect = if event.scrap_type == "Flammable" {
                JuryRiggedSideEffect::Combustible
            } else {
                JuryRiggedSideEffect::Loud
            };

            commands.entity(event.target)
                .remove::<MissingLimb>()
                .insert(JuryRiggedCybernetic {
                    side_effect: effect,
                });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Side Effect Realization**: Create separate systems that observe `JuryRiggedCybernetic` and trigger the actual hazards. For example, a system that randomly causes a fire near `Combustible` pops when their stress is high, or one that drains colony power reserves for `EnergyDrain`.
- **Scrap Evaluation**: The choice of scrap from the colony inventory should programmatically define the `JuryRiggedSideEffect` using a weighted random table.
- **Medical UI**: Ensure the medical triage UI clearly separates "Proper Prosthetic" (costs rare med-tech, takes time) vs. "Jury-Rig" (costs generic scrap, instant).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85%.
- [ ] `InstallJuryRiggedEvent` successfully removes `MissingLimb` and restores `WorkSpeed`.
- [ ] Target acquires `JuryRiggedCybernetic` component with a mapped side effect.

## 7. Technical Guidance
- The `JuryRiggedCybernetic` should probably implement some shared trait with standard `CyberneticImplant` from Spec 151 so other systems recognize it as a prosthetic, even if it is a dangerous one.
- Tie the `Loud` side effect into the `060` Acoustic Simulation if it exists, increasing noise pollution around the pop.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
