<<<<<<< SEARCH
pub fn public_grievance_grudge_bridge(
    mut events: bevy_ecs::event::EventReader<crate::layer1::social::grievances::PostGrievanceEvent>,
    mut query: bevy_ecs::system::Query<&mut crate::layer1::social::inherited_grudges::GrudgeList>,
    mut commands: bevy_ecs::system::Commands,
) {
    for event in events.read() {
        if event.impact < 0.0 { // It's a grievance
            if let Ok(mut grudges) = query.get_mut(event.poster) {
                // Check if already exists
                let mut found = false;
                for grudge in &mut grudges.0 {
                    if grudge.target_entity == event.target {
                        grudge.intensity += event.impact.abs();
                        found = true;
                        break;
                    }
                }

                if !found {
                    grudges.0.push(crate::layer1::social::inherited_grudges::Grudge {
                        target_entity: event.target,
                        intensity: event.impact.abs(),
                        origin_reason: "Public grievance".to_string(),
                    });
                }
            } else {
                commands.entity(event.poster).insert(crate::layer1::social::inherited_grudges::GrudgeList(vec![
                    crate::layer1::social::inherited_grudges::Grudge {
                        target_entity: event.target,
                        intensity: event.impact.abs(),
                        origin_reason: "Public grievance".to_string(),
                    }
                ]));
            }
        }
    }
}
=======
pub fn public_grievance_grudge_bridge(
    mut events: bevy_ecs::event::EventReader<crate::layer1::social::grievances::PostGrievanceEvent>,
    mut query: bevy_ecs::system::Query<&mut crate::layer1::social::inherited_grudges::GrudgeList>,
    mut commands: bevy_ecs::system::Commands,
) {
    use bevy_ecs::entity::Entity;
    use std::collections::HashMap;

    // To prevent multiple inserts on the same entity overwriting each other in the same frame
    let mut pending_inserts: HashMap<Entity, crate::layer1::social::inherited_grudges::GrudgeList> = HashMap::new();

    for event in events.read() {
        if event.impact < 0.0 { // It's a grievance
            if let Ok(mut grudges) = query.get_mut(event.poster) {
                // Check if already exists
                let mut found = false;
                for grudge in &mut grudges.0 {
                    if grudge.target_entity == event.target {
                        grudge.intensity += event.impact.abs();
                        found = true;
                        break;
                    }
                }

                if !found {
                    grudges.0.push(crate::layer1::social::inherited_grudges::Grudge {
                        target_entity: event.target,
                        intensity: event.impact.abs(),
                        origin_reason: "Public grievance".to_string(),
                    });
                }
            } else {
                let grudge_list = pending_inserts.entry(event.poster).or_insert_with(|| crate::layer1::social::inherited_grudges::GrudgeList(Vec::new()));

                let mut found = false;
                for grudge in &mut grudge_list.0 {
                    if grudge.target_entity == event.target {
                        grudge.intensity += event.impact.abs();
                        found = true;
                        break;
                    }
                }

                if !found {
                    grudge_list.0.push(crate::layer1::social::inherited_grudges::Grudge {
                        target_entity: event.target,
                        intensity: event.impact.abs(),
                        origin_reason: "Public grievance".to_string(),
                    });
                }
            }
        }
    }

    for (entity, grudge_list) in pending_inserts {
        commands.entity(entity).insert(grudge_list);
    }
}
>>>>>>> REPLACE
