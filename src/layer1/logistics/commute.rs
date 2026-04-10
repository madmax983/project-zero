use crate::layer1::economy::ColonyResources;
use bevy::prelude::*;
// use crate::layer1::entities::Pop;
// use crate::layer2::system::OrbitalBody;

#[derive(Component)]
pub struct CurrentLocation(pub Entity);

#[derive(Component)]
pub struct CommuteAction {
    pub destination: Entity,
    pub fuel_cost: u32,
    pub duration: u32,
    pub progress: u32,
}

pub fn process_orbital_commutes(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(Entity, &mut CommuteAction, &mut CurrentLocation)>,
) {
    for (entity, mut commute, mut loc) in query.iter_mut() {
        if resources.fuel >= commute.fuel_cost as f32 {
            resources.fuel -= commute.fuel_cost as f32;
            commute.progress += 1;

            if commute.progress >= commute.duration {
                loc.0 = commute.destination;
                commands.entity(entity).remove::<CommuteAction>();
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::layer1::entities::Pop;
    use crate::layer2::system::OrbitalBody;

    #[test]
    fn test_orbital_commute_consumes_fuel_and_updates_location() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ColonyResources>();
        app.world_mut().resource_mut::<ColonyResources>().fuel = 10.0;
        app.add_systems(Update, process_orbital_commutes);

        let planet_entity = app.world_mut().spawn(OrbitalBody::default()).id();
        let station_entity = app.world_mut().spawn(OrbitalBody::default()).id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                CurrentLocation(planet_entity),
                CommuteAction {
                    destination: station_entity,
                    fuel_cost: 2,
                    duration: 3,
                    progress: 0,
                },
            ))
            .id();

        // Tick 1
        app.update();
        let commute = app.world().get::<CommuteAction>(pop).unwrap();
        assert_eq!(commute.progress, 1);
        assert_eq!(app.world().resource::<ColonyResources>().fuel, 8.0);
        assert_eq!(
            app.world().get::<CurrentLocation>(pop).unwrap().0,
            planet_entity
        ); // Still en route

        // Tick 2
        app.update();

        // Tick 3 - Commute complete
        app.update();

        let loc = app.world().get::<CurrentLocation>(pop).unwrap();
        assert_eq!(loc.0, station_entity);
        assert!(app.world().get::<CommuteAction>(pop).is_none()); // Action removed
    }

    #[test]
    fn test_commute_stalls_without_fuel() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ColonyResources>();
        app.world_mut().resource_mut::<ColonyResources>().fuel = 0.0; // No fuel
        app.add_systems(Update, process_orbital_commutes);

        let planet_entity = app.world_mut().spawn(OrbitalBody::default()).id();
        let station_entity = app.world_mut().spawn(OrbitalBody::default()).id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                CurrentLocation(planet_entity),
                CommuteAction {
                    destination: station_entity,
                    fuel_cost: 2,
                    duration: 3,
                    progress: 0,
                },
            ))
            .id();

        app.update();

        let commute = app.world().get::<CommuteAction>(pop).unwrap();
        assert_eq!(commute.progress, 0); // No progress
        assert_eq!(
            app.world().get::<CurrentLocation>(pop).unwrap().0,
            planet_entity
        );
    }
}
