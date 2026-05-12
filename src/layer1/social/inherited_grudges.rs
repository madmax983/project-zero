#[allow(unused_imports)]
use bevy_ecs::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum WrongReason {
    UnjustImprisonment,
    StarvationNeglect,
}

#[derive(Event)]
pub struct SevereWrongEvent {
    pub victim: Entity,
    pub perpetrator: Entity,
    pub reason: WrongReason,
}

#[derive(Event)]
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
    mut query: Query<&mut Vendettas>,
) {
    let mut inheritances: Vec<(Entity, Vec<Entity>)> = Vec::new();

    for event in events.read() {
        if let Ok(parent_vendettas) = query.get(event.parent) {
            inheritances.push((event.child, parent_vendettas.targets.clone()));
        }
    }

    for (child, targets) in inheritances {
        if let Ok(mut child_vendettas) = query.get_mut(child) {
            for target in &targets {
                if !child_vendettas.targets.contains(target) {
                    child_vendettas.targets.push(*target);
                }
            }
        } else {
            commands.entity(child).insert(Vendettas {
                targets: targets.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::PopBundle;
    #[allow(unused_imports)]
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use rand::SeedableRng;

    #[test]
    fn test_severe_wrong_creates_vendetta() {
        let mut world = World::new();
        world.init_resource::<Events<SevereWrongEvent>>();

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let victim = world.spawn(PopBundle::random(0, 0, &mut rng)).id();
        let perpetrator = world.spawn(PopBundle::random(0, 0, &mut rng)).id();

        // Simulate a severe wrong
        world
            .resource_mut::<Events<SevereWrongEvent>>()
            .send(SevereWrongEvent {
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
        world.init_resource::<Events<ChildBornEvent>>();

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let perpetrator = world.spawn(PopBundle::random(0, 0, &mut rng)).id();

        let parent = world
            .spawn((
                PopBundle::random(0, 0, &mut rng),
                Vendettas {
                    targets: vec![perpetrator],
                },
            ))
            .id();

        let child = world.spawn(PopBundle::random(0, 0, &mut rng)).id();

        world
            .resource_mut::<Events<ChildBornEvent>>()
            .send(ChildBornEvent { parent, child });

        world.run_system_once(inherit_vendettas_system).unwrap();

        let child_vendettas = world.get::<Vendettas>(child).unwrap();
        // Child should inherit the grudge against the perpetrator
        assert!(child_vendettas.targets.contains(&perpetrator));
    }
}
