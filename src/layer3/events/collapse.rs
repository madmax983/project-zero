use bevy::prelude::*;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::administration::invasive_bureaucracy::EmpireStability;

#[derive(Resource, Default)]
pub struct Population {
    pub total: u32,
}

#[derive(Event, Debug)]
pub struct CivilizationCollapseEvent {
    pub civ_id: Entity,
    pub population_lost: u32,
    pub tech_level: u32,
}

#[derive(Event, Debug)]
pub struct RefugeeWaveEvent {
    pub incoming_population: u32,
    pub tech_fragments: u32,
}

pub fn process_civilization_collapse_system(
    mut collapse_events: EventReader<CivilizationCollapseEvent>,
    mut refugee_events: EventWriter<RefugeeWaveEvent>,
    stability: Option<Res<EmpireStability>>,
) {
    let stab_val = stability.map(|r| r.value).unwrap_or(0.0);

    for event in collapse_events.read() {
        if stab_val >= 50.0 {
            refugee_events.send(RefugeeWaveEvent {
                incoming_population: event.population_lost / 100,
                tech_fragments: event.tech_level * 10,
            });
        }
    }
}

pub fn process_refugee_arrival_system(
    mut arrival_events: EventReader<RefugeeWaveEvent>,
    population: Option<ResMut<Population>>,
    resources: Option<ResMut<ColonyResources>>,
    stability: Option<ResMut<EmpireStability>>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    let mut population = population;
    let mut resources = resources;
    let mut stability = stability;
    for event in arrival_events.read() {
        if let Some(pop) = population.as_deref_mut() {
            pop.total += event.incoming_population;
        }

        if let Some(res) = resources.as_deref_mut() {
            res.add_knowledge(event.tech_fragments as f32);
        }

        if let Some(stab) = stability.as_deref_mut() {
            stab.value -= 5.0; // Rapid influx lowers stability
        }

        chronicle.send(AddChronicleEvent {
            text: format!(
                "A massive refugee fleet has arrived, bringing {} new souls and fragments of lost technology.",
                event.incoming_population
            ),
            importance: EventImportance::Major,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_civilization_collapse_emits_refugee_event() {
        let mut app = App::new();
        app.add_event::<CivilizationCollapseEvent>();
        app.add_event::<RefugeeWaveEvent>();
        app.insert_resource(EmpireStability { value: 80.0 });
        app.add_systems(Update, process_civilization_collapse_system);

        app.world_mut()
            .resource_mut::<Events<CivilizationCollapseEvent>>()
            .send(CivilizationCollapseEvent {
                civ_id: Entity::PLACEHOLDER,
                population_lost: 1_000_000,
                tech_level: 5,
            });

        app.update();

        let refugee_events = app.world().resource::<Events<RefugeeWaveEvent>>();
        let mut reader = refugee_events.get_cursor();
        let events: Vec<_> = reader.read(refugee_events).collect();

        assert_eq!(events.len(), 1);
        assert!(events[0].incoming_population > 0);
        assert!(events[0].tech_fragments > 0);
    }

    #[test]
    fn test_low_stability_ignores_refugees() {
        let mut app = App::new();
        app.add_event::<CivilizationCollapseEvent>();
        app.add_event::<RefugeeWaveEvent>();
        app.insert_resource(EmpireStability { value: 30.0 });
        app.add_systems(Update, process_civilization_collapse_system);

        app.world_mut()
            .resource_mut::<Events<CivilizationCollapseEvent>>()
            .send(CivilizationCollapseEvent {
                civ_id: Entity::PLACEHOLDER,
                population_lost: 1_000_000,
                tech_level: 5,
            });

        app.update();

        let refugee_events = app.world().resource::<Events<RefugeeWaveEvent>>();
        assert!(refugee_events.is_empty());
    }

    #[test]
    fn test_refugee_wave_arrival_impacts_resources() {
        let mut app = App::new();
        app.add_event::<RefugeeWaveEvent>();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(Population { total: 1000 });
        app.insert_resource(ColonyResources { knowledge: 100.0, max_knowledge: 1000.0, food: 5000.0, ..default() });
        app.add_systems(Update, process_refugee_arrival_system);

        app.world_mut()
            .resource_mut::<Events<RefugeeWaveEvent>>()
            .send(RefugeeWaveEvent {
                incoming_population: 500,
                tech_fragments: 50,
            });

        app.update();

        let pop = app.world().resource::<Population>();
        let res = app.world().resource::<ColonyResources>();

        assert_eq!(pop.total, 1500);
        assert_eq!(res.knowledge, 150.0);
    }
}
