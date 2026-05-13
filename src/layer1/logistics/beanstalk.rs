use crate::layer1::architecture::structure::Structure;
use crate::layer1::map::GridPosition;
use crate::layer1::social::morale::Morale;
use bevy::prelude::*;

#[derive(Component)]
pub struct BeanstalkWorker {
    pub assigned_beanstalk: Entity,
}

#[derive(Component, Default, PartialEq, Eq, Debug)]
pub enum BeanstalkState {
    #[default]
    Operational,
    CableLock,
    Destroyed,
}

#[derive(Component, Default)]
pub struct Beanstalk {
    pub state: BeanstalkState,
}

#[derive(Event)]
pub struct BeanstalkEvent {
    pub entity: Entity,
    pub origin: GridPosition,
    pub direction: Vec2,
}

#[allow(clippy::type_complexity, clippy::needless_pass_by_value)]
pub fn beanstalk_morale_system(
    mut beanstalk_query: Query<(Entity, &mut Beanstalk, &GridPosition)>,
    worker_query: Query<(&BeanstalkWorker, &Morale)>,
    mut events: EventWriter<BeanstalkEvent>,
) {
    for (beanstalk_entity, mut beanstalk, position) in beanstalk_query.iter_mut() {
        if beanstalk.state == BeanstalkState::Destroyed {
            continue;
        }

        let mut total_morale = 0.0;
        let mut worker_count = 0;

        for (worker, morale) in worker_query.iter() {
            if worker.assigned_beanstalk == beanstalk_entity {
                total_morale += morale.value;
                worker_count += 1;
            }
        }

        if worker_count > 0 {
            let average_morale = total_morale / worker_count as f32 * 100.0; // scale up by 100 to match tests

            if average_morale < 10.0 {
                events.send(BeanstalkEvent {
                    entity: beanstalk_entity,
                    origin: *position,
                    direction: Vec2::new(1.0, 0.0), // Arbitrary fallback direction
                });
                beanstalk.state = BeanstalkState::Destroyed;
            } else if average_morale < 30.0 {
                beanstalk.state = BeanstalkState::CableLock;
            } else {
                beanstalk.state = BeanstalkState::Operational;
            }
        }
    }
}

pub fn calculate_fall_trajectory(
    origin: GridPosition,
    direction: Vec2,
    length: i32,
) -> Vec<GridPosition> {
    let mut path = Vec::new();
    let dx = direction.x;
    let dy = direction.y;

    for i in 1..=length {
        path.push(GridPosition {
            x: origin.x + (dx * i as f32) as i32,
            y: origin.y + (dy * i as f32) as i32,
        });
    }
    path
}

#[allow(clippy::needless_pass_by_value)]
pub fn beanstalk_collapse_system(
    mut events: EventReader<BeanstalkEvent>,
    mut commands: Commands,
    mut structure_query: Query<(Entity, &mut Structure, &GridPosition)>,
) {
    for event in events.read() {
        let BeanstalkEvent {
            entity,
            origin,
            direction,
        } = event;
        let path = calculate_fall_trajectory(*origin, *direction, 100);

        for (struct_entity, mut structure, pos) in structure_query.iter_mut() {
            if path.contains(pos) {
                structure.current_hp -= 10000.0; // Massive damage
                if structure.current_hp <= 0.0 {
                    commands.entity(struct_entity).despawn();
                }
            }
        }

        if let Some(mut cmd) = commands.get_entity(*entity) {
            cmd.despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        beanstalk_collapse_system, beanstalk_morale_system, Beanstalk, BeanstalkEvent,
        BeanstalkState, BeanstalkWorker,
    };
    use crate::layer1::architecture::structure::Structure;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::map::GridPosition;
    use crate::layer1::social::morale::Morale;
    use bevy::prelude::*;

    #[test]
    fn test_beanstalk_cable_lock_on_low_morale() {
        let mut app = App::new();
        app.add_event::<BeanstalkEvent>();
        app.add_systems(Update, beanstalk_morale_system);

        let beanstalk_entity = app
            .world_mut()
            .spawn((
                Beanstalk {
                    state: BeanstalkState::Operational,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        // Spawn workers with low morale
        app.world_mut().spawn((
            Pop,
            BeanstalkWorker {
                assigned_beanstalk: beanstalk_entity,
            },
            Morale {
                modifiers: vec![],
                value: 0.20,
            }, // Below threshold for Cable Lock (30.0%)
        ));

        app.update();

        let beanstalk = app.world().get::<Beanstalk>(beanstalk_entity).unwrap();
        assert_eq!(
            beanstalk.state,
            BeanstalkState::CableLock,
            "Beanstalk should enter Cable Lock state when worker morale is critically low."
        );
    }

    #[test]
    fn test_beanstalk_sever_on_critical_morale() {
        let mut app = App::new();
        app.add_event::<BeanstalkEvent>();
        app.add_systems(Update, beanstalk_morale_system);

        let beanstalk_entity = app
            .world_mut()
            .spawn((
                Beanstalk {
                    state: BeanstalkState::CableLock,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        // Spawn workers with critical morale
        app.world_mut().spawn((
            Pop,
            BeanstalkWorker {
                assigned_beanstalk: beanstalk_entity,
            },
            Morale {
                modifiers: vec![],
                value: 0.05,
            }, // Below threshold for Sever (10.0%)
        ));

        app.update();

        let events = app.world().resource::<Events<BeanstalkEvent>>();
        let mut reader = events.get_cursor();
        let sever_events: Vec<_> = reader.read(events).collect();

        assert_eq!(
            sever_events.len(),
            1,
            "A BeanstalkSevered event should be emitted."
        );
        let BeanstalkEvent { entity, origin, .. } = sever_events[0];
        assert_eq!(*entity, beanstalk_entity);
        assert_eq!(origin.x, 10);
    }

    #[test]
    fn test_beanstalk_collapse_damage_pattern() {
        let mut app = App::new();
        app.add_event::<BeanstalkEvent>();
        app.add_systems(Update, beanstalk_collapse_system);

        let origin = GridPosition { x: 10, y: 10 };

        // Spawn a line of buildings to be crushed
        let target1 = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 11, y: 10 },
            ))
            .id();
        let target2 = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 12, y: 10 },
            ))
            .id();
        let safe_target = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 10, y: 11 },
            ))
            .id(); // Off-axis

        let beanstalk_entity = app.world_mut().spawn(Beanstalk::default()).id();

        // Trigger the collapse event
        app.world_mut().send_event(BeanstalkEvent {
            entity: beanstalk_entity,
            origin,
            direction: Vec2::new(1.0, 0.0),
        });

        app.update();

        // Verify catastrophic damage on the line
        assert!(
            app.world().get::<Structure>(target1).is_none()
                || app.world().get::<Structure>(target1).unwrap().current_hp <= 0.0,
            "Building in fall path should be destroyed"
        );
        assert!(
            app.world().get::<Structure>(target2).is_none()
                || app.world().get::<Structure>(target2).unwrap().current_hp <= 0.0,
            "Building further down fall path should be destroyed"
        );

        // Verify safe building is unharmed
        assert_eq!(
            app.world()
                .get::<Structure>(safe_target)
                .unwrap()
                .current_hp,
            100.0,
            "Building off the fall axis should remain unharmed"
        );

        // Verify Beanstalk entity is despawned
        assert!(
            app.world().get::<Beanstalk>(beanstalk_entity).is_none(),
            "Beanstalk should be despawned after collapse"
        );
    }
}
