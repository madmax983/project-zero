use crate::layer1::access_control::{AccessControl, AccessMode};
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::admin::{AdminProvider, Office};
use crate::layer1::building::{Building, OccupiedTiles};
use crate::layer1::combat::HitStop;
use crate::layer1::defense::Gate;
use crate::layer1::erosion::{ErosionGrid, MOVEMENT_EROSION_AMOUNT};
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::farm::Farm;
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Role, Speed};
use crate::layer1::social::Tavern;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::traits::{Traits, get_trait_move_speed_modifier};
use crate::layer1::utility_types::{ActionType, StartPlan};
use bevy_ecs::prelude::*;

/// Removes pops from farms/housing when they switch to a different action.
///
/// This system runs before `process_start_plan_system` to ensure pops are
/// properly removed from their previous assignment before moving to a new one.
pub fn cleanup_previous_assignment_system(
    pops_query: Query<(Entity, &AssignedTo), With<StartPlan>>,
    mut farms: Query<&mut Farm>,
    mut housing: Query<&mut Housing>,
    mut taverns: Query<&mut Tavern>,
    mut offices: Query<&mut Office>,
    mut commands: Commands,
) {
    for (pop_entity, assigned) in &pops_query {
        let assigned_entity = assigned.entity;
        match assigned.assignment_type {
            AssignmentType::FarmWorker => {
                if let Ok(mut farm) = farms.get_mut(assigned_entity) {
                    farm.workers.retain(|&w| w != pop_entity);
                }
            }
            AssignmentType::HousingResident => {
                if let Ok(mut h) = housing.get_mut(assigned_entity) {
                    h.residents.retain(|&r| r != pop_entity);
                }
            }
            AssignmentType::TavernVisitor => {
                if let Ok(mut tavern) = taverns.get_mut(assigned_entity) {
                    tavern.visitors.retain(|&v| v != pop_entity);
                }
            }
            AssignmentType::Administrator => {
                // Remove AdminProvider
                commands.entity(pop_entity).remove::<AdminProvider>();
                if let Ok(mut office) = offices.get_mut(assigned_entity) {
                    office.workers.retain(|&w| w != pop_entity);
                }
            }
            AssignmentType::LibraryWorker
            | AssignmentType::Patient
            | AssignmentType::Funeral
            | AssignmentType::ObservatoryWorker
            | AssignmentType::Surgery => {}
        }

        commands.entity(pop_entity).remove::<AssignedTo>();
    }
}

/// Consumes `StartPlan` markers and creates `MovementTarget` components.
///
/// This system bridges the utility AI's decision (`StartPlan`) with the
/// movement system by creating `MovementTarget` for each pop.
pub fn process_start_plan_system(
    plans: Query<(Entity, &StartPlan)>,
    positions: Query<&GridPosition>,
    mut commands: Commands,
) {
    for (pop_entity, start_plan) in &plans {
        let action = start_plan.action;
        let target = start_plan.target;

        // Remove the StartPlan marker and any existing movement components
        commands
            .entity(pop_entity)
            .remove::<StartPlan>()
            .remove::<MovementTarget>()
            .remove::<AtTarget>();

        // If there's no target, skip (e.g., Idle action)
        let Some(target_entity) = target else {
            continue;
        };

        // Get the target's position (also validates entity existence)
        let Ok(&target_position) = positions.get(target_entity) else {
            continue;
        };

        // Insert new MovementTarget
        commands.entity(pop_entity).insert(MovementTarget {
            target_entity,
            target_position,
            for_action: action,
        });
    }
}

/// Moves pops 1 tile per tick toward their target (Manhattan-style).
///
/// When a pop arrives at its target position (or adjacent for work), this system
/// marks it with `AtTarget`.
#[allow(clippy::type_complexity)]
pub fn movement_system(
    mut pops: Query<
        (
            Entity,
            &mut GridPosition,
            &MovementTarget,
            Option<&mut Speed>,
            Option<&Traits>,
            Option<&HitStop>,
            Option<&Role>,
        ),
        (Without<AtTarget>, Without<Building>),
    >,
    mut erosion: ResMut<ErosionGrid>,
    terrain: Res<TerrainGrid>,
    occupied_tiles: Option<Res<OccupiedTiles>>,
    wind_grid: Option<Res<crate::layer1::wind::WindGrid>>,
    tide: Option<Res<crate::layer1::atmosphere::AtmosphericTide>>,
    buildings: Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
    mut commands: Commands,
) {
    for (pop_entity, mut current_pos, mt, mut speed_opt, traits, hit_stop, role) in &mut pops {
        // Ludwig: Check Hit Stop
        if let Some(hs) = hit_stop {
            if hs.ticks_remaining > 0 {
                continue;
            }
        }

        let trait_mod = traits.map_or(1.0, get_trait_move_speed_modifier);

        // Accumulate speed
        if let Some(ref mut speed) = speed_opt {
            speed.accumulator += speed.current * trait_mod;
        }

        let target_pos = mt.target_position;
        let action = mt.for_action;

        // Check pre-move adjacency for work
        if check_work_adjacency(
            *current_pos,
            target_pos,
            action,
            &terrain,
            occupied_tiles.as_deref(),
            &buildings,
            pop_entity,
            role.copied(),
        ) {
            commands.entity(pop_entity).insert(AtTarget);
            continue;
        }

        // Check if already at target (for non-work actions requiring exact position)
        if *current_pos == target_pos {
            commands.entity(pop_entity).insert(AtTarget);
            continue;
        }

        let (primary, secondary) = calculate_next_positions(*current_pos, target_pos);

        // Selection Phase: Find first walkable candidate
        let chosen_pos = try_get_walkable_pos(
            primary,
            &terrain,
            occupied_tiles.as_deref(),
            &buildings,
            pop_entity,
            role.copied(),
        )
        .or_else(|| {
            try_get_walkable_pos(
                secondary,
                &terrain,
                occupied_tiles.as_deref(),
                &buildings,
                pop_entity,
                role.copied(),
            )
        });

        let Some(new_pos) = chosen_pos else {
            continue;
        };

        // Determine base movement cost
        let base_cost =
            if let (Ok(x), Ok(y)) = (usize::try_from(new_pos.x), usize::try_from(new_pos.y)) {
                terrain
                    .get(x, y)
                    .map_or(1.0, crate::layer1::terrain::TerrainType::movement_cost)
            } else {
                1.0
            };

        // Wind penalty
        let wind_mod = if let Some(ref w) = wind_grid {
            let wind_vec = w.get_wind(current_pos.x, current_pos.y);
            let move_dir = crate::layer1::wind::Vec2::new(
                (new_pos.x - current_pos.x) as f32,
                (new_pos.y - current_pos.y) as f32,
            );
            crate::layer1::wind::calculate_wind_movement_penalty(wind_vec, move_dir)
        } else {
            1.0
        };

        // Pressure penalty
        let pressure_mod = tide.as_ref().map_or(1.0, |t| {
            crate::layer1::atmosphere::calculate_atmospheric_movement_cost(t.pressure)
        });

        let movement_cost = base_cost * wind_mod * pressure_mod;

        // Check if we can move
        let can_move = if let Some(ref mut speed) = speed_opt {
            // Ludwig: "Coyote Speed" - Allow moving if we are *almost* there.
            // This prevents the feeling of "just missing the bus" by 0.01 speed.
            // Increased to 0.20 for even better flow.
            const COYOTE_THRESHOLD: f32 = 0.20;
            if speed.accumulator >= (movement_cost - COYOTE_THRESHOLD) {
                speed.accumulator -= movement_cost;
                true
            } else {
                false
            }
        } else {
            true
        };

        if !can_move {
            continue;
        }

        current_pos.x = new_pos.x;
        current_pos.y = new_pos.y;

        // Apply Erosion
        if let (Ok(x), Ok(y)) = (usize::try_from(new_pos.x), usize::try_from(new_pos.y)) {
            erosion.add_erosion(x, y, MOVEMENT_EROSION_AMOUNT);
        }

        if new_pos == target_pos {
            commands.entity(pop_entity).insert(AtTarget);
        }

        // Check post-move adjacency for work
        if check_work_adjacency(
            new_pos,
            target_pos,
            action,
            &terrain,
            occupied_tiles.as_deref(),
            &buildings,
            pop_entity,
            role.copied(),
        ) {
            commands.entity(pop_entity).insert(AtTarget);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn check_work_adjacency(
    current_pos: GridPosition,
    target_pos: GridPosition,
    action: ActionType,
    terrain: &TerrainGrid,
    occupied: Option<&OccupiedTiles>,
    buildings: &Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
    pop_entity: Entity,
    pop_role: Option<Role>,
) -> bool {
    if action != ActionType::Work
        && action != ActionType::Repair
        && action != ActionType::ScrawlMemeticSigil
    {
        return false;
    }

    if is_tile_walkable(
        terrain,
        occupied,
        buildings,
        target_pos.x,
        target_pos.y,
        pop_entity,
        pop_role,
    ) {
        return false;
    }

    let distance = (current_pos.x - target_pos.x).abs() + (current_pos.y - target_pos.y).abs();
    distance == 1
}

pub(crate) fn try_get_walkable_pos(
    pos: Option<GridPosition>,
    terrain: &TerrainGrid,
    occupied_tiles: Option<&OccupiedTiles>,
    buildings: &Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
    pop_entity: Entity,
    pop_role: Option<Role>,
) -> Option<GridPosition> {
    let p = pos?;
    if is_tile_walkable(
        terrain,
        occupied_tiles,
        buildings,
        p.x,
        p.y,
        pop_entity,
        pop_role,
    ) {
        Some(p)
    } else {
        None
    }
}

pub(crate) fn is_tile_walkable(
    terrain: &TerrainGrid,
    occupied: Option<&OccupiedTiles>,
    buildings: &Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
    x: i32,
    y: i32,
    pop_entity: Entity,
    pop_role: Option<Role>,
) -> bool {
    // Check Terrain bounds and type
    let Ok(x_idx) = usize::try_from(x) else {
        return false;
    };
    let Ok(y_idx) = usize::try_from(y) else {
        return false;
    };

    if !terrain
        .get(x_idx, y_idx)
        .is_some_and(TerrainType::is_walkable)
    {
        return false;
    }

    // Check Buildings
    let Some(occupied_tiles) = occupied else {
        return true;
    };
    if !occupied_tiles.0.contains(&(x, y)) {
        return true;
    }

    for (pos, building, _gate, access_opt) in buildings.iter() {
        if pos.x == x && pos.y == y {
            // Priority: AccessControl
            if let Some(access) = access_opt {
                return match access.mode {
                    AccessMode::Public => true,
                    AccessMode::Lockdown => false,
                    AccessMode::Restricted => {
                        access.allowed_pops.contains(&pop_entity)
                            || pop_role.is_some_and(|r| access.allowed_roles.contains(&r))
                    }
                };
            }

            // Fallback: Gate (Legacy) - deprecated but kept for safety if AccessControl missing
            // If we decide to fully remove logic, we can just skip this.
            // But spec said "Replace Gate.is_locked".
            // Since we are adding AccessControl to Gates, we should rely on AccessControl.
            // If AccessControl is missing, we check building.is_obstacle().
            // Gate is an obstacle.
            if building.building_type.is_obstacle() {
                return false;
            }
            return true;
        }
    }
    true
}

#[allow(clippy::missing_const_for_fn, clippy::unnecessary_wraps)]
pub(crate) fn calculate_next_positions(
    current: GridPosition,
    target: GridPosition,
) -> (Option<GridPosition>, Option<GridPosition>) {
    // Calculate movement direction (Manhattan)
    let dx = (target.x - current.x).signum();
    let dy = (target.y - current.y).signum();

    let move_x = if dx != 0 {
        Some(GridPosition {
            x: current.x + dx,
            y: current.y,
        })
    } else {
        None
    };

    let move_y = if dy != 0 {
        Some(GridPosition {
            x: current.x,
            y: current.y + dy,
        })
    } else {
        None
    };

    // Prefer horizontal movement
    if dx != 0 {
        (move_x, move_y)
    } else {
        (move_y, None)
    }
}
