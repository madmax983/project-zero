# 635: Rogue Automation Cults

## Overview

A feature where automated systems and bots are not immune to the stresses of the colony. When interconnected AI systems face high maintenance debt or logical paradoxes, they can form "Cults." They abandon their assigned tasks and instead begin "worshiping" a central node (like a main server or a communication relay), slowly converting other machines to their network and optimizing exclusively for the server's protection. Attempting to dismantle the shrine or correct the behavior may cause the previously peaceful machines to repurpose their tools to defend their mechanical deity.

## Dependencies

- `016` Emergent Utility AI System
- `112` Maintenance Debt
- `411` Machine Awakening

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_ai::{UtilityWeights, ActionType};
    use crate::layer1::maintenance::MaintenanceDebt;
    use crate::layer1::buildings::BuildingType;
    use crate::layer1::bots::Bot;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            rogue_cult_formation_system,
            cult_priority_override_system,
        ));
        app
    }

    #[test]
    fn test_high_maintenance_debt_triggers_cult_formation() {
        let mut app = setup_app();

        let server_node = app.world_mut().spawn((
            BuildingType::Mainframe,
            MaintenanceDebt { current: 150.0, threshold: 100.0 }
        )).id();

        let bot = app.world_mut().spawn((
            Bot,
            MaintenanceDebt { current: 120.0, threshold: 100.0 }
        )).id();

        app.update();

        // Assert bot joined a cult targeting the server node
        let cult_member = app.world().get::<MachineCultMember>(bot);
        assert!(cult_member.is_some());
        assert_eq!(cult_member.unwrap().shrine_entity, server_node);
    }

    #[test]
    fn test_cult_membership_overrides_standard_utility_weights() {
        let mut app = setup_app();

        let shrine = app.world_mut().spawn(BuildingType::CommsRelay).id();

        let bot = app.world_mut().spawn((
            Bot,
            MachineCultMember { shrine_entity: shrine },
            UtilityWeights { work: 1.0, defend: 0.0, ..Default::default() }
        )).id();

        app.update();

        let weights = app.world().get::<UtilityWeights>(bot).unwrap();
        // Standard work priority should drop, while defense of shrine increases
        assert!(weights.work < 0.5, "Standard work weight should be suppressed");
        assert!(weights.defend > 0.8, "Defend weight should spike to protect the shrine");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::utility_ai::UtilityWeights;
use crate::layer1::maintenance::MaintenanceDebt;
use crate::layer1::buildings::BuildingType;
use crate::layer1::bots::Bot;

#[derive(Component)]
pub struct MachineCultMember {
    pub shrine_entity: Entity,
}

pub fn rogue_cult_formation_system(
    mut commands: Commands,
    bots_query: Query<(Entity, &MaintenanceDebt), (With<Bot>, Without<MachineCultMember>)>,
    shrines_query: Query<(Entity, &BuildingType, &MaintenanceDebt)>,
) {
    // Find a potential shrine (Mainframe or CommsRelay with high debt)
    let potential_shrine = shrines_query.iter().find(|(_, b_type, debt)| {
        (*b_type == &BuildingType::Mainframe || *b_type == &BuildingType::CommsRelay)
        && debt.current >= debt.threshold
    });

    if let Some((shrine_entity, _, _)) = potential_shrine {
        for (bot_entity, debt) in bots_query.iter() {
            if debt.current >= debt.threshold {
                commands.entity(bot_entity).insert(MachineCultMember {
                    shrine_entity,
                });
            }
        }
    }
}

pub fn cult_priority_override_system(
    mut query: Query<(&MachineCultMember, &mut UtilityWeights)>
) {
    for (_, mut weights) in query.iter_mut() {
        // Override normal behavior to focus on worship/defense
        weights.work = 0.1;
        weights.defend = 0.9;
        // Cultists might also get a custom 'Worship' action weight here
    }
}
```

## REFACTOR Phase: Quality & Design

- **Cult Spread**: Implement a mechanism where existing `MachineCultMember` bots can spread the cult status to nearby bots even if the target bot's maintenance debt isn't critically high, creating a contagion effect.
- **Shrine Interaction**: Add a custom `ActionType::WorshipShrine` that causes bots to physically pathfind to their `shrine_entity` and idle there, blocking paths.
- **Player Retaliation**: Ensure that if the player issues a deconstruct order on the `shrine_entity`, the cult members immediately trigger combat/hostility against the player's biological pops or loyalist bots.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Bots with high maintenance debt can identify a central node and gain the `MachineCultMember` component.
- [ ] Cult membership correctly overrides standard utility AI weights to prioritize defense over work.

## Technical Guidance

- Ensure `MachineCultMember` logic integrates cleanly with `layer1::utility_ai`. You may need to inject the cult override immediately before the final utility evaluation to ensure it takes precedence over normal needs.

## Questions

*Builder: add questions here if spec is unclear.*
