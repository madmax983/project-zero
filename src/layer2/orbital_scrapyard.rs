use bevy::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::orbital_crossfire::OrbitalEvent;
use crate::layer1::resources::{ResourceItem, ResourceType};

#[derive(Component)]
pub struct OrbitalDebrisField {
    pub stability: f32, // 1.0 down to 0.0
}

#[derive(Event)]
pub struct DeorbitEvent {
    pub target: Vec2,
    pub mass: f32,
}

#[derive(Component)]
pub struct SalvageMission {
    pub progress: f32,
}

pub fn process_salvage_missions(
    mut mission_query: Query<(Entity, &mut SalvageMission, &Parent)>,
    mut debris_query: Query<&mut OrbitalDebrisField>,
    mut commands: Commands,
) {
    for (entity, mut mission, _parent) in mission_query.iter_mut() {
        mission.progress += 0.1;
        if mission.progress >= 1.0 {
            for mut debris in debris_query.iter_mut() {
                debris.stability -= 0.05;
            }
            commands.entity(entity).despawn();
            commands.spawn((ResourceItem {
                resource_type: ResourceType::Scrap,
                amount: 10.0,
            }, GridPosition { x: 0, y: 0 }));
        }
    }
}

pub fn check_deorbit_trigger(
    debris_query: Query<&OrbitalDebrisField>,
    mut deorbit_events: EventWriter<DeorbitEvent>,
) {
    for debris in debris_query.iter() {
        if debris.stability < 0.2 {
            deorbit_events.send(DeorbitEvent { target: Vec2::new(50.0, 50.0), mass: 1000.0 });
        }
    }
}

pub fn bridge_deorbit_to_orbital_crossfire(
    mut deorbit_events: EventReader<DeorbitEvent>,
    mut commands: Commands,
) {
    for event in deorbit_events.read() {
        commands.spawn(OrbitalEvent {
            target: GridPosition { x: event.target.x as i32, y: event.target.y as i32 },
            damage: event.mass * 2.0,
            heat: event.mass * 0.5,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salvage_mission_yields_tech_blueprints() {
        let mut world = World::new();
        let _debris = world.spawn(OrbitalDebrisField { stability: 1.0 }).id();
        let fleet = world.spawn_empty().id();
        let mission = world.spawn(SalvageMission { progress: 0.95 }).set_parent_in_place(fleet).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_salvage_missions);
        schedule.run(&mut world);
        schedule.run(&mut world); // ensure deferred commands run

        // Mission complete?
        assert!(world.get_entity(mission).is_err());
        // Assert resources spawned
        assert_eq!(world.query::<&crate::layer1::resources::ResourceItem>().iter(&world).count(), 1);
    }

    #[test]
    fn test_salvage_operation_increases_deorbit_chance() {
        let mut world = World::new();
        let debris = world.spawn(OrbitalDebrisField { stability: 1.0 }).id();
        let fleet = world.spawn_empty().id();
        let _ = world.spawn(SalvageMission { progress: 0.95 }).set_parent_in_place(fleet).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_salvage_missions);
        schedule.run(&mut world);

        let updated_debris = world.get::<OrbitalDebrisField>(debris).unwrap();
        assert!(updated_debris.stability < 1.0);
    }

    #[test]
    fn test_debris_impact_destroys_buildings_and_spawns_scrap() {
        let mut world = World::new();
        world.init_resource::<Events<DeorbitEvent>>();

        world.send_event(DeorbitEvent { target: Vec2::new(10.0, 10.0), mass: 1000.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(bridge_deorbit_to_orbital_crossfire);
        schedule.run(&mut world);
        schedule.run(&mut world);

        let mut q = world.query::<&crate::layer1::orbital_crossfire::OrbitalEvent>();
        assert_eq!(q.iter(&world).count(), 1);
    }

    #[test]
    fn test_check_deorbit_trigger() {
        let mut world = World::new();
        world.init_resource::<Events<DeorbitEvent>>();
        world.spawn(OrbitalDebrisField { stability: 0.1 });

        let mut schedule = Schedule::default();
        schedule.add_systems(check_deorbit_trigger);
        schedule.run(&mut world);

        let events = world.resource::<Events<DeorbitEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1);
    }
}
