use bevy::prelude::*;
use crate::layer3::diplomacy::proxy_wars::ThreatMap;

#[derive(Component, Debug, Default)]
pub struct MegastructureScaffolding {
    pub completion_percent: f32,
    pub resources_invested: u64,
}

#[derive(Event, Debug)]
pub struct MegastructureProgressEvent {
    pub entity: Entity,
    pub resources_added: u64,
    pub completion_increment: f32,
}

#[derive(Event, Debug)]
pub struct SolarAnomalyEvent {
    pub severity: f32,
    pub source_entity: Entity,
}

pub fn process_megastructure_construction_system(
    mut events: EventReader<MegastructureProgressEvent>,
    mut query: Query<&mut MegastructureScaffolding>,
) {
    for event in events.read() {
        if let Ok(mut scaffolding) = query.get_mut(event.entity) {
            scaffolding.completion_percent += event.completion_increment;
            scaffolding.resources_invested += event.resources_added;
        }
    }
}

pub fn trigger_solar_anomaly_system(
    query: Query<(Entity, &MegastructureScaffolding)>,
    mut events: EventWriter<SolarAnomalyEvent>,
) {
    for (entity, scaffolding) in query.iter() {
        // Use RNG based on completion percentage
        let probability = scaffolding.completion_percent / 100.0;
        if rand::random::<f32>() < probability {
            events.send(SolarAnomalyEvent {
                severity: scaffolding.completion_percent / 100.0,
                source_entity: entity,
            });
        }
    }
}

pub fn update_threat_from_megastructures_system(
    query: Query<&MegastructureScaffolding>,
    mut threat_map: ResMut<ThreatMap>,
) {
    let mut total_threat = 0.0;
    for scaffolding in query.iter() {
        total_threat += scaffolding.completion_percent * 0.5; // Threat scales with completion
    }
    threat_map.global_threat_modifier = total_threat;
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::layer2::megastructure::{
        MegastructureScaffolding, MegastructureProgressEvent,
        process_megastructure_construction_system,
        trigger_solar_anomaly_system,
        SolarAnomalyEvent,
    };
    use crate::layer3::diplomacy::proxy_wars::ThreatMap;

    #[test]
    fn test_megastructure_progress_increases_completion() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_megastructure_construction_system);
        app.add_event::<MegastructureProgressEvent>();

        let scaffolding_entity = app.world_mut().spawn(MegastructureScaffolding {
            completion_percent: 10.0,
            resources_invested: 1000,
        }).id();

        // Act
        app.world_mut()
            .resource_mut::<Events<MegastructureProgressEvent>>()
            .send(MegastructureProgressEvent {
                entity: scaffolding_entity,
                resources_added: 500,
                completion_increment: 2.5,
            });

        app.update();

        // Assert
        let scaffolding = app.world().entity(scaffolding_entity).get::<MegastructureScaffolding>().unwrap();
        assert_eq!(scaffolding.completion_percent, 12.5);
        assert_eq!(scaffolding.resources_invested, 1500);
    }

    #[test]
    fn test_high_completion_triggers_solar_anomaly() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, trigger_solar_anomaly_system);
        app.add_event::<SolarAnomalyEvent>();

        // Spawn scaffolding that is sufficiently large to block/alter starlight
        app.world_mut().spawn(MegastructureScaffolding {
            completion_percent: 100.0, // Set to 100% to guarantee RNG triggers (probability = 1.0)
            resources_invested: 50000,
        });

        // Act
        app.update();

        // Assert
        let anomaly_events = app.world().resource::<Events<SolarAnomalyEvent>>();
        let mut cursor = anomaly_events.get_cursor();
        let events: Vec<_> = cursor.read(anomaly_events).collect();
        assert!(!events.is_empty(), "A solar anomaly should have been triggered by the massive scaffolding.");
    }

    #[test]
    fn test_megastructure_generates_threat() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, crate::layer2::megastructure::update_threat_from_megastructures_system);

        app.world_mut().spawn(MegastructureScaffolding {
            completion_percent: 20.0,
            resources_invested: 20000,
        });

        app.insert_resource(ThreatMap::default());

        // Act
        app.update();

        // Assert
        let threat_map = app.world().resource::<ThreatMap>();
        // Assume we check threat against neighboring factions
        // Megastructure construction should globally raise threat levels
        assert!(threat_map.global_threat_modifier > 0.0);
    }
}
