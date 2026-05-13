use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Grudge {
    pub target_entity: Entity,
    pub intensity: f32,
    pub origin_reason: String,
}

#[derive(Component, Clone, Default)]
pub struct GrudgeList(pub Vec<Grudge>);

#[derive(Component)]
pub struct Lineage {
    pub parent_entity: Option<Entity>,
}

#[allow(clippy::type_complexity)]
pub fn inherit_grudges_on_birth_system(
    mut events: EventReader<crate::layer1::pop::PopBorn>,
    mut queries: ParamSet<(
        Query<(&Lineage, &mut GrudgeList)>,
        Query<&GrudgeList>,
    )>,
) {
    for event in events.read() {
        // Collect parent grudges first
        let mut parent_grudges_clone = Vec::new();
        if let Ok((lineage, _)) = queries.p0().get(event.entity) {
            if let Some(parent_entity) = lineage.parent_entity {
                if let Ok(parent_grudges) = queries.p1().get(parent_entity) {
                    parent_grudges_clone = parent_grudges.0.clone();
                }
            }
        }

        // Then apply to child
        if !parent_grudges_clone.is_empty() {
            if let Ok((_, mut child_grudges)) = queries.p0().get_mut(event.entity) {
                for grudge in parent_grudges_clone {
                    child_grudges.0.push(grudge);
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn transfer_grudges_on_death_system(
    mut events: EventReader<crate::layer1::pop::PopDied>,
    mut queries: ParamSet<(
        Query<&GrudgeList>,
        Query<(&Lineage, &mut GrudgeList)>,
    )>,
) {
    for event in events.read() {
        let mut dead_grudges_clone = Vec::new();
        if let Ok(dead_grudges) = queries.p0().get(event.entity) {
            dead_grudges_clone = dead_grudges.0.clone();
        }

        if dead_grudges_clone.is_empty() {
            continue;
        }

        let dead_entity = event.entity;
        for (lineage, mut child_grudges) in queries.p1().iter_mut() {
            if lineage.parent_entity == Some(dead_entity) {
                for grudge in &dead_grudges_clone {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{PopBorn, PopDied};
    use crate::layer1::pop::Pop;

    #[test]
    fn test_child_inherits_parent_grudges() {
        let mut app = bevy::app::App::new();
        app.add_event::<PopBorn>();
        app.add_systems(bevy::app::Update, inherit_grudges_on_birth_system);

        let target_entity = Entity::from_raw(2);

        let parent = app.world_mut().spawn((
            Pop,
            GrudgeList(vec![Grudge {
                target_entity,
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
            source: "Birth".to_string(),
        });

        app.update();

        let child_grudges = app.world().get::<GrudgeList>(child).unwrap();
        assert_eq!(child_grudges.0.len(), 1);
        assert_eq!(child_grudges.0[0].target_entity, target_entity);
        assert_eq!(child_grudges.0[0].origin_reason, "Stole a ration");
    }

    #[test]
    fn test_grudge_transfers_on_death() {
        let mut app = bevy::app::App::new();
        app.add_event::<PopDied>();
        app.add_systems(bevy::app::Update, transfer_grudges_on_death_system);

        let target_entity = Entity::from_raw(2);

        let parent = app.world_mut().spawn((
            Pop,
            GrudgeList(vec![Grudge {
                target_entity,
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
            reason: "Murder".to_string(),
        });

        app.update();

        let child_grudges = app.world().get::<GrudgeList>(child).unwrap();
        assert_eq!(child_grudges.0.len(), 1);
        assert_eq!(child_grudges.0[0].target_entity, target_entity);
        assert!(child_grudges.0[0].intensity >= 80.0);
    }
}
