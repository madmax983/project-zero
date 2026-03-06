# Spec 343: The Rumor Web

## 1. Overview
The colony feels alive when information travels person-to-person. Pops carry "knowledge tokens" (e.g., resource locations, job openings, events). When pops are adjacent, they swap tokens. Some tokens are "False Rumors," causing panic or bad decisions. This makes information spread a physical phenomenon in the colony.

## 2. Dependencies
- Spatial proximity checks (Grid/Transform)
- Pop `Needs` and `Traits` (to generate false rumors based on mood)
- Job or Task assignment system (to be affected by rumors)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_adjacent_pops_exchange_rumors() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, spread_rumors_system);

        let pop1 = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            RumorTracker {
                rumors: vec![Rumor::FoodShortage],
            },
        )).id();

        let pop2 = app.world_mut().spawn((
            Transform::from_xyz(1.0, 0.0, 0.0), // Adjacent
            RumorTracker {
                rumors: vec![],
            },
        )).id();

        // Act
        app.update();

        // Assert
        let tracker2 = app.world().get::<RumorTracker>(pop2).unwrap();
        assert!(tracker2.rumors.contains(&Rumor::FoodShortage));
    }

    #[test]
    fn test_false_rumor_triggers_panic() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, react_to_rumors_system);

        let pop = app.world_mut().spawn((
            RumorTracker {
                rumors: vec![Rumor::FalseMutantAttack],
            },
            Needs::default(), // Assuming stress/panic is part of Needs
        )).id();

        // Act
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        // The panic should significantly increase stress or lower morale
        assert!(needs.stress > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Rumor {
    FoodShortage,
    JobOpening(Entity),
    FalseMutantAttack,
}

#[derive(Component, Default)]
pub struct RumorTracker {
    pub rumors: Vec<Rumor>,
}

#[derive(Component, Default)]
pub struct Needs {
    pub stress: f32,
    pub morale: f32,
}

pub fn spread_rumors_system(
    mut query: Query<(Entity, &Transform, &mut RumorTracker)>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(e1, t1, mut r1), (e2, t2, mut r2)]) = combinations.fetch_next() {
        if t1.translation.distance(t2.translation) < 1.5 { // Adjacent
            let new_r1: Vec<Rumor> = r2.rumors.clone();
            let new_r2: Vec<Rumor> = r1.rumors.clone();

            for rumor in new_r1 {
                if !r1.rumors.contains(&rumor) { r1.rumors.push(rumor); }
            }
            for rumor in new_r2 {
                if !r2.rumors.contains(&rumor) { r2.rumors.push(rumor); }
            }
        }
    }
}

pub fn react_to_rumors_system(
    mut query: Query<(&RumorTracker, &mut Needs)>,
) {
    for (tracker, mut needs) in query.iter_mut() {
        if tracker.rumors.contains(&Rumor::FalseMutantAttack) {
            needs.stress += 10.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Partitioning**: The `iter_combinations_mut` is $O(N^2)$ and will kill performance. MUST refactor to use a spatial grid or KD-tree to only check actual neighbors.
- **Rumor Decay**: Rumors should have a time-to-live (TTL) so they don't stay on a pop forever.
- **Traits Influence**: Pops with "Gossip" trait spread rumors further; "Skeptic" trait pops have a chance to destroy false rumors.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for rumor spread and reaction.
- [ ] O(N^2) combination checking is avoided using the spatial grid.

## 7. Technical Guidance
- **Layer**: Layer 1 (Colony/Pop scale).
- Utilize the existing Grid implementation (e.g., `ColonyGrid` or similar) to find neighbors quickly instead of checking distance between all pops.
- Rumors could be a powerful tool for the player to intentionally seed, e.g., spending influence to seed a "Morale Boost" rumor.

## 8. Questions
*Builder: add questions here if spec is unclear.*
