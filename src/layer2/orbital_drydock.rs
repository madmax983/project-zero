use crate::layer1::resources::ResourceType;
use crate::layer2::mining::FleetCargo;
use crate::layer2::station::{Station, StationType};
use bevy::prelude::*;

#[derive(Component)]
pub struct ShipConstruction {
    pub target_ship_class: String,
    pub metal_required: f32,
    pub metal_delivered: f32,
    pub is_complete: bool,
}

pub fn process_drydock_construction_system(
    mut query: Query<(&Station, &mut ShipConstruction, &mut FleetCargo)>,
) {
    for (station, mut construction, mut cargo) in query.iter_mut() {
        if station.station_type != StationType::OrbitalDrydock || construction.is_complete {
            continue;
        }

        let needed = construction.metal_required - construction.metal_delivered;
        if needed <= 0.0 {
            construction.is_complete = true;
            continue;
        }

        for stack in cargo.contents.iter_mut() {
            if stack.resource_type == ResourceType::Metal && stack.amount > 0.0 {
                let to_take = stack.amount.min(needed);
                stack.amount -= to_take;
                construction.metal_delivered += to_take;
                break; // Only taking from one stack for MVP
            }
        }

        if construction.metal_delivered >= construction.metal_required {
            construction.is_complete = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ResourceType;
    use crate::layer2::mining::{CargoStack, FleetCargo};
    use crate::layer2::station::{Station, StationType};

    #[test]
    fn test_orbital_drydock_construction_progress() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_drydock_construction_system);

        let required_metal = 1000.0;

        let drydock_entity = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::OrbitalDrydock,
                },
                ShipConstruction {
                    target_ship_class: "Dreadnought".to_string(),
                    metal_required: required_metal,
                    metal_delivered: 0.0,
                    is_complete: false,
                },
            ))
            .id();

        // Deliver some cargo to the drydock
        app.world_mut()
            .entity_mut(drydock_entity)
            .insert(FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Metal,
                    amount: 500.0,
                }],
                capacity: 2000.0,
            });

        // Act
        app.update();

        // Assert
        let construction = app.world().get::<ShipConstruction>(drydock_entity).unwrap();
        assert_eq!(construction.metal_delivered, 500.0);
        assert!(!construction.is_complete);

        let cargo = app.world().get::<FleetCargo>(drydock_entity).unwrap();
        assert_eq!(
            cargo
                .contents
                .iter()
                .find(|s| s.resource_type == ResourceType::Metal)
                .map(|s| s.amount)
                .unwrap_or(0.0),
            0.0
        );

        // Deliver the rest
        app.world_mut()
            .get_mut::<FleetCargo>(drydock_entity)
            .unwrap()
            .contents
            .push(CargoStack {
                resource_type: ResourceType::Metal,
                amount: 500.0,
            });

        app.update();

        // Assert Completion
        let construction = app.world().get::<ShipConstruction>(drydock_entity).unwrap();
        assert_eq!(construction.metal_delivered, 1000.0);
        assert!(construction.is_complete);
    }
}
