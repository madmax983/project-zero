# 263: The Cadet Branch

## 1. Overview
**Layer:** 3 -> 1
**Fantasy:** Babysitting the Emperor's nephew.
**Mechanic:** You accept "Noble Scions" from the Homeworld. They have terrible stats and "Snob" traits but come with a monthly "Allowance" (Funding) from their rich families. If they die, funding stops and relations tank.
**Emergence:** You build a luxurious, safe playground for the idiots just to keep the funding flowing, while the real workers live in squalor.
**Tension:** Free money vs. Incompetent/High-maintenance population.

## 2. Dependencies
- Layer 1 `Pop` and Needs system (for high-maintenance).
- Economy/Credits system (for the Allowance).
- Time/Simulation Tick (for monthly income).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::economy::ColonyWealth;

    #[test]
    fn test_noble_scion_provides_allowance() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_allowance_system);
        app.insert_resource(ColonyWealth { credits: 0.0 });

        // Spawn a Noble Scion
        app.world_mut().spawn((
            Pop { skill_level: 1 },
            NobleScion { monthly_allowance: 500.0 },
        ));

        // Act - Simulate a month passing (event or timer trigger)
        app.world_mut().send_event(MonthTickEvent);
        app.update();

        // Assert
        let wealth = app.world().resource::<ColonyWealth>();
        assert_eq!(wealth.credits, 500.0, "The colony should receive the noble's allowance.");
    }

    #[test]
    fn test_noble_scion_death_stops_allowance() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_allowance_system);
        app.insert_resource(ColonyWealth { credits: 0.0 });

        // Spawn a Noble Scion
        let scion_entity = app.world_mut().spawn((
            Pop { skill_level: 1 },
            NobleScion { monthly_allowance: 500.0 },
        )).id();

        // Kill the scion
        app.world_mut().entity_mut(scion_entity).despawn();

        // Act
        app.world_mut().send_event(MonthTickEvent);
        app.update();

        // Assert
        let wealth = app.world().resource::<ColonyWealth>();
        assert_eq!(wealth.credits, 0.0, "Dead nobles pay no allowance.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::economy::ColonyWealth;

#[derive(Component)]
pub struct NobleScion {
    pub monthly_allowance: f32,
}

#[derive(Event)]
pub struct MonthTickEvent;

pub fn process_allowance_system(
    mut events: EventReader<MonthTickEvent>,
    mut wealth: ResMut<ColonyWealth>,
    nobles: Query<&NobleScion>,
) {
    for _ in events.read() {
        let mut total_allowance = 0.0;
        for noble in nobles.iter() {
            total_allowance += noble.monthly_allowance;
        }
        wealth.credits += total_allowance;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Combine the monthly tick check with existing temporal cycle systems (`SimulationTime`) rather than a bespoke `MonthTickEvent` if one already exists.
- **Code Smells:** Hardcoding allowance values in spawners. Consider a `NobleScionBundle` that sets random low stats and random high allowances.
- **Integration Points:** Hook into the `DeathEvent` or similar cleanup system to trigger a "Relations Dropped" notification when the scion dies.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The `NobleScion` component correctly provides recurring income.
- [ ] Income ceases immediately if the entity is despawned.

## 7. Technical Guidance
- The scion's demanding nature should be handled by giving them unique traits (e.g., `Snob`) that accelerate their need decay for luxuries, leveraging the existing `Needs` system.

## 8. Questions
*Builder: add questions here if spec is unclear.*
