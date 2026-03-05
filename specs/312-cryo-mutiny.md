# Specification 312: The Cryo-Mutiny

## 1. Overview
This feature introduces "The Cryo-Mutiny." An ancient failing Cryo-Ship (Layer 2) lands on the colony. The awakened Pioneers, possessing obsolete traits and primitive ethics, suffer massive "Culture Shock" upon seeing the advanced colony and immediately form a hostile faction to "reclaim" the colony for humanity.

## 2. Dependencies
- `chronicle` system (`src/layer1/chronicle.rs`).
- `grievance` / Faction system (`src/layer1/social/grievances.rs`).
- `combat` system (if the mutiny turns violent).

## 3. RED Phase: Tests First

```rust
// src/layer2/cryo_mutiny.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, trigger_cryo_ship_landing_system);
        app.init_resource::<CryoShipEvent>();
        app.init_resource::<MutinyTracker>();
        app
    }

    #[test]
    fn test_cryo_ship_lands_and_spawns_mutineers() {
        let mut app = setup_app();

        // Trigger a landing event
        app.world_mut().resource_mut::<CryoShipEvent>().active = true;
        app.update();

        // Mutiny tracker should show active mutineers spawned
        let tracker = app.world().resource::<MutinyTracker>();
        assert_eq!(tracker.mutineers_spawned, 10, "10 Mutineer Pops should be spawned.");
        assert_eq!(tracker.active, true, "A mutiny should be active.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/cryo_mutiny.rs
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CryoShipEvent {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct MutinyTracker {
    pub active: bool,
    pub mutineers_spawned: u32,
}

#[derive(Component)]
pub struct MutineerPop {
    pub culture_shock: f32,
}

pub fn trigger_cryo_ship_landing_system(
    mut commands: Commands,
    mut event: ResMut<CryoShipEvent>,
    mut tracker: ResMut<MutinyTracker>,
) {
    if event.active {
        tracker.mutineers_spawned = 10;
        tracker.active = true;

        for _ in 0..10 {
            commands.spawn(MutineerPop {
                culture_shock: 100.0,
            });
        }
        event.active = false;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The newly spawned Pops must integrate with the regular `Pop` system but with extremely low `Affinity` or high `Grievance` against the established colony.
- **Lore**: A unique event should be chronicled showing the culture clash between the "Ancients" and the current "Abominations."
- **Mechanics**: Instead of instantaneous violence, the mutineers could start a strike or seize buildings until their "Culture Shock" slowly decays or they are defeated in combat.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `cryo_mutiny.rs`.
- [ ] Triggering the event spawns a set number of mutineer entities.

## 7. Technical Guidance
- The actual spawning should place the Pops at a valid `GridPosition` near a "Crash Site" or "Landing Pad".
- Consider leveraging `Faction` components if they exist to handle the group's collective hostility.

## 8. Questions
*Builder: add questions here if spec is unclear.*
