# 940 - Commuter's Exile

## 1. Overview
**Layer:** Cross-layer (Layer 1 / 2)
**Fantasy:** The grueling reality of interplanetary commutes causing families to fall apart.
**Mechanic:** Pops living on a high-amenity residential moon but working on an industrial planet spend chunks of their "Lifespan" in sub-light transit. This creates a "Time-Debt" where their family on the moon ages faster in their perceived interaction window, tanking relationship scores between the commuter and their stationary relatives.
**Emergence:** Your core world relies on cheap labor from a nearby orbital ring. Eventually, an entire generation of workers snaps because their children grew up without them due to transit lag. They refuse to return home, setting up a hyper-militarized squatters' camp in the factory zone.
**Tension:** Maximizing efficiency by separating residential luxury from industrial toxicity, versus the psychological toll it takes on the pops who have to bridge that gap every day.

## 2. Dependencies
- Needs Layer 1 `Pop` entity.
- Needs Layer 1 `Relationship` component or system to track family/friend ties.
- Needs Layer 2 `Transit` or equivalent mechanism to represent off-world commuting.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_transit_time_accrues_time_debt() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, accrue_time_debt_system);

        // Spawn a Pop actively in transit
        let commuter = app.world_mut().spawn((
            Pop,
            CommuterTransit { time_in_transit: 5.0 },
            TimeDebt { accumulated: 0.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let debt = app.world().get::<TimeDebt>(commuter).unwrap();
        assert!(debt.accumulated > 0.0, "Time debt should increase while in transit");
    }

    #[test]
    fn test_time_debt_decays_relationships() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, time_debt_relationship_decay_system);

        let commuter = app.world_mut().spawn((
            Pop,
            TimeDebt { accumulated: 10.0 }
        )).id();

        let family_member = app.world_mut().spawn(Pop).id();

        app.world_mut().spawn(Relationship {
            pop_a: commuter,
            pop_b: family_member,
            score: 100.0,
            bond_type: BondType::Family,
        });

        // Act
        app.update();

        // Assert
        let rel = app.world().query::<&Relationship>().iter(app.world()).next().unwrap();
        assert!(rel.score < 100.0, "Relationship score should decrease due to time debt");
    }

    #[test]
    fn test_squatter_revolt_threshold() {
        // Test that high time debt and low relationships eventually trigger a status change
        // preventing them from returning home (Squatter/Exile).
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct CommuterTransit {
    pub time_in_transit: f32,
}

#[derive(Component, Default)]
pub struct TimeDebt {
    pub accumulated: f32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum BondType {
    Family,
    Friend,
}

#[derive(Component)]
pub struct Relationship {
    pub pop_a: Entity,
    pub pop_b: Entity,
    pub score: f32,
    pub bond_type: BondType,
}

pub fn accrue_time_debt_system(
    mut query: Query<(&CommuterTransit, &mut TimeDebt)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (_transit, mut debt) in query.iter_mut() {
        // Assuming every second in transit adds to debt
        debt.accumulated += dt;
    }
}

pub fn time_debt_relationship_decay_system(
    debtors: Query<&TimeDebt>,
    mut relationships: Query<&mut Relationship>,
) {
    let decay_rate = 0.5; // Base decay per unit of debt
    for mut rel in relationships.iter_mut() {
        let mut total_debt = 0.0;
        if let Ok(debt) = debtors.get(rel.pop_a) {
            total_debt += debt.accumulated;
        }
        if let Ok(debt) = debtors.get(rel.pop_b) {
            total_debt += debt.accumulated;
        }

        if total_debt > 0.0 {
            // Apply decay based on accumulated debt
            rel.score -= decay_rate * total_debt;
            rel.score = rel.score.max(0.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring**: Relationship decay shouldn't run every frame. It should run on a periodic tick (e.g., once a day or when transit is completed). `TimeDebt` should probably be cleared or mitigated slightly when spending time at home.
- **Data Structure**: Iterating all relationships linearly is fine for small numbers but could become O(E) where E is the number of edges in the social graph. Ensure relationships are queried efficiently.
- **Code Smell**: `TimeDebt` shouldn't just be raw float seconds; it should map cleanly to the actual simulation time disparity.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops in transit accumulate Time Debt.
- [ ] High Time Debt actively decays `Relationship` scores with stationary Pops.

## 7. Technical Guidance
- **Integration**: Tie this into existing `Movement` or `Assignment` systems. If a Pop's assigned `Workplace` is on a different Node than their `Residence`, flag them with `CommuterTransit` during their travel cycle.
- **Lore hooks**: Trigger `CommuterExileEvent` when relationships hit rock bottom, allowing the Lore system to narrate the creation of a squatter camp.
- **Performance**: Use sparse updates for relationship decay to keep the simulation tick lean.

## 8. Questions
*Builder: add questions here if spec is unclear.*
