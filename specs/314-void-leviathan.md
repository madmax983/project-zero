# Specification 314: The Void Leviathan

## 1. Overview
This feature introduces "The Void Leviathan," a colossal space-dwelling creature (Layer 2). Occasionally entering the system to feed on solar radiation, its sheer size causes a temporary global eclipse on Layer 1. The eclipse kills solar power and drops temperatures. It also hunts passing trade fleets. Destroying it yields unique "Bio-Alloys".

## 2. Dependencies
- `lighting` system (`src/layer1/lighting.rs`).
- `temperature` system (`src/layer1/atmosphere.rs` or similar).

## 3. RED Phase: Tests First

```rust
// src/layer2/void_leviathan.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, update_leviathan_eclipse_system);
        app.insert_resource(VoidLeviathan { active: false, duration: 0 });
        app.init_resource::<LeviathanEclipse>();
        app
    }

    #[test]
    fn test_leviathan_triggers_eclipse() {
        let mut app = setup_app();

        // Spawn leviathan
        app.world_mut().resource_mut::<VoidLeviathan>().active = true;
        app.world_mut().resource_mut::<VoidLeviathan>().duration = 100;
        app.update();

        let eclipse = app.world().resource::<LeviathanEclipse>();
        assert_eq!(eclipse.active, true, "Eclipse should be active when leviathan is active.");
    }

    #[test]
    fn test_leviathan_leaves_after_duration() {
        let mut app = setup_app();

        app.world_mut().resource_mut::<VoidLeviathan>().active = true;
        app.world_mut().resource_mut::<VoidLeviathan>().duration = 1;

        app.update(); // Tick 1 (active)
        app.update(); // Tick 2 (duration expires)

        let leviathan = app.world().resource::<VoidLeviathan>();
        assert_eq!(leviathan.active, false, "Leviathan should depart when its duration expires.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/void_leviathan.rs
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct VoidLeviathan {
    pub active: bool,
    pub duration: u32,
}

#[derive(Resource, Default)]
pub struct LeviathanEclipse {
    pub active: bool,
}

pub fn update_leviathan_eclipse_system(
    mut leviathan: ResMut<VoidLeviathan>,
    mut eclipse: ResMut<LeviathanEclipse>,
) {
    if leviathan.active {
        eclipse.active = true;

        if leviathan.duration > 0 {
            leviathan.duration -= 1;
        } else {
            leviathan.active = false;
        }
    } else {
        eclipse.active = false;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The `LeviathanEclipse` needs to override or modify the output of the standard `DayNightCycle` to simulate total darkness and plummeting temperatures during the day.
- **Lore**: A massive alert and chronicle entry should trigger when the Leviathan arrives and departs.
- **Reward**: If a combat system exists for Layer 2, add the logic to grant `BioAlloys` upon the Leviathan's destruction.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `void_leviathan.rs`.
- [ ] Leviathan sets `LeviathanEclipse` active when present.
- [ ] Leviathan duration ticks down and deactivates correctly.

## 7. Technical Guidance
- The `LeviathanEclipse` resource should be checked by the systems that calculate solar power generation to zero it out while active.
- For temperature drops, apply a flat negative modifier to the ambient temperature of the entire grid while the eclipse persists.

## 8. Questions
*Builder: add questions here if spec is unclear.*
