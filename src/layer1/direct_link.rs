use bevy_ecs::prelude::*;
use crate::shared::input::{Input, KeyCode, InputContext, InputContextStack};
use crate::ui::state::UiState;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Speed, Role};
use crate::layer1::utility_types::StartPlan;
use crate::layer1::execution::components::{MovementTarget, AtTarget};
use crate::layer1::actions::AssignedTo;
use crate::layer1::execution::movement::is_tile_walkable;
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::building::{OccupiedTiles, Building};
use crate::layer1::defense::Gate;
use crate::layer1::access_control::AccessControl;

#[derive(Component)]
pub struct Possessed;

#[derive(Event)]
pub struct PossessEntityEvent(pub Entity);

#[derive(Event)]
pub struct UnpossessEvent;

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
            commands.entity(entity).remove::<Possessed>();
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
                commands.entity(existing).remove::<Possessed>();
            }
        }

        // Add Possessed to new target
        commands.entity(entity).insert(Possessed);

        // Clear AI components
        commands.entity(entity)
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
    mut query: Query<(Entity, &mut GridPosition, Option<&Role>), (With<Possessed>, Without<Building>)>,
    terrain: Res<TerrainGrid>,
    occupied_tiles: Option<Res<OccupiedTiles>>,
    buildings: Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
) {
    for (entity, mut pos, role) in &mut query {
        let mut dx = 0;
        let mut dy = 0;

        if input.just_pressed(KeyCode::W) || input.just_pressed(KeyCode::Up) {
            dy -= 1;
        } else if input.just_pressed(KeyCode::S) || input.just_pressed(KeyCode::Down) {
            dy += 1;
        } else if input.just_pressed(KeyCode::A) || input.just_pressed(KeyCode::Left) {
            dx -= 1;
        } else if input.just_pressed(KeyCode::D) || input.just_pressed(KeyCode::Right) {
            dx += 1;
        }

        if dx == 0 && dy == 0 {
            continue;
        }

        let new_x = pos.x + dx;
        let new_y = pos.y + dy;

        // Check collision
        if is_tile_walkable(
            &terrain,
            occupied_tiles.as_deref(),
            &buildings,
            new_x,
            new_y,
            entity,
            role.copied()
        ) {
            pos.x = new_x;
            pos.y = new_y;
        }
    }
}

pub fn apply_buffs(
    mut removed: RemovedComponents<Possessed>,
    mut queries: ParamSet<(
        Query<&mut Speed, Added<Possessed>>,
        Query<&mut Speed>,
    )>,
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
    use crate::layer1::pop::{PopBundle, Pop};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Speed;
    use crate::layer1::execution::components::MovementTarget;
    use crate::layer1::utility_types::ActionType;
    use bevy_ecs::schedule::Schedule;
    use crate::layer1::terrain::generate_terrain;

    fn setup_world() -> World {
        let mut world = World::new();
        // Register events
        world.init_resource::<Events<PossessEntityEvent>>();
        world.init_resource::<Events<UnpossessEvent>>();
        // Register resources
        world.insert_resource(InputContextStack::default());
        world.insert_resource(UiState::default());
        world.init_resource::<Input<KeyCode>>();

        // Needed for movement
        let terrain = generate_terrain(20, 20);
        world.insert_resource(terrain);
        world.insert_resource(OccupiedTiles::default());

        world
    }

    #[test]
    fn test_possession_toggle() {
        let mut world = setup_world();
        let pop = world.spawn(PopBundle::random(0, 0, &mut rand::thread_rng())).id();

        // Act: Trigger possession command
        world.resource_mut::<Events<PossessEntityEvent>>().send(PossessEntityEvent(pop));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(handle_possession);
        schedule.run(&mut world);

        // Assert: Pop has Possessed component
        assert!(world.entity(pop).contains::<Possessed>());

        // Assert: Input context is DirectControl
        assert_eq!(world.resource::<InputContextStack>().current(), InputContext::DirectControl);

        // Assert: UI suppressed
        assert!(world.resource::<UiState>().suppress_global_ui);

        // Act: Trigger unpossess
        world.resource_mut::<Events<UnpossessEvent>>().send(UnpossessEvent);
        schedule.run(&mut world);

        // Assert: Pop no longer has Possessed
        assert!(!world.entity(pop).contains::<Possessed>());

        // Assert: UI not suppressed
        assert!(!world.resource::<UiState>().suppress_global_ui);

        // Assert: Input context popped (back to MainMenu default)
        assert_eq!(world.resource::<InputContextStack>().current(), InputContext::MainMenu);
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

        let pop = world.spawn((
            PopBundle::random(10, 10, &mut rand::thread_rng()),
            Possessed
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_direct_movement);
        // We also need to clear input like the main loop does
        schedule.add_systems(clear_input_system.after(handle_direct_movement));

        // Step 1: Press W
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        schedule.run(&mut world);

        let pos1 = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos1.y, 9);

        // Step 2: Press W again (Input should have been cleared and re-pressed)
        world.resource_mut::<Input<KeyCode>>().press(KeyCode::W);
        schedule.run(&mut world);

        let pos2 = world.entity(pop).get::<GridPosition>().unwrap();
        assert_eq!(pos2.y, 8, "Should move a second time");
    }

    #[test]
    fn test_ai_components_cleared_on_possession() {
        let mut world = setup_world();
        let pop = world.spawn((
            PopBundle::random(0, 0, &mut rand::thread_rng()),
            MovementTarget {
                target_entity: Entity::from_raw(999),
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Idle,
            }
        )).id();

        // Possess
        world.resource_mut::<Events<PossessEntityEvent>>().send(PossessEntityEvent(pop));

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
        let pop = world.spawn(PopBundle::random(0, 0, &mut rand::thread_rng())).id();
        let initial_speed = world.entity(pop).get::<Speed>().unwrap().base;

        // Possess
        world.resource_mut::<Events<PossessEntityEvent>>().send(PossessEntityEvent(pop));

        let mut schedule = Schedule::default();
        schedule.add_systems((handle_possession, apply_buffs.after(handle_possession)));
        schedule.run(&mut world);

        let new_speed = world.entity(pop).get::<Speed>().unwrap().base;
        assert!(new_speed > initial_speed, "Speed should increase");

        // Unpossess
        world.resource_mut::<Events<UnpossessEvent>>().send(UnpossessEvent);
        schedule.run(&mut world);

        let restored_speed = world.entity(pop).get::<Speed>().unwrap().base;
        assert_eq!(restored_speed, initial_speed, "Speed should be restored");
    }
}
