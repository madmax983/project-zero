import sys

with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

# Fix the Query component reference. It should be Query<&GridPosition> not Query<GridPosition>.
# But we already did Query<&crate::layer1::map::GridPosition>.
# Wait, let's look at the errors:
# error[E0229]: associated type bindings are not allowed here
#   --> src/layer1/core/integration.rs:2881:95
# 2881 |     pops_query: bevy_ecs::system::Query<&crate::layer1::map::GridPosition, bevy_ecs::query::With<crate::layer1::pop::Pop>>,
#      |                                                                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ associated type not allowed in this position

# Ah! It's because Query takes two generic arguments: `Query<D, F = ()>`. It doesn't take associated type bindings like `With=...`.
# So it should be `Query<&crate::layer1::map::GridPosition, bevy_ecs::query::With<crate::layer1::pop::Pop>>`. Wait, it is. But `bevy_ecs::query::With` doesn't take associated type binding. It is just `With<T>`. Wait, `bevy_ecs::query::With<crate::layer1::pop::Pop>` is correct.
# Wait, let me check what we actually wrote: `bevy_ecs::query::With<crate::layer1::pop::Pop>`
# In Rust, `bevy_ecs::query::With` is a struct, but I might have written `bevy_ecs::query::With<T>`. Wait! The error says `With<crate::layer1::pop::Pop>` associated type not allowed... Oh, I wrote `With<crate::layer1::pop::Pop>` but it is an item, maybe it thought I meant `With = crate::layer1::pop::Pop`? No, wait.

# Let's fix the parameter list completely.
new_params = """
pub fn track_negative_events_bridge_system(
    mut pop_died_events: bevy_ecs::event::EventReader<crate::layer1::pop::PopDied>,
    mut building_removed_events: bevy_ecs::event::EventReader<crate::layer1::core::events::BuildingRemovedEvent>,
    mut pop_died_accident_events: bevy_ecs::event::EventReader<crate::layer1::haunted_assembly_lines::PopDiedInAccidentEvent>,
    pops_query: bevy_ecs::system::Query<&crate::layer1::core::map::GridPosition>,
    pos_query: bevy_ecs::system::Query<&crate::layer1::core::map::GridPosition>,
    mut building_query: bevy_ecs::system::Query<(
        &crate::layer1::core::map::GridPosition,
        &mut crate::layer1::architecture_superstition::NegativeEventHistory,
    ), bevy_ecs::query::With<crate::layer1::architecture::Building>>,
    time: bevy_ecs::system::Res<crate::shared::time::SimulationTime>,
) {
"""

# Let's read the current integration.rs and replace the function declaration.
import re

content = re.sub(
    r"pub fn track_negative_events_bridge_system\(.*?\) \{",
    new_params.strip() + " {",
    content,
    flags=re.DOTALL
)

with open("src/layer1/core/integration.rs", "w") as f:
    f.write(content)
