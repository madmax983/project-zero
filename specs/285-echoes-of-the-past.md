# 285: Echoes of the Past

## 1. Overview
A colony haunted by its predecessors.

**Layer:** 1

**Fantasy:** When settling on ruins or ancient battlefields, "Echoes" of past inhabitants occasionally manifest as holographic ghosts. They provide huge XP bonuses to Pops studying them but rapidly increase Stress due to the eerie environment.

**Mechanic:** Some maps or specific ruined tiles contain `EchoSource` components. Randomly, these sources emit `Echo` entities (non-corporeal ghosts). Pops near an `Echo` gain a massive boost to their `Skills` XP gain but simultaneously accumulate `Stress` at a highly accelerated rate. Prolonged exposure can lead to madness or strange cult formations.

**Emergence:** Your researchers discover incredible lost tech by following the ghosts, but the prolonged exposure drives them mad, leading to a faction that worships the dead and demands you stop all new construction.

**Tension:** Massive knowledge gains vs. severe psychological deterioration.

## 2. Dependencies
- **051 Pop Skills and Experience:** Needs the skill/XP system.
- **127 Stress Breakdowns:** Needs the stress/mental health system.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Skills, SkillType};
    use crate::layer1::needs::StressTracker;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_echo_spawns_from_source() {
        let mut world = World::new();
        // Create an EchoSource
        let source = world.spawn((EchoSource { spawn_chance: 1.0 }, GridPosition { x: 5, y: 5 })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_spawn_system);
        schedule.run(&mut world);

        let mut echo_query = world.query::<&Echo>();
        assert_eq!(echo_query.iter(&world).count(), 1, "An Echo should have spawned from the source.");
    }

    #[test]
    fn test_pop_near_echo_gains_bonus_xp_and_stress() {
        let mut world = World::new();
        // Spawn an Echo
        world.spawn((Echo { radius: 2.0 }, GridPosition { x: 5, y: 5 }));

        // Spawn a Pop nearby
        let pop = world.spawn((
            GridPosition { x: 6, y: 5 },
            Skills::default(),
            StressTracker::default()
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_aura_system);

        // Run a few ticks
        for _ in 0..5 {
            schedule.run(&mut world);
        }

        let skills = world.get::<Skills>(pop).unwrap();
        let stress = world.get::<StressTracker>(pop).unwrap();

        // The pop is technically idle in this test but the aura provides passive XP to the "Science" or general pool
        assert!(skills.get_xp(SkillType::Science) > 0.0, "Pop should gain XP from being near the Echo.");
        assert!(stress.accumulated_stress > 0.0, "Pop should gain stress from being near the Echo.");
    }

    #[test]
    fn test_pop_far_from_echo_unaffected() {
        let mut world = World::new();
        // Spawn an Echo
        world.spawn((Echo { radius: 2.0 }, GridPosition { x: 5, y: 5 }));

        // Spawn a Pop far away
        let pop = world.spawn((
            GridPosition { x: 20, y: 20 },
            Skills::default(),
            StressTracker::default()
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(echo_aura_system);
        schedule.run(&mut world);

        let skills = world.get::<Skills>(pop).unwrap();
        let stress = world.get::<StressTracker>(pop).unwrap();

        assert_eq!(skills.get_xp(SkillType::Science), 0.0, "Pop far away should not gain XP.");
        assert_eq!(stress.accumulated_stress, 0.0, "Pop far away should not gain stress.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Skills, SkillType};
use crate::layer1::needs::StressTracker;

#[derive(Component)]
pub struct EchoSource {
    pub spawn_chance: f32, // e.g. 0.01 per tick
}

#[derive(Component)]
pub struct Echo {
    pub radius: f32,
}

pub fn echo_spawn_system(
    mut commands: Commands,
    query: Query<(&EchoSource, &GridPosition)>,
) {
    for (source, pos) in query.iter() {
        // Simplified chance logic for MVP
        if source.spawn_chance >= 1.0 {
            commands.spawn((
                Echo { radius: 3.0 },
                *pos,
            ));
        }
    }
}

pub fn echo_aura_system(
    echoes: Query<(&Echo, &GridPosition)>,
    mut pops: Query<(&GridPosition, &mut Skills, &mut StressTracker)>,
) {
    for (pop_pos, mut skills, mut stress) in pops.iter_mut() {
        for (echo, echo_pos) in echoes.iter() {
            let dx = (pop_pos.x as f32 - echo_pos.x as f32).abs();
            let dy = (pop_pos.y as f32 - echo_pos.y as f32).abs();
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= echo.radius {
                // Give massive XP boost to a generic/research skill
                skills.add_xp(SkillType::Science, 10.0);
                // Apply severe stress
                stress.accumulate(5.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Action Context:** The XP boost shouldn't just be passive. It should act as a massive multiplier to the `XP_GAIN_RATE` if the Pop is actively performing an `ActionType::Research` or `ActionType::Study` near the echo.
- **Echo Lifecycle:** Echoes should be transient. They spawn, wander slightly, and then despawn after a set duration.
- **Line of Sight:** Consider adding line of sight checks so Echoes don't cause stress through thick walls (unless they can pass through walls, which fits the ghost fantasy but might be too punishing).
- **Insanity Cult:** Tie prolonged exposure to a high chance of acquiring a `Cultist` trait that forces the Pop to spend their free time near `EchoSource` tiles.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new `echoes.rs` module.
- [ ] Pops near an `Echo` gain significant XP and Stress.
- [ ] Pops far away are unaffected.

## 7. Technical Guidance
- Create `src/layer1/social/echoes.rs`.
- Register systems in `src/layer1/systems/social.rs` or `observation.rs`.
- Link this with the `ActionType::Study` if available to make the XP gain conditional on the Pop intentionally interacting with the Echo, while the stress is a passive aura.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
