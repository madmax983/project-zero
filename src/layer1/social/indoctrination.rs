use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ethic {
    #[default]
    Pacifist,
    Militarist,
    Collectivist,
    Individualist,
    FreeThinker,
    StateLoyalist,
}

#[derive(Component, Debug, Clone, Default)]
pub struct PopEthics {
    pub ethic: Ethic,
    pub stubbornness: f32, // 0.0 to 1.0
}

#[derive(Component, Debug, Clone)]
pub struct IndoctrinationAura {
    pub target_ethic: Ethic,
    pub strength: f32,
    pub radius: f32,
}

pub fn process_indoctrination_system(
    time: Res<crate::shared::time::SimulationTime>,
    auras: Query<(&IndoctrinationAura, &GridPosition)>,
    mut pops: Query<(
        &mut PopEthics,
        &GridPosition,
        Option<&mut crate::layer1::needs::Needs>,
    )>,
) {
    if time.tick == 0 {
        return;
    }

    for (aura, aura_pos) in auras.iter() {
        for (mut ethics, pop_pos, mut needs) in pops.iter_mut() {
            let dist = crate::layer1::pathfinding::manhattan_distance(
                (aura_pos.x, aura_pos.y),
                (pop_pos.x, pop_pos.y),
            );
            if dist as f32 <= aura.radius {
                if ethics.ethic == aura.target_ethic {
                    continue; // Already aligned
                }

                if ethics.stubbornness > 0.8 {
                    // Too stubborn -> dissent
                    if let Some(ref mut n) = needs {
                        n.leisure = (n.leisure - 0.1 * aura.strength).max(0.0);
                    }
                } else {
                    // Shift ethic
                    ethics.stubbornness -= aura.strength * 0.05;
                    if ethics.stubbornness <= 0.0 {
                        ethics.ethic = aura.target_ethic;
                        ethics.stubbornness = 1.0; // Reset stubbornness for new ethic
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_indoctrination_shifts_pop_ethics_over_time() {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime {
            tick: 1,
            ..Default::default()
        });

        let pop = world
            .spawn((
                PopEthics {
                    ethic: Ethic::Pacifist,
                    stubbornness: 0.1, // Will shift after 2 ticks of strength 1.0
                },
                GridPosition { x: 0, y: 0 },
                Needs::default(),
            ))
            .id();

        world.spawn((
            IndoctrinationAura {
                target_ethic: Ethic::Militarist,
                strength: 1.0,
                radius: 5.0,
            },
            GridPosition { x: 2, y: 0 },
        ));

        // Act: Run twice
        world
            .run_system_once(process_indoctrination_system)
            .unwrap();
        world
            .run_system_once(process_indoctrination_system)
            .unwrap();

        let ethics = world.get::<PopEthics>(pop).unwrap();
        assert_eq!(ethics.ethic, Ethic::Militarist);
        assert_eq!(ethics.stubbornness, 1.0);
    }

    #[test]
    fn test_indoctrination_failure_causes_dissent() {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime {
            tick: 1,
            ..Default::default()
        });

        let pop = world
            .spawn((
                PopEthics {
                    ethic: Ethic::FreeThinker,
                    stubbornness: 0.9, // Very stubborn
                },
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            IndoctrinationAura {
                target_ethic: Ethic::StateLoyalist,
                strength: 1.0,
                radius: 5.0,
            },
            GridPosition { x: 0, y: 1 },
        ));

        world
            .run_system_once(process_indoctrination_system)
            .unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 1.0); // Dissent reduces morale/leisure

        let ethics = world.get::<PopEthics>(pop).unwrap();
        assert_eq!(ethics.ethic, Ethic::FreeThinker); // Did not shift
    }
}
