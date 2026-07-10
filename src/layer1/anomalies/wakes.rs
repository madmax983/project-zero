use crate::layer1::building::Building;
use crate::layer1::flora::Flora;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer2::ftl::wakes::SubspaceWakeEvent;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct SubspaceFlora;

pub fn resolve_subspace_wake_system(
    mut commands: Commands,
    mut wake_events: EventReader<SubspaceWakeEvent>,
    pops: Query<(Entity, &GridPosition), With<Pop>>,
    buildings: Query<(Entity, &GridPosition), With<Building>>,
) {
    let mut rng = rand::thread_rng();

    for wake in wake_events.read() {
        if wake.severity > 0.5 {
            // High severity: teleport entities away
            for (entity, _pos) in pops.iter() {
                if rng.gen::<f32>() < wake.severity * 0.1 {
                    commands.entity(entity).despawn();
                }
            }

            for (entity, _pos) in buildings.iter() {
                if rng.gen::<f32>() < wake.severity * 0.05 {
                    commands.entity(entity).despawn();
                }
            }
        } else {
            // Low severity: teleport subspace flora in
            let count = (wake.severity * 10.0) as u32;
            for _ in 0..count {
                let x = rng.gen_range(0..50);
                let y = rng.gen_range(0..50);
                commands.spawn((Flora::default(), SubspaceFlora, GridPosition { x, y }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;
    use crate::layer1::health::Health;
    use crate::layer1::items::Equipment;
    use crate::layer1::memory::Memories;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Speed;
    use crate::layer1::rumor::Knowledge;
    use crate::layer1::skills::Skills;
    use crate::layer1::utility_ai::PopAction;
    use crate::layer1::utility_ai::UtilityWeights;
    use bevy_ecs::schedule::Schedule;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Events::<SubspaceWakeEvent>::default());
        world
    }

    #[test]
    fn test_high_severity_wake_teleports_entities() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                Speed::default(),
                UtilityWeights::default(),
                Health::default(),
                Needs::default(),
                Skills::default(),
                PopAction::default(),
                Memories::default(),
                Equipment::default(),
                Knowledge::default(),
            ))
            .id();

        let building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 20, y: 20 },
            ))
            .id();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: Entity::PLACEHOLDER,
                severity: 21.0, // Ensures severity * 0.05 > 1.0
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wake_system);
        schedule.run(&mut world);

        assert!(world.get_entity(pop).is_err());
        assert!(world.get_entity(building).is_err());
    }

    #[test]
    fn test_low_severity_wake_teleports_flora() {
        let mut world = setup_world();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: Entity::PLACEHOLDER,
                severity: 0.4,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wake_system);
        schedule.run(&mut world);

        let flora_count = world.query::<&SubspaceFlora>().iter(&world).count();
        assert_eq!(flora_count, 4);
    }
}
