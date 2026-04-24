# 1151: The Immortal Mascot

## 1. Overview
A seemingly harmless alien creature wanders into the colony, providing a massive colony-wide morale boost upon adoption. However, it is biologically immortal, its food consumption grows exponentially as it ages, and its extreme armor makes it nearly impossible to kill. The player must choose between sustaining the resource drain for the morale boost or attempting a dangerous disposal of the beloved pet.

## 2. Dependencies
- Needs system (`src/layer1/needs.rs`)
- Morale/Mood system
- Entity combat/damage system
- Inventory/Resource consumption system

## 3. RED Phase: Tests First

```rust
// src/layer1/fauna/immortal_mascot_tests.rs
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_mascot_provides_global_morale_boost() {
        let mut app = App::new();
        app.add_systems(Update, apply_mascot_morale_system);

        let pop = app.world_mut().spawn(PopMorale { value: 50.0 }).id();

        app.update();
        assert_eq!(app.world().get::<PopMorale>(pop).unwrap().value, 50.0);

        // Spawn the mascot
        app.world_mut().spawn(ImmortalMascot {
            age: 0,
            food_consumption_rate: 1.0,
            armor_rating: 1000.0,
        });

        app.update();
        assert_eq!(
            app.world().get::<PopMorale>(pop).unwrap().value,
            75.0,
            "Mascot should boost pop morale"
        );
    }

    #[test]
    fn test_mascot_food_consumption_increases_exponentially() {
        let mut app = App::new();
        app.add_systems(Update, mascot_aging_system);

        let mascot = app.world_mut().spawn(ImmortalMascot {
            age: 0,
            food_consumption_rate: 1.0,
            armor_rating: 1000.0,
        }).id();

        app.update(); // age = 1
        let rate_1 = app.world().get::<ImmortalMascot>(mascot).unwrap().food_consumption_rate;
        assert!(rate_1 > 1.0);

        app.update(); // age = 2
        let rate_2 = app.world().get::<ImmortalMascot>(mascot).unwrap().food_consumption_rate;
        assert!(rate_2 > rate_1 * 1.1, "Growth must be at least exponential");
    }

    #[test]
    fn test_mascot_has_extreme_armor() {
        let mut mascot = ImmortalMascot {
            age: 0,
            food_consumption_rate: 1.0,
            armor_rating: 1000.0,
        };

        // Hypothetical damage calculation
        let damage_taken = calculate_damage(100.0, mascot.armor_rating);
        assert!(damage_taken < 1.0, "Mascot should be virtually immune to normal damage");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The morale boost needs to integrate directly with `src/layer1/needs.rs` instead of the dummy `PopMorale` component. It should likely be an `AuraEffect` or global state modifier.
- **Consumption Logic**: Food consumption should be tied to actual game ticks (`SimulationTime`) and the colony's central inventory logic (`src/layer1/inventory.rs`).
- **Riot Mechanics**: If the mascot is attacked or killed by the player, it must emit a global event that triggers massive Unrest and Riots among the `Pop`s.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/fauna/immortal_mascot.rs`.
- [ ] Mascot grants a significant global morale bonus.
- [ ] Mascot food consumption scales exponentially with age.
- [ ] Mascot possesses `armor_rating` high enough to mitigate >99% of standard weapon damage.

## 7. Technical Guidance
- The mascot should be spawned via an event, not directly, so it can wander in dynamically.
- To prevent exponential `f32::INFINITY` crashes on consumption, cap the consumption rate at an absurdly high number (e.g., 90% of the colony's maximum theoretical output).
- Ensure the Mascot is registered with the combat system as a valid (but highly armored) target.

## 8. Questions
*Builder: add questions here if spec is unclear.*
