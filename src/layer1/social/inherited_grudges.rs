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
    mut queries: ParamSet<(Query<(&Lineage, &mut GrudgeList)>, Query<&GrudgeList>)>,
) {
    for event in events.read() {
        // Extract parent entity
        let parent_entity = queries
            .p0()
            .get(event.entity)
            .ok()
            .and_then(|(lineage, _)| lineage.parent_entity);

        if let Some(parent_entity) = parent_entity {
            // Get parent grudges
            let parent_grudges = if let Ok(grudges) = queries.p1().get(parent_entity) {
                Some(grudges.0.clone())
            } else {
                None
            };

            // Then apply to child
            if let Some(grudges) = parent_grudges {
                if !grudges.is_empty() {
                    if let Ok((_, mut child_grudges)) = queries.p0().get_mut(event.entity) {
                        // ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<_>>()`
                        // and `.push()` loop, extending directly.
                        child_grudges.0.extend(grudges);
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn prevent_grudge_work_system(
    mut commands: Commands,
    grudge_query: Query<(Entity, &GrudgeList, &crate::layer1::actions::AssignedTo)>,
    target_query: Query<&crate::layer1::actions::AssignedTo>,
) {
    for (entity, grudges, assigned_to) in grudge_query.iter() {
        for grudge in &grudges.0 {
            if let Ok(target_assigned) = target_query.get(grudge.target_entity) {
                // If both are assigned to the same workplace, and it's not a generic housing
                if target_assigned.entity == assigned_to.entity {
                    // Refuse to work there
                    commands.entity(entity).remove::<crate::layer1::actions::AssignedTo>();
                    break;
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn transfer_grudges_on_death_system(
    mut events: EventReader<crate::layer1::pop::PopDied>,
    mut queries: ParamSet<(Query<&GrudgeList>, Query<(&Lineage, &mut GrudgeList)>)>,
) {
    for event in events.read() {
        let dead_grudges_clone = if let Ok(dead_grudges) = queries.p0().get(event.entity) {
            dead_grudges.0.clone()
        } else {
            continue;
        };

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
    use crate::layer1::pop::Pop;
    use crate::layer1::pop::{PopBorn, PopDied};

    #[test]
    fn test_child_inherits_parent_grudges() {
        let mut app = bevy::app::App::new();
        app.add_event::<PopBorn>();
        app.add_systems(bevy::app::Update, inherit_grudges_on_birth_system);

        let target_entity = Entity::from_raw(2);

        let parent = app
            .world_mut()
            .spawn((
                Pop,
                GrudgeList(vec![Grudge {
                    target_entity,
                    intensity: 50.0,
                    origin_reason: "Stole a ration".to_string(),
                }]),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                Pop,
                Lineage {
                    parent_entity: Some(parent),
                },
                GrudgeList(vec![]),
            ))
            .id();

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

    use crate::layer1::actions::{AssignedTo, AssignmentType};

    #[test]
    fn test_grudge_work_refusal() {
        let mut app = bevy::app::App::new();
        // Setup systems that prevent grudges from working together
        app.add_systems(bevy::app::Update, prevent_grudge_work_system);

        // Arrange: Setup two pops with a grudge against each other, assigned to the same workplace
        let target = app.world_mut().spawn_empty().id();
        let workplace = app.world_mut().spawn_empty().id();

        // Target works there
        app.world_mut().entity_mut(target).insert((
            Pop,
            AssignedTo {
                entity: workplace,
                assignment_type: AssignmentType::FarmWorker,
            },
        ));

        // Pop hates target, also assigned there
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                GrudgeList(vec![Grudge {
                    target_entity: target,
                    intensity: 1.0,
                    origin_reason: "Unknown".to_string(),
                }]),
                AssignedTo {
                    entity: workplace,
                    assignment_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        // Act: Run systems to prevent this
        app.update();

        // Assert: Verify assignment fails (e.g., AssignedTo is removed)
        assert!(app.world().get::<AssignedTo>(pop).is_none());
    }

    #[test]
    fn test_grudge_transfers_on_death() {
        let mut app = bevy::app::App::new();
        app.add_event::<PopDied>();
        app.add_systems(bevy::app::Update, transfer_grudges_on_death_system);

        let target_entity = Entity::from_raw(2);

        let parent = app
            .world_mut()
            .spawn((
                Pop,
                GrudgeList(vec![Grudge {
                    target_entity,
                    intensity: 80.0,
                    origin_reason: "Killed my kin".to_string(),
                }]),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                Pop,
                Lineage {
                    parent_entity: Some(parent),
                },
                GrudgeList(vec![]),
            ))
            .id();

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
