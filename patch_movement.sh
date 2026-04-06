#!/bin/bash
cat << 'INNER_EOF' > /tmp/movement_patch.txt
<<<<<<< SEARCH
/// Moves pops 1 tile per tick toward their target (Manhattan-style).
///
/// When a pop arrives at its target position (or adjacent for work), this system
/// marks it with `AtTarget`.
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
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
            Option<&MentalFog>,
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
    for (pop_entity, mut current_pos, mt, mut speed_opt, traits, hit_stop, role, fog) in &mut pops {
        // Ludwig: Check Hit Stop
        if let Some(hs) = hit_stop {
            if hs.ticks_remaining > 0 {
                continue;
            }
        }

        let trait_mod = traits.map_or(1.0, get_trait_move_speed_modifier);
        let fog_mod = fog.map_or(1.0, |f| f.movement_penalty);

        // Accumulate speed
        if let Some(ref mut speed) = speed_opt {
            speed.accumulator += speed.current * trait_mod * fog_mod;
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

        let movement_cost = calculate_movement_cost(
            new_pos,
            *current_pos,
            &terrain,
            wind_grid.as_deref(),
            tide.as_deref(),
        );

        if !try_apply_movement_speed(&mut speed_opt, movement_cost, &mut commands, *current_pos) {
            continue;
        }

        current_pos.x = new_pos.x;

        current_pos.y = new_pos.y;

        apply_erosion(&mut erosion, new_pos);

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
=======
use bevy_ecs::system::SystemParam;

#[derive(SystemParam)]
pub struct MovementContext<'w, 's> {
    erosion: ResMut<'w, ErosionGrid>,
    terrain: Res<'w, TerrainGrid>,
    occupied_tiles: Option<Res<'w, OccupiedTiles>>,
    wind_grid: Option<Res<'w, crate::layer1::wind::WindGrid>>,
    tide: Option<Res<'w, crate::layer1::atmosphere::AtmosphericTide>>,
    buildings: Query<
        'w,
        's,
        (
            &'static GridPosition,
            &'static Building,
            Option<&'static Gate>,
            Option<&'static AccessControl>,
        ),
    >,
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
            Option<&MentalFog>,
        ),
        (Without<AtTarget>, Without<Building>),
    >,
    mut ctx: MovementContext,
    mut commands: Commands,
) {
    for (pop_entity, mut current_pos, mt, mut speed_opt, traits, hit_stop, role, fog) in &mut pops {
        process_single_movement(
            pop_entity,
            &mut current_pos,
            mt,
            &mut speed_opt,
            traits,
            hit_stop,
            role,
            fog,
            &mut ctx,
            &mut commands,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn process_single_movement(
    pop_entity: Entity,
    current_pos: &mut GridPosition,
    mt: &MovementTarget,
    speed_opt: &mut Option<Mut<Speed>>,
    traits: Option<&Traits>,
    hit_stop: Option<&HitStop>,
    role: Option<&Role>,
    fog: Option<&MentalFog>,
    ctx: &mut MovementContext,
    commands: &mut Commands,
) {
    // Ludwig: Check Hit Stop
    if let Some(hs) = hit_stop {
        if hs.ticks_remaining > 0 {
            return;
        }
    }

    let trait_mod = traits.map_or(1.0, get_trait_move_speed_modifier);
    let fog_mod = fog.map_or(1.0, |f| f.movement_penalty);

    // Accumulate speed
    if let Some(ref mut speed) = speed_opt {
        speed.accumulator += speed.current * trait_mod * fog_mod;
    }

    let target_pos = mt.target_position;
    let action = mt.for_action;

    // Check pre-move adjacency for work
    if check_work_adjacency(
        *current_pos,
        target_pos,
        action,
        &ctx.terrain,
        ctx.occupied_tiles.as_deref(),
        &ctx.buildings,
        pop_entity,
        role.copied(),
    ) {
        commands.entity(pop_entity).insert(AtTarget);
        return;
    }

    // Check if already at target (for non-work actions requiring exact position)
    if *current_pos == target_pos {
        commands.entity(pop_entity).insert(AtTarget);
        return;
    }

    let (primary, secondary) = calculate_next_positions(*current_pos, target_pos);

    // Selection Phase: Find first walkable candidate
    let chosen_pos = try_get_walkable_pos(
        primary,
        &ctx.terrain,
        ctx.occupied_tiles.as_deref(),
        &ctx.buildings,
        pop_entity,
        role.copied(),
    )
    .or_else(|| {
        try_get_walkable_pos(
            secondary,
            &ctx.terrain,
            ctx.occupied_tiles.as_deref(),
            &ctx.buildings,
            pop_entity,
            role.copied(),
        )
    });

    let Some(new_pos) = chosen_pos else {
        return;
    };

    let movement_cost = calculate_movement_cost(
        new_pos,
        *current_pos,
        &ctx.terrain,
        ctx.wind_grid.as_deref(),
        ctx.tide.as_deref(),
    );

    if !try_apply_movement_speed(speed_opt, movement_cost, commands, *current_pos) {
        return;
    }

    current_pos.x = new_pos.x;
    current_pos.y = new_pos.y;

    apply_erosion(&mut ctx.erosion, new_pos);

    if new_pos == target_pos {
        commands.entity(pop_entity).insert(AtTarget);
    }

    // Check post-move adjacency for work
    if check_work_adjacency(
        new_pos,
        target_pos,
        action,
        &ctx.terrain,
        ctx.occupied_tiles.as_deref(),
        &ctx.buildings,
        pop_entity,
        role.copied(),
    ) {
        commands.entity(pop_entity).insert(AtTarget);
    }
}
>>>>>>> REPLACE
INNER_EOF
python3 -c '
import sys
with open("src/layer1/execution/movement.rs", "r") as f: content = f.read()
with open("/tmp/movement_patch.txt", "r") as f: patch = f.read()

parts = patch.split("<<<<<<< SEARCH\n")
for part in parts[1:]:
    search, rest = part.split("=======\n", 1)
    replace, _ = rest.split(">>>>>>> REPLACE\n", 1)
    content = content.replace(search, replace)

with open("src/layer1/execution/movement.rs", "w") as f: f.write(content)
'
