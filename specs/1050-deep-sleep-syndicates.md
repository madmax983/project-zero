# 1050: Deep Sleep Syndicates

## 1. Overview
Players can discover derelict "Cryo-Corporate Ships" from a bygone era. Thawing out these "Deep Sleep Syndicates" provides massive, immediate boosts to production or logistics. However, they implement archaic, hyper-exploitative policies (like indentured servitude or unsafe working conditions) that cause massive unrest and trauma in modern pops. The tension lies in using desperate, unethical solutions from the past to solve impossible crises in the present, knowing you'll have to deal with the moral and social fallout later.

## 2. Dependencies
- Layer 1 `psychology` (specifically `TraumaTracker` for adding trauma).
- Layer 1 `economy` or `production` system for granting the massive boosts.
- Layer 2 `exploration` or `events` for discovering the derelict ships.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::psychology::TraumaTracker;
    use crate::layer1::economy::ProductionModifier;

    #[test]
    fn test_thawing_syndicate_grants_boost_and_adds_trauma() {
        let mut app = App::new();
        app.add_systems(Update, process_thaw_syndicate_event);
        app.add_event::<ThawSyndicateEvent>();

        // Setup initial TraumaTracker
        app.insert_resource(TraumaTracker {
            global_trauma_level: 0.0,
            recent_trauma_events: vec![],
        });

        // Setup a global production modifier
        app.insert_resource(ProductionModifier {
            multiplier: 1.0,
        });

        // Fire the event to thaw the syndicate
        app.world_mut().resource_mut::<Events<ThawSyndicateEvent>>().send(ThawSyndicateEvent {
            syndicate_type: SyndicateType::Industrial,
        });

        app.update();

        // Verify production was boosted
        let production = app.world().resource::<ProductionModifier>();
        assert!(production.multiplier > 1.0, "Thawing a syndicate should provide a massive production boost.");

        // Verify trauma was added to the TraumaTracker
        let trauma = app.world().resource::<TraumaTracker>();
        assert!(trauma.global_trauma_level > 0.0, "Thawing a syndicate must add significant trauma to the colony.");
        assert!(!trauma.recent_trauma_events.is_empty(), "TraumaTracker should record the reason for the trauma.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/deep_sleep_syndicates.rs
use bevy::prelude::*;
use crate::layer1::psychology::TraumaTracker;
use crate::layer1::economy::ProductionModifier;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SyndicateType {
    Industrial,
    Logistical,
    Military,
}

#[derive(Event)]
pub struct ThawSyndicateEvent {
    pub syndicate_type: SyndicateType,
}

pub fn process_thaw_syndicate_event(
    mut events: EventReader<ThawSyndicateEvent>,
    mut production_modifier: ResMut<ProductionModifier>,
    mut trauma_tracker: ResMut<TraumaTracker>,
) {
    for event in events.read() {
        // Apply the immediate boost based on the type
        match event.syndicate_type {
            SyndicateType::Industrial => {
                production_modifier.multiplier += 2.0; // Massive +200% boost
            }
            _ => {
                production_modifier.multiplier += 1.0;
            }
        }

        // Add the severe drawback: Trauma and Unrest
        trauma_tracker.global_trauma_level += 50.0;
        trauma_tracker.recent_trauma_events.push("Indentured Servitude Implemented".to_string());
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Edicts/Policies:** Instead of just magically modifying `ProductionModifier` and `TraumaTracker` via an event, the thawed syndicate should force specific archaic `Policy` or `Edict` entities to be enacted. These policies would then naturally provide the boost and cause the trauma through existing systems.
- **Duration/Removal:** The player needs a way to "overthrow" or remove the syndicate later. This could involve a massive resource cost or fighting a rebellion.
- **Localized Effects:** If possible, apply the trauma to specific pops working in the affected sectors rather than just globally.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_thawing_syndicate_grants_boost_and_adds_trauma` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Ensure that the `TraumaTracker` resource is modified instead of emitting a standalone `TraumaEvent`, as per architectural guidelines.
- Consider how the event connects to Layer 2 exploration (e.g., finding the derelict ship triggers a dialogue/decision that fires the `ThawSyndicateEvent`).

## 8. Questions
*Builder: add questions here if spec is unclear.*
