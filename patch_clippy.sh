#!/bin/bash
cat << 'INNER_EOF' > /tmp/clippy_patch.txt
<<<<<<< SEARCH
fn process_arrival(
    action: ActionType,
    pop_entity: Entity,
    target_entity: Entity,
    pop_pos: GridPosition,
    target_pos: GridPosition,
    equipment_opt: &mut Option<Mut<Equipment>>,
    chemical_state_opt: &mut Option<Mut<crate::layer1::chemical::ChemicalState>>,
    health_opt: &mut Option<Mut<crate::layer1::health::Health>>,
    stress_opt: &mut Option<Mut<crate::layer1::stress::StressTracker>>,
    commands: &mut Commands,
    ctx: &mut ArrivalContext,
) -> bool {
=======
#[allow(clippy::too_many_arguments)]
fn process_arrival(
    action: ActionType,
    pop_entity: Entity,
    target_entity: Entity,
    pop_pos: GridPosition,
    target_pos: GridPosition,
    equipment_opt: &mut Option<Mut<Equipment>>,
    chemical_state_opt: &mut Option<Mut<crate::layer1::chemical::ChemicalState>>,
    health_opt: &mut Option<Mut<crate::layer1::health::Health>>,
    stress_opt: &mut Option<Mut<crate::layer1::stress::StressTracker>>,
    commands: &mut Commands,
    ctx: &mut ArrivalContext,
) -> bool {
>>>>>>> REPLACE
INNER_EOF
python3 -c '
import sys
with open("src/layer1/execution/arrival.rs", "r") as f: content = f.read()
with open("/tmp/clippy_patch.txt", "r") as f: patch = f.read()

parts = patch.split("<<<<<<< SEARCH\n")
for part in parts[1:]:
    search, rest = part.split("=======\n", 1)
    replace, _ = rest.split(">>>>>>> REPLACE\n", 1)
    content = content.replace(search, replace)

with open("src/layer1/execution/arrival.rs", "w") as f: f.write(content)
'
