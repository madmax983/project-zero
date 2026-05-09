use bevy_ecs::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum WrongReason {
    UnjustImprisonment,
    StarvationNeglect,
}

#[derive(Event, Clone, Debug)]
pub struct SevereWrongEvent {
    pub victim: Entity,
    pub perpetrator: Entity,
    pub reason: WrongReason,
}

#[derive(Event, Clone, Debug)]
pub struct ChildBornEvent {
    pub parent: Entity,
    pub child: Entity,
}

#[derive(Component, Default)]
pub struct Vendettas {
    pub targets: Vec<Entity>,
}

pub fn handle_severe_wrongs_system(
    mut commands: Commands,
    mut events: EventReader<SevereWrongEvent>,
    mut query: Query<&mut Vendettas>,
) {
    for event in events.read() {
        if let Ok(mut vendettas) = query.get_mut(event.victim) {
            if !vendettas.targets.contains(&event.perpetrator) {
                vendettas.targets.push(event.perpetrator);
            }
        } else {
            commands.entity(event.victim).insert(Vendettas {
                targets: vec![event.perpetrator],
            });
        }
    }
}

pub fn inherit_vendettas_system(
    mut commands: Commands,
    mut events: EventReader<ChildBornEvent>,
    mut set: ParamSet<(
        Query<&Vendettas>,
        Query<&mut Vendettas>,
    )>,
) {
    for event in events.read() {
        let parent_targets = if let Ok(parent_vendettas) = set.p0().get(event.parent) {
            Some(parent_vendettas.targets.clone())
        } else {
            None
        };

        if let Some(targets) = parent_targets {
            if let Ok(mut child_vendettas) = set.p1().get_mut(event.child) {
                for target in targets {
                    if !child_vendettas.targets.contains(&target) {
                        child_vendettas.targets.push(target);
                    }
                }
            } else {
                commands.entity(event.child).insert(Vendettas {
                    targets,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_severe_wrong_creates_vendetta() {
        let mut world = World::new();
        world.insert_resource(Events::<SevereWrongEvent>::default());

        let victim = world.spawn(Pop).id();
        let perpetrator = world.spawn(Pop).id();

        // Simulate a severe wrong
        world.resource_mut::<Events<SevereWrongEvent>>().send(SevereWrongEvent {
            victim,
            perpetrator,
            reason: WrongReason::UnjustImprisonment,
        });

        world.run_system_once(handle_severe_wrongs_system).unwrap();

        let vendettas = world.get::<Vendettas>(victim).unwrap();
        assert!(vendettas.targets.contains(&perpetrator));
    }

    #[test]
    fn test_vendetta_is_inherited_by_offspring() {
        let mut world = World::new();
        world.insert_resource(Events::<ChildBornEvent>::default());

        let perpetrator = world.spawn(Pop).id();

        let parent = world.spawn((
            Pop,
            Vendettas { targets: vec![perpetrator] },
        )).id();

        let child = world.spawn(Pop).id();

        world.resource_mut::<Events<ChildBornEvent>>().send(ChildBornEvent {
            parent,
            child,
        });

        world.run_system_once(inherit_vendettas_system).unwrap();

        let child_vendettas = world.get::<Vendettas>(child).unwrap();
        // Child should inherit the grudge against the perpetrator
        assert!(child_vendettas.targets.contains(&perpetrator));
    }
}
