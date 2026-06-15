with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

# Fix inspector memory docstring
old_doc_part2 = """/// INT-1121: Bridges `PostGrievanceEvent` to `Grudge` components to link negative social interactions
/// to the formation of long-lasting generational grudges.
fn upsert_grudge(
    grudge_list: &mut crate::layer1::social::inherited_grudges::GrudgeList,
    target: bevy_ecs::entity::Entity,
    impact: f32,
) {
    let mut found = false;
    for grudge in &mut grudge_list.0 {
        if grudge.target_entity == target {
            grudge.intensity += impact.abs();
            found = true;
            break;
        }
    }

    if !found {
        grudge_list
            .0
            .push(crate::layer1::social::inherited_grudges::Grudge {
                target_entity: target,
                intensity: impact.abs(),
                origin_reason: "Public grievance".to_string(),
            });
    }
}

pub fn public_grievance_grudge_bridge("""

new_doc_part2 = """fn upsert_grudge(
    grudge_list: &mut crate::layer1::social::inherited_grudges::GrudgeList,
    target: bevy_ecs::entity::Entity,
    impact: f32,
) {
    let mut found = false;
    for grudge in &mut grudge_list.0 {
        if grudge.target_entity == target {
            grudge.intensity += impact.abs();
            found = true;
            break;
        }
    }

    if !found {
        grudge_list
            .0
            .push(crate::layer1::social::inherited_grudges::Grudge {
                target_entity: target,
                intensity: impact.abs(),
                origin_reason: "Public grievance".to_string(),
            });
    }
}

/// INT-1121: Bridges `PostGrievanceEvent` to `Grudge` components to link negative social interactions
/// to the formation of long-lasting generational grudges.
pub fn public_grievance_grudge_bridge("""

content = content.replace(old_doc_part2, new_doc_part2)

with open("src/layer1/core/integration.rs", "w") as f:
    f.write(content)
print("done")
