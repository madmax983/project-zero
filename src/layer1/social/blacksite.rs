use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::economy::ideological_contraband::Ethics;
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::map::GridPosition;
use crate::layer1::Enemy;

#[derive(Component)]
pub struct Blacksite {
    pub instability: f32,
    pub payout: f32,
}

#[derive(Component)]
pub struct Prisoner {
    pub radical_ethics: Ethics,
}

#[derive(Component)]
pub struct Warden;

#[derive(Event)]
pub struct PayoutEvent;

#[derive(Event)]
pub struct PrisonBreakEvent;

pub fn blacksite_payout_system(
    mut events: EventReader<PayoutEvent>,
    mut resources: ResMut<ColonyResources>,
    blacksites: Query<&Blacksite>,
) {
    for _ in events.read() {
        for blacksite in blacksites.iter() {
            resources.credits += blacksite.payout;
        }
    }
}

pub fn radicalization_aura_system(
    prisoners: Query<(&Prisoner, &GridPosition)>,
    mut wardens: Query<(&mut Ethics, &GridPosition), With<Warden>>,
) {
    for (prisoner, p_pos) in prisoners.iter() {
        for (mut w_ethics, w_pos) in wardens.iter_mut() {
            if p_pos.distance_manhattan(*w_pos) < 5 {
                if prisoner.radical_ethics.collectivism > w_ethics.collectivism {
                    w_ethics.collectivism += 1;
                }
                if prisoner.radical_ethics.elitism > w_ethics.elitism {
                    w_ethics.elitism += 1;
                }
            }
        }
    }
}

pub fn prison_break_system(
    mut commands: Commands,
    mut events: EventReader<PrisonBreakEvent>,
    mut chronicle: EventWriter<AddChronicleEvent>,
    prisoners: Query<(Entity, &Prisoner)>,
) {
    for _ in events.read() {
        chronicle.send(AddChronicleEvent {
            text: "A prison break has occurred at the Blacksite!".to_string(),
            importance: EventImportance::Major,
        });

        for (entity, _) in prisoners.iter() {
            commands.entity(entity)
                .remove::<Prisoner>()
                .insert(Enemy { action_speed: 1.0 });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacksite_contract_payout() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, blacksite_payout_system);
        app.add_event::<PayoutEvent>();

        let mut res = ColonyResources::default();
        res.credits = 0.0;
        app.insert_resource(res);

        app.world_mut().spawn(Blacksite {
            instability: 10.0,
            payout: 5000.0,
        });

        app.world_mut().send_event(PayoutEvent);
        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 5000.0);
    }

    #[test]
    fn test_prisoner_radicalization() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, radicalization_aura_system);

        app.world_mut().spawn((
            Prisoner {
                radical_ethics: Ethics { collectivism: 10, elitism: 10 },
            },
            GridPosition { x: 5, y: 5 },
        ));

        let warden = app.world_mut().spawn((
            Warden,
            Ethics { collectivism: 0, elitism: 0 },
            GridPosition { x: 6, y: 5 },
        )).id();

        app.update();

        let w_ethics = app.world().get::<Ethics>(warden).unwrap();
        assert_eq!(w_ethics.collectivism, 1);
        assert_eq!(w_ethics.elitism, 1);
    }

    #[test]
    fn test_prison_break_event() {
        let mut app = bevy_app::App::new();
        app.add_event::<PrisonBreakEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(bevy_app::Update, prison_break_system);

        let prisoner = app.world_mut().spawn((
            Prisoner {
                radical_ethics: Ethics { collectivism: 10, elitism: 10 }
            },
        )).id();

        app.world_mut().send_event(PrisonBreakEvent);
        app.update();

        assert!(app.world().get::<Enemy>(prisoner).is_some());
        assert!(app.world().get::<Prisoner>(prisoner).is_none());

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        assert_eq!(events.len(), 1);
    }
}
