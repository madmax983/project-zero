# The Commuter Tax

## 1. Overview
**Layer:** Cross-layer (1, 2)
**Fantasy:** The soul-crushing reality of interplanetary commuting to afford a life back home.
**Mechanic:** Pops can live on an agricultural or low-tech world (Layer 1) where cost of living is extremely low, but work on an adjacent high-tech orbital ring or industrial world (Layer 2) where wages are high. They travel daily or weekly via mass transit ships. However, if hyperlane traffic or local orbital debris slows the transit times, their "Commuter Tax" debuff increases. They spend half their life in transit, drastically reducing their maximum rest and leisure, leading to chronic depression and reduced life expectancy.

## 2. Dependencies
- Layer 1 Population System
- Layer 2 Fleet/Transit System
- Needs and Morale System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_commuter_tax_increases_with_delay() {
        // Arrange
        let mut app = App::new();
        let pop = app.world_mut().spawn((
            Pop,
            Commuter { transit_delay: 5.0, base_commute_time: 2.0 },
            Needs { rest: 1.0, ..Default::default() }
        )).id();

        // Act
        app.add_systems(Update, apply_commuter_tax_system);
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.rest < 1.0, "Extended commute should reduce rest need");
    }

    #[test]
    fn test_commuter_tax_causes_depression_debuff() {
        // Arrange
        let mut app = App::new();
        let pop = app.world_mut().spawn((
            Pop,
            Commuter { transit_delay: 20.0, base_commute_time: 2.0 }, // Massive delay
            Needs { rest: 0.1, ..Default::default() }
        )).id();

        // Act
        app.add_systems(Update, apply_commuter_tax_system);
        app.update();

        // Assert
        assert!(app.world().get::<Depressed>(pop).is_some(), "Massive commute delays should cause depression");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component, Default)]
pub struct Needs {
    pub rest: f32,
    pub morale: f32,
}

#[derive(Component)]
pub struct Commuter {
    pub transit_delay: f32,
    pub base_commute_time: f32,
}

#[derive(Component)]
pub struct Depressed;

pub fn apply_commuter_tax_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Commuter, &mut Needs)>,
) {
    for (entity, commuter, mut needs) in query.iter_mut() {
        let total_commute = commuter.base_commute_time + commuter.transit_delay;

        if total_commute > commuter.base_commute_time {
            // Deduct from rest based on delay
            let penalty = (commuter.transit_delay * 0.05).min(0.5);
            needs.rest = (needs.rest - penalty).max(0.0);

            // Extreme delay causes depression
            if commuter.transit_delay > 10.0 {
                commands.entity(entity).insert(Depressed);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `transit_delay` should be dynamic, driven by actual Layer 2 `TransitShip` delays rather than a hardcoded value on the Pop.
- Implement varying degrees of `Depressed` or `Exhausted` debuffs.
- Consider economic factors: long commutes should also impact the `Wage` or `Credits` if they miss their shifts.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Transit delays reduce Pop rest/morale needs.
- [ ] Excessive delays apply negative status components.

## 7. Technical Guidance
- **Transit Calculation:** You'll likely need a system that maps `TransitRoute` delays to the specific `Commuter` pops using that route.
- **Cross-Layer:** Ensure Layer 1 handles the resulting riots and Layer 2 handles the traffic jams concurrently.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
