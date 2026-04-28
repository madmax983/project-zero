use bevy_ecs::prelude::*;
use bevy_time::Time;

#[derive(Resource)]
pub struct ImperialStanding {
    pub value: i32,
}

#[derive(Resource)]
pub struct GlobalEfficiency {
    pub value: f32,
}

#[derive(Component)]
pub struct VainGovernor {
    pub is_vain: bool,
}

#[derive(Resource, Default)]
pub struct ActiveDemands {
    pub vanity_demand_active: bool,
    pub time_since_demand: f32,
    pub active_governor_entity: Option<Entity>,
}

#[derive(Component)]
pub struct VanityProject;

pub fn vanity_building_listener_system(
    mut events: EventReader<crate::layer1::core::events::BuildingCompletedEvent>,
    standing: Option<ResMut<ImperialStanding>>,
    demands: Option<ResMut<ActiveDemands>>,
    buildings: Query<(&crate::layer1::architecture::building::Building, Option<&VanityProject>)>,
) {
    if let (Some(mut standing_res), Some(mut active_demands)) = (standing, demands) {
        for event in events.read() {
            if let Ok((building, vanity)) = buildings.get(event.entity) {
                if (vanity.is_some() || building.building_type == crate::layer1::architecture::building::BuildingType::Statue)
                    && active_demands.vanity_demand_active {
                        active_demands.vanity_demand_active = false;
                        standing_res.value += 10;
                }
            }
        }
    }
}

pub fn vanity_sabotage_system(
    time: Option<Res<Time>>,
    demands: Option<ResMut<ActiveDemands>>,
    efficiency: Option<ResMut<GlobalEfficiency>>,
) {
    if let (Some(mut eff), Some(mut active_demands)) = (efficiency, demands) {
        if active_demands.vanity_demand_active {
            let delta = time.map_or(0.0, |t| t.delta_secs());
            active_demands.time_since_demand += delta;

            if active_demands.time_since_demand >= 100.0 {
                eff.value *= 0.9;
                active_demands.time_since_demand = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer2::governance::Governor;
    use crate::layer1::core::events::BuildingCompletedEvent;

    #[test]
    fn test_vanity_project_fulfilled_boosts_standing() {
        let mut app = App::new();
        // Setup ...
        app.add_event::<BuildingCompletedEvent>();
        app.add_systems(Update, vanity_building_listener_system);

        // Spawn governor
        let governor = app.world_mut().spawn((
            Governor { pop_entity: Entity::from_raw(0), assigned_at: 0 },
            VainGovernor { is_vain: true },
            // ...
        )).id();

        app.world_mut().insert_resource(ImperialStanding { value: 50 });
        app.world_mut().insert_resource(ActiveDemands {
            vanity_demand_active: true,
            time_since_demand: 0.0,
            active_governor_entity: Some(governor),
        });

        // Act: Player builds the vanity project
        let b = app.world_mut().spawn((
            Building { building_type: BuildingType::Statue },
            VanityProject,
        )).id();

        app.world_mut().resource_mut::<Events<BuildingCompletedEvent>>().send(BuildingCompletedEvent { entity: b });

        app.update();

        // Assert: Standing increased, demand satisfied
        assert_eq!(app.world().resource::<ImperialStanding>().value, 60);
        assert_eq!(app.world().resource::<ActiveDemands>().vanity_demand_active, false);
    }

    #[test]
    fn test_vanity_project_ignored_causes_sabotage() {
        let mut app = App::new();
        // Setup ...
        app.add_systems(Update, vanity_sabotage_system);

        app.world_mut().insert_resource(GlobalEfficiency { value: 1.0 });

        let governor = app.world_mut().spawn((
            Governor { pop_entity: Entity::from_raw(0), assigned_at: 0 },
            VainGovernor { is_vain: true },
        )).id();

        app.world_mut().insert_resource(ActiveDemands {
            vanity_demand_active: true,
            time_since_demand: 100.0,
            active_governor_entity: Some(governor),
        });

        // Act: Time passes, demand ignored
        app.update(); // triggers sabotage

        // Assert: Efficiency reduced
        let efficiency = app.world().resource::<GlobalEfficiency>().value;
        assert!(efficiency < 1.0);
    }
}
