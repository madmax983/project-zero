use crate::layer1::geology::tectonic::TectonicStress;
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct TectonicFracker;

#[derive(Event)]
pub struct FrackEvent {
    pub entity: Entity,
}

pub fn tectonic_fracking_system(
    mut events: EventReader<FrackEvent>,
    query: Query<&TectonicFracker>,
    resources: Option<ResMut<ColonyResources>>,
    stress: Option<ResMut<TectonicStress>>,
) {
    if let (Some(mut resources), Some(mut stress)) = (resources, stress) {
        for event in events.read() {
            if query.get(event.entity).is_ok() && resources.waste >= 10.0 {
                // Consume waste
                resources.waste -= 10.0;

                // Generate resources (e.g., fuel)
                resources.fuel += 20.0;

                // Increase tectonic stress significantly
                stress.current += 15.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::geology::tectonic::TectonicStress;
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_fracking_consumes_waste_generates_resources() {
        let mut app = bevy_app::App::new();
        app.add_event::<FrackEvent>();
        app.add_systems(bevy_app::Update, tectonic_fracking_system);

        // Arrange
        let fracker = app.world_mut().spawn(TectonicFracker).id();
        app.world_mut().insert_resource(ColonyResources {
            waste: 100.0,
            ..Default::default()
        });
        app.world_mut().insert_resource(TectonicStress::default());

        // Act
        app.world_mut().send_event(FrackEvent { entity: fracker });
        app.update();

        // Assert
        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.waste < 100.0);
        assert!(resources.fuel > 0.0);
    }

    #[test]
    fn test_fracking_increases_seismic_instability() {
        let mut app = bevy_app::App::new();
        app.add_event::<FrackEvent>();
        app.add_systems(bevy_app::Update, tectonic_fracking_system);

        let fracker = app.world_mut().spawn(TectonicFracker).id();
        app.world_mut().insert_resource(TectonicStress {
            current: 0.0,
            threshold: 100.0,
            dissipation_rate: 0.1,
        });
        app.world_mut().insert_resource(ColonyResources {
            waste: 100.0,
            ..Default::default()
        });

        // Act
        app.world_mut().send_event(FrackEvent { entity: fracker });
        app.update();

        // Assert
        let instability = app.world().resource::<TectonicStress>();
        assert!(instability.current > 0.0);
    }
}
