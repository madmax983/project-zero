//! Exploration
//!
//! Manages the exploration of new areas and the discovery of points of interest.

pub mod void_whispers {
    use bevy::prelude::*;
    use rand::seq::IteratorRandom;

    #[derive(Component)]
    pub struct ExplorerFleet;

    #[derive(Component)]
    pub struct DeepSpaceExposure {
        pub ticks: u32,
    }

    #[derive(Component)]
    pub struct VoidWhispers {
        pub intensity: f32,
    }

    #[derive(Component)]
    pub struct ColonyPop {
        pub colony: Entity,
    }

    #[derive(Event)]
    pub struct FleetReturnedEvent {
        pub fleet: Entity,
        pub colony: Entity,
    }

    #[allow(clippy::type_complexity)]
    pub fn accumulate_void_whispers_in_deep_space(
        mut commands: Commands,
        fleets: Query<(Entity, &DeepSpaceExposure), (With<ExplorerFleet>, Without<VoidWhispers>)>,
    ) {
        for (entity, exposure) in fleets.iter() {
            if exposure.ticks >= 100 {
                commands
                    .entity(entity)
                    .insert(VoidWhispers { intensity: 10.0 });
            }
        }
    }

    pub fn spread_whispers_to_colony(
        mut commands: Commands,
        mut events: EventReader<FleetReturnedEvent>,
        fleets_with_whispers: Query<&VoidWhispers>,
        pops: Query<(Entity, &ColonyPop)>,
    ) {
        let mut rng = rand::thread_rng();
        for event in events.read() {
            if fleets_with_whispers.get(event.fleet).is_ok() {
                // Infect a single random pop in the colony
                if let Some(pop_entity) = pops
                    .iter()
                    .filter_map(|(e, pop)| {
                        if pop.colony == event.colony {
                            Some(e)
                        } else {
                            None
                        }
                    })
                    .choose(&mut rng)
                {
                    commands
                        .entity(pop_entity)
                        .insert(crate::layer1::memetics::MemeticCarrier);
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_explorers_accumulate_void_whispers() {
            let mut app = App::new();
            app.add_systems(Update, accumulate_void_whispers_in_deep_space);

            let fleet_entity = app
                .world_mut()
                .spawn((ExplorerFleet, DeepSpaceExposure { ticks: 100 }))
                .id();

            app.update();

            let whispers = app.world().get::<VoidWhispers>(fleet_entity);
            assert!(
                whispers.is_some(),
                "Fleet exposed to deep space should accumulate Void Whispers"
            );
            assert_eq!(whispers.unwrap().intensity, 10.0);
        }

        #[test]
        fn test_returning_fleet_infects_colony() {
            let mut app = App::new();
            app.add_event::<FleetReturnedEvent>();
            app.add_systems(Update, spread_whispers_to_colony);

            let fleet_entity = app
                .world_mut()
                .spawn((VoidWhispers { intensity: 50.0 },))
                .id();

            let colony_entity = app.world_mut().spawn_empty().id();
            let other_colony_entity = app.world_mut().spawn_empty().id();

            let colony_pop_entity = app
                .world_mut()
                .spawn((ColonyPop {
                    colony: colony_entity,
                },))
                .id();

            let other_colony_pop_entity = app
                .world_mut()
                .spawn((ColonyPop {
                    colony: other_colony_entity,
                },))
                .id();

            app.world_mut().send_event(FleetReturnedEvent {
                fleet: fleet_entity,
                colony: colony_entity,
            });

            app.update();

            // Target colony pop should now have an infection
            let meme = app
                .world()
                .get::<crate::layer1::memetics::MemeticCarrier>(colony_pop_entity);
            assert!(
            meme.is_some(),
            "Target colony pop should receive crate::layer1::memetics::MemeticCarrier from returning fleet"
        );

            // Other colony pop should NOT be infected
            let other_meme = app
                .world()
                .get::<crate::layer1::memetics::MemeticCarrier>(other_colony_pop_entity);
            assert!(
                other_meme.is_none(),
                "Other colony pop should not be infected"
            );
        }
    }
}
