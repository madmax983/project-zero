use crate::layer1::core::chronicle::AddChronicleEvent;
use crate::layer1::economy::resources::ColonyResources;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct DerelictEcosystem {
    pub threat_level: f32,
    pub salvage_yield: f32,
}

#[derive(Component)]
pub struct ShipbreakerMission {
    pub target_derelict: Entity,
    pub progress: f32,
}

pub fn simulate_shipbreaker_salvage(
    mut commands: Commands,
    mut q_missions: Query<(Entity, &mut ShipbreakerMission)>,
    mut q_derelicts: Query<&mut DerelictEcosystem>,
    mut resources: ResMut<ColonyResources>,
    mut evt_chronicle: EventWriter<AddChronicleEvent>,
) {
    for (mission_entity, mut mission) in q_missions.iter_mut() {
        if let Ok(mut ecosystem) = q_derelicts.get_mut(mission.target_derelict) {
            let progress_step = 10.0;
            mission.progress += progress_step;
            ecosystem.threat_level -= progress_step;

            let yield_step = ecosystem.salvage_yield * (progress_step / 100.0);
            resources.metal += yield_step;

            if ecosystem.threat_level <= 0.0 {
                commands.entity(mission.target_derelict).despawn();
                commands.entity(mission_entity).despawn();

                evt_chronicle.send(AddChronicleEvent {
                    text: "SYMBIOTIC_SALVAGE".to_string(),
                    importance: crate::layer1::core::chronicle::EventImportance::Major,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use bevy_ecs::event::Events;

    #[test]
    fn test_shipbreaker_salvage_progress_reduces_threat_and_yields_salvage() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(ColonyResources::default());
        app.add_systems(Update, simulate_shipbreaker_salvage);

        let derelict = app
            .world_mut()
            .spawn(DerelictEcosystem {
                threat_level: 100.0,
                salvage_yield: 50.0,
            })
            .id();

        let mission = app
            .world_mut()
            .spawn(ShipbreakerMission {
                target_derelict: derelict,
                progress: 0.0,
            })
            .id();

        app.update();

        let ecosystem = app.world().get::<DerelictEcosystem>(derelict).unwrap();
        assert!(
            ecosystem.threat_level < 100.0,
            "Ecosystem threat should decrease as shipbreakers make progress"
        );

        let resources = app.world().resource::<ColonyResources>();
        assert!(
            resources.metal > 0.0,
            "Salvage mission should yield metal resources"
        );

        let mission_data = app.world().get::<ShipbreakerMission>(mission).unwrap();
        assert!(
            mission_data.progress > 0.0,
            "Mission progress should increase"
        );
    }

    #[test]
    fn test_shipbreaker_salvage_complete_fires_chronicle() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(ColonyResources::default());
        app.add_systems(Update, simulate_shipbreaker_salvage);

        let derelict = app
            .world_mut()
            .spawn(DerelictEcosystem {
                threat_level: 5.0,
                salvage_yield: 10.0,
            })
            .id();

        app.world_mut().spawn(ShipbreakerMission {
            target_derelict: derelict,
            progress: 95.0,
        });

        app.update();

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let ev_list: Vec<_> = reader.read(events).collect();
        assert_eq!(
            ev_list.len(),
            1,
            "Should fire chronicle event when derelict salvage completes"
        );
    }
}
