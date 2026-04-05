# 799: The Synthetic Prophet

## Overview

A low-level AI module (like an automated sorting system or a traffic controller) begins to glitch, outputting strange, highly poetic, but nonsensical predictions instead of standard error logs. Stressed pops begin reading these logs and interpreting them as divine prophecy, forming a cult. The cult obeys the machine's random outputs over direct orders, creating tension between destroying a slightly buggy machine (and risking a massive holy war) or leaving it running (and potentially triggering a colony-wide crisis based on an error code).

## Dependencies

- `210` — Pop factions and belief systems (must exist for the cult to form)
- `145` — Structure components/AI modules (must exist for the glitch to occur)

## RED Phase: Tests First

```rust
// tests/layer1/synthetic_prophet.rs

use scale::layer1::ai::{AIModule, SyntheticProphecyEvent};
use scale::layer1::pops::{Pop, Stress, BeliefSystem, CultMembership};
use bevy::prelude::*;

#[test]
fn test_stressed_pops_join_synthetic_cult() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, scale::layer1::cult::process_synthetic_prophecies);

    // Spawn a glitched AI module
    let ai_entity = app.world.spawn((
        AIModule { is_glitched: true },
    )).id();

    // Spawn a highly stressed pop
    let pop_entity = app.world.spawn((
        Pop,
        Stress { level: 90 },
        BeliefSystem::default(),
    )).id();

    app.world.insert_resource(Events::<SyntheticProphecyEvent>::default());

    // Act
    app.world.resource_mut::<Events<SyntheticProphecyEvent>>().send(
        SyntheticProphecyEvent {
            source_ai: ai_entity,
            message: "RED PROTOCOL ENGAGED. THE END IS NIGH.".to_string(),
        }
    );
    app.update();

    // Assert
    let membership = app.world.get::<CultMembership>(pop_entity);
    assert!(membership.is_some(), "Highly stressed pops should join the cult upon hearing prophecies");
    assert_eq!(membership.unwrap().cult_leader, ai_entity);
}

#[test]
fn test_cult_interprets_random_color_as_command() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, scale::layer1::cult::execute_cult_commands);

    let ai_entity = app.world.spawn(AIModule { is_glitched: true }).id();

    let pop_entity = app.world.spawn((
        Pop,
        CultMembership { cult_leader: ai_entity },
        scale::layer1::utility_ai::CurrentAction::default(),
    )).id();

    app.world.insert_resource(Events::<SyntheticProphecyEvent>::default());

    // Act
    app.world.resource_mut::<Events<SyntheticProphecyEvent>>().send(
        SyntheticProphecyEvent {
            source_ai: ai_entity,
            message: "RED".to_string(), // Interpreted as 'Attack' or 'Riot'
        }
    );
    app.update();

    // Assert
    let action = app.world.get::<scale::layer1::utility_ai::CurrentAction>(pop_entity).unwrap();
    assert_eq!(action.action_type, scale::layer1::utility_ai::ActionType::Riot, "Cult should interpret the RED prophecy as a command to riot");
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/cult.rs

use bevy::prelude::*;
use crate::layer1::pops::{Pop, Stress};
use crate::layer1::utility_ai::{CurrentAction, ActionType};

#[derive(Component)]
pub struct CultMembership {
    pub cult_leader: Entity,
}

#[derive(Event, Debug)]
pub struct SyntheticProphecyEvent {
    pub source_ai: Entity,
    pub message: String,
}

pub fn process_synthetic_prophecies(
    mut commands: Commands,
    mut events: EventReader<SyntheticProphecyEvent>,
    pops: Query<(Entity, &Stress), Without<CultMembership>>,
) {
    for event in events.read() {
        for (pop_entity, stress) in pops.iter() {
            // Highly stressed pops are susceptible to the prophecies
            if stress.level > 80 {
                commands.entity(pop_entity).insert(CultMembership {
                    cult_leader: event.source_ai,
                });
            }
        }
    }
}

pub fn execute_cult_commands(
    mut events: EventReader<SyntheticProphecyEvent>,
    mut cultists: Query<(&CultMembership, &mut CurrentAction)>,
) {
    for event in events.read() {
        if event.message.contains("RED") {
            for (membership, mut action) in cultists.iter_mut() {
                if membership.cult_leader == event.source_ai {
                    action.action_type = ActionType::Riot;
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Message Parsing**: Hardcoding "RED" is brittle. The prophecy string parsing should be connected to the procedural generation (Lore System) to derive meaning from generated text.
- **Cult Dynamics**: Cult members should try to spread the prophecy to other, less-stressed pops.
- **Destruction Consequence**: Add an event hook for when the `AIModule` is destroyed, triggering an immediate and violent response from its followers.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Stressed pops can join a cult led by a machine, and alter their behavior based on random machine outputs.

## Technical Guidance

- Place the systems in `src/layer1/cult.rs` or `src/layer1/pops/beliefs.rs` depending on existing file structure.
- Register `SyntheticProphecyEvent` and add the `process_synthetic_prophecies` and `execute_cult_commands` systems to the simulation tick.

## Questions
*Builder: add questions here if spec is unclear.*
