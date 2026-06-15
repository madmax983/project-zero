with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

# Fix inspector memory docstring
old_doc_part1 = """/// Creates chronicle entries from [`crate::layer1::social::grievances::PostGrievanceEvent`] events.
///
/// Bridges the Inspector system (Observation) and Pop/Resources system (Psychology/Economy).
#[allow(clippy::cast_precision_loss)]
fn grant_inspector_memory"""

old_doc_part2 = """/// INT-1121: Bridges `PostGrievanceEvent` to `Grudge` components to link negative social interactions
/// to the formation of long-lasting generational grudges.
fn upsert_grudge"""

new_doc_part1 = """fn grant_inspector_memory"""

new_doc_part2 = """fn upsert_grudge"""

# Oh wait, the inspector docstring was originally:
# /// Creates chronicle entries from [`crate::layer1::observation::Inspector`] reports.
# wait, let me just find it and replace the order
