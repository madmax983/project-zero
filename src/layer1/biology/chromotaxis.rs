use crate::layer1::architecture::building::Building;
use crate::layer1::biology::health::Health;
use crate::layer1::fauna::Fauna;
use bevy::math::Vec3;
use bevy::transform::components::Transform;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralColor {
    Red,
    Blue,
    Beige,
}

#[derive(Component, Debug, Clone)]
pub struct ChromotaxisTrait {
    pub attractive_color: StructuralColor,
    pub repulsive_color: StructuralColor,
}

#[derive(Component)]
pub struct ChromotaxisVelocity(pub Vec3);

#[derive(Component)]
pub struct ChromotaxisDamageProvider {
    pub amount: f32,
}

#[allow(clippy::type_complexity)]
pub fn chromotaxis_attraction_system(
    mut fauna_query: Query<(&mut ChromotaxisVelocity, &Transform, &ChromotaxisTrait), With<Fauna>>,
    building_query: Query<(&Transform, &StructuralColor), With<Building>>,
) {
    for (mut velocity, fauna_transform, chromotaxis) in fauna_query.iter_mut() {
        for (building_transform, color) in building_query.iter() {
            if *color == chromotaxis.attractive_color {
                let direction = (building_transform.translation - fauna_transform.translation)
                    .normalize_or_zero();
                velocity.0 += direction * 5.0; // simple move towards
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn chromotaxis_aggro_system(
    fauna_query: Query<(&Transform, &ChromotaxisTrait, &ChromotaxisDamageProvider), With<Fauna>>,
    mut building_query: Query<(&Transform, &StructuralColor, &mut Health), With<Building>>,
) {
    for (fauna_transform, chromotaxis, damage) in fauna_query.iter() {
        for (building_transform, color, mut health) in building_query.iter_mut() {
            if *color == chromotaxis.repulsive_color {
                let distance = fauna_transform
                    .translation
                    .distance(building_transform.translation);
                if distance < 2.0 {
                    // Attack range
                    let amount = damage.amount;
                    health.take_damage(amount);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::fauna::FaunaType;
    use bevy::app::App;
    use bevy::app::Update;
    use bevy::MinimalPlugins;

    #[test]
    fn test_chromotaxis_attraction() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, chromotaxis_attraction_system);

        let _building_id = app
            .world_mut()
            .spawn((
                Building::default(),
                StructuralColor::Red,
                Transform::from_xyz(10.0, 10.0, 0.0),
            ))
            .id();

        let fauna_id = app
            .world_mut()
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    state: Default::default(),
                    target: None,
                    detection_range: 10.0,
                    attack_cooldown: 0,
                },
                ChromotaxisTrait {
                    attractive_color: StructuralColor::Red,
                    repulsive_color: StructuralColor::Blue,
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                ChromotaxisVelocity(Vec3::ZERO),
            ))
            .id();

        app.update(); // Run system

        // Assert that the fauna is moving towards the building
        let fauna_velocity = app
            .world()
            .entity(fauna_id)
            .get::<ChromotaxisVelocity>()
            .unwrap();
        assert!(
            fauna_velocity.0.x > 0.0 && fauna_velocity.0.y > 0.0,
            "Fauna should move towards the attractive color."
        );
    }

    #[test]
    fn test_chromotaxis_repulsion_aggro() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, chromotaxis_aggro_system);

        let building_id = app
            .world_mut()
            .spawn((
                Building::default(),
                StructuralColor::Blue,
                Transform::from_xyz(5.0, 5.0, 0.0),
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let _fauna_id = app
            .world_mut()
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    state: Default::default(),
                    target: None,
                    detection_range: 10.0,
                    attack_cooldown: 0,
                },
                ChromotaxisTrait {
                    attractive_color: StructuralColor::Red,
                    repulsive_color: StructuralColor::Blue,
                },
                Transform::from_xyz(4.0, 4.0, 0.0), // Close enough to attack
                ChromotaxisDamageProvider { amount: 10.0 },
            ))
            .id();

        app.update();

        // Assert the building took damage due to the aggro color
        let building_health = app.world().entity(building_id).get::<Health>().unwrap();
        assert!(
            building_health.current < 100.0,
            "Building should take damage from aggro fauna due to repulsive color."
        );
    }

    #[test]
    fn test_chromotaxis_neutral_color() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(
            Update,
            (chromotaxis_attraction_system, chromotaxis_aggro_system),
        );

        let building_id = app
            .world_mut()
            .spawn((
                Building::default(),
                StructuralColor::Beige,
                Transform::from_xyz(10.0, 10.0, 0.0),
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let fauna_id = app
            .world_mut()
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    state: Default::default(),
                    target: None,
                    detection_range: 10.0,
                    attack_cooldown: 0,
                },
                ChromotaxisTrait {
                    attractive_color: StructuralColor::Red,
                    repulsive_color: StructuralColor::Blue,
                },
                Transform::from_xyz(9.0, 9.0, 0.0),
                ChromotaxisVelocity(Vec3::ZERO),
                ChromotaxisDamageProvider { amount: 10.0 },
            ))
            .id();

        app.update();

        // Fauna should not move towards or attack the building
        let fauna_velocity = app
            .world()
            .entity(fauna_id)
            .get::<ChromotaxisVelocity>()
            .unwrap();
        assert_eq!(
            fauna_velocity.0,
            Vec3::ZERO,
            "Fauna should not move towards neutral color."
        );

        let building_health = app.world().entity(building_id).get::<Health>().unwrap();
        assert_eq!(
            building_health.current, 100.0,
            "Building should not take damage from neutral color."
        );
    }
}
