use bevy_ecs::prelude::*;
use crate::layer1::entities::pop::{PopBorn, PopDied, Pop};

#[derive(Component)]
pub struct Lineage {
    pub parent_entity: Option<Entity>,
}

#[derive(Component, Debug, Clone)]
pub struct Grudge {
    pub target_entity: Entity,
    pub intensity: f32,
    pub origin_reason: String,
}

#[derive(Component, Clone, Default)]
pub struct GrudgeList(pub Vec<Grudge>);

#[allow(clippy::type_complexity)]
pub fn inherit_grudges_on_birth_system(
    mut events: EventReader<PopBorn>,
    mut param_set: ParamSet<(
        Query<&GrudgeList>,
        Query<(&Lineage, &mut GrudgeList)>,
    )>
) {
    for event in events.read() {
        let mut parent_grudges_clone: Option<Vec<Grudge>> = None;
        let mut child_has_parent = None;

        // Find child's parent entity
        if let Ok((lineage, _)) = param_set.p1().get(event.entity) {
            child_has_parent = lineage.parent_entity;
        }

        // Get parent's grudges
        if let Some(parent_entity) = child_has_parent {
            if let Ok(parent_grudges) = param_set.p0().get(parent_entity) {
                parent_grudges_clone = Some(parent_grudges.0.clone());
            }
        }

        // Apply grudges to child
        if let Some(grudges) = parent_grudges_clone {
            if let Ok((_, mut child_grudges)) = param_set.p1().get_mut(event.entity) {
                for grudge in grudges {
                    child_grudges.0.push(grudge);
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn transfer_grudges_on_death_system(
    mut events: EventReader<PopDied>,
    mut param_set: ParamSet<(
        Query<&GrudgeList, With<Pop>>,
        Query<(&Lineage, &mut GrudgeList)>,
    )>
) {
    for event in events.read() {
        let mut dead_grudges_clone: Option<Vec<Grudge>> = None;

        if let Ok(dead_grudges) = param_set.p0().get(event.entity) {
            dead_grudges_clone = Some(dead_grudges.0.clone());
        }

        if let Some(dead_grudges) = dead_grudges_clone {
            let mut children_query = param_set.p1();
            for (lineage, mut child_grudges) in children_query.iter_mut() {
                if lineage.parent_entity == Some(event.entity) {
                    for grudge in &dead_grudges {
                        let mut found = false;
                        for existing_grudge in &mut child_grudges.0 {
                            if existing_grudge.target_entity == grudge.target_entity {
                                existing_grudge.intensity += grudge.intensity * 0.5;
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            let mut inherited_grudge = grudge.clone();
                            inherited_grudge.intensity *= 1.2;
                            child_grudges.0.push(inherited_grudge);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PopBorn>();
        app.add_event::<PopDied>();
        app.add_systems(Update, (inherit_grudges_on_birth_system, transfer_grudges_on_death_system));
        app
    }

    #[test]
    fn test_child_inherits_parent_grudges() {
        let mut app = setup_app();

        let target = app.world_mut().spawn(Pop).id();

        let parent = app.world_mut().spawn((
            Pop,
            Lineage { parent_entity: None }, // Parent also has lineage
            GrudgeList(vec![Grudge {
                target_entity: target,
                intensity: 50.0,
                origin_reason: "Stole a ration".to_string(),
            }]),
        )).id();

        let child = app.world_mut().spawn((
            Pop,
            Lineage { parent_entity: Some(parent) },
            GrudgeList(vec![]),
        )).id();

        app.world_mut().send_event(PopBorn {
            entity: child,
            name: "Child".to_string(),
            tick: 0,
            source: "Test".to_string()
        });

        app.update();

        let child_grudges = app.world().get::<GrudgeList>(child).unwrap();
        assert_eq!(child_grudges.0.len(), 1);
        assert_eq!(child_grudges.0[0].target_entity, target);
        assert_eq!(child_grudges.0[0].origin_reason, "Stole a ration");
    }

    #[test]
    fn test_grudge_transfers_on_death() {
        let mut app = setup_app();

        let target = app.world_mut().spawn(Pop).id();

        let parent = app.world_mut().spawn((
            Pop,
            Lineage { parent_entity: None }, // Parent also has lineage
            GrudgeList(vec![Grudge {
                target_entity: target,
                intensity: 80.0,
                origin_reason: "Killed my kin".to_string(),
            }]),
        )).id();

        let child = app.world_mut().spawn((
            Pop,
            Lineage { parent_entity: Some(parent) },
            GrudgeList(vec![]),
        )).id();

        app.world_mut().send_event(PopDied {
            entity: parent,
            name: "Parent".to_string(),
            tick: 0,
            reason: "Test".to_string()
        });

        app.update();

        let child_grudges = app.world().get::<GrudgeList>(child).unwrap();
        assert_eq!(child_grudges.0.len(), 1);
        assert_eq!(child_grudges.0[0].target_entity, target);
        assert!(child_grudges.0[0].intensity >= 80.0);
        assert_eq!(child_grudges.0[0].intensity, 96.0); // 80.0 * 1.2
    }
}
