//! Pop lifecycle and aging system.
//!
//! Handles aging of Pops, life stage transitions (Child -> Adult -> Elder),
//! and natural death due to old age.

use crate::layer1::balance::{AGE_ADULT, AGE_ELDER, TICKS_PER_YEAR};
use crate::layer1::pop::Speed;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Life stages of a Pop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LifeStage {
    /// Initial stage.
    #[default]
    Child,
    /// Productive stage (18+ years).
    Adult,
    /// Senior stage (60+ years), with potential penalties.
    Elder,
}

/// Tracks the age and life stage of a Pop.
#[derive(Component, Debug, Clone, Default)]
pub struct Age {
    /// Total ticks the Pop has been alive.
    pub ticks_alive: u64,
    /// Current life stage.
    pub stage: LifeStage,
}

impl Age {
    /// Creates a new Age component from years.
    pub fn new(years: u64) -> Self {
        let ticks = years * TICKS_PER_YEAR;
        let stage = if ticks < AGE_ADULT {
            LifeStage::Child
        } else if ticks < AGE_ELDER {
            LifeStage::Adult
        } else {
            LifeStage::Elder
        };
        Self {
            ticks_alive: ticks,
            stage,
        }
    }
}

/// System to increment age and handle life stage transitions.
pub fn aging_system(
    mut query: Query<(Entity, &mut Age, Option<&mut Speed>)>,
    mut log: Option<ResMut<MessageLog>>,
) {
    for (_entity, mut age, mut speed) in query.iter_mut() {
        age.ticks_alive += 1;

        let new_stage = if age.ticks_alive >= AGE_ELDER {
            LifeStage::Elder
        } else if age.ticks_alive >= AGE_ADULT {
            LifeStage::Adult
        } else {
            LifeStage::Child
        };

        if new_stage != age.stage {
            // Apply transition effects
            if new_stage == LifeStage::Elder {
                if let Some(s) = speed.as_mut() {
                    s.base *= 0.8; // 20% slow down
                }
                if let Some(l) = log.as_mut() {
                    l.add("A colonist has become an Elder.");
                }
            }
            age.stage = new_stage;
        }
    }
}

/// System to handle natural death from old age.
///
/// Probability of death increases with age for Elders.
pub fn natural_death_system(mut query: Query<(&Age, &mut crate::layer1::health::Health)>) {
    let mut rng = rand::thread_rng();

    for (age, mut health) in query.iter_mut() {
        if age.stage == LifeStage::Elder {
            #[allow(clippy::cast_precision_loss)]
            let years = age.ticks_alive as f64 / TICKS_PER_YEAR as f64;

            // Only start applying chance after 60 years.
            if years > 60.0 {
                // Chance per tick.
                // Example: At 80 years (20 years past 60), chance is 0.0002 per tick.
                // Over 1000 ticks (1 year), chance is ~18%.
                let chance = (years - 60.0) * 0.00001;

                if rng.gen_bool(chance.max(0.0).min(1.0)) {
                    health.current = 0.0; // Die
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::balance::{AGE_ADULT, AGE_ELDER, TICKS_PER_YEAR};
    use crate::layer1::health::Health;
    use crate::layer1::pop::{Pop, Speed};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_pop_has_age_component() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Age::default())).id();

        let age = world.get::<Age>(entity);
        assert!(age.is_some());
        assert_eq!(age.unwrap().ticks_alive, 0);
    }

    #[test]
    fn test_aging_system_increments_age() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(aging_system);

        let entity = world
            .spawn((
                Pop,
                Age {
                    ticks_alive: 100,
                    ..Default::default()
                },
                Speed::default(),
            ))
            .id();

        schedule.run(&mut world);

        let age = world.get::<Age>(entity).unwrap();
        assert_eq!(age.ticks_alive, 101);
    }

    #[test]
    fn test_lifestage_transitions() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(aging_system);

        // 1. Child -> Adult
        let child = world
            .spawn((
                Pop,
                Age {
                    ticks_alive: AGE_ADULT - 1,
                    stage: LifeStage::Child,
                },
                Speed::default(),
            ))
            .id();

        // 2. Adult -> Elder
        let adult = world
            .spawn((
                Pop,
                Age {
                    ticks_alive: AGE_ELDER - 1,
                    stage: LifeStage::Adult,
                },
                Speed::default(),
            ))
            .id();

        schedule.run(&mut world);

        let child_age = world.get::<Age>(child).unwrap();
        assert_eq!(child_age.stage, LifeStage::Adult);

        let adult_age = world.get::<Age>(adult).unwrap();
        assert_eq!(adult_age.stage, LifeStage::Elder);
    }

    #[test]
    fn test_elder_speed_penalty() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(aging_system);

        // Spawn a pop about to become Elder
        let entity = world
            .spawn((
                Pop,
                Age {
                    ticks_alive: AGE_ELDER - 1,
                    stage: LifeStage::Adult,
                },
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        schedule.run(&mut world);

        let speed = world.get::<Speed>(entity).unwrap();
        // Expect penalty (e.g. 0.8x)
        assert!(speed.base < 1.0);
        assert!((speed.base - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_natural_death_chance() {
        // Probabilistic test: excessive age should eventually kill
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(natural_death_system);

        let entity = world
            .spawn((
                Pop,
                Age {
                    ticks_alive: 120 * TICKS_PER_YEAR, // Very old
                    stage: LifeStage::Elder,
                },
                Health {
                    current: 10.0,
                    max: 10.0,
                },
            ))
            .id();

        // Run many times to trigger probability.
        // With 0.00001 base chance * (120-60) = 0.0006 per tick.
        // 10,000 ticks gives ~99.7% chance of death.
        let mut died = false;
        for _ in 0..10_000 {
            schedule.run(&mut world);
            let health = world.get::<Health>(entity).unwrap();
            if !health.is_alive() {
                died = true;
                break;
            }
        }

        assert!(died, "Very old pop should eventually die naturally");
    }

    #[test]
    fn test_aging_system_without_speed() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(aging_system);

        let entity = world
            .spawn((
                Pop,
                Age {
                    ticks_alive: 100,
                    ..Default::default()
                },
                // No Speed component
            ))
            .id();

        schedule.run(&mut world);

        let age = world.get::<Age>(entity).unwrap();
        assert_eq!(age.ticks_alive, 101);
    }
}
