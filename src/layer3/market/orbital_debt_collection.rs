use bevy::prelude::*;

#[derive(Resource)]
pub struct ColonyFinances {
    pub debt: f32,
    pub defaulted: bool,
}

#[derive(Component)]
pub struct BuildingValue {
    pub value: f32,
}

#[derive(Event)]
pub struct RepoDroneArrivalEvent;

#[derive(Component)]
pub struct RepoDrone {
    pub target: Entity,
}

pub fn process_loan_default_system(
    finances: Res<ColonyFinances>,
    mut events: EventWriter<RepoDroneArrivalEvent>,
) {
    if finances.defaulted && finances.debt > 0.0 {
        events.send(RepoDroneArrivalEvent);
    }
}

pub fn execute_repo_drone_extraction_system(
    mut commands: Commands,
    mut finances: ResMut<ColonyFinances>,
    repo_query: Query<(Entity, &RepoDrone)>,
    building_query: Query<&BuildingValue>,
) {
    for (drone_entity, drone) in repo_query.iter() {
        if let Ok(building) = building_query.get(drone.target) {
            finances.debt -= building.value; // Pay off debt with the building's value
            if finances.debt <= 0.0 {
                finances.defaulted = false;
                finances.debt = 0.0;
            }
            commands.entity(drone.target).despawn_recursive(); // Extract building
            commands.entity(drone_entity).despawn(); // Drone leaves
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaulting_on_loan_triggers_repo_drones() {
        let mut app = App::new();
        app.add_event::<RepoDroneArrivalEvent>();
        app.add_systems(Update, process_loan_default_system);

        app.world_mut().insert_resource(ColonyFinances {
            debt: 100_000.0,
            defaulted: true,
        });

        app.update();

        let repo_events = app.world().resource::<Events<RepoDroneArrivalEvent>>();
        let mut reader = repo_events.get_cursor();
        assert!(reader.read(repo_events).len() > 0, "Loan default should spawn repo drones in orbit");
    }

    #[test]
    fn test_repo_drones_remove_high_value_buildings_from_layer_1() {
        let mut app = App::new();
        app.add_systems(Update, execute_repo_drone_extraction_system);
        app.world_mut().insert_resource(ColonyFinances { debt: 100_000.0, defaulted: true });

        let hospital = app.world_mut().spawn(
            BuildingValue { value: 50_000.0 }, // High value
        ).id();

        app.world_mut().spawn(RepoDrone { target: hospital });

        app.update();

        assert!(app.world().get::<BuildingValue>(hospital).is_none(), "Repo drone should extract (despawn) the high-value building");
    }

    #[test]
    fn test_repo_drones_reduce_colony_debt_upon_extraction() {
        let mut app = App::new();
        app.add_systems(Update, execute_repo_drone_extraction_system);

        app.world_mut().insert_resource(ColonyFinances { debt: 100_000.0, defaulted: true });
        let generator = app.world_mut().spawn(BuildingValue { value: 40_000.0 }).id();
        app.world_mut().spawn(RepoDrone { target: generator });

        app.update();

        let finances = app.world().resource::<ColonyFinances>();
        assert_eq!(finances.debt, 60_000.0, "Extracting a building should reduce the debt by its value");
    }
}
