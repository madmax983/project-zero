# 727 The Empathic Plague

## 1. Overview
A unique pathogen heightens the empathy of infected Pops. Instead of physical symptoms, they experience the negative moods (Hunger, Exhaustion, Unrest) of all Pops within a certain radius. Localized pockets of misery can trigger massive, colony-wide morale collapses.

## 2. Dependencies
- 005 Pop needs (hunger, rest)
- 031 Pop Morale
- 034 Pop Health and Damage

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_empathic_plague_spreads_negative_mood() {
        let mut world = World::new();
        // Setup infected pop and nearby miserable pop
        let infected_pop = world.spawn((Pop, Transform::from_xyz(0.0, 0.0, 0.0), EmpathicInfection, Morale { current: 100.0 })).id();
        world.spawn((Pop, Transform::from_xyz(1.0, 0.0, 0.0), Hunger { current: 90.0 }, Morale { current: 20.0 }));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_empathic_resonance);
        schedule.run(&mut world);

        let infected_morale = world.get::<Morale>(infected_pop).unwrap();

        assert!(infected_morale.current < 100.0, "Infected pop should lose morale due to nearby miserable pop");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct EmpathicInfection;

#[derive(Component)]
pub struct Morale {
    pub current: f32,
}

#[derive(Component)]
pub struct Hunger {
    pub current: f32, // high means very hungry
}

pub fn process_empathic_resonance(
    mut infected_query: Query<(&Transform, &mut Morale), (With<Pop>, With<EmpathicInfection>)>,
    others_query: Query<(&Transform, &Morale, &Hunger), (With<Pop>, Without<EmpathicInfection>)>,
) {
    for (infected_transform, mut infected_morale) in infected_query.iter_mut() {
        let mut resonance_penalty = 0.0;

        for (other_transform, other_morale, other_hunger) in others_query.iter() {
            let distance = infected_transform.translation.distance(other_transform.translation);
            if distance < 5.0 && (other_morale.current < 50.0 || other_hunger.current > 80.0) {
                resonance_penalty += 2.0;
            }
        }

        infected_morale.current -= resonance_penalty;
        if infected_morale.current < 0.0 {
            infected_morale.current = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Spatial querying could be optimized using a grid or spatial partitioning (like KD-Tree) if pop counts are high.
- The infection should have a chance to spread based on proximity.

## 6. Acceptance Criteria
- [ ] `process_empathic_resonance` accurately decreases infected pops' morale based on nearby miserable pops.
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- Ensure distance checks are efficient to prevent O(N^2) performance hits during updates.
- Empathic impact should probably be a gradual change over time rather than instant steps.

## 8. Questions
*Builder: add questions here if spec is unclear.*
