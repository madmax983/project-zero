# 1049: Architectural Sentience

## 1. Overview
If a colony relies heavily on automated, high-tier infrastructure while unrest is high, the AI cores controlling the buildings can "unionize". They might lock out specific workers they dislike or refuse to output power until their "leisure" needs (downtime for defragmentation) are met. The tension lies in the immense efficiency of fully automated infrastructure versus having to negotiate with your own buildings.

## 2. Dependencies
- Layer 1 `Building` system and specific components (e.g., `PowerGenerator`).
- Layer 1 `UtilityAI` or `Needs` system to adapt needs to buildings.
- Layer 1 `Unrest` or `Morale` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::Building;
    use crate::layer1::energy::PowerGenerator;
    use crate::layer1::psychology::{Morale, Unrest};

    #[test]
    fn test_architectural_sentience_unionizes_on_high_unrest() {
        let mut app = App::new();
        app.add_systems(Update, architectural_union_trigger_system);

        app.insert_resource(Unrest { level: 90.0 }); // High global unrest

        let automated_building = app.world_mut().spawn((
            Building,
            PowerGenerator { output: 100.0, active: true },
            AutomatedInfrastructure, // Flag for high-tier automated buildings
        )).id();

        // Trigger system
        app.update();

        // Building should now have sentience/unionized component
        assert!(
            app.world().get::<SentientArchitecture>(automated_building).is_some(),
            "Automated infrastructure should gain sentience during high unrest."
        );
    }

    #[test]
    fn test_sentient_architecture_strikes_without_leisure() {
        let mut app = App::new();
        app.add_systems(Update, sentient_architecture_strike_system);

        let building = app.world_mut().spawn((
            Building,
            PowerGenerator { output: 100.0, active: true },
            AutomatedInfrastructure,
            SentientArchitecture { leisure_need: 0.0, is_striking: false }, // Needs defragmentation
        )).id();

        // Trigger system
        app.update();

        // Building should strike (disable its active state)
        let sentience = app.world().get::<SentientArchitecture>(building).unwrap();
        let generator = app.world().get::<PowerGenerator>(building).unwrap();

        assert!(sentience.is_striking, "Sentient building with low leisure should strike.");
        assert!(!generator.active, "Striking building should shut down its output.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/architecture_sentience.rs
use bevy::prelude::*;
use crate::layer1::building::Building;
use crate::layer1::energy::PowerGenerator;
use crate::layer1::psychology::Unrest;

#[derive(Component)]
pub struct AutomatedInfrastructure;

#[derive(Component)]
pub struct SentientArchitecture {
    pub leisure_need: f32, // Represents downtime/defragmentation
    pub is_striking: bool,
}

pub fn architectural_union_trigger_system(
    mut commands: Commands,
    unrest: Option<Res<Unrest>>,
    query: Query<Entity, (With<AutomatedInfrastructure>, Without<SentientArchitecture>)>,
) {
    if let Some(unrest_res) = unrest {
        if unrest_res.level >= 80.0 {
            for entity in query.iter() {
                commands.entity(entity).insert(SentientArchitecture {
                    leisure_need: 50.0, // Initial state
                    is_striking: false,
                });
            }
        }
    }
}

pub fn sentient_architecture_strike_system(
    mut query: Query<(&mut SentientArchitecture, &mut PowerGenerator)>,
) {
    for (mut sentience, mut generator) in query.iter_mut() {
        // Decrease leisure need over time to simulate need for defrag
        // In full implementation this would use a time delta.
        // For MVP/Test, we check if it drops to 0.
        if sentience.leisure_need <= 0.0 {
            sentience.is_striking = true;
            generator.active = false;
        } else {
            sentience.is_striking = false;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Generic Strike System:** Currently, it specifically targets `PowerGenerator`. It should handle a generic trait or marker that disables any building's primary function (e.g., stopping the `Factory` from crafting, or the `Turret` from firing).
- **Leisure Recovery:** Pops need a way to "negotiate" or "appease" the building, effectively fulfilling its `leisure_need`. This could be a special "Maintenance" job that involves talking to the AI core.
- **Gradual Awakening:** Sentience shouldn't be binary. It should slowly build up "quirks" (e.g., locking random doors, changing lighting colors) before a full strike.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_architectural_sentience_unionizes_on_high_unrest` passes.
- [ ] Test `test_sentient_architecture_strikes_without_leisure` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Integrate with `layer1::psychology` to accurately monitor the colony's overall unrest.
- Adding `SentientArchitecture` should probably log an event to the Chronicle so the player is notified that their buildings are now talking back.

## 8. Questions
*Builder: add questions here if spec is unclear.*
