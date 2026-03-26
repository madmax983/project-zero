# Temporal Echo Chambers

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** Capturing the ghost of a past golden age—or a future crisis—by locking a room in a localized time dilation field.
**Mechanic:** Specialized containment rooms where time moves at a fraction of the normal rate. You can seal Pops or resources inside to perfectly preserve them for decades. However, the energy cost increases exponentially the longer it runs, and catastrophic failure releases a "temporal shockwave."

## 2. Dependencies
- `042-energy-system.md`
- `062-pop-lifecycle.md`

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_chamber_preserves_pops_by_reducing_aging() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, temporal_chamber_aging_system);

        let pop = app.world.spawn((
            Pop,
            Age { ticks: 0 },
            InsideChamber { chamber_entity: Entity::from_raw(1) },
        )).id();

        app.world.spawn_empty().insert((
            TemporalChamber {
                time_dilation_factor: 0.1,
                active: true,
                energy_cost: 10.0,
            },
        )).id(); // ID 1

        // Act
        app.update();

        // Assert
        let pop_age = app.world.get::<Age>(pop).unwrap();
        // Normal aging would add 1.0, chamber adds 0.1
        assert_eq!(pop_age.ticks, 0); // Need fractional tracking or tick accumulation
    }

    #[test]
    fn test_chamber_energy_cost_increases_over_time() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, temporal_chamber_energy_system);

        let chamber = app.world.spawn((
            TemporalChamber {
                time_dilation_factor: 0.1,
                active: true,
                energy_cost: 10.0,
            },
            TimeActive { ticks: 0 },
        )).id();

        // Act
        app.update();

        // Assert
        let updated_chamber = app.world.get::<TemporalChamber>(chamber).unwrap();
        assert!(updated_chamber.energy_cost > 10.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Age {
    pub ticks: u32,
}

#[derive(Component)]
pub struct FractionalAge {
    pub accumulated: f32,
}

#[derive(Component)]
pub struct InsideChamber {
    pub chamber_entity: Entity,
}

#[derive(Component)]
pub struct TemporalChamber {
    pub time_dilation_factor: f32,
    pub active: bool,
    pub energy_cost: f32,
}

#[derive(Component)]
pub struct TimeActive {
    pub ticks: u32,
}

pub fn temporal_chamber_aging_system(
    mut pops: Query<(&mut Age, &mut FractionalAge, Option<&InsideChamber>)>,
    chambers: Query<&TemporalChamber>,
) {
    for (mut age, mut fractional_age, inside_chamber) in pops.iter_mut() {
        let mut time_factor = 1.0;

        if let Some(inside) = inside_chamber {
            if let Ok(chamber) = chambers.get(inside.chamber_entity) {
                if chamber.active {
                    time_factor = chamber.time_dilation_factor;
                }
            }
        }

        fractional_age.accumulated += time_factor;
        if fractional_age.accumulated >= 1.0 {
            age.ticks += fractional_age.accumulated.floor() as u32;
            fractional_age.accumulated = fractional_age.accumulated.fract();
        }
    }
}

pub fn temporal_chamber_energy_system(
    mut chambers: Query<(&mut TemporalChamber, &mut TimeActive)>,
) {
    for (mut chamber, mut time_active) in chambers.iter_mut() {
        if chamber.active {
            time_active.ticks += 1;
            // Exponential increase
            chamber.energy_cost = 10.0 * (1.01_f32).powi(time_active.ticks as i32);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an `ApplyTemporalShockwave` event to handle catastrophic failure when power runs out.
- Ensure the `FractionalAge` component is added to all Pops at spawn or integrated into the standard `Age` mechanism.
- The `InsideChamber` relation should properly handle pop entrance/exit via pathfinding/jobs.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Chamber effectively slows the aging and need decay of contents.
- [ ] Energy cost grows exponentially while active.

## 7. Technical Guidance
- Time dilation should also apply to `Needs` (hunger, rest), not just age. Add a `Needs` system multiplier.
- Be careful with Bevy's timestep; ensure `FractionalAge` properly accounts for standard delta time if it moves away from tick-based.

## 8. Questions
*Builder: add questions here if spec is unclear.*
