# 486 - Debt-Prison Colonies

## Overview

A neighboring empire offers to pay off your immense debts if you accept their worst criminals, radically shifting the culture and crime rates of your colony.

When your colony is on the verge of bankruptcy, a powerful Layer 3 faction offers a bailout. The catch: they drop thousands of their most dangerous, highly-skilled criminal Pops onto your Layer 1 colony. These Pops have incredible stats but permanently high Unrest and dangerous traits.

The tension comes from the immediate financial salvation vs. importing a massive, highly-competent hostile faction into your own home.

## Dependencies

- `072 Justice System` (crime, arrests, unrest)
- `454 Gravitational Debt` (for debt mechanisms or general colony debt)
- `068 Pop Factions` (for managing the criminal faction)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, PopBundle, Traits};
    use crate::layer1::needs::Morale;
    use crate::layer1::factions::Faction;
    use crate::layer2::trade::blockade::ColonyDebt;
    use crate::layer3::diplomacy::Faction as L3Faction;

    #[test]
    fn test_bailout_offer_triggers_on_high_debt() {
        // Arrange
        let mut world = World::new();
        // Set colony debt to a critical level
        world.insert_resource(ColonyDebt(1000000.0));

        // Act
        // Run system that checks for debt bailout

        // Assert
        // Verify a BailoutOffer event or resource is generated
    }

    #[test]
    fn test_accept_bailout_clears_debt_and_spawns_criminals() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(ColonyDebt(1000000.0));
        // Setup initial population count

        // Act
        // Trigger acceptance of the bailout

        // Assert
        // Verify ColonyDebt is 0.0
        // Verify a large number of Pops with criminal traits and high stats are spawned
    }

    #[test]
    fn test_criminal_faction_formation() {
        // Arrange
        let mut world = World::new();
        // Spawn criminal pops

        // Act
        // Run faction update systems

        // Assert
        // Verify a criminal faction exists and the new pops belong to it
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer3/events/debt_prison.rs

use bevy::prelude::*;
use crate::layer2::trade::blockade::ColonyDebt;
use crate::layer1::pop::{PopBundle, Traits, Trait};

#[derive(Event)]
pub struct BailoutOfferEvent;

#[derive(Event)]
pub struct AcceptBailoutEvent;

pub fn check_bailout_condition_system(
    debt: Res<ColonyDebt>,
    mut bailout_events: EventWriter<BailoutOfferEvent>,
) {
    if debt.0 >= 1000000.0 {
        bailout_events.send(BailoutOfferEvent);
    }
}

pub fn process_bailout_acceptance_system(
    mut commands: Commands,
    mut events: EventReader<AcceptBailoutEvent>,
    mut debt: ResMut<ColonyDebt>,
) {
    for _ in events.read() {
        // Clear debt
        debt.0 = 0.0;

        // Spawn criminals
        for _ in 0..100 { // Arbitrary number for MVP
            commands.spawn((
                PopBundle {
                    // Setup high stats but dangerous traits
                    traits: Traits(vec![Trait::Rebellious, Trait::Violent]),
                    ..Default::default()
                },
            ));
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Pop Generation**: The criminals should have a specific distribution of high skills and negative traits to make them useful but dangerous. This should use the central procedural pop generation logic.
- **Faction Integration**: Ensure the spawned criminals automatically form or join a "Cartel" faction that exerts pressure on the colony.
- **UI Feedback**: Accepting the bailout should be a major, dramatic UI decision. The consequences should be clearly communicated.
- **Scaling**: The number of criminals spawned should scale with the size of the debt paid off.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Reaching a critical debt threshold triggers a bailout offer.
- [ ] Accepting the offer sets debt to zero and spawns high-skill, high-unrest Pops.

## Technical Guidance

- Integrate tightly with `ColonyDebt` from `src/layer2/trade/blockade.rs` (Spec 436).
- Use `Trait::Rebellious` and similar traits to ensure the new Pops naturally generate unrest and crime as part of the existing Justice system.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
