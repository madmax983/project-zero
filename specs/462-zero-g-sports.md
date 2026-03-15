# Specification 462: Zero-G Sports

## 1. Overview
This feature introduces a new "Zero-G Sports" system. The concept is that Pops engage in culturally evolved sports in Low Gravity or Zero-G environments (e.g. specialized arenas or orbital stations). Participating in these sports gives a massive Morale boost for the winner but inflicts injuries or severe exhaustion on the loser. This creates a risk/reward mechanic for dealing with low morale in advanced colonies, while adding cultural flavor to space habitats.

## 2. Dependencies
- `031` Pop Morale (`Morale` component)
- `034` Pop Health and Damage (`Health` component, `apply_damage` system)
- `097` Social Need and Tavern (`Social` interaction base)
- `152` Orbital Stations (Zero-G or Low Gravity environments)

## 3. RED Phase: Tests First

```rust
// tests/layer1/social/zero_g_sports_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::social::zero_g_sports::{ZeroGArena, PlayZeroGSportsAction, zero_g_sports_system};
    use scale::layer1::pop::{PopBundle, Morale, Health};
    use scale::layer1::utility_ai::ActionType;

    fn setup_world() -> World {
        let mut world = World::new();
        world
    }

    #[test]
    fn test_zero_g_sports_applies_buff_and_debuff() {
        let mut world = setup_world();

        let arena = world.spawn(ZeroGArena).id();

        let pop1 = world.spawn((
            PopBundle::default(),
            PlayZeroGSportsAction { target_arena: arena, opponent: None },
        )).id();

        let pop2 = world.spawn((
            PopBundle::default(),
            PlayZeroGSportsAction { target_arena: arena, opponent: Some(pop1) },
        )).id();

        // Update opponent link
        world.get_mut::<PlayZeroGSportsAction>(pop1).unwrap().opponent = Some(pop2);

        let mut schedule = Schedule::default();
        schedule.add_systems(zero_g_sports_system);
        schedule.run(&mut world);

        // One pop should have increased morale, the other decreased health.
        let m1 = world.get::<Morale>(pop1).unwrap().current;
        let h1 = world.get::<Health>(pop1).unwrap().current;

        let m2 = world.get::<Morale>(pop2).unwrap().current;
        let h2 = world.get::<Health>(pop2).unwrap().current;

        // Either pop1 won or pop2 won.
        // A win = Morale > default (50.0). A loss = Health < default (100.0).
        let p1_won = m1 > 50.0 && h1 == 100.0;
        let p2_won = m2 > 50.0 && h2 == 100.0;

        assert!(p1_won || p2_won);

        if p1_won {
            assert!(h2 < 100.0);
        } else {
            assert!(h1 < 100.0);
        }
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/social/zero_g_sports.rs

use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::pop::{Morale, Health, MoodModifier};
use crate::layer1::utility_ai::ActionType;

#[derive(Component, Debug, Clone)]
pub struct ZeroGArena;

#[derive(Component, Debug, Clone)]
pub struct PlayZeroGSportsAction {
    pub target_arena: Entity,
    pub opponent: Option<Entity>,
}

pub fn zero_g_sports_system(
    mut commands: Commands,
    mut query: Query<(Entity, &PlayZeroGSportsAction, &mut Morale, &mut Health)>,
) {
    let mut resolved_matches = Vec::new();

    // First pass: resolve matches and determine winners/losers
    for (entity, action, _, _) in query.iter() {
        if let Some(opponent) = action.opponent {
            // Ensure we only process each pair once
            if entity < opponent && !resolved_matches.contains(&(entity, opponent)) {
                let mut rng = rand::thread_rng();
                let winner = if rng.gen_bool(0.5) { entity } else { opponent };
                let loser = if winner == entity { opponent } else { entity };

                resolved_matches.push((winner, loser));
            }
        }
    }

    // Second pass: apply effects
    for (winner, loser) in resolved_matches {
        if let Ok((_, _, mut morale, _)) = query.get_mut(winner) {
            morale.current = (morale.current + 30.0).min(100.0); // Massive morale boost
            commands.entity(winner).remove::<PlayZeroGSportsAction>();
        }
        if let Ok((_, _, _, mut health)) = query.get_mut(loser) {
            health.current = (health.current - 20.0).max(0.0); // Injury
            commands.entity(loser).remove::<PlayZeroGSportsAction>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Skills System**: Implement a sports/agility skill that influences the win chance rather than a pure 50/50 coin flip.
- **Utility AI Integration**: Pops with low morale should heavily weight `PlayZeroGSportsAction` if an arena is nearby, but avoid it if their health is already low.
- **Spectator Effects**: Other pops in the arena should receive a smaller morale boost just from watching the match.
- **Chronicle Integration**: Emit a `SportsMatchEvent` that the Chronicle system can record if a notable Pop (e.g. Governor) gets injured during a match.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Two Pops engaging in `PlayZeroGSportsAction` result in one gaining Morale and one losing Health.

## 7. Technical Guidance
- The `zero_g_sports_system` currently uses entity IDs to ensure pairs are only processed once (`entity < opponent`). This is a standard and safe way to handle mutual interactions in ECS.
- Ensure the `ZeroGArena` component is actually added to a valid building/zone entity so Pops have a place to pathfind to.
- Remember to register `zero_g_sports_system` in the appropriate schedule (likely `Layer1SystemSet::Execution` or `Observation`).

## 8. Questions
*Builder: add questions here if spec is unclear.*
