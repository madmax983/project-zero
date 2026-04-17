use bevy::prelude::{App, Plugin, Transform, Update, Vec3};
use bevy_ecs::prelude::*;
use bevy_time::Time;

#[allow(unused_imports)]
use crate::layer2::ship::Ship;

#[derive(Component)]
pub struct Velocity {
    pub value: Vec3,
}

#[derive(Component)]
pub struct Mass {
    pub value: f32,
}

#[derive(Component)]
pub struct ThrustCapability {
    pub max_thrust: f32,
}

#[derive(Component)]
pub struct NavTarget {
    pub destination: Vec3,
}

#[derive(Component)]
pub struct FuelStorage {
    pub current: f32,
    pub capacity: f32,
}

#[derive(Resource)]
pub struct ShipPhysicsConfig {
    pub base_turn_rate_modifier: f32,
    pub retro_burn_fuel_cost_per_sec: f32,
    pub retro_burn_deceleration_multiplier: f32,
}

impl Default for ShipPhysicsConfig {
    fn default() -> Self {
        Self {
            base_turn_rate_modifier: 100.0,
            retro_burn_fuel_cost_per_sec: 10.0,
            retro_burn_deceleration_multiplier: 5.0,
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct RetroBurnInitiatedEvent {
    pub ship: Entity,
}

pub fn calculate_acceleration(direction: Vec3, thrust: f32, mass: f32) -> Vec3 {
    (direction * thrust) / mass
}

pub fn apply_thrust_system(
    mut query: Query<(
        &mut Velocity,
        &Transform,
        &NavTarget,
        &ThrustCapability,
        &Mass,
    )>,
    time: Res<Time>,
) {
    for (mut velocity, transform, target, thrust, mass) in query.iter_mut() {
        let direction = (target.destination - transform.translation).normalize_or_zero();
        let acceleration = calculate_acceleration(direction, thrust.max_thrust, mass.value);
        velocity.value += acceleration * time.delta_secs();
    }
}

pub fn calculate_turn_rate_system(
    mut query: Query<(&mut Velocity, &Transform, &NavTarget, &Mass)>,
    time: Res<Time>,
    config: Res<ShipPhysicsConfig>,
) {
    for (mut velocity, transform, target, mass) in query.iter_mut() {
        let desired_dir = (target.destination - transform.translation).normalize_or_zero();
        let current_dir = velocity.value.normalize_or_zero();

        // Turn rate inversely proportional to mass
        let turn_rate = config.base_turn_rate_modifier / mass.value;

        let new_dir = current_dir
            .lerp(desired_dir, turn_rate * time.delta_secs())
            .normalize_or_zero();
        let speed = velocity.value.length();
        velocity.value = new_dir * speed;
    }
}

#[allow(clippy::type_complexity)]
pub fn retro_burn_system(
    mut query: Query<(
        Entity,
        &mut Velocity,
        &Transform,
        &NavTarget,
        &mut FuelStorage,
        Option<&ThrustCapability>,
        Option<&Mass>,
    )>,
    time: Res<Time>,
    config: Res<ShipPhysicsConfig>,
    mut events: EventWriter<RetroBurnInitiatedEvent>,
) {
    for (entity, mut velocity, transform, target, mut fuel, thrust_opt, mass_opt) in
        query.iter_mut()
    {
        let direction_to_target = (target.destination - transform.translation).normalize_or_zero();
        let current_dir = velocity.value.normalize_or_zero();

        // If traveling away from target (overshot), perform retro burn
        if current_dir.dot(direction_to_target) < 0.0 && fuel.current > 0.0 {
            let burn_amount = config.retro_burn_fuel_cost_per_sec * time.delta_secs();
            fuel.current -= burn_amount;

            let deceleration_rate = if let (Some(thrust), Some(mass)) = (thrust_opt, mass_opt) {
                calculate_acceleration(current_dir, thrust.max_thrust, mass.value).length()
                    * config.retro_burn_deceleration_multiplier
            } else {
                config.retro_burn_deceleration_multiplier
            };

            let deceleration = current_dir * deceleration_rate * time.delta_secs();
            velocity.value -= deceleration;

            events.send(RetroBurnInitiatedEvent { ship: entity });
        }
    }
}

pub struct InertialLogisticsPlugin;

impl Plugin for InertialLogisticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShipPhysicsConfig>()
            .add_event::<RetroBurnInitiatedEvent>()
            .add_systems(
                Update,
                (
                    apply_thrust_system,
                    calculate_turn_rate_system,
                    retro_burn_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ship_gains_momentum_during_travel() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<Time>();
        app.init_resource::<ShipPhysicsConfig>();
        app.add_event::<RetroBurnInitiatedEvent>();
        app.add_systems(Update, apply_thrust_system);

        let ship_id = app
            .world_mut()
            .spawn((
                Ship {
                    ship_type: crate::layer2::ship::ShipType::Transport,
                    health: 100.0,
                    max_health: 100.0,
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity { value: Vec3::ZERO },
                Mass { value: 1000.0 },
                ThrustCapability { max_thrust: 50.0 },
                NavTarget {
                    destination: Vec3::new(100.0, 0.0, 0.0),
                },
            ))
            .id();

        // Act
        // Advance time manually to get non-zero delta_secs
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(1));

        app.update();

        // Assert
        let velocity = app.world().get::<Velocity>(ship_id).unwrap();
        assert!(
            velocity.value.x > 0.0,
            "Ship should gain momentum towards destination."
        );
    }

    #[test]
    fn test_heavy_ship_turns_slower() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<Time>();
        app.init_resource::<ShipPhysicsConfig>();
        app.add_event::<RetroBurnInitiatedEvent>();
        app.add_systems(Update, calculate_turn_rate_system);

        let light_ship = app
            .world_mut()
            .spawn((
                Ship {
                    ship_type: crate::layer2::ship::ShipType::Scout,
                    health: 100.0,
                    max_health: 100.0,
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity {
                    value: Vec3::new(10.0, 0.0, 0.0),
                },
                Mass { value: 100.0 },
                NavTarget {
                    destination: Vec3::new(10.0, 10.0, 0.0),
                },
            ))
            .id();

        let heavy_ship = app
            .world_mut()
            .spawn((
                Ship {
                    ship_type: crate::layer2::ship::ShipType::Transport,
                    health: 100.0,
                    max_health: 100.0,
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity {
                    value: Vec3::new(10.0, 0.0, 0.0),
                },
                Mass { value: 5000.0 },
                NavTarget {
                    destination: Vec3::new(10.0, 10.0, 0.0),
                },
            ))
            .id();

        // Act
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(1));

        app.update();

        // Assert
        let light_vel = app.world().get::<Velocity>(light_ship).unwrap();
        let heavy_vel = app.world().get::<Velocity>(heavy_ship).unwrap();

        // Light ship should have turned its velocity vector more towards Y than the heavy ship
        assert!(
            light_vel.value.y > heavy_vel.value.y,
            "Lighter ship should turn faster than heavy ship."
        );
    }

    #[test]
    fn test_retro_burn_consumes_fuel() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<Time>();
        app.init_resource::<ShipPhysicsConfig>();
        app.add_event::<RetroBurnInitiatedEvent>();
        app.add_systems(Update, retro_burn_system);

        let ship_id = app
            .world_mut()
            .spawn((
                Ship {
                    ship_type: crate::layer2::ship::ShipType::Transport,
                    health: 100.0,
                    max_health: 100.0,
                },
                Transform::from_xyz(20.0, 0.0, 0.0),
                Velocity {
                    value: Vec3::new(100.0, 0.0, 0.0),
                },
                NavTarget {
                    destination: Vec3::new(10.0, 0.0, 0.0),
                }, // Overshot
                FuelStorage {
                    current: 500.0,
                    capacity: 1000.0,
                },
            ))
            .id();

        // Act
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(1));

        app.update();

        // Assert
        let fuel = app.world().get::<FuelStorage>(ship_id).unwrap();
        let velocity = app.world().get::<Velocity>(ship_id).unwrap();

        assert!(fuel.current < 500.0, "Retro burn should consume fuel.");
        assert!(
            velocity.value.x < 100.0,
            "Retro burn should reduce forward velocity."
        );
    }
}
