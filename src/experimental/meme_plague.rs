//! Meme Plague (Contagious Memes) System (Nova Feature).
//!
//! A system that allows ideas to spread like a virus through the colony,
//! overriding behavior until the pop is "cured" or the meme decays.

use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;
use rand::Rng;

/// The type of meme infecting a Pop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemeType {
    /// A memetic obsession with productivity, ignoring all other needs.
    WorkCult,
    /// An overwhelming urge to dance and socialize, ignoring work and rest.
    DanceMeme,
    /// A deep paranoia that causes the pop to isolate themselves.
    ParanoiaMeme,
}

/// Component indicating that a Pop is carrying a memetic infection.
#[derive(Component, Clone, Debug)]
pub struct MemeCarrier {
    /// The type of meme.
    pub meme_type: MemeType,
    /// The remaining duration of the infection in ticks.
    pub duration: u32,
}

/// Spreads the meme to nearby uninfected Pops.
#[allow(clippy::type_complexity)]
pub fn process_meme_contagion(
    mut commands: Commands,
    infected_query: Query<(&GridPosition, &MemeCarrier), With<Pop>>,
    uninfected_query: Query<(Entity, &GridPosition), (With<Pop>, Without<MemeCarrier>)>,
) {
    let mut rng = rand::thread_rng();

    for (uninfected_entity, uninfected_pos) in &uninfected_query {
        for (infected_pos, carrier) in &infected_query {
            // Check if they are adjacent (distance <= 1)
            let dx = infected_pos.x.abs_diff(uninfected_pos.x);
            let dy = infected_pos.y.abs_diff(uninfected_pos.y);

            if dx <= 1 && dy <= 1 && (dx > 0 || dy > 0) {
                // 10% chance to infect per tick per adjacent infected pop
                if rng.r#gen::<f32>() < 0.10 {
                    commands.entity(uninfected_entity).insert(MemeCarrier {
                        meme_type: carrier.meme_type,
                        duration: 500, // Hardcoded duration for now
                    });
                    break; // Only get infected once per tick
                }
            }
        }
    }
}

/// Applies the effects of the meme and handles decay.
pub fn apply_meme_effects(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MemeCarrier, &mut Needs, &mut StressTracker), With<Pop>>,
) {
    for (entity, mut carrier, mut needs, mut stress) in &mut query {
        if carrier.duration == 0 {
            commands.entity(entity).remove::<MemeCarrier>();
            continue;
        }

        carrier.duration -= 1;

        match carrier.meme_type {
            MemeType::WorkCult => {
                // Artificially suppress rest and leisure needs, but increase stress
                needs.rest = needs.rest.max(0.8);
                needs.leisure = needs.leisure.max(0.8);
                stress.accumulated_stress = (stress.accumulated_stress + 0.05).min(100.0);
            }
            MemeType::DanceMeme => {
                // Force leisure need to be extremely high (low value = high need)
                needs.leisure = needs.leisure.min(0.1);
                // Also exhaust them
                needs.rest = (needs.rest - 0.01).max(0.0);
            }
            MemeType::ParanoiaMeme => {
                // Maximize stress rapidly
                stress.accumulated_stress = (stress.accumulated_stress + 0.1).min(100.0);
                // Suppress social needs
                needs.leisure = needs.leisure.max(0.9);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_meme_contagion_spreads() {
        let mut world = World::new();

        // Spawn infected pop
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            MemeCarrier {
                meme_type: MemeType::DanceMeme,
                duration: 100,
            },
        ));

        // Spawn uninfected pop adjacent
        let target = world.spawn((Pop, GridPosition { x: 5, y: 6 })).id();

        // Run the system multiple times to guarantee the 10% spread chance hits
        let mut schedule = Schedule::default();
        schedule.add_systems(process_meme_contagion);

        for _ in 0..100 {
            schedule.run(&mut world);
        }

        // The target should now be infected
        let carrier = world.get::<MemeCarrier>(target);
        assert!(carrier.is_some(), "Meme should have spread");
        assert_eq!(carrier.unwrap().meme_type, MemeType::DanceMeme);
    }

    #[test]
    fn test_work_cult_effects() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                MemeCarrier {
                    meme_type: MemeType::WorkCult,
                    duration: 10,
                },
                Needs {
                    rest: 0.1,    // Very tired
                    leisure: 0.1, // Very bored
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(apply_meme_effects).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        let stress = world.get::<StressTracker>(pop).unwrap();

        assert!(needs.rest >= 0.8, "Work cult should suppress rest need");
        assert!(
            needs.leisure >= 0.8,
            "Work cult should suppress leisure need"
        );
        assert!(
            stress.accumulated_stress > 0.0,
            "Work cult should increase stress"
        );
    }

    #[test]
    fn test_dance_meme_effects() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                MemeCarrier {
                    meme_type: MemeType::DanceMeme,
                    duration: 10,
                },
                Needs {
                    rest: 1.0,    // Fully rested
                    leisure: 1.0, // Fully satisfied
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(apply_meme_effects).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();

        assert!(
            needs.leisure <= 0.1,
            "Dance meme should force high leisure need"
        );
        assert!(needs.rest < 1.0, "Dance meme should exhaust the pop");
    }

    #[test]
    fn test_meme_decays() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                MemeCarrier {
                    meme_type: MemeType::ParanoiaMeme,
                    duration: 1, // Will decay next tick
                },
                Needs::default(),
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(apply_meme_effects).unwrap();

        // Duration should be 0 now
        let carrier = world.get::<MemeCarrier>(pop).unwrap();
        assert_eq!(carrier.duration, 0);

        // Run again to remove it
        world.run_system_once(apply_meme_effects).unwrap();

        assert!(
            world.get::<MemeCarrier>(pop).is_none(),
            "Meme should be removed after duration hits 0"
        );
    }
}
