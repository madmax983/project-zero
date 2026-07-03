use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::economy::ideological_contraband::Ethics;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::rearguard::Enemy;
use bevy::prelude::*;

#[derive(Resource)]
pub struct BlacksiteContract {
    pub ticks_to_payout: u32,
    pub payout_amount: f32,
}

#[derive(Component)]
pub struct Prisoner;

#[derive(Event)]
pub struct PrisonBreakEvent;

pub fn process_blacksite_payout_system(
    blacksite: Option<ResMut<BlacksiteContract>>,
    mut resources: ResMut<ColonyResources>,
) {
    if let Some(mut contract) = blacksite {
        if contract.ticks_to_payout > 0 {
            contract.ticks_to_payout -= 1;
        } else {
            resources.credits += contract.payout_amount;
            contract.ticks_to_payout = 10; // reset
        }
    }
}

pub fn prisoner_radicalization_system(
    mut wardens: Query<&mut Ethics, Without<Prisoner>>,
    prisoners: Query<&Ethics, With<Prisoner>>,
) {
    for prisoner_ethics in prisoners.iter() {
        for mut warden_ethics in wardens.iter_mut() {
            warden_ethics.collectivism +=
                (prisoner_ethics.collectivism - warden_ethics.collectivism) / 10;
        }
    }
}

pub fn prison_break_system(
    mut events: EventReader<PrisonBreakEvent>,
    mut commands: Commands,
    prisoners: Query<Entity, With<Prisoner>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _ in events.read() {
        for entity in prisoners.iter() {
            commands.entity(entity).insert(Enemy { action_speed: 1.0 });
        }
        chronicle_events.send(AddChronicleEvent {
            text: "A massive prison break occurred at the blacksite!".to_string(),
            importance: EventImportance::Major,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.insert_resource(BlacksiteContract {
            ticks_to_payout: 1,
            payout_amount: 500.0,
        });
        app.add_event::<PrisonBreakEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(
            Update,
            (
                process_blacksite_payout_system,
                prisoner_radicalization_system,
                prison_break_system,
            ),
        );
        app
    }

    #[test]
    fn test_blacksite_contract_payout() {
        let mut app = setup_app();
        let initial_credits = app.world().resource::<ColonyResources>().credits;
        app.update();
        app.update();
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, initial_credits + 500.0);
    }

    #[test]
    fn test_prisoner_radicalization() {
        let mut app = setup_app();
        app.world_mut().spawn(Ethics {
            collectivism: 10,
            elitism: 0,
        }); // Warden
        app.world_mut().spawn((
            Prisoner,
            Ethics {
                collectivism: 100,
                elitism: 0,
            },
        )); // Prisoner
        app.update();
        let mut query = app
            .world_mut()
            .query_filtered::<&Ethics, Without<Prisoner>>();
        let warden_ethics = query.iter(app.world()).next().unwrap();
        assert_eq!(warden_ethics.collectivism, 19);
    }

    #[test]
    fn test_prison_break_event() {
        let mut app = setup_app();
        let prisoner_entity = app.world_mut().spawn(Prisoner).id();
        app.world_mut().send_event(PrisonBreakEvent);
        app.update();
        assert!(app.world().get::<Enemy>(prisoner_entity).is_some());

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let ev = reader.read(events).next().unwrap();
        assert!(matches!(ev.importance, EventImportance::Major));
    }
}
