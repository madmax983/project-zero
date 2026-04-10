# 928 - The Black Market Airlock

## 1. Overview

**Layer:** 1
**Fantasy:** Contraband entering the colony outside official channels, creating an illicit economy.
**Mechanic:** Pops with low morale and high greed secretly repurpose a malfunctioning or remote airlock to smuggle restricted goods (like exotic spices or banned media), bypassing colony storage.
**Emergence:** An "abandoned" maintenance corridor becomes the busiest area in the colony. If the airlock experiences a critical failure, it could decompress the hidden black market, instantly killing key figures of your underground economy.
**Tension:** Do you repair the airlock and crush the black market (causing massive withdrawal stress for the population), or leave it broken and risk explosive decompression?

## 2. Dependencies

- Needs System (Morale)
- Pops Trait System (Greed)
- Building/Airlock entity representations
- Inventory/Resource storage systems
- Environment System (Decompression mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct Airlock { is_broken: bool }

    #[derive(Component)]
    struct SmugglerHideout;

    #[derive(Component)]
    struct PopTraits { greed: f32, morale: f32 }

    #[derive(Component)]
    struct ActionTarget(Entity);

    #[derive(Component)]
    struct SmugglingAction;

    #[test]
    fn test_smuggling_action_triggered_by_broken_airlock() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_smuggling_action_system);

        let airlock = app.world_mut().spawn((
            Airlock { is_broken: true },
            SmugglerHideout,
        )).id();

        let pop = app.world_mut().spawn((
            PopTraits { greed: 0.9, morale: 0.2 },
        )).id();

        app.update();

        // The pop should decide to smuggle due to low morale and high greed near a broken airlock
        assert!(app.world().get::<SmugglingAction>(pop).is_some());
        assert_eq!(app.world().get::<ActionTarget>(pop).unwrap().0, airlock);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn evaluate_smuggling_action_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Hook this into the Utility AI system correctly, adding a new action type rather than raw components where appropriate.
- Ensure the contraband resources are handled by a secondary/hidden inventory system, or give Pops their own private inventories if they don't already have one.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops with high greed and low morale prioritize smuggling goods using broken airlocks.

## 7. Technical Guidance

- Utilize the existing Utility AI system (`evaluate_actions_system`) to add a new `ActionType::Smuggle` that gets scored highly when conditions are met.
- Make sure repairing the airlock dynamically removes the `SmugglerHideout` component or prevents the smuggling action from being scored.

## 8. Questions
*Builder: add questions here if spec is unclear.*
