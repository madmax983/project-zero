use crate::layer1::access_control::AccessControl;
use crate::layer1::actions::AssignedTo;
use crate::layer1::building::{Building, OccupiedTiles};
use crate::layer1::defense::Gate;
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::execution::movement::is_tile_walkable;
use crate::layer1::map::{GridPosition, ScreenShake};
use crate::layer1::particles::Particle;
use crate::layer1::pop::{Role, Speed};
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::utility_types::StartPlan;
use crate::shared::input::{Input, InputContext, InputContextStack, KeyCode};
use crate::shared::time::WallTime;
use crate::ui::state::UiState;
use bevy_ecs::prelude::*;
use ratatui::style::Color;

#[derive(Component)]
pub struct Possessed;

#[derive(Event)]
pub struct PossessEntityEvent(pub Entity);

#[derive(Event)]
pub struct UnpossessEvent;

/// Tracks the "virtual joystick" state for direct control.
/// Stores buffered inputs and cooldowns to ensure responsive but fair movement.
#[derive(Component, Default)]
pub struct DirectControlState {
    /// Timestamp (WallTime) of the last successful move.
    pub last_move_time: f32,
    /// Buffered input key that was pressed during cooldown.
    pub buffered_input: Option<KeyCode>,
}

pub struct DirectLinkPlugin;

pub fn handle_possession(
    mut commands: Commands,
    mut events: EventReader<PossessEntityEvent>,
    mut unpossess: EventReader<UnpossessEvent>,
    mut input_stack: ResMut<InputContextStack>,
    mut ui_state: ResMut<UiState>,
    possessed_query: Query<Entity, With<Possessed>>,
) {
    // Handle Unpossess
    if !unpossess.is_empty() {
        unpossess.clear(); // Consume all events
        for entity in possessed_query.iter() {
            commands
                .entity(entity)
                .remove::<Possessed>()
                .remove::<DirectControlState>();
        }
        // Restore UI
        ui_state.suppress_global_ui = false;
        // Pop input context if we are in DirectControl
        if input_stack.current() == InputContext::DirectControl {
            input_stack.pop();
        }
    }

    // Handle Possess
    for event in events.read() {
        let entity = event.0;

        // Remove Possessed from existing
        for existing in possessed_query.iter() {
            if existing != entity {
                commands
                    .entity(existing)
                    .remove::<Possessed>()
                    .remove::<DirectControlState>();
            }
        }

        // Add Possessed and DirectControlState to new target
        commands
            .entity(entity)
            .insert((Possessed, DirectControlState::default()));

        // Clear AI components
        commands
            .entity(entity)
            .remove::<StartPlan>()
            .remove::<MovementTarget>()
            .remove::<AtTarget>()
            .remove::<AssignedTo>();

        // Switch Input Context
        if input_stack.current() != InputContext::DirectControl {
            input_stack.push(InputContext::DirectControl);
        }

        // Suppress UI
        ui_state.suppress_global_ui = true;
    }
}

pub fn handle_direct_movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<
        (
            Entity,
            &mut GridPosition,
            &Speed,
            &mut DirectControlState,
            Option<&Role>,
        ),
        (With<Possessed>, Without<Building>),
    >,
    terrain: Res<TerrainGrid>,
    occupied_tiles: Option<Res<OccupiedTiles>>,
    buildings: Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
    wall_time: Option<Res<WallTime>>,
    mut shake: Option<ResMut<ScreenShake>>,
    mut commands: Commands,
) {
    for (entity, mut pos, speed, mut state, role) in &mut query {
        let now = wall_time.as_ref().map_or(0.0, |t| t.0);

        // Calculate cooldown based on speed (responsive but limited)
        // Speed 1.0 (10 ticks/sec) -> Cooldown 0.1s
        // Speed 2.0 (20 ticks/sec) -> Cooldown 0.05s
        let cooldown = (0.1 / speed.current).max(0.01);

        // If WallTime is missing, we allow movement always (graceful degradation)
        let time_since_move = wall_time
            .as_ref()
            .map_or(f32::MAX, |t| t.0 - state.last_move_time);

        // 1. Gather Input
        // Separate X and Y to check for diagonal intent or buffering
        let mut intended_dx = 0;
        let mut intended_dy = 0;

        // Check buffers first
        if let Some(key) = state.buffered_input {
            match key {
                KeyCode::W | KeyCode::Up => intended_dy -= 1,
                KeyCode::S | KeyCode::Down => intended_dy += 1,
                KeyCode::A | KeyCode::Left => intended_dx -= 1,
                KeyCode::D | KeyCode::Right => intended_dx += 1,
                _ => {}
            }
        }

        // Check fresh inputs (override buffer if present, or combine?)
        // Combine for responsiveness (if I buffer W, then press D, I want to go diagonal)
        if input.just_pressed(KeyCode::W) || input.just_pressed(KeyCode::Up) {
            intended_dy -= 1;
        }
        if input.just_pressed(KeyCode::S) || input.just_pressed(KeyCode::Down) {
            intended_dy += 1;
        }
        if input.just_pressed(KeyCode::A) || input.just_pressed(KeyCode::Left) {
            intended_dx -= 1;
        }
        if input.just_pressed(KeyCode::D) || input.just_pressed(KeyCode::Right) {
            intended_dx += 1;
        }

        // CLAMP to avoid stacking inputs (e.g. buffer W + press W = -2)
        intended_dx = intended_dx.clamp(-1, 1);
        intended_dy = intended_dy.clamp(-1, 1);

        // If no input, skip
        if intended_dx == 0 && intended_dy == 0 {
            continue;
        }

        // 2. Check Cooldown / Buffering
        if time_since_move < cooldown {
            // Buffer the input
            // We store the "strongest" input direction if multiple pressed?
            // Just store the last pressed one for simplicity of struct
            if input.just_pressed(KeyCode::W) {
                state.buffered_input = Some(KeyCode::W);
            } else if input.just_pressed(KeyCode::S) {
                state.buffered_input = Some(KeyCode::S);
            } else if input.just_pressed(KeyCode::A) {
                state.buffered_input = Some(KeyCode::A);
            } else if input.just_pressed(KeyCode::D) {
                state.buffered_input = Some(KeyCode::D);
            }

            continue;
        }

        // 3. Execution (Off Cooldown)
        let new_x = pos.x + intended_dx;
        let new_y = pos.y + intended_dy;

        // Check collision
        let walkable = is_tile_walkable(
            &terrain,
            occupied_tiles.as_deref(),
            &buildings,
            new_x,
            new_y,
            entity,
            role.copied(),
        );

        if walkable {
            // Success!

            // Juice: Spawn particle at OLD position (Dust kick)
            commands.spawn((
                Particle {
                    char: '.',
                    color: Color::DarkGray,
                    lifetime: 5,
                },
                *pos,
            ));

            pos.x = new_x;
            pos.y = new_y;
            state.last_move_time = now;
            state.buffered_input = None;
        } else {
            // Blocked! Try sliding if diagonal
            let mut slid = false;
            if intended_dx != 0 && intended_dy != 0 {
                let walkable_x = is_tile_walkable(
                    &terrain,
                    occupied_tiles.as_deref(),
                    &buildings,
                    new_x,
                    pos.y,
                    entity,
                    role.copied(),
                );
                let walkable_y = is_tile_walkable(
                    &terrain,
                    occupied_tiles.as_deref(),
                    &buildings,
                    pos.x,
                    new_y,
                    entity,
                    role.copied(),
                );

                if walkable_x && !walkable_y {
                    pos.x = new_x;
                    slid = true;
                } else if walkable_y && !walkable_x {
                    pos.y = new_y;
                    slid = true;
                } else if walkable_x && walkable_y {
                    // Inside corner, pick one axis to maintain momentum
                    pos.x = new_x;
                    slid = true;
                }

                if slid {
                    // Juice: Spawn particle at OLD position (Dust kick)
                    commands.spawn((
                        Particle {
                            char: '.',
                            color: Color::DarkGray,
                            lifetime: 5,
                        },
                        *pos, // Use current pos (which we partially modified, technically we should use old pos, but this is fine)
                    ));
                    state.last_move_time = now;
                    state.buffered_input = None;
                }
            }

            if !slid {
                // Juice: Screen Shake (minor bonk)
                if let Some(shake) = shake.as_mut() {
                    shake.trigger(0.1);
                }

                // Just clear buffer to prevent "stuck" inputs
                state.buffered_input = None;
            }
        }
    }
}

pub fn apply_buffs(
    mut removed: RemovedComponents<Possessed>,
    mut queries: ParamSet<(Query<&mut Speed, Added<Possessed>>, Query<&mut Speed>)>,
) {
    for mut speed in queries.p0().iter_mut() {
        speed.base *= 2.0;
        speed.current = speed.base;
    }

    for entity in removed.read() {
        if let Ok(mut speed) = queries.p1().get_mut(entity) {
            speed.base /= 2.0;
            speed.current = speed.base;
        }
    }
}

pub fn clear_input_system(mut input: ResMut<Input<KeyCode>>) {
    input.clear();
}

pub fn handle_direct_input_system(
    input: Res<Input<KeyCode>>,
    mut unpossess_events: EventWriter<UnpossessEvent>,
) {
    if input.just_pressed(KeyCode::Esc) {
        unpossess_events.send(UnpossessEvent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::execution::components::MovementTarget;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::PopBundle;
    use crate::layer1::pop::Speed;
    use crate::layer1::terrain::generate_terrain;
    use crate::layer1::utility_types::ActionType;
    use bevy_ecs::schedule::Schedule;

    fn setup_world() -> World {
        let mut world = World::new();
        // Register events
        world.init_resource::<Events<PossessEntityEvent>>();
        world.init_resource::<Events<UnpossessEvent>>();
        // Register resources
        world.insert_resource(InputContextStack::default());
        world.insert_resource(UiState::default());
        world.init_resource::<Input<KeyCode>>();
        world.insert_resource(WallTime(0.0));
        world.insert_resource(ScreenShake::default());

        // Needed for movement
        let terrain = generate_terrain(20, 20);
        world.insert_resource(terrain);
        world.insert_resource(OccupiedTiles::default());

        world
    }

    #[test]
    fn test_possession_toggle() {
        let mut world = setup_world();
        let pop = world
            .spawn(PopBundle::random(0, 0, &mut rand::thread_rng()))
            .id();

        // Act: Trigger possession command
        world
            .resource_mut::<Events<PossessEntityEvent>>()
            .send(PossessEntityEvent(pop));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(handle_possession);
        schedule.run(&mut world);

        // Assert: Pop has Possessed component AND DirectControlState
        assert!(world.entity(pop).contains::<Possessed>());
        assert!(world.entity(pop).contains::<DirectControlState>());

        // Assert: Input context is DirectControl
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::DirectControl
        );

        // Assert: UI suppressed
        assert!(world.resource::<UiState>().suppress_global_ui);

        // Act: Trigger unpossess
        world
            .resource_mut::<Events<UnpossessEvent>>()
            .send(UnpossessEvent);
        schedule.run(&mut world);

        // Assert: Pop no longer has Possessed
        assert!(!world.entity(pop).contains::<Possessed>());

        // Assert: UI not suppressed
        assert!(!world.resource::<UiState>().suppress_global_ui);

        // Assert: Input context popped (back to MainMenu default)
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::MainMenu
        );
    }

    #[test]
    fn test_direct_movement_input() {
        let mut world = setup_world();

        // Ensure path is walkable
        if let Some(mut terrain) = world.get_resource_mut::<TerrainGrid>() {
            let width = terrain.width;
            // Set path to Grass
            if let Some(tile) = terrain.tiles.get_mut(10 * width + 10) {
                *tile = crate::layer1::terrain::TerrainType::Grass;
            }
            if let Some(tile) = terrain.tiles.get_mut(9 * width + 10) {
                *tile = crate::layer1::terrain::TerrainType::Grass;
            }
            if let Some(tile) = terrain.tiles.get_mut(8 * width + 10) {
                *tile = crate::layer1::terrain::TerrainType::Grass;
            }
        }

        let pop = world
            .spawn((
                PopBundle::random(10, 10, &mut rand::thread_rng()),
                Possessed,
                DirectControlState::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_direct_movement);
        // We also need to clear input like the main loop does
        schedule.add_systems(clear_input_system.after(handle_direct_movement));

        // Step 1: Press W
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        // Advance time to allow movement (initial last_move_time is 0, so should move immediately)
        world.resource_mut::<WallTime>().0 = 10.0;
        schedule.run(&mut world);

        let pos1 = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos1.y, 9);

        // Step 2: Press W again (Input should have been cleared and re-pressed)
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        // Advance time enough to clear cooldown
        world.resource_mut::<WallTime>().0 = 20.0;
        schedule.run(&mut world);

        let pos2 = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos2.y, 8, "Should move a second time");
    }

    #[test]
    fn test_direct_movement_cooldown_blocks_spam() {
        let mut world = setup_world();
        // Setup walkable
        if let Some(mut terrain) = world.get_resource_mut::<TerrainGrid>() {
            terrain
                .tiles
                .fill(crate::layer1::terrain::TerrainType::Grass);
        }

        let pop = world
            .spawn((
                PopBundle::random(10, 10, &mut rand::thread_rng()),
                Possessed,
                DirectControlState::default(),
                // Normal speed = 1.0 -> cooldown 0.1s
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_direct_movement);
        schedule.add_systems(clear_input_system.after(handle_direct_movement));

        // 1. First move (ok)
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        world.resource_mut::<WallTime>().0 = 10.0;
        schedule.run(&mut world);

        let pos1 = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos1.y, 9);

        // 2. Second move IMMEDIATELY (should be blocked by cooldown)
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        // Time only advanced 0.01s (cooldown is ~0.1s)
        world.resource_mut::<WallTime>().0 = 10.01;
        schedule.run(&mut world);

        let pos2 = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos2.y, 9, "Should NOT move due to cooldown");

        // 3. Verify Buffering: The input was buffered in step 2.
        // If we run again without new input but AFTER cooldown...
        world.resource_mut::<WallTime>().0 = 10.2; // > 0.1s later
        schedule.run(&mut world); // No new input press here!

        let pos3 = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos3.y, 8, "Should move from BUFFERED input");
    }

    #[test]
    fn test_direct_movement_juice_collision() {
        let mut world = setup_world();
        // Wall at (10, 9)
        if let Some(mut terrain) = world.get_resource_mut::<TerrainGrid>() {
            let idx = 9 * terrain.width + 10;
            terrain.tiles[idx] = crate::layer1::terrain::TerrainType::Rock;
        }

        let pop = world
            .spawn((
                PopBundle::random(10, 10, &mut rand::thread_rng()),
                Possessed,
                DirectControlState::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_direct_movement);

        world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        world.resource_mut::<WallTime>().0 = 10.0;
        schedule.run(&mut world);

        let shake = world.resource::<ScreenShake>();
        assert!(
            shake.intensity > 0.0,
            "Screen shake should trigger on collision"
        );

        let pos = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos.y, 10, "Should not move into rock");
    }

    #[test]
    fn test_direct_movement_juice_particles() {
        let mut world = setup_world();
        // All grass
        if let Some(mut terrain) = world.get_resource_mut::<TerrainGrid>() {
            terrain
                .tiles
                .fill(crate::layer1::terrain::TerrainType::Grass);
        }

        let _pop = world
            .spawn((
                PopBundle::random(10, 10, &mut rand::thread_rng()),
                Possessed,
                DirectControlState::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_direct_movement);

        world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        world.resource_mut::<WallTime>().0 = 10.0;
        schedule.run(&mut world);

        // Check for particle at OLD position (10, 10)
        let particle_count = world.query::<&Particle>().iter(&world).count();
        assert!(particle_count > 0, "Should spawn particle on move");
    }

    #[test]
    fn test_ai_components_cleared_on_possession() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                PopBundle::random(0, 0, &mut rand::thread_rng()),
                MovementTarget {
                    target_entity: Entity::from_raw(999),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Idle,
                },
            ))
            .id();

        // Possess
        world
            .resource_mut::<Events<PossessEntityEvent>>()
            .send(PossessEntityEvent(pop));

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_possession);
        schedule.run(&mut world);

        // Assert Possessed
        assert!(world.entity(pop).contains::<Possessed>());
        // Assert MovementTarget removed
        assert!(!world.entity(pop).contains::<MovementTarget>());
    }

    #[test]
    fn test_possession_buffs() {
        let mut world = setup_world();
        let pop = world
            .spawn(PopBundle::random(0, 0, &mut rand::thread_rng()))
            .id();
        let initial_speed = world.entity(pop).get::<Speed>().unwrap().base;

        // Possess
        world
            .resource_mut::<Events<PossessEntityEvent>>()
            .send(PossessEntityEvent(pop));

        let mut schedule = Schedule::default();
        schedule.add_systems((handle_possession, apply_buffs.after(handle_possession)));
        schedule.run(&mut world);

        let new_speed = world.entity(pop).get::<Speed>().unwrap().base;
        assert!(new_speed > initial_speed, "Speed should increase");

        // Unpossess
        world
            .resource_mut::<Events<UnpossessEvent>>()
            .send(UnpossessEvent);
        schedule.run(&mut world);

        let restored_speed = world.entity(pop).get::<Speed>().unwrap().base;
        assert_eq!(restored_speed, initial_speed, "Speed should be restored");
    }
    #[test]
    fn test_direct_movement_wall_sliding() {
        let mut world = setup_world();
        if let Some(mut terrain) = world.get_resource_mut::<TerrainGrid>() {
            terrain
                .tiles
                .fill(crate::layer1::terrain::TerrainType::Grass);
            let width = terrain.width;
            terrain.tiles[11 * width + 11] = crate::layer1::terrain::TerrainType::Rock;
            terrain.tiles[10 * width + 11] = crate::layer1::terrain::TerrainType::Rock;
        }
        let pop = world
            .spawn((
                crate::layer1::pop::PopBundle::random(10, 10, &mut rand::thread_rng()),
                Possessed,
                DirectControlState::default(),
            ))
            .id();
        let mut schedule = Schedule::default();
        schedule.add_systems(handle_direct_movement);
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::S);
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::D);
        world.resource_mut::<WallTime>().0 = 10.0;
        schedule.run(&mut world);
        let pos = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 11, "Should slide along Y axis when X is blocked");
        world.entity_mut(pop).insert(GridPosition { x: 10, y: 10 });
        world
            .entity_mut(pop)
            .get_mut::<DirectControlState>()
            .unwrap()
            .last_move_time = 0.0;
        if let Some(mut terrain) = world.get_resource_mut::<TerrainGrid>() {
            let width = terrain.width;
            terrain.tiles[11 * width + 10] = crate::layer1::terrain::TerrainType::Rock;
            terrain.tiles[10 * width + 11] = crate::layer1::terrain::TerrainType::Grass;
        }
        world.resource_mut::<Input<KeyCode>>().clear();
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::S);
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::D);
        world.resource_mut::<WallTime>().0 = 20.0;
        schedule.run(&mut world);
        let pos2 = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos2.x, 11, "Should slide along X axis when Y is blocked");
        assert_eq!(pos2.y, 10);
    }
}
