use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::unrest::{MentalBreakType, MentalState};
use crate::layer1::utility_types::{ActionType, PopAction, StartPlan};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component tracking how many ticks a sleepwalking episode lasts.
#[derive(Component, Debug, Clone, Copy)]
pub struct SleepwalkTimer(pub u32);

/// Marker component for an entity serving as a target for sleepwalking movement.
#[derive(Component)]
pub struct SleepwalkTarget;

/// Configuration for sleepwalking mechanics.
#[derive(Resource, Debug, Clone)]
pub struct SleepwalkingConfig {
    /// Probability per tick of starting sleepwalking when conditions are met.
    pub chance: f64,
}

impl Default for SleepwalkingConfig {
    fn default() -> Self {
        Self { chance: 0.01 }
    }
}

/// System to check if a Pop should start sleepwalking.
///
/// Triggered when a Pop is attempting to satisfy rest (or resting) and has low morale.
pub fn check_sleepwalking_start_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Needs, &PopAction, &mut MentalState), Without<SleepwalkTimer>>,
    config: Option<Res<SleepwalkingConfig>>,
) {
    let mut rng = rand::thread_rng();
    let chance = config.map_or(0.01, |c| c.chance);

    for (entity, needs, action, mut state) in &mut query {
        // Trigger condition: Trying to rest AND low morale
        // Check if currently resting or about to rest (SatisfyRest)
        if action.current == ActionType::SatisfyRest
            && needs.morale() < 0.25
            && rng.gen_bool(chance)
        {
            *state = MentalState::Broken(MentalBreakType::Sleepwalking);
            commands.entity(entity).insert(SleepwalkTimer(100)); // Sleepwalk for 100 ticks
        }
    }
}

/// System to manage sleepwalking duration and recovery.
pub fn sleepwalk_end_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MentalState, &mut SleepwalkTimer)>,
) {
    for (entity, mut state, mut timer) in &mut query {
        if matches!(*state, MentalState::Broken(MentalBreakType::Sleepwalking)) {
            timer.0 = timer.0.saturating_sub(1);
            if timer.0 == 0 {
                *state = MentalState::Normal;
                commands.entity(entity).remove::<SleepwalkTimer>();
            }
        } else {
            // Safety cleanup if state changed externally
            commands.entity(entity).remove::<SleepwalkTimer>();
        }
    }
}

/// Spawns a target entity for sleepwalkers if target is None.
///
/// This runs after the AI decides to Sleepwalk but before the execution system processes the plan.
#[allow(clippy::cast_sign_loss, clippy::collapsible_if)]
pub fn assign_sleepwalk_target_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut StartPlan, &GridPosition)>,
    terrain: Res<TerrainGrid>,
) {
    let mut rng = rand::thread_rng();

    for (_entity, mut plan, pos) in &mut query {
        if plan.action == ActionType::Sleepwalking && plan.target.is_none() {
            // Pick a random valid position within radius 10
            let mut target_pos = *pos;
            for _ in 0..10 {
                // Try 10 times to find a valid spot
                let dx = rng.gen_range(-10..=10);
                let dy = rng.gen_range(-10..=10);
                let new_x = pos.x + dx;
                let new_y = pos.y + dy;

                // Ensure within map bounds
                if new_x >= 0
                    && (new_x as usize) < terrain.width
                    && new_y >= 0
                    && (new_y as usize) < terrain.height
                {
                    if let Some(tile) = terrain.get(new_x as usize, new_y as usize) {
                        if tile.is_walkable() {
                            target_pos = GridPosition { x: new_x, y: new_y };
                            break;
                        }
                    }
                }
            }

            // Spawn a target marker entity
            let target_entity = commands.spawn((target_pos, SleepwalkTarget)).id();
            plan.target = Some(target_entity);
        }
    }
}
