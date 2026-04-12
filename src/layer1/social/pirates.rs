use crate::layer1::core::integration::PirateAmnestyEvent;
use crate::layer1::economy::Wallet;
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use bevy::prelude::*;

use crate::layer1::law::justice::{CrimeCommittedEvent, CrimeType};
use crate::layer3::physics::relativity::SimulationTime;
use rand::Rng;

/// Spawns Layer 1 Pirate pops in response to a Layer 3 pirate fleet accepting amnesty.
pub fn process_pirate_amnesty_system(
    mut commands: Commands,
    amnesty_events: Option<Res<'_, bevy::prelude::Events<PirateAmnestyEvent>>>,
) {
    if let Some(events) = amnesty_events {
        let mut cursor = events.get_cursor();
        for _ev in cursor.read(&events) {
            // Spawn multiple pirate pops per fleet
            for _ in 0..5 {
                let mut pirate_traits = Traits::default();
                pirate_traits.add(Trait::Pirate);

                commands.spawn((
                    Pop,
                    pirate_traits,
                    Wallet { credits: 1000.0 }, // Massive credit boost
                ));
            }
        }
    }
}

/// Pirates frequently shirk work and engage in brawls/crime randomly.
pub fn pirate_crime_system(
    mut crime_events: Option<ResMut<'_, bevy::prelude::Events<CrimeCommittedEvent>>>,
    pirates: Query<(Entity, &Traits), With<Pop>>,
    _sim_time: Res<SimulationTime>,
) {
    let mut rng = rand::thread_rng();

    for (entity, traits) in pirates.iter() {
        if traits.has(Trait::Pirate) {
            // Determine base on arbitrary periodic check
            // e.g. 5% chance per tick to commit vandalism
            if rng.gen_bool(0.05) {
                if let Some(ref mut events) = crime_events {
                    events.send(CrimeCommittedEvent {
                        perpetrator: entity,
                        crime_type: CrimeType::Vandalism,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pirate_crime() {
        let mut app = App::new();
        app.add_event::<CrimeCommittedEvent>();
        app.init_resource::<SimulationTime>();
        // Required so events clear each frame if added directly.
        // We will just read the cursor without clearing since we loop update.
        // Wait, Bevy clears events every frame if not explicitly updated in schedule via event buffer systems.
        // Actually, App::update() runs all schedules, including clearing events.
        // Let's read events directly instead of relying on `App::update()` loop OR don't run `App::update()`.
        // Better: Just manually run the system!

        let mut pirate_traits = Traits::default();
        pirate_traits.add(Trait::Pirate);

        // A pirate pop
        let pirate_pop = app.world_mut().spawn((Pop, pirate_traits)).id();

        // A normal pop
        let normal_pop = app.world_mut().spawn((Pop, Traits::default())).id();

        let mut pirate_crimes = 0;
        let mut normal_crimes = 0;

        // Run the system manually many times
        for _ in 0..1000 {
            let mut system_state = bevy::ecs::system::IntoSystem::into_system(pirate_crime_system);
            system_state.initialize(app.world_mut());
            system_state.run((), app.world_mut());
            system_state.apply_deferred(app.world_mut());

            let events = app.world().resource::<Events<CrimeCommittedEvent>>();
            let mut cursor = events.get_cursor();
            for ev in cursor.read(events) {
                if ev.perpetrator == pirate_pop {
                    pirate_crimes += 1;
                } else if ev.perpetrator == normal_pop {
                    normal_crimes += 1;
                }
            }
            app.world_mut()
                .resource_mut::<Events<CrimeCommittedEvent>>()
                .clear();
        }

        // Ensure pirate committed a crime and normal pop did not
        assert!(
            pirate_crimes > 0,
            "Pirate pop should have committed at least one crime"
        );
        assert_eq!(
            normal_crimes, 0,
            "Normal pop should not have committed a crime"
        );
    }

    #[test]
    fn test_process_pirate_amnesty() {
        let mut app = App::new();
        app.add_event::<PirateAmnestyEvent>();

        app.world_mut().send_event(PirateAmnestyEvent {
            fleet: Entity::PLACEHOLDER,
        });

        app.add_systems(Update, process_pirate_amnesty_system);
        app.update();

        // Verify that pirate pops were spawned
        let mut query = app.world_mut().query::<(&Pop, &Traits, &Wallet)>();
        let mut count = 0;
        for (_pop, traits, wallet) in query.iter(app.world()) {
            assert!(traits.has(Trait::Pirate));
            assert_eq!(wallet.credits, 1000.0);
            count += 1;
        }

        assert_eq!(count, 5); // Spawns 5 per fleet
    }
}
