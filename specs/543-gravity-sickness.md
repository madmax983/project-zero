# 543 - Gravity Sickness

## 1. Overview
**Layer:** 1 / 2
**Fantasy:** The physical toll of life between the stars.
**Mechanic:** Pops born on low-gravity worlds or orbital stations develop a permanent "Low-G Adapted" trait. If they are relocated to a high-gravity world (like a heavy mining planet), they suffer severe debuffs to movement speed, work efficiency, and health, eventually leading to early death unless provided with expensive "Exo-suits."

## 2. Dependencies
- `084` Pop Traits
- `132` Equipment & Wear
- `468` Escape Velocity Economics (for Planetary Gravity)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_low_g_adapted_pop_in_high_g_suffers_debuffs() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_gravity_sickness_system);

        app.insert_resource(PlanetaryGravity { g_force: 1.5 }); // High G

        let pop_id = app.world_mut().spawn((
            Pop,
            Traits { list: vec![Trait::LowGAdapted] },
            MovementSpeed { base: 10.0, current: 10.0 },
            Health { current: 100.0, max: 100.0 },
            Equipment { suit: None },
        )).id();

        // Act
        app.update();

        // Assert
        let speed = app.world().get::<MovementSpeed>(pop_id).unwrap();
        assert!(speed.current < 10.0, "Speed should be reduced in high G without exo-suit");

        let health = app.world().get::<Health>(pop_id).unwrap();
        assert!(health.current < 100.0, "Health should decay over time in high G without exo-suit");
    }

    #[test]
    fn test_exo_suit_prevents_gravity_sickness() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_gravity_sickness_system);

        app.insert_resource(PlanetaryGravity { g_force: 1.5 }); // High G

        let pop_id = app.world_mut().spawn((
            Pop,
            Traits { list: vec![Trait::LowGAdapted] },
            MovementSpeed { base: 10.0, current: 10.0 },
            Health { current: 100.0, max: 100.0 },
            Equipment { suit: Some(SuitType::ExoSuit) }, // Has exo-suit
        )).id();

        // Act
        app.update();

        // Assert
        let speed = app.world().get::<MovementSpeed>(pop_id).unwrap();
        assert_eq!(speed.current, 10.0, "Speed should NOT be reduced with exo-suit");

        let health = app.world().get::<Health>(pop_id).unwrap();
        assert_eq!(health.current, 100.0, "Health should NOT decay with exo-suit");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Clone, PartialEq, Eq)]
pub enum Trait { LowGAdapted }

#[derive(Component)]
pub struct Traits { pub list: Vec<Trait> }

#[derive(Component)]
pub struct MovementSpeed { pub base: f32, pub current: f32 }

#[derive(Component)]
pub struct Health { pub current: f32, pub max: f32 }

#[derive(Clone, PartialEq, Eq)]
pub enum SuitType { ExoSuit }

#[derive(Component)]
pub struct Equipment { pub suit: Option<SuitType> }

#[derive(Resource)]
pub struct PlanetaryGravity { pub g_force: f32 }

pub fn process_gravity_sickness_system(
    gravity: Res<PlanetaryGravity>,
    mut query: Query<(&Traits, &Equipment, &mut MovementSpeed, &mut Health), With<Pop>>,
) {
    if gravity.g_force <= 1.0 {
        return; // No gravity sickness in low or normal G
    }

    for (traits, equipment, mut speed, mut health) in query.iter_mut() {
        if traits.list.contains(&Trait::LowGAdapted) {
            let has_exo_suit = equipment.suit == Some(SuitType::ExoSuit);
            if !has_exo_suit {
                let penalty_factor = 1.0 - ((gravity.g_force - 1.0) * 0.5).clamp(0.0, 0.8);
                speed.current = speed.base * penalty_factor;
                health.current -= (gravity.g_force - 1.0) * 1.0; // Decay health slightly
            } else {
                speed.current = speed.base; // Ensure base speed is maintained
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactor:** `MovementSpeed` penalties shouldn't overwrite other status effects. Consider adding a `StatModifier` system where traits apply a specific `MovementPenaltyModifier` rather than modifying `current` directly.
- **Improvement:** Connect the health decay to the existing Metabolism or Medical Needs systems instead of just decrementing `Health.current`.
- **Code Smell:** Hardcoding the penalty factors (`0.5`, `1.0`) directly in the logic. Extract these into configurable constants or an overarching balancing resource.
- **Integration:** Update the `birth_system` or migration mechanics to assign `Trait::LowGAdapted` automatically when a pop spawns on a low-G world.

## 6. Acceptance Criteria
- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] `LowGAdapted` pops suffer health and speed penalties on High-G worlds.
- [ ] Wearing an `ExoSuit` entirely negates the gravity sickness debuff.
- [ ] Test coverage is above 85%.

## 7. Technical Guidance
- `PlanetaryGravity` resource was introduced in Spec `468`. Ensure you import or reference it correctly if already defined in `src/layer2/trade/escape_velocity.rs` or `src/shared/physics.rs`.
- Take care to avoid underflowing `Health` values. Implement `clamp(0.0, health.max)` or utilize damage events.

## 8. Questions
*Builder: add questions here if spec is unclear.*
