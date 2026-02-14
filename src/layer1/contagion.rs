use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

/// Component to prevent contagion spam from a single source.
#[derive(Component, Default)]
pub struct ContagionCooldown {
    /// Ticks remaining until the pop can spread emotion again.
    pub timer: u32,
}

const CONTAGION_RANGE: i32 = 5;
const CONTAGION_COOLDOWN: u32 = 200;
const LOW_MORALE_THRESHOLD: f32 = 0.2;
const HIGH_MORALE_THRESHOLD: f32 = 0.9;

/// System to spread strong emotions (Joy/Breakdown) to nearby pops.
pub fn emotional_contagion_system(
    mut _commands: Commands,
    mut pops: Query<(Entity, &GridPosition, &mut Morale, &mut ContagionCooldown)>,
) {
    // 1. Identify Sources
    let mut sources = Vec::new();
    for (entity, pos, morale, mut cooldown) in &mut pops {
        if cooldown.timer > 0 {
            cooldown.timer -= 1;
            continue;
        }

        if morale.value <= LOW_MORALE_THRESHOLD {
            sources.push((entity, *pos, "Witnessed Breakdown", -0.05));
            cooldown.timer = CONTAGION_COOLDOWN;
        } else if morale.value >= HIGH_MORALE_THRESHOLD {
            sources.push((entity, *pos, "Witnessed Joy", 0.05));
            cooldown.timer = CONTAGION_COOLDOWN;
        }
    }

    if sources.is_empty() {
        return;
    }

    // 2. Apply to Targets
    for (target_entity, target_pos, mut morale, _) in &mut pops {
        for (source_entity, source_pos, label, value) in &sources {
            if *source_entity == target_entity {
                continue;
            }

            // Check distance (Chebyshev)
            let dist = (target_pos.x - source_pos.x)
                .abs()
                .max((target_pos.y - source_pos.y).abs());

            if dist <= CONTAGION_RANGE {
                morale.modifiers.push(MoodModifier {
                    label: label.to_string(),
                    value: *value,
                    duration: 100, // Short duration
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::contagion::{ContagionCooldown, emotional_contagion_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use bevy_ecs::schedule::Schedule;
    use bevy_ecs::world::World;

    #[test]
    fn test_contagion_spreads_negative_mood() {
        let mut world = World::new();

        // 1. Create Source Pop (Low Morale)
        let _source = world
            .spawn((
                GridPosition { x: 10, y: 10 },
                Morale {
                    value: 0.1,
                    modifiers: vec![],
                }, // Very Low
                ContagionCooldown::default(),
            ))
            .id();

        // 2. Create Target Pop (Neutral Morale, nearby)
        let target = world
            .spawn((
                GridPosition { x: 11, y: 10 }, // Adjacent
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                }, // Neutral
                ContagionCooldown::default(),
            ))
            .id();

        // 3. Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(emotional_contagion_system);
        schedule.run(&mut world);

        // 4. Assert Target received negative modifier
        let target_morale = world.get::<Morale>(target).unwrap();
        assert!(
            target_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Witnessed Breakdown"),
            "Target should have 'Witnessed Breakdown' modifier"
        );
    }

    #[test]
    fn test_contagion_spreads_positive_mood() {
        let mut world = World::new();

        // 1. Create Source Pop (High Morale)
        let _source = world
            .spawn((
                GridPosition { x: 10, y: 10 },
                Morale {
                    value: 0.95,
                    modifiers: vec![],
                }, // Very High
                ContagionCooldown::default(),
            ))
            .id();

        // 2. Create Target Pop (Neutral Morale, nearby)
        let target = world
            .spawn((
                GridPosition { x: 11, y: 11 }, // Diagonal
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                ContagionCooldown::default(),
            ))
            .id();

        // 3. Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(emotional_contagion_system);
        schedule.run(&mut world);

        // 4. Assert Target received positive modifier
        let target_morale = world.get::<Morale>(target).unwrap();
        assert!(
            target_morale
                .modifiers
                .iter()
                .any(|m| m.label == "Witnessed Joy"),
            "Target should have 'Witnessed Joy' modifier"
        );
    }

    #[test]
    fn test_contagion_range_limit() {
        let mut world = World::new();

        // Source
        let _source = world
            .spawn((
                GridPosition { x: 10, y: 10 },
                Morale {
                    value: 0.05,
                    modifiers: vec![],
                },
                ContagionCooldown::default(),
            ))
            .id();

        // Distant Target (Outside range, e.g., range is 5)
        let distant_target = world
            .spawn((
                GridPosition { x: 20, y: 20 },
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                ContagionCooldown::default(),
            ))
            .id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(emotional_contagion_system);
        schedule.run(&mut world);

        // Assert NO modifier
        let target_morale = world.get::<Morale>(distant_target).unwrap();
        assert!(
            target_morale.modifiers.is_empty(),
            "Distant target should not be affected"
        );
    }

    #[test]
    fn test_contagion_cooldown() {
        let mut world = World::new();

        // Source
        let _source = world
            .spawn((
                GridPosition { x: 10, y: 10 },
                Morale {
                    value: 0.05,
                    modifiers: vec![],
                },
                ContagionCooldown { timer: 100 }, // Recently triggered
            ))
            .id();

        // Target
        let target = world
            .spawn((
                GridPosition { x: 11, y: 10 },
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                ContagionCooldown::default(),
            ))
            .id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(emotional_contagion_system);
        schedule.run(&mut world);

        // Assert NO modifier (Source on cooldown)
        let target_morale = world.get::<Morale>(target).unwrap();
        assert!(
            target_morale.modifiers.is_empty(),
            "Source on cooldown should not spread emotion"
        );
    }
}
