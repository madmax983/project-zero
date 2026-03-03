# 255: Hypno-Learning

## Overview

"I know Kung Fu."

**Hypno-Learning** introduces a high-tech alternative to standard beds: the **HypnoPod**. Pops assigned to sleep in these pods undergo rapid neural restructuring. They gain Skill XP significantly faster than through practice, but this comes at a physical and mental cost.

Upon waking, the Pop suffers from **Mental Fog**, a temporary condition that severely reduces Movement Speed and Work Speed. They also wake up with significantly higher **Hunger** than normal sleep (the brain burns calories).

This creates a tension: rapid up-skilling vs. immediate combat/work readiness. Do you train a master soldier overnight if they will be too groggy to fight effectively in the morning?

## Dependencies

- `051` — Pop Skills (XP gain logic)
- `007` — Housing (Bed functionality)
- `005` — Pop Needs (Hunger mechanics)
- `031` — Pop Morale (Mental states/effects)

## RED Phase: Tests First

Write these tests in `src/layer1/tech/hypno_learning_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Needs, PopState};
    use crate::layer1::skills::{Skill, SkillType, Skills};
    use crate::layer1::tech::hypno_learning::{HypnoPod, MentalFog, hypno_sleep_system, wake_up_hypno_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::Building;

    #[test]
    fn test_hypno_pod_grants_xp_while_sleeping() {
        let mut world = World::new();
        // Spawn HypnoPod with target skill
        let pod = world.spawn(HypnoPod {
            target_skill: SkillType::Mining,
            xp_rate: 10.0,
        }).id();

        // Spawn Pop sleeping in the pod
        let pop = world.spawn((
            Pop,
            Skills::default(),
            PopState::Sleeping { bed: Some(pod) }, // Assuming PopState stores bed entity
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify XP gain
        let skills = world.get::<Skills>(pop).unwrap();
        let mining_xp = skills.get_xp(SkillType::Mining);
        assert!(mining_xp >= 10.0);
    }

    #[test]
    fn test_hypno_sleep_drains_hunger_faster() {
        let mut world = World::new();
        let pod = world.spawn(HypnoPod::default()).id();

        let pop = world.spawn((
            Pop,
            Needs { hunger: 100.0, ..Default::default() },
            PopState::Sleeping { bed: Some(pod) },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(hypno_sleep_system);
        schedule.run(&mut world);

        // Verify extra hunger drain (Standard sleep might consume X, Hypno consumes X + Y)
        let needs = world.get::<Needs>(pop).unwrap();
        // Assuming normal decay is handled elsewhere or we model the delta
        // We expect a significant drop.
        assert!(needs.hunger < 99.0); // Assuming decay rate > 1.0 per tick for hypno
    }

    #[test]
    fn test_waking_from_hypno_applies_mental_fog() {
        let mut world = World::new();
        let pod = world.spawn(HypnoPod::default()).id();

        // Pop was sleeping in pod, now transitions to Idle (Waking up)
        // This requires tracking previous state or intercepting the state change event.
        // For GREEN phase simplicity, we might iterate Pops who *just* stopped sleeping in a pod.
        // Or we use an event `WakeUpEvent`.

        // Let's assume we send a WakeUpEvent.
        world.insert_resource(Events::<crate::layer1::pop::WakeUpEvent>::default());

        let pop = world.spawn((
            Pop,
            // Marker component or check event data
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(wake_up_hypno_system);

        // Send event saying Pop woke up from Pod
        world.send_event(crate::layer1::pop::WakeUpEvent {
            entity: pop,
            bed_entity: Some(pod),
        });

        schedule.run(&mut world);

        // Verify MentalFog component
        let fog = world.get::<MentalFog>(pop);
        assert!(fog.is_some());
        assert!(fog.unwrap().duration > 0.0);
    }

    #[test]
    fn test_mental_fog_penalizes_movement_and_work() {
        // This test verifies the effect of the component, usually in movement/work systems.
        // For this spec, we just verify the component properties imply penalties.
        let fog = MentalFog {
            duration: 10.0,
            movement_penalty: 0.5,
            work_speed_penalty: 0.5,
        };

        assert_eq!(fog.movement_penalty, 0.5);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/tech/hypno_learning.rs

use bevy_ecs::prelude::*;
use crate::layer1::skills::SkillType;

#[derive(Component, Debug, Clone, Default)]
pub struct HypnoPod {
    pub target_skill: SkillType, // Default to something or allow "General" learning
    pub xp_rate: f32, // XP per tick
}

#[derive(Component, Debug, Clone, Default)]
pub struct MentalFog {
    pub duration: f32, // Ticks or Seconds
    pub movement_penalty: f32, // 0.0 to 1.0 (multiplier)
    pub work_speed_penalty: f32,
}
```

### 2. Systems

```rust
use crate::layer1::pop::{Pop, Needs, PopState};
use crate::layer1::skills::Skills;

pub fn hypno_sleep_system(
    mut pops: Query<(&mut Skills, &mut Needs, &PopState), With<Pop>>,
    pods: Query<&HypnoPod>,
) {
    for (mut skills, mut needs, state) in pops.iter_mut() {
        if let PopState::Sleeping { bed: Some(bed_entity) } = state {
            if let Ok(pod) = pods.get(*bed_entity) {
                // Grant XP
                skills.add_xp(pod.target_skill, pod.xp_rate);

                // Drain Hunger extra
                needs.hunger = (needs.hunger - 0.5).max(0.0); // Arbitrary drain rate
            }
        }
    }
}

use crate::layer1::pop::WakeUpEvent; // Assuming event exists or create it

pub fn wake_up_hypno_system(
    mut commands: Commands,
    mut events: EventReader<WakeUpEvent>,
    pods: Query<&HypnoPod>,
) {
    for event in events.read() {
        if let Some(bed) = event.bed_entity {
            if pods.get(bed).is_ok() {
                // Apply Fog
                commands.entity(event.entity).insert(MentalFog {
                    duration: 1000.0, // Long duration
                    movement_penalty: 0.5, // Half speed
                    work_speed_penalty: 0.5,
                });
            }
        }
    }
}

pub fn update_mental_fog_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MentalFog)>,
) {
    for (entity, mut fog) in query.iter_mut() {
        fog.duration -= 1.0;
        if fog.duration <= 0.0 {
            commands.entity(entity).remove::<MentalFog>();
        }
    }
}
```

### 3. Integration Points

- **Movement System**: Needs to query `MentalFog` and apply `movement_penalty`.
- **Work System**: Needs to query `MentalFog` and apply `work_speed_penalty`.
- **UI**: Add `HypnoPod` to building menu. Add UI to select `target_skill` on the pod.

## REFACTOR Phase: Quality & Design

- **Skill Selection**: `HypnoPod` should have a UI or logic to set the `target_skill`. Maybe defaults to the Pop's highest passion, or player-set.
- **Fog Severity**: Fog duration could scale with the amount of XP gained (longer sleep = worse fog).
- **Dreams**: Integration with `Spec 195 (Cryo-Dreams)`—Hypno-learning might trigger vivid dreams or nightmares.
- **Safety**: Chance of "Brain Burn" (Skill loss or Trauma) if used too frequently or while stressed.

## Acceptance Criteria

- [ ] `HypnoPod` building exists.
- [ ] Pops sleeping in `HypnoPod` gain XP in the assigned skill.
- [ ] Pops sleeping in `HypnoPod` lose Hunger faster than normal sleep.
- [ ] Waking up from `HypnoPod` applies `MentalFog` component.
- [ ] `MentalFog` decays over time.
- [ ] Tests pass.

## Technical Guidance

- If `WakeUpEvent` does not exist in `005` or `031`, hook into the transition logic in `sleep_system`. Ideally, define the event in `src/layer1/pop.rs` if needed.
- Ensure `MentalFog` penalty is multiplicative (e.g., `speed *= fog.movement_penalty`).

## Questions

- *Builder: Does the pod require power?*
- *Architect:* Yes, high power consumption while active.
- *Builder: Can children use it?*
- *Architect:* Yes, allowing them to rapidly gain adult skills.
