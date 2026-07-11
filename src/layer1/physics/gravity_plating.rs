//! Artificial Gravity Management
//!
//! This module manages artificial gravity generation across different zones in the colony.
//! It tracks the power state of gravity generators and determines if a `Zone` loses gravity,
//! transitioning into a `ZeroG` state. Pops and Items in `ZeroG` zones have their movement
//! drastically altered.
//!
//! ## Examples
//!
//! ```
//! use bevy_ecs::prelude::*;
//! use scale::layer1::physics::gravity_plating::{GravityState, Zone};
//!
//! let mut world = World::new();
//! // Spawn a zone with normal gravity
//! world.spawn((
//!     Zone { id: 1 },
//!     GravityState::Normal,
//! ));
//! ```
use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

// GREEN Phase Minimal Implementation
/// Represents a node within the colony's power grid.
///
/// Tracks the current power supplied versus the required power for the attached system to function.
///
/// ## Examples
/// ```
/// use scale::layer1::physics::gravity_plating::PowerNode;
/// let node = PowerNode { current_power: 10, required_power: 50 };
/// ```
#[derive(Component)]
pub struct PowerNode {
    pub current_power: i32,
    pub required_power: i32,
}

/// Events related to changes in the power grid's status.
///
/// ## Examples
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::physics::gravity_plating::PowerGridEvent;
/// let event = PowerGridEvent::NodeFailed(Entity::from_raw(1));
/// ```
#[derive(Event)]
pub enum PowerGridEvent {
    NodeFailed(Entity),
}

/// A distinct spatial area within the colony that shares environmental properties.
///
/// ## Examples
/// ```
/// use scale::layer1::physics::gravity_plating::Zone;
/// let zone = Zone { id: 42 };
/// ```
#[derive(Component)]
pub struct Zone {
    pub id: i32,
}

/// The current gravitational condition of a `Zone`.
///
/// ## Examples
/// ```
/// use scale::layer1::physics::gravity_plating::GravityState;
/// let state = GravityState::ZeroG;
/// ```
#[derive(Component, PartialEq, Eq, Debug)]
pub enum GravityState {
    Normal,
    ZeroG,
}

/// Defines how an entity traverses space.
///
/// ## Examples
/// ```
/// use scale::layer1::physics::gravity_plating::MovementType;
/// let movement = MovementType::Drifting;
/// ```
#[derive(Component, PartialEq, Eq, Debug)]
pub enum MovementType {
    Walking,
    Drifting,
    ZeroGControlled,
}

/// The physical vector representing the entity's speed and direction.
///
/// ## Examples
/// ```
/// use scale::layer1::physics::gravity_plating::Velocity;
/// let velocity = Velocity { x: 0.5, y: -0.2 };
/// ```
#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

/// A collection of traits possessed by a Pop, such as 'ZeroGTraining'.
///
/// ## Examples
/// ```
/// use scale::layer1::physics::gravity_plating::TraitList;
/// let traits = TraitList { traits: vec!["ZeroGTraining".to_string()] };
/// ```
#[derive(Component)]
pub struct TraitList {
    pub traits: Vec<String>,
}

/// A building component that provides artificial gravity to a specific `target_zone`.
///
/// ## Examples
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::physics::gravity_plating::GravityGenerator;
/// let generator = GravityGenerator { target_zone: Entity::from_raw(2) };
/// ```
#[derive(Component)]
pub struct GravityGenerator {
    pub target_zone: Entity,
}

/// Indicates which `Zone` an entity currently occupies.
///
/// ## Examples
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::physics::gravity_plating::CurrentZone;
/// let zone = CurrentZone { zone: Entity::from_raw(3) };
/// ```
#[derive(Component)]
pub struct CurrentZone {
    pub zone: Entity,
}

/// Monitors the power grid for failing nodes and disables gravity in affected zones.
///
/// Listens for `PowerGridEvent::NodeFailed`. If the failing node belongs to a `GravityGenerator`,
/// the corresponding `target_zone` is placed into a `GravityState::ZeroG`.
///
/// ## Examples
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::physics::gravity_plating::{PowerGridEvent, monitor_gravity_generator_power_system};
/// use bevy_app::Update;
/// let mut app = bevy_app::App::new();
/// app.add_event::<PowerGridEvent>();
/// app.add_systems(Update, monitor_gravity_generator_power_system);
/// ```
pub fn monitor_gravity_generator_power_system(
    mut events: EventReader<PowerGridEvent>,
    generator_query: Query<&GravityGenerator>,
    mut zone_query: Query<&mut GravityState>,
) {
    for event in events.read() {
        let PowerGridEvent::NodeFailed(node_entity) = event;
        if let Ok(gen) = generator_query.get(*node_entity) {
            if let Ok(mut grav_state) = zone_query.get_mut(gen.target_zone) {
                *grav_state = GravityState::ZeroG;
            }
        }
    }
}

// Import Item
use crate::layer1::economy::Item;

/// Penalties applied to an entity's combat effectiveness due to environmental hazards (like Zero-G).
///
/// ## Examples
/// ```
/// use scale::layer1::physics::gravity_plating::CombatModifier;
/// let penalty = CombatModifier { aim_penalty: 0.8, melee_penalty: 0.8 };
/// ```
#[derive(Component)]
pub struct CombatModifier {
    pub aim_penalty: f32,
    pub melee_penalty: f32,
}

type ItemQueryFilter = (With<Item>, Without<Pop>);
type ItemQueryComponents<'a> = (
    Entity,
    &'a CurrentZone,
    &'a mut MovementType,
    Option<&'a mut Velocity>,
);

/// Updates entity movement and combat states based on the gravity of their current zone.
///
/// - Pops without `ZeroGTraining` transition to `MovementType::Drifting` and receive combat penalties.
/// - Pops with `ZeroGTraining` transition to `MovementType::ZeroGControlled`.
/// - Items without gravity enter `MovementType::Drifting` and gain random initial velocity.
/// - Entities returning to `GravityState::Normal` have penalties removed and return to `Walking`.
///
/// ## Examples
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::physics::gravity_plating::apply_zero_g_movement_system;
/// use bevy_app::Update;
/// let mut app = bevy_app::App::new();
/// app.add_systems(Update, apply_zero_g_movement_system);
/// ```
pub fn apply_zero_g_movement_system(
    mut commands: Commands,
    zone_query: Query<&GravityState>,
    mut pop_query: Query<(Entity, &CurrentZone, &mut MovementType, &TraitList), With<Pop>>,
    mut item_query: Query<ItemQueryComponents<'_>, ItemQueryFilter>,
) {
    for (entity, current_zone, mut move_type, traits) in pop_query.iter_mut() {
        if let Ok(grav_state) = zone_query.get(current_zone.zone) {
            if *grav_state == GravityState::ZeroG {
                if traits.traits.contains(&"ZeroGTraining".to_string()) {
                    *move_type = MovementType::ZeroGControlled;
                } else {
                    *move_type = MovementType::Drifting;
                    commands.entity(entity).insert(CombatModifier {
                        aim_penalty: 0.8,
                        melee_penalty: 0.8,
                    });
                }
            } else {
                *move_type = MovementType::Walking;
                commands.entity(entity).remove::<CombatModifier>();
            }
        }
    }

    for (_entity, current_zone, mut move_type, velocity_opt) in item_query.iter_mut() {
        if let Ok(grav_state) = zone_query.get(current_zone.zone) {
            if *grav_state == GravityState::ZeroG {
                *move_type = MovementType::Drifting;
                if let Some(mut vel) = velocity_opt {
                    if vel.x == 0.0 && vel.y == 0.0 {
                        // Apply random drift velocity (placeholder values)
                        vel.x = 0.5;
                        vel.y = 0.5;
                    }
                }
            } else {
                *move_type = MovementType::Walking; // Items stop drifting
                if let Some(mut vel) = velocity_opt {
                    vel.x = 0.0;
                    vel.y = 0.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    #[test]
    fn test_power_failure_disables_zone_gravity() {
        let mut app = bevy_app::App::new();
        app.add_event::<PowerGridEvent>();
        app.add_systems(Update, monitor_gravity_generator_power_system);

        let zone = app
            .world_mut()
            .spawn((Zone { id: 1 }, GravityState::Normal))
            .id();

        let generator = app
            .world_mut()
            .spawn((
                PowerNode {
                    current_power: 0,
                    required_power: 100,
                }, // Failed
                GravityGenerator { target_zone: zone },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<PowerGridEvent>>()
            .send(PowerGridEvent::NodeFailed(generator));

        app.update();

        // Zone should now have ZeroG state
        let gravity = app.world().get::<GravityState>(zone).unwrap();
        assert_eq!(
            *gravity,
            GravityState::ZeroG,
            "Zone should enter Zero-G when its gravity generator loses power."
        );
    }

    #[test]
    fn test_pops_in_zero_g_zone_switch_to_drifting() {
        let mut app = bevy_app::App::new();
        app.add_systems(Update, apply_zero_g_movement_system);

        let zone = app
            .world_mut()
            .spawn((Zone { id: 1 }, GravityState::ZeroG))
            .id();

        // Un-trained Pop
        let pop = app
            .world_mut()
            .spawn((
                Pop,
                MovementType::Walking,
                Velocity { x: 0.0, y: 0.0 },
                CurrentZone { zone },
                TraitList { traits: vec![] },
            ))
            .id();

        // Trained Pop
        let trained_pop = app
            .world_mut()
            .spawn((
                Pop,
                MovementType::Walking,
                Velocity { x: 0.0, y: 0.0 },
                CurrentZone { zone },
                TraitList {
                    traits: vec!["ZeroGTraining".to_string()],
                },
            ))
            .id();

        app.update();

        let move_type = app.world().get::<MovementType>(pop).unwrap();
        assert_eq!(
            *move_type,
            MovementType::Drifting,
            "Untrained Pops in Zero-G must drift."
        );

        let trained_move_type = app.world().get::<MovementType>(trained_pop).unwrap();
        assert_eq!(
            *trained_move_type,
            MovementType::ZeroGControlled,
            "Trained Pops in Zero-G should retain controlled movement."
        );
    }
}

use bevy::prelude::Vec3;

#[derive(Component)]
pub struct GravityPlate {
    pub down_vector: Vec3,
    pub powered: bool,
}

#[derive(Component)]
pub struct Orientation {
    pub up: Vec3,
}

#[derive(Component, Default)]
pub struct MapTile {
    pub down_vector: Vec3,
}

#[derive(Resource)]
pub struct GlobalGravity {
    pub down_vector: Vec3,
}

impl Default for GlobalGravity {
    fn default() -> Self {
        Self {
            down_vector: Vec3::NEG_Y,
        }
    }
}

pub fn update_tile_gravity(plate_query: Query<&GravityPlate>, mut tile_query: Query<&mut MapTile>) {
    if let Some(plate) = plate_query.iter().find(|p| p.powered) {
        for mut tile in tile_query.iter_mut() {
            tile.down_vector = plate.down_vector;
        }
    }
}

pub fn update_pop_orientation(
    mut pop_query: Query<
        (&crate::layer1::core::map::GridPosition, &mut Orientation),
        With<crate::layer1::entities::pop::Pop>,
    >,
    tile_query: Query<(&crate::layer1::core::map::GridPosition, &MapTile)>,
) {
    for (pop_pos, mut orientation) in pop_query.iter_mut() {
        if let Some((_, tile)) = tile_query
            .iter()
            .find(|(t_pos, _)| t_pos.x == pop_pos.x && t_pos.y == pop_pos.y)
        {
            orientation.up = tile.down_vector * -1.0;
        }
    }
}

pub fn apply_falling_mechanics(
    global_gravity: Res<GlobalGravity>,
    plate_query: Query<&GravityPlate>,
    mut pop_query: Query<&mut Orientation, With<crate::layer1::entities::pop::Pop>>,
) {
    if plate_query.iter().find(|p| p.powered).is_none() {
        for mut orientation in pop_query.iter_mut() {
            orientation.up = global_gravity.down_vector * -1.0;
        }
    }
}

#[cfg(test)]
mod gravity_plate_tests {
    use super::*;

    use crate::layer1::map::GridPosition;

    #[test]
    fn test_gravity_plate_defines_down() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, update_tile_gravity);

        // Act: place gravity plate with UP orientation
        let _plate = app
            .world_mut()
            .spawn(GravityPlate {
                down_vector: Vec3::Y,
                powered: true,
            })
            .id();
        let tile = app
            .world_mut()
            .spawn((GridPosition { x: 0, y: 1 }, MapTile::default()))
            .id();

        app.update();

        // Assert: tile 'down' matches plate orientation
        let down = app.world().get::<MapTile>(tile).unwrap().down_vector;
        assert_eq!(down, Vec3::Y);
    }

    #[test]
    fn test_pop_transitions_orientation() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, update_pop_orientation);

        let pop = app
            .world_mut()
            .spawn((
                crate::layer1::entities::pop::Pop,
                GridPosition { x: 0, y: 0 },
                Orientation { up: Vec3::Y },
            ))
            .id();
        let _tile = app
            .world_mut()
            .spawn((
                GridPosition { x: 0, y: 0 },
                MapTile {
                    down_vector: Vec3::NEG_X,
                },
            ))
            .id();

        // Act: pop walks over a curved plate that shifts 'down' to X
        app.update();

        // Assert: Pop's Orientation changes smoothly
        let up = app.world().get::<Orientation>(pop).unwrap().up;
        assert_eq!(up, Vec3::X);
    }

    #[test]
    fn test_gravity_plate_failure_causes_falling() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, apply_falling_mechanics);

        let pop = app
            .world_mut()
            .spawn((
                crate::layer1::entities::pop::Pop,
                GridPosition { x: 5, y: 10 },
                Orientation { up: Vec3::NEG_Y },
            ))
            .id();
        let _tile = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 10 },
                MapTile {
                    down_vector: Vec3::NEG_Y,
                },
            ))
            .id();
        let _plate = app
            .world_mut()
            .spawn(GravityPlate {
                down_vector: Vec3::Y,
                powered: false,
            })
            .id();

        // Assuming a resource handles global gravity.
        app.world_mut().insert_resource(GlobalGravity {
            down_vector: Vec3::NEG_Y,
        });
        // Act: plate loses power
        app.update();

        // Assert: pop falls down towards Vec3::NEG_Y (global down) - orientation aligns with global gravity
        let up = app.world().get::<Orientation>(pop).unwrap().up;
        assert_eq!(up, Vec3::Y);
    }
}
