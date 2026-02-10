// Utility AI evaluate shader — GPU-side replica of the CPU utility scoring logic.
//
// Each invocation processes one pop, iterating all buildings to find the
// highest-utility action. The result (action, utility, target, switched)
// is written to the decisions buffer for the CPU to read back.

// ---------- Struct definitions (must match CPU-side bytemuck layout) ----------

struct PopInput {
    pos_x: i32,
    pos_y: i32,
    hunger: f32,
    rest: f32,
    leisure: f32,
    distance_weight: f32,
    availability_weight: f32,
    social_weight: f32,
    success_count: array<u32, 19>,
    attempt_count: array<u32, 19>,
    current_utility: f32,
    _pad: array<u32, 1>,
}

struct BuildingInput {
    pos_x: i32,
    pos_y: i32,
    building_type: u32,
    capacity: u32,
    occupied: u32,
    resource_has_room: u32,
    _pad: array<u32, 2>,
}

struct GlobalState {
    switch_threshold: f32,
    knowledge_full: u32,
    has_stockpile: u32,
    pop_count: u32,
    building_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

struct PopDecision {
    best_action: u32,
    best_utility: f32,
    target_index: u32,
    switched: u32,
}

// ---------- Bind groups ----------

@group(0) @binding(0) var<storage, read> pops: array<PopInput>;
@group(0) @binding(1) var<storage, read> buildings: array<BuildingInput>;
@group(0) @binding(2) var<uniform> globals: GlobalState;
@group(0) @binding(3) var<storage, read_write> decisions: array<PopDecision>;

// ---------- Helper functions ----------

/// Quadratic response curve: lower need value -> higher urgency.
/// Mirrors `need_response_curve` in math.rs: `1.0 - v * v`.
fn need_response_curve(v: f32) -> f32 {
    return 1.0 - v * v;
}

/// Context score combining distance and availability factors.
/// Mirrors `calculate_context_score` in math.rs.
fn calculate_context_score(
    pop_x: i32, pop_y: i32,
    bldg_x: i32, bldg_y: i32,
    capacity: u32, occupied: u32,
    dist_weight: f32, avail_weight: f32
) -> f32 {
    var score: f32 = 1.0;

    // Distance factor (Manhattan distance, closer = better)
    let dx = abs(pop_x - bldg_x);
    let dy = abs(pop_y - bldg_y);
    let distance = f32(dx + dy);
    let distance_factor = 1.0 / (distance * 0.1 + 1.0);
    score *= pow(distance_factor, dist_weight);

    // Availability factor (less crowded = better)
    if capacity > 0u {
        let availability = 1.0 - f32(occupied) / f32(capacity);
        if availability > 0.0 {
            score *= pow(availability, avail_weight);
        } else {
            score = 0.0;
        }
    }

    return clamp(score, 0.0, 1.0);
}

/// Success-rate modifier in the 0.8-1.2 range.
/// Mirrors `calculate_success_modifier` in math.rs.
fn calculate_success_modifier(action_idx: u32, pop: PopInput) -> f32 {
    let attempts = pop.attempt_count[action_idx];
    let successes = pop.success_count[action_idx];

    if attempts == 0u {
        return 1.0;
    }

    let success_rate = f32(successes) / f32(attempts);
    return success_rate * 0.4 + 0.8;
}

// ---------- Main entry point ----------

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pop_idx = global_id.x;
    if pop_idx >= globals.pop_count {
        return;
    }

    let pop = pops[pop_idx];

    var best_action: u32 = 11u;         // Default: Idle
    var best_utility: f32 = 0.05;       // Idle baseline utility
    var best_target: u32 = 0xFFFFFFFFu; // No target (u32::MAX sentinel)

    // Evaluate every building to find the highest-utility action
    for (var i: u32 = 0u; i < globals.building_count; i = i + 1u) {
        let bldg = buildings[i];

        var urgency: f32 = 0.0;
        var action_idx: u32 = 11u;
        var skip: bool = false;

        switch bldg.building_type {
            case 0u: {
                // Farm -> SatisfyHunger (action 0)
                urgency = need_response_curve(pop.hunger);
                action_idx = 0u;
            }
            case 1u: {
                // Housing -> SatisfyRest (action 1)
                urgency = need_response_curve(pop.rest);
                action_idx = 1u;
            }
            case 2u: {
                // Tavern -> Socialize (action 2)
                urgency = need_response_curve(pop.leisure);
                action_idx = 2u;
            }
            case 3u: {
                // Library -> Research (action 6)
                if globals.knowledge_full == 1u {
                    skip = true;
                }
                urgency = 0.4;
                action_idx = 6u;
            }
            case 4u: {
                // Work Designation -> Work (action 4)
                urgency = 0.5;
                action_idx = 4u;
            }
            case 5u: {
                // ResourceItem -> Haul (action 7)
                if globals.has_stockpile == 0u || bldg.resource_has_room == 0u {
                    skip = true;
                }
                urgency = 0.6;
                action_idx = 7u;
            }
            case 6u: {
                // Repair Designation -> Repair (action 5)
                urgency = 0.6;
                action_idx = 5u;
            }
            case 7u: {
                // Hospital -> SeekMedicalCare (action 8)
                // For GPU we approximate: urgency = 1.0 (TODO: pass health to GPU)
                // Since we don't have health in GpuPopInput yet, we can't fully evaluate.
                // But for now let's just make it possible.
                urgency = 1.0;
                action_idx = 8u;
            }
            case 9u: {
                // Corpse -> BuryCorpse (action 9)
                if bldg.resource_has_room == 0u {
                    skip = true;
                }
                urgency = 0.8;
                action_idx = 9u;
            }
            case 10u: {
                // Wanted -> Warden (action 18)
                urgency = 0.8; // High priority for maintaining order
                action_idx = 18u;
            }
            default: {
                skip = true;
            }
        }

        if !skip {
            let context = calculate_context_score(
                pop.pos_x, pop.pos_y,
                bldg.pos_x, bldg.pos_y,
                bldg.capacity, bldg.occupied,
                pop.distance_weight, pop.availability_weight
            );

            let success_mod = calculate_success_modifier(action_idx, pop);
            let utility = urgency * context * success_mod;

            if utility > best_utility {
                best_utility = utility;
                best_action = action_idx;
                best_target = i;
            }
        }
    }

    // Only switch if new utility exceeds current by the threshold
    var switched: u32 = 0u;
    if best_utility > pop.current_utility + globals.switch_threshold {
        switched = 1u;
    }

    decisions[pop_idx] = PopDecision(best_action, best_utility, best_target, switched);
}
