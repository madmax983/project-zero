# 1167: Gravitational Empathy

## 1. Overview
On high-gravity worlds, Pops initially suffer a "Crushing Weight" mood debuff. Over generations, they develop "Gravitational Empathy"—a latent psychic connection where the collective mood of the colony alters the local gravity field. High morale decreases gravity (boosting movement/industry), while low morale increases it (crushing fragile buildings and slowing movement).

## 2. Dependencies
- Morale/Mood System (`src/layer1/needs.rs`)
- Environment/Planet traits (`src/layer1/environment.rs`)
- Building Damage/Integrity System
- Entity Movement Speed Modifiers

## 3. RED Phase: Tests First

```rust
// src/layer1/gravitational_empathy_tests.rs
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_high_gravity_applies_crushing_weight() {
        let mut app = App::new();
        app.add_systems(Update, apply_crushing_weight_system);

        let planet = app.world_mut().spawn(PlanetGravity { value: 2.0 }).id();
        let pop = app.world_mut().spawn((
            Pop,
            Location { planet_id: planet },
            PopMorale { value: 100.0 },
        )).id();

        app.update();

        // Check if crushing weight modifier is applied
        assert!(app.world().get::<CrushingWeight>(pop).is_some(), "Pop should gain CrushingWeight on high gravity world");
    }

    #[test]
    fn test_low_morale_increases_local_gravity_and_damages_buildings() {
        let mut app = App::new();
        app.add_systems(Update, apply_gravitational_empathy_system);

        let planet = app.world_mut().spawn((
            PlanetGravity { value: 2.0 },
            ColonyMorale { average: 10.0 }, // Very low morale
            GravitationalEmpathyUnassigned, // Marker for unlocked empathy
        )).id();

        let building = app.world_mut().spawn((
            Building,
            Location { planet_id: planet },
            Integrity { current: 100.0, max: 100.0, fragile: true },
        )).id();

        app.update();

        let new_gravity = app.world().get::<PlanetGravity>(planet).unwrap();
        assert!(new_gravity.value > 2.0, "Gravity should increase due to low morale");

        let b_integrity = app.world().get::<Integrity>(building).unwrap();
        assert!(b_integrity.current < 100.0, "Fragile buildings should take damage from increased gravity");
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
- **Integration**: `PlanetGravity` updates should dynamically recalculate the speed of Pops traversing the grid in Layer 1.
- **Generational Unlock**: Gravitational Empathy should not be active immediately on a high-gravity world. Introduce a generational counter or tech unlock that eventually applies the `GravitationalEmpathy` component to the colony.
- **UI Indicators**: When the effect kicks in, visual indicators (screen shake, building sparks) should warn the player of the crushing doom.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/gravitational_empathy.rs`.
- [ ] High-gravity worlds apply a baseline mood debuff ("Crushing Weight").
- [ ] If `GravitationalEmpathy` is active, low average morale increases the local gravity modifier.
- [ ] Increased gravity modifier damages buildings tagged as `fragile` and reduces Pop movement speed.

## 7. Technical Guidance
- The calculation for the dynamic gravity modifier should be clamped to prevent `f32` overflow or instant destruction of the entire colony.
- Morale sampling should probably use a smoothing function or a moving average to avoid extreme gravity fluctuations on every tick.

## 8. Questions
*Builder: add questions here if spec is unclear.*
