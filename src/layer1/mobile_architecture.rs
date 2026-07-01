use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::economy::resources::ColonyResources;

#[derive(Clone, PartialEq, Debug)]
pub enum MobilityMode {
    Stationary,
    Mobile,
}

#[derive(Clone, PartialEq, Debug, Component)]
pub enum BuildingState {
    Operational,
    Offline,
}

#[derive(Component)]
pub struct MobileChassis {
    pub mode: MobilityMode,
}

#[derive(Event)]
pub struct TransformCommand {
    pub target: Entity,
    pub new_mode: MobilityMode,
}

#[derive(Event)]
pub struct MoveCommand {
    pub entity: Entity,
    pub destination: GridPosition,
}

const MOVE_FUEL_COST: f32 = 10.0;

#[allow(clippy::needless_pass_by_value)]
pub fn transform_building_system(
    mut events: EventReader<TransformCommand>,
    mut query: Query<(&mut MobileChassis, &mut BuildingState)>,
) {
    for event in events.read() {
        if let Ok((mut chassis, mut state)) = query.get_mut(event.target) {
            chassis.mode = event.new_mode.clone();
            if chassis.mode == MobilityMode::Mobile {
                *state = BuildingState::Offline;
            } else {
                *state = BuildingState::Operational;
            }
        }
    }
}

pub fn move_mobile_building_system(
    mut events: EventReader<MoveCommand>,
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(&MobileChassis, &mut GridPosition)>,
) {
    for event in events.read() {
        if let Ok((chassis, mut pos)) = query.get_mut(event.entity) {
            if chassis.mode == MobilityMode::Mobile && resources.fuel >= MOVE_FUEL_COST {
                // Update position
                *pos = event.destination;
                // Consume fuel
                resources.fuel -= MOVE_FUEL_COST;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use crate::layer1::architecture::building::Building;

    #[test]
    fn test_mobile_building_transforms_state() {
        let mut app = App::new();
        app.add_event::<TransformCommand>();
        app.add_systems(bevy_app::Update, transform_building_system);

        let building = app.world_mut().spawn((
            Building { building_type: crate::layer1::architecture::building::BuildingType::Housing },
            MobileChassis { mode: MobilityMode::Stationary },
            BuildingState::Operational,
        )).id();

        app.world_mut().resource_mut::<Events<TransformCommand>>().send(TransformCommand {
            target: building,
            new_mode: MobilityMode::Mobile,
        });

        app.update();

        let chassis = app.world().get::<MobileChassis>(building).unwrap();
        let state = app.world().get::<BuildingState>(building).unwrap();

        assert_eq!(chassis.mode, MobilityMode::Mobile, "Building chassis should change to Mobile mode.");
        assert_eq!(*state, BuildingState::Offline, "Building must go Offline while mobile.");
    }

    #[test]
    fn test_mobile_building_consumes_fuel_to_move() {
        let mut app = App::new();
        let mut resources = ColonyResources::default();
        resources.fuel = 100.0;
        app.insert_resource(resources);
        app.add_event::<MoveCommand>();
        app.add_systems(bevy_app::Update, move_mobile_building_system);

        let building = app.world_mut().spawn((
            Building { building_type: crate::layer1::architecture::building::BuildingType::Housing },
            GridPosition { x: 0, y: 0 },
            MobileChassis { mode: MobilityMode::Mobile },
        )).id();

        app.world_mut().resource_mut::<Events<MoveCommand>>().send(MoveCommand {
            entity: building,
            destination: GridPosition { x: 1, y: 0 },
        });

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        let pos = app.world().get::<GridPosition>(building).unwrap();

        assert_eq!(resources.fuel, 90.0, "Moving a building should consume fuel (MVP cost 10).");
        assert_eq!(pos.x, 1, "Building should have moved to the destination.");
    }
}
