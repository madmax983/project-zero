//! Cryo-Dreams system (Spec 195).
//!
//! Handles subconscious knowledge generation for pops in Cryo-Stasis,
//! and the risk of Cryo-Nightmares.

use bevy_ecs::prelude::*;
use rand::Rng;

/// State tracking the progress of subconscious thought aggregation.
#[derive(Component, Default, Debug)]
pub struct CryoDreamState {
    /// Progress towards the next point of Knowledge.
    pub accumulator: f32,
}

/// A permanent or semi-permanent trauma caused by a Cryo-Nightmare.
///
/// This component persists after waking up until treated or decayed,
/// causing significant debuffs.
#[derive(Component, Debug)]
pub struct CryoTrauma {
    /// Severity of the trauma (affects debuff magnitude).
    pub severity: f32,
}

/// System that processes dreams for frozen pops.
///
/// Generates Knowledge points over time, boosted by traits.
/// Has a small chance to cause `CryoTrauma`.
pub fn cryo_dream_system(
    mut query: Query<
        (Entity, &mut CryoDreamState, &crate::layer1::traits::Traits),
        With<crate::layer1::cryo::CryoStasis>,
    >,
    mut resources: ResMut<crate::layer1::resources::ColonyResources>,
    mut commands: Commands,
) {
    let mut total_knowledge = 0.0;
    let mut rng = rand::thread_rng();

    for (entity, mut dream, traits) in &mut query {
        let mut rate = 0.01; // Base rate per tick

        if traits.has(crate::layer1::traits::Trait::Creative) {
            rate *= 1.5;
        }
        if traits.has(crate::layer1::traits::Trait::Intellectual) {
            rate *= 1.2;
        }

        dream.accumulator += rate;

        // Process generated knowledge chunks
        loop {
            if dream.accumulator < 1.0 {
                break;
            }
            total_knowledge += 1.0;
            dream.accumulator -= 1.0;

            // Roll for Nightmare (1% chance per knowledge point generated)
            if rng.gen_bool(0.01) {
                commands.entity(entity).insert(CryoTrauma {
                    severity: rng.gen_range(0.5..1.0),
                });
            }
        }
    }

    if total_knowledge > 0.0 {
        resources.add_knowledge(total_knowledge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::cryo::CryoStasis;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::traits::{Trait, Traits};

    #[test]
    fn test_cryo_pop_generates_knowledge() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn a frozen pop
        world.spawn((
            Pop,
            CryoStasis,
            CryoDreamState::default(),
            Traits::default(),
        ));

        // Run system many times to ensure accumulation >= 1.0
        // Base rate 0.01, so 100 ticks = 1.0
        let mut schedule = Schedule::default();
        schedule.add_systems(cryo_dream_system);

        for _ in 0..101 {
            schedule.run(&mut world);
        }

        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge > 0.0, "Frozen pop should generate knowledge");
    }

    #[test]
    fn test_creative_trait_boosts_dreams() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let normal = world
            .spawn((
                Pop,
                CryoStasis,
                CryoDreamState::default(),
                Traits::default(),
            ))
            .id();

        let mut traits = Traits::default();
        traits.add(Trait::Creative);
        let creative = world
            .spawn((Pop, CryoStasis, CryoDreamState::default(), traits))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(cryo_dream_system);
        schedule.run(&mut world);

        let normal_state = world.get::<CryoDreamState>(normal).unwrap();
        let creative_state = world.get::<CryoDreamState>(creative).unwrap();

        assert!(
            creative_state.accumulator > normal_state.accumulator,
            "Creative pop should dream faster"
        );
    }

    #[test]
    fn test_nightmare_acquisition() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn pop with high accumulator to trigger roll immediately
        world.spawn((
            Pop,
            CryoStasis,
            CryoDreamState { accumulator: 1.0 },
            Traits::default(),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(cryo_dream_system);

        let mut caught_nightmare = false;
        for _ in 0..1000 {
            let pop = world
                .spawn((
                    Pop,
                    CryoStasis,
                    CryoDreamState { accumulator: 1.0 },
                    Traits::default(),
                ))
                .id();

            schedule.run(&mut world);

            if world.get::<CryoTrauma>(pop).is_some() {
                caught_nightmare = true;
                break;
            }

            world.despawn(pop);
        }

        assert!(caught_nightmare, "Should eventually trigger a nightmare");
    }

    #[test]
    fn test_trauma_affects_waking() {
        use crate::layer1::cryo::{exit_cryo_system, CryoSickness, ThawOrder};
        use crate::layer1::pop::Speed;

        let mut world = World::new();
        // Use a valid ID for testing, but ideally we rely on spawn
        let pop = world
            .spawn((
                Pop,
                CryoStasis,
                ThawOrder,
                CryoDreamState::default(),
                CryoTrauma { severity: 0.8 },
                Speed::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(exit_cryo_system);
        schedule.run(&mut world);

        let sickness = world
            .get::<CryoSickness>(pop)
            .expect("Should have sickness");
        // Base duration 500, doubled to 1000
        assert_eq!(sickness.duration, 1000);
        // Base severity 0.5 + 0.8 = 1.3 -> min(0.9)
        assert!((sickness.severity - 0.9).abs() < f32::EPSILON);

        // Trauma removed?
        assert!(world.get::<CryoTrauma>(pop).is_none());
    }
}
