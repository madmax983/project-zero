# Specification 315: The Propaganda Engine

## 1. Overview
This feature introduces "The Propaganda Engine," a massive communications array that broadcasts fake intel (e.g., exaggerating your fleet size, fabricating economic data) to Layer 3 empires. It increases diplomatic weight and deters attacks. However, if an "Inspector" ship discovers the lie, you suffer a catastrophic "Loss of Face," immediately turning neutral factions hostile.

## 2. Dependencies
- `diplomacy` system (Layer 3) or local prestige tracking.
- `chronicle` system (`src/layer1/chronicle.rs`).

## 3. RED Phase: Tests First

```rust
// src/layer2/propaganda_engine.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, update_propaganda_system);
        app.insert_resource(PropagandaEngine { active: false, bluff_level: 0 });
        app.insert_resource(DiplomaticWeight { value: 100 });
        app.init_resource::<InspectorEvent>();
        app
    }

    #[test]
    fn test_propaganda_increases_weight() {
        let mut app = setup_app();

        // Activate propaganda
        app.world_mut().resource_mut::<PropagandaEngine>().active = true;
        app.world_mut().resource_mut::<PropagandaEngine>().bluff_level = 50;

        app.update();

        let weight = app.world().resource::<DiplomaticWeight>();
        assert!(weight.value > 100, "Diplomatic weight should increase when the propaganda engine is active.");
    }

    #[test]
    fn test_inspector_calls_bluff() {
        let mut app = setup_app();

        // Active bluff
        app.world_mut().resource_mut::<PropagandaEngine>().active = true;
        app.world_mut().resource_mut::<PropagandaEngine>().bluff_level = 100;

        app.update();

        // Trigger inspector
        app.world_mut().resource_mut::<InspectorEvent>().triggered = true;

        app.update();

        let weight = app.world().resource::<DiplomaticWeight>();
        let engine = app.world().resource::<PropagandaEngine>();
        assert_eq!(weight.value, 0, "Diplomatic weight should plummet to zero when a bluff is called.");
        assert_eq!(engine.active, false, "The bluff should end.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/propaganda_engine.rs
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PropagandaEngine {
    pub active: bool,
    pub bluff_level: u32,
}

#[derive(Resource, Default)]
pub struct DiplomaticWeight {
    pub value: u32,
}

#[derive(Resource, Default)]
pub struct InspectorEvent {
    pub triggered: bool,
}

pub fn update_propaganda_system(
    mut engine: ResMut<PropagandaEngine>,
    mut weight: ResMut<DiplomaticWeight>,
    mut inspector: ResMut<InspectorEvent>,
) {
    if engine.active {
        // Simple buff logic: Add the bluff level to the weight (if not already applied in reality, this is simplified)
        weight.value = 100 + engine.bluff_level; // Assuming base weight is 100

        if inspector.triggered {
            // Bluff called! Catastrophic loss of face.
            weight.value = 0;
            engine.active = false;
            engine.bluff_level = 0;
            inspector.triggered = false; // Reset event
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: `DiplomaticWeight` should integrate with the actual global faction system. The "Inspector" should be an actual unit/event traversing the system.
- **Morale**: Could tie local Pops' morale to the success of the propaganda (they might believe it too, or feel guilty about the lie).
- **Lore**: Important Chronicle logs should be added for when the bluff is started and spectacularly when it fails.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `propaganda_engine.rs`.
- [ ] Propaganda correctly buffs diplomatic weight.
- [ ] Inspector event correctly calls the bluff and tanks diplomatic weight.

## 7. Technical Guidance
- The `DiplomaticWeight` logic above is simplified for testing. In reality, it should be a modifier added dynamically during a `calculate_diplomatic_weight` system so that base values can still change independently of the bluff.
- You can add an "expose" mechanic where nearby rival factions have an RNG chance per tick to discover the bluff based on the `bluff_level` and their intel capacity.

## 8. Questions
*Builder: add questions here if spec is unclear.*
