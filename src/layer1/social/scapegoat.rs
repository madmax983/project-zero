use crate::layer1::pop::Pop;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::social::unrest::ScapegoatTarget;
use crate::layer1::social::unrest::Unrest;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Guilt;

#[derive(Component)]
pub struct Banished;

#[derive(Event)]
pub struct ScapegoatEvent {
    pub target: Entity,
}

#[derive(Event)]
pub struct ScapegoatPunishedEvent {
    pub target: Entity,
}

#[allow(clippy::type_complexity)]
pub fn generate_scapegoat_system(
    unrest: Res<Unrest>,
    pops: Query<(Entity, Option<&Traits>), (With<Pop>, Without<ScapegoatTarget>)>,
    existing: Query<(), With<ScapegoatTarget>>,
    mut commands: Commands,
    mut events: EventWriter<ScapegoatEvent>,
) {
    if !existing.is_empty() {
        return;
    }
    if unrest.level >= 0.8 {
        let mut best_target = None;
        for (entity, traits_opt) in pops.iter() {
            if let Some(traits) = traits_opt {
                if traits.has(Trait::Outsider) || traits.has(Trait::Mutant) {
                    best_target = Some(entity);
                    break;
                }
            }
            if best_target.is_none() {
                best_target = Some(entity);
            }
        }
        if let Some(target) = best_target {
            commands.entity(target).insert(ScapegoatTarget);
            events.send(ScapegoatEvent { target });
        }
    }
}

pub fn punish_scapegoat_system(
    mut events: EventReader<ScapegoatPunishedEvent>,
    mut unrest: ResMut<Unrest>,
    mut commands: Commands,
    bystanders: Query<Entity, (With<Pop>, Without<ScapegoatTarget>)>,
) {
    for event in events.read() {
        unrest.level = (unrest.level - 0.3).max(0.0);
        commands.entity(event.target).insert(Banished);

        for bystander in bystanders.iter() {
            commands.entity(bystander).insert(Guilt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_high_unrest_generates_scapegoat() {
        let mut app = App::new();
        app.add_event::<ScapegoatEvent>();
        app.init_resource::<Unrest>();
        app.add_systems(Update, generate_scapegoat_system);

        let pop_id = app.world_mut().spawn(Pop).id();
        app.world_mut().resource_mut::<Unrest>().level = 0.8;

        app.update();

        let events = app.world().resource::<Events<ScapegoatEvent>>();
        let mut reader = events.get_cursor();
        let evs: Vec<_> = reader.read(events).collect();

        assert_eq!(evs.len(), 1, "Should generate exactly one ScapegoatEvent");
        assert_eq!(
            evs[0].target, pop_id,
            "The spawned pop should be the target"
        );
        assert!(
            app.world().get::<ScapegoatTarget>(pop_id).is_some(),
            "Target pop must have the ScapegoatTarget component"
        );
    }

    #[test]
    fn test_punishing_scapegoat_reduces_unrest_and_adds_guilt() {
        let mut app = App::new();
        app.add_event::<ScapegoatPunishedEvent>();
        app.init_resource::<Unrest>();
        app.add_systems(Update, punish_scapegoat_system);

        let scapegoat_id = app.world_mut().spawn((Pop, ScapegoatTarget)).id();
        let bystander_id = app.world_mut().spawn(Pop).id();

        app.world_mut().resource_mut::<Unrest>().level = 0.8;

        app.world_mut().send_event(ScapegoatPunishedEvent {
            target: scapegoat_id,
        });
        app.update();

        let unrest = app.world().resource::<Unrest>();
        assert!(
            unrest.level < 0.8,
            "Unrest should be reduced after punishing the scapegoat"
        );
        assert!(
            app.world().get::<Guilt>(bystander_id).is_some(),
            "Bystander pop should receive the Guilt trait"
        );
        assert!(
            app.world().get::<Banished>(scapegoat_id).is_some(),
            "Scapegoat should receive the Banished component"
        );
    }
}
