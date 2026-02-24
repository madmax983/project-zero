use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::style::Color;

use crate::layer1::map::{GridPosition, ScreenShake};
use crate::layer1::orbital_crossfire::{mine_scrap, ImpactSite};
use crate::layer1::particles::{spawn_moving_particle, spawn_particle};
use crate::layer1::resources::{process_logging, process_mining};
use crate::shared::log::MessageLog;

/// Handles mining work at a designation.
///
/// Reduces terrain health or mining progress, spawns resources, and removes rock/ore.
pub fn handle_mining_work(
    world: &mut World,
    entity: Entity,
    work_amount: f32,
    pos: Option<GridPosition>,
) -> bool {
    let mut rng = rand::thread_rng();
    let is_crit = rng.gen_bool(0.05);

    let mut effective_work = work_amount;
    if is_crit {
        effective_work *= 5.0;
        if let Some(p) = pos {
            spawn_particle(world, p, '*', Color::Yellow, 10);
            trigger_shake(world, 0.3);
        }
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored("Critical Mine!", Color::Yellow);
        }
    }

    if let Some(p) = pos {
        crate::layer1::geology::add_seismic_stress(world, p, 1.0);
    }

    let is_scrap = if let Some(p) = pos {
        let mut found = false;
        let mut query = world.query::<(&GridPosition, &ImpactSite)>();
        for (gp, _) in query.iter(world) {
            if *gp == p {
                found = true;
                break;
            }
        }
        found
    } else {
        false
    };

    if is_scrap {
        mine_scrap(world, entity, effective_work);
    } else {
        process_mining(world, entity, effective_work);
    }

    if let Some(p) = pos {
        if world.get_entity(entity).is_err() {
            // Finished: Big shake + Debris
            trigger_shake(world, 0.5);
            spawn_particle(world, p, '*', Color::White, 10);

            // Ludwig: Explosive Debris
            let mut rng = rand::thread_rng();
            for _ in 0..5 {
                let dx = rng.gen_range(-1.0..1.0);
                let dy = rng.gen_range(-1.0..1.0);
                spawn_moving_particle(world, p, '.', Color::DarkGray, 15, dx, dy);
            }
        } else {
            // Working: Dynamic shake + Dust
            if !is_crit {
                let intensity = world
                    .get::<crate::layer1::resources::MiningProgress>(entity)
                    .map_or(0.05, |prog| (prog.current / prog.max).mul_add(0.15, 0.05));

                trigger_shake(world, intensity);
                spawn_particle(world, p, '.', Color::DarkGray, 3);

                // Ludwig: Occasional flying chip
                if rng.gen_bool(0.3) {
                    let dx = rng.gen_range(-0.5..0.5);
                    let dy = rng.gen_range(-0.5..0.5);
                    spawn_moving_particle(world, p, '.', Color::Gray, 10, dx, dy);
                }
            }
        }
    }
    true
}

/// Handles wood chopping work at a designation.
///
/// Reduces tree HP, spawns wood, and removes the tree entity.
pub fn handle_chopping_work(
    world: &mut World,
    entity: Entity,
    work_amount: f32,
    pos: Option<GridPosition>,
) -> bool {
    let mut rng = rand::thread_rng();
    let is_crit = rng.gen_bool(0.05);

    let mut effective_work = work_amount;
    if is_crit {
        effective_work *= 5.0;
        if let Some(p) = pos {
            spawn_particle(world, p, '^', Color::LightGreen, 10);
            trigger_shake(world, 0.3);
        }
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored("Critical Chop!", Color::LightGreen);
        }
    }

    process_logging(world, entity, effective_work);

    if let Some(p) = pos {
        if world.get_entity(entity).is_err() {
            // Finished
            trigger_shake(world, 0.3);
            spawn_particle(world, p, '^', Color::Green, 10);

            // Ludwig: Wood chips flying
            let mut rng = rand::thread_rng();
            for _ in 0..4 {
                let dx = rng.gen_range(-0.8..0.8);
                let dy = rng.gen_range(-0.8..0.8);
                spawn_moving_particle(world, p, '\'', Color::Rgb(139, 69, 19), 15, dx, dy);
            }
        } else {
            // Working
            if !is_crit {
                let intensity = world
                    .get::<crate::layer1::resources::ForestryProgress>(entity)
                    .map_or(0.02, |prog| (prog.current / prog.max).mul_add(0.1, 0.02));

                trigger_shake(world, intensity);
                spawn_particle(world, p, '\'', Color::Rgb(139, 69, 19), 3);

                // Ludwig: Occasional flying chip
                if rng.gen_bool(0.3) {
                    let dx = rng.gen_range(-0.5..0.5);
                    let dy = rng.gen_range(-0.5..0.5);
                    spawn_moving_particle(world, p, '\'', Color::Rgb(160, 82, 45), 10, dx, dy);
                }
            }
        }
    }
    true
}

fn trigger_shake(world: &mut World, intensity: f32) {
    if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
        shake.trigger(intensity);
    }
}
