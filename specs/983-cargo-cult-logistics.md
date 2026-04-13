# Spec 983: Cargo Cult Logistics

## 1. Overview
"Cargo Cult Logistics" explores the concept of doing the right thing for the wrong reason. When a random positive event like a supply drop happens to coincide with a Pop's arbitrary action, they might mistakenly associate the two. This leads Pops to repeatedly perform that action (like "dancing" or "rituals") near landing pads in the hope of summoning more supplies. While this superstitious behavior slows down their overall work efficiency, it significantly boosts their morale through a sense of ritual comfort.

## 2. Dependencies
- Layer 1 Pop Actions/Needs simulation (`needs.rs`, `action.rs`)
- Layer 1 Logistics/Supply drop system (`logistics/mod.rs` or `supply_drop.rs`)
- Layer 1 Traits/Utility weights (`traits.rs`)

## 3. RED Phase: Tests First

```rust
// tests/layer1_cargo_cult_logistics_tests.rs
use bevy::prelude::*;
use crate::layer1::social::cargo_cult::{CargoCultBelief, apply_cargo_cult_belief_system, process_ritual_actions_system};
use crate::layer1::needs::{NeedMorale, ActionType, CurrentAction};
use crate::layer1::logistics::SupplyDropEvent;

#[test]
fn test_supply_drop_triggers_cargo_cult_belief() {
    let mut app = App::new();
    app.add_systems(Update, apply_cargo_cult_belief_system);
    app.init_resource::<Events<SupplyDropEvent>>();

    // Spawn a pop currently dancing
    let pop_entity = app.world_mut().spawn((
        CurrentAction { action_type: ActionType::Dancing, duration: 10.0 },
    )).id();

    // Trigger a supply drop nearby
    app.world_mut().resource_mut::<Events<SupplyDropEvent>>().send(SupplyDropEvent {
        position: UVec2::new(5, 5),
    });

    app.update();

    // Assert that the pop developed a CargoCultBelief
    assert!(app.world().entity(pop_entity).has::<CargoCultBelief>());
    let belief = app.world().entity(pop_entity).get::<CargoCultBelief>().unwrap();
    assert_eq!(belief.associated_action, ActionType::Dancing);
}

#[test]
fn test_cargo_cult_rituals_boost_morale_but_lower_efficiency() {
    let mut app = App::new();
    app.add_systems(Update, process_ritual_actions_system);

    let pop_entity = app.world_mut().spawn((
        CargoCultBelief { associated_action: ActionType::Dancing, belief_strength: 1.0 },
        CurrentAction { action_type: ActionType::Dancing, duration: 5.0 },
        NeedMorale { value: 50.0, max: 100.0 },
        // Efficiency modifier component or similar
    )).id();

    app.update();

    let morale = app.world().entity(pop_entity).get::<NeedMorale>().unwrap();
    // Morale should increase during the ritual
    assert!(morale.value > 50.0);
    // There should also be a modifier that decreases their work efficiency
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/social/cargo_cult.rs
use bevy::prelude::*;
use crate::layer1::needs::{NeedMorale, CurrentAction};
use crate::layer1::logistics::SupplyDropEvent;

#[derive(Component)]
pub struct CargoCultBelief {
    pub associated_action: crate::layer1::needs::ActionType,
    pub belief_strength: f32,
}

pub fn apply_cargo_cult_belief_system(
    mut commands: Commands,
    mut supply_events: EventReader<SupplyDropEvent>,
    query: Query<(Entity, &CurrentAction), Without<CargoCultBelief>>,
) {
    if !supply_events.is_empty() {
        // Just consume the event for now
        supply_events.clear();
        for (entity, action) in query.iter() {
            // Assign belief based on the current action
            commands.entity(entity).insert(CargoCultBelief {
                associated_action: action.action_type.clone(),
                belief_strength: 1.0,
            });
        }
    }
}

pub fn process_ritual_actions_system(
    mut query: Query<(&CargoCultBelief, &CurrentAction, &mut NeedMorale)>,
) {
    for (belief, action, mut morale) in query.iter_mut() {
        if belief.associated_action == action.action_type {
            morale.value = (morale.value + 1.0).min(morale.max);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Awareness:** Supply drops should only trigger beliefs for Pops in the immediate vicinity, not globally. Add a radius check using the `SupplyDropEvent` position and the Pop's `GridPosition`.
- **Probability:** Not every Pop should develop a belief immediately. Add a random chance (e.g., 10%) or base it on existing traits (e.g., `Superstitious` pops are more susceptible).
- **Efficiency Debuff:** Instead of just boosting morale, explicitly apply an `EfficiencyDebuff` component or update `UtilityWeights` so they prioritize their ritual action over actual work tasks when near the landing pad.
- **Decay:** Belief strength should slowly decay over time if no further supply drops reinforce it.

## 6. Acceptance Criteria (Testable!)
- [ ] `cargo test` returns 0 failures, including `test_supply_drop_triggers_cargo_cult_belief`.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new `cargo_cult.rs` module.
- [ ] Pops correctly gain `CargoCultBelief` when a `SupplyDropEvent` occurs near them while they are performing an action.
- [ ] Performing the associated ritual action increases morale but applies an efficiency penalty or takes time away from actual work tasks.

## 7. Technical Guidance
- Create a new module `src/layer1/social/cargo_cult.rs`.
- Ensure `CargoCultBelief` derives standard Bevy traits (`Component`, `Debug`, `Clone`).
- Register `apply_cargo_cult_belief_system` and `process_ritual_actions_system` in the appropriate simulation schedule (likely within `Layer1SystemSet::Update` or similar).
- Make sure to initialize the `SupplyDropEvent` resource in your test setups to avoid panics.

## 8. Questions
*Builder: add questions here if spec is unclear.*
