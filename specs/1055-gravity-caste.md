# 1055: Gravity Caste

## 1. Overview
Biology diverges based on the gravitational environment a Pop resides in. Over time, Pops born or living in Low-G environments adapt to become "Spacers" (tall, frail, intelligent), while those in High-G become "Squats" (short, dense, strong). Transferring a Spacer to a Super-Earth (High-G) results in severe health and movement penalties, enforcing biological specialization or requiring the construction of segregated habitats or exo-suits.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- `Time` resource to track adaptation
- Pops with attributes (`Strength`, `Intelligence`, `Health`, `MovementSpeed`)
- Environments with gravity metrics (e.g., `GravityWell` or tile-based gravity modifiers)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_prolonged_low_g_creates_spacer() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Time::default());
        app.add_systems(Update, adapt_gravity_caste_system);

        let pop = app.world_mut().spawn((
            GravityExposure { current_g: 0.2, exposure_time: 0.0 },
            PopAttributes { strength: 10, intelligence: 10, health: 100 },
        )).id();

        // Act: Advance time significantly
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(1000));
        app.update();

        // Assert: Pop should have Spacer caste and altered attributes
        let caste = app.world().get::<GravityCaste>(pop);
        assert!(caste.is_some());
        assert_eq!(*caste.unwrap(), GravityCaste::Spacer);

        let attrs = app.world().get::<PopAttributes>(pop).unwrap();
        assert!(attrs.intelligence > 10);
        assert!(attrs.strength < 10);
    }

    #[test]
    fn test_spacer_in_high_g_suffers_penalties() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_gravity_penalties_system);

        let pop = app.world_mut().spawn((
            GravityCaste::Spacer,
            GravityExposure { current_g: 2.0, exposure_time: 0.0 }, // In High-G
            MovementSpeed { current: 5.0 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Act
        app.update();

        // Assert: Movement speed should be crippled, and health should drain
        let speed = app.world().get::<MovementSpeed>(pop).unwrap();
        let health = app.world().get::<Health>(pop).unwrap();

        assert!(speed.current < 5.0);
        assert!(health.current < 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub enum GravityCaste {
    Standard,
    Spacer,
    Squat,
}

#[derive(Component)]
pub struct GravityExposure {
    pub current_g: f32,
    pub exposure_time: f32,
}

#[derive(Component)]
pub struct PopAttributes {
    pub strength: i32,
    pub intelligence: i32,
    pub health: i32,
}

#[derive(Component)]
pub struct MovementSpeed {
    pub current: f32,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

// 1000 seconds of exposure required to shift caste (for testing)
const ADAPTATION_THRESHOLD: f32 = 1000.0;

pub fn adapt_gravity_caste_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut GravityExposure, &mut PopAttributes, Option<&GravityCaste>)>,
) {
    let dt = time.delta_secs();

    for (entity, mut exposure, mut attrs, caste_opt) in query.iter_mut() {
        exposure.exposure_time += dt;

        if exposure.exposure_time >= ADAPTATION_THRESHOLD {
            if exposure.current_g <= 0.5 && caste_opt != Some(&GravityCaste::Spacer) {
                commands.entity(entity).insert(GravityCaste::Spacer);
                attrs.intelligence += 5;
                attrs.strength -= 5;
            } else if exposure.current_g >= 1.5 && caste_opt != Some(&GravityCaste::Squat) {
                commands.entity(entity).insert(GravityCaste::Squat);
                attrs.strength += 5;
                attrs.intelligence -= 2;
            }
            // Reset exposure time after a shift
            exposure.exposure_time = 0.0;
        }
    }
}

pub fn apply_gravity_penalties_system(
    mut query: Query<(&GravityCaste, &GravityExposure, &mut MovementSpeed, &mut Health)>,
) {
    for (caste, exposure, mut speed, mut health) in query.iter_mut() {
        if *caste == GravityCaste::Spacer && exposure.current_g > 1.0 {
            // Spacer in high G
            speed.current *= 0.5;
            health.current -= 1.0; // Flat damage per tick
        } else if *caste == GravityCaste::Squat && exposure.current_g < 1.0 {
            // Squat in low G - clumsy but not taking damage
            speed.current *= 0.8;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Exo-Suits Integration**: If a Pop has an `ExoSuit` component equipped, it should negate the penalties applied in `apply_gravity_penalties_system`.
- **Generational Inheritance**: Gravity castes should have a chance to be inherited by offspring (if reproduction systems exist), rather than just adapted over a lifetime.
- **Gradual Adaptation**: Instead of a hard threshold, caste adaptation could be a continuous spectrum (-1.0 to 1.0) that modifies stats fluidly, though discreet castes are easier for UI and player comprehension.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for new code is >= 85%.
- [ ] Prolonged exposure to extreme gravity alters Pop attributes.
- [ ] Mismatched environments apply continuous penalties to affected castes.

## 7. Technical Guidance
- The `current_g` value should be updated by a separate system that reads the Pop's current `GridPosition` and checks the local `GravityGrid` or tile properties.
- Ensure health damage applied by gravity penalties respects minimum health bounds (0.0) and emits death events if fatal.

## 8. Questions
*Builder: add questions here if spec is unclear.*
