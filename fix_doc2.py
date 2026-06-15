with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

# Fix inspector memory docstring
old_doc_part1 = """/// Applies consequences of an Inspector's report.
///
/// Bridges the Inspector system (Observation) and Pop/Resources system (Psychology/Economy).
#[allow(clippy::cast_precision_loss)]
fn grant_inspector_memory(
    pop_memories: &mut Query<&mut Memories, With<Pop>>,
    memory_type: MemoryType,
    tick: u64,
) {
    pop_memories.par_iter_mut().for_each(|mut memories| {
        if !memories.items.iter().any(|m| m.memory_type == memory_type) {
            memories.add(memory_type, tick);
        }
    });
}

pub fn inspector_outcome_bridge_system("""

new_doc_part1 = """fn grant_inspector_memory(
    pop_memories: &mut Query<&mut Memories, With<Pop>>,
    memory_type: MemoryType,
    tick: u64,
) {
    pop_memories.par_iter_mut().for_each(|mut memories| {
        if !memories.items.iter().any(|m| m.memory_type == memory_type) {
            memories.add(memory_type, tick);
        }
    });
}

/// Applies consequences of an Inspector's report.
///
/// Bridges the Inspector system (Observation) and Pop/Resources system (Psychology/Economy).
#[allow(clippy::cast_precision_loss)]
pub fn inspector_outcome_bridge_system("""

content = content.replace(old_doc_part1, new_doc_part1)

with open("src/layer1/core/integration.rs", "w") as f:
    f.write(content)
print("done")
