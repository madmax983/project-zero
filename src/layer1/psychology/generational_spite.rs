use bevy_ecs::prelude::*;
use crate::layer1::entities::pop::ReproductionEvent;
use crate::layer2::leadership::{Admiral, FleetMutinyEvent};

#[derive(Component, Clone)]
pub struct SpiteTrait {
    pub intensity: u32,
    pub target_faction: u32,
}

const SPITE_MUTINY_THRESHOLD: u32 = 10;

pub fn inherit_spite_system(
    mut commands: Commands,
    mut repro_events: EventReader<ReproductionEvent>,
    query: Query<&SpiteTrait>,
) {
    for event in repro_events.read() {
        // Simple inheritance: if parent A has spite, child gets it
        if let Ok(spite) = query.get(event.parent_a) {
            commands.entity(event.child).insert(spite.clone());
        }
    }
}

pub fn evaluate_spiteful_leader_system(
    query: Query<(Entity, &Admiral, &SpiteTrait)>,
    mut mutiny_events: EventWriter<FleetMutinyEvent>,
) {
    for (entity, admiral, spite) in query.iter() {
        // Assume faction 0 is player for MVP
        if spite.intensity >= SPITE_MUTINY_THRESHOLD && spite.target_faction == 0 {
            mutiny_events.send(FleetMutinyEvent {
                admiral: entity,
                fleet: admiral.fleet,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::entities::pop::Pop;

    #[test]
    fn test_spite_is_inherited_by_offspring() {
        let mut app = App::new();
        app.add_event::<ReproductionEvent>();
        app.add_systems(Update, inherit_spite_system);

        // Setup Parent with Spite
        let parent = app.world_mut().spawn((
            Pop,
            SpiteTrait { intensity: 5, target_faction: 1 },
        )).id();

        // Setup Child
        let child = app.world_mut().spawn(Pop).id();

        // Simulate reproduction
        app.world_mut().resource_mut::<Events<ReproductionEvent>>().send(ReproductionEvent {
            parent_a: parent,
            parent_b: parent, // Asexual for test simplicity
            child,
        });

        app.update();

        // Verify child inherited spite
        let child_spite = app.world().get::<SpiteTrait>(child).expect("Child should have inherited SpiteTrait.");
        assert_eq!(child_spite.intensity, 5, "Child should inherit the intensity of the parent's spite.");
        assert_eq!(child_spite.target_faction, 1, "Child should inherit the target of the parent's spite.");
    }

    #[test]
    fn test_admiral_with_deep_spite_triggers_mutiny() {
        let mut app = App::new();
        app.add_event::<FleetMutinyEvent>();
        app.add_systems(Update, evaluate_spiteful_leader_system);

        // Setup Spiteful Admiral
        let admiral = app.world_mut().spawn((
            Pop,
            Admiral { fleet: Entity::from_raw(99) },
            SpiteTrait { intensity: 10, target_faction: 0 }, // 0 is Player Faction
        )).id();

        app.update();

        // Verify Mutiny event was fired
        let mutiny_events = app.world().resource::<Events<FleetMutinyEvent>>();
        let mut reader = mutiny_events.get_cursor();
        let mut found = false;
        for event in reader.read(&mutiny_events) {
            if event.admiral == admiral {
                found = true;
            }
        }

        assert!(found, "An admiral with high spite against the current faction should mutiny.");
    }
}
