use crate::layer1::disasters::{DisasterEvent, DisasterType};
use crate::layer1::terraforming::TerraformEvent;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct PlanetaryStress {
    pub value: f32,
    pub threshold: f32,
}

pub fn apply_terraforming_stress_system(
    mut stress: ResMut<PlanetaryStress>,
    mut events: EventReader<TerraformEvent>,
) {
    for event in events.read() {
        // Simple linear scaling for MVP
        stress.value += event.delta_temperature.abs() * 0.5;
    }
}

pub fn trigger_autoimmune_response_system(
    mut stress: ResMut<PlanetaryStress>,
    mut events: EventReader<TerraformEvent>,
    mut disaster_events: EventWriter<DisasterEvent>,
) {
    if stress.value >= stress.threshold {
        // Find the last known terraforming location to target
        if let Some(last_event) = events.read().last() {
            disaster_events.send(DisasterEvent {
                position: last_event.position,
                disaster_type: DisasterType::Fissure, // MVP: Just tear the ground open
            });
            // Reset stress after disaster
            stress.value = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::core::map::GridPosition;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_rapid_terraforming_increases_planetary_stress() {
        let mut app = App::new();
        app.insert_resource(PlanetaryStress {
            value: 0.0,
            threshold: 100.0,
        });
        app.add_event::<TerraformEvent>();
        app.add_systems(Update, apply_terraforming_stress_system);

        // Send a massive terraform event
        app.world_mut()
            .resource_mut::<Events<TerraformEvent>>()
            .send(TerraformEvent {
                position: GridPosition { x: 5, y: 5 },
                delta_temperature: 50.0, // Aggressive change
            });

        app.update();

        let stress = app.world().resource::<PlanetaryStress>();
        assert!(
            stress.value > 0.0,
            "Terraforming should increase planetary stress."
        );
    }

    #[test]
    fn test_critical_stress_triggers_disaster_at_terraform_site() {
        let mut app = App::new();
        app.insert_resource(PlanetaryStress {
            value: 95.0,
            threshold: 100.0,
        });
        app.add_event::<TerraformEvent>();
        app.add_event::<DisasterEvent>();
        app.add_systems(
            Update,
            (
                apply_terraforming_stress_system,
                trigger_autoimmune_response_system,
            )
                .chain(),
        );

        // Push stress over the threshold
        app.world_mut()
            .resource_mut::<Events<TerraformEvent>>()
            .send(TerraformEvent {
                position: GridPosition { x: 5, y: 5 },
                delta_temperature: 10.0,
            });

        app.update();

        // Verify a disaster was spawned at the location
        let disaster_events = app.world().resource::<Events<DisasterEvent>>();
        let mut cursor = disaster_events.get_cursor();
        let mut found = false;
        for event in cursor.read(disaster_events) {
            if event.position.x == 5 && event.position.y == 5 {
                found = true;
            }
        }

        assert!(
            found,
            "A disaster should trigger at the terraforming site when stress exceeds the threshold."
        );

        let stress = app.world().resource::<PlanetaryStress>();
        assert_eq!(
            stress.value, 0.0,
            "Planetary stress should reset after triggering a disaster."
        );
    }
}
