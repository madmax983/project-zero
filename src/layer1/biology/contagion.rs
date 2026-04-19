use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
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
const HIGH_MORALE_THRESHOLD: f32 = 0.8;

/// System to spread strong emotions (Joy/Terror) to nearby pops.
pub fn emotional_contagion_system(
    mut pops: Query<(Entity, &GridPosition, &mut Needs, &mut ContagionCooldown)>,
) {
    // 1. Identify Sources
    let mut sources = Vec::new();
    for (entity, pos, needs, mut cooldown) in &mut pops {
        if cooldown.timer > 0 {
            cooldown.timer -= 1;
            continue;
        }

        let morale = needs.morale();

        if morale <= LOW_MORALE_THRESHOLD {
            sources.push((entity, *pos, -0.05));
            cooldown.timer = CONTAGION_COOLDOWN;
        } else if morale >= HIGH_MORALE_THRESHOLD {
            sources.push((entity, *pos, 0.05));
            cooldown.timer = CONTAGION_COOLDOWN;
        }
    }

    if sources.is_empty() {
        return;
    }

    // 2. Apply to Targets
    for (target_entity, target_pos, mut needs, _) in &mut pops {
        for (source_entity, source_pos, value) in &sources {
            if *source_entity == target_entity {
                continue;
            }

            // Check distance (Chebyshev)
            let dist = (target_pos.x - source_pos.x)
                .abs()
                .max((target_pos.y - source_pos.y).abs());

            if dist <= CONTAGION_RANGE {
                needs.leisure = (needs.leisure + *value).clamp(0.0, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_emotional_contagion_spreads_joy() {
        // Arrange
        let mut world = World::new();
        let _happy_pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 1.0,
                    rest: 1.0,
                    hunger: 1.0,
                    hygiene: 1.0,
                },
                GridPosition { x: 0, y: 0 },
                ContagionCooldown::default(),
            ))
            .id();
        let neutral_pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    hunger: 0.5,
                    hygiene: 0.5,
                },
                GridPosition { x: 1, y: 0 },
                ContagionCooldown::default(),
            ))
            .id();

        // Act
        world.run_system_once(emotional_contagion_system).expect("Component should exist or System should run");

        // Assert
        let needs = world.get::<Needs>(neutral_pop).expect("Component should exist or System should run");
        assert!(
            needs.leisure > 0.5,
            "Neutral pop should gain leisure from nearby happy pop"
        );
    }

    #[test]
    fn test_emotional_contagion_spreads_terror() {
        // Arrange
        let mut world = World::new();
        let _terrified_pop = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.1,
                    leisure: 0.1,
                    hunger: 0.1,
                    hygiene: 0.1,
                },
                GridPosition { x: 0, y: 0 },
                ContagionCooldown::default(),
            ))
            .id();
        let neutral_pop = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.8,
                    leisure: 0.8,
                    hunger: 0.8,
                    hygiene: 0.8,
                },
                GridPosition { x: 1, y: 0 },
                ContagionCooldown::default(),
            ))
            .id();

        // Act
        world.run_system_once(emotional_contagion_system).expect("Component should exist or System should run");

        // Assert
        let needs = world.get::<Needs>(neutral_pop).expect("Component should exist or System should run");
        assert!(
            needs.leisure < 0.8,
            "Neutral pop should lose leisure from nearby terrified pop"
        );
    }

    #[test]
    fn test_emotional_contagion_range_limit() {
        // Arrange
        let mut world = World::new();
        let _happy_pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 1.0,
                    rest: 1.0,
                    hunger: 1.0,
                    hygiene: 1.0,
                },
                GridPosition { x: 0, y: 0 },
                ContagionCooldown::default(),
            ))
            .id();
        let far_pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    hunger: 0.5,
                    hygiene: 0.5,
                },
                GridPosition { x: 10, y: 10 },
                ContagionCooldown::default(),
            ))
            .id();

        // Act
        world.run_system_once(emotional_contagion_system).expect("Component should exist or System should run");

        // Assert
        let needs = world.get::<Needs>(far_pop).expect("Component should exist or System should run");
        assert_eq!(needs.leisure, 0.5, "Far pop should be unaffected");
    }

    #[test]
    fn test_contagion_cooldown() {
        let mut world = World::new();

        // Source pop on cooldown
        let _source = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.1,
                    rest: 0.1,
                    hunger: 0.1,
                    hygiene: 0.1,
                },
                GridPosition { x: 0, y: 0 },
                ContagionCooldown { timer: 100 }, // Recently triggered
            ))
            .id();

        // Target
        let target = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    hunger: 0.5,
                    hygiene: 0.5,
                },
                GridPosition { x: 1, y: 0 },
                ContagionCooldown::default(),
            ))
            .id();

        // Act
        world.run_system_once(emotional_contagion_system).expect("Component should exist or System should run");

        // Assert NO modifier (Source on cooldown)
        let needs = world.get::<Needs>(target).expect("Component should exist or System should run");
        assert_eq!(
            needs.leisure, 0.5,
            "Source on cooldown should not spread emotion"
        );
    }
}
