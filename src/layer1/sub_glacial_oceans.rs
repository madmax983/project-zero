use bevy::prelude::*;

#[derive(Component)]
pub struct Position {
    pub z: f32, // 0.0 is Ice crust, negative is deep water
}

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct Buoyancy(pub f32);

#[derive(Component, PartialEq, Eq)]
pub enum AnchorState {
    Intact,
    Snapped,
}

#[derive(Component)]
pub struct Velocity(pub f32);

#[derive(Component)]
pub struct StructuralIntegrity(pub f32);

pub fn buoyancy_movement_system(
    mut query: Query<(&mut Position, &mut Velocity, &Mass, &Buoyancy, &AnchorState)>,
) {
    for (mut pos, mut vel, mass, buoyancy, anchor) in query.iter_mut() {
        if *anchor == AnchorState::Snapped {
            let net_force = buoyancy.0 - mass.0;
            let acceleration = net_force / mass.0;
            vel.0 += acceleration * 0.1; // Delta time simplification
            pos.z += vel.0;
        } else {
            vel.0 = 0.0;
        }
    }
}

pub fn ice_collision_system(
    mut query: Query<(&mut Position, &mut Velocity, &mut StructuralIntegrity)>,
) {
    for (mut pos, mut vel, mut health) in query.iter_mut() {
        if pos.z >= 0.0 {
            pos.z = 0.0; // Stop at crust
            if vel.0 > 0.0 {
                // Take impact damage
                health.0 -= vel.0 * 5.0;
                vel.0 = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapped_anchor_causes_upward_floating() {
        let mut app = App::new();
        app.add_systems(Update, buoyancy_movement_system);

        let module = app
            .world_mut()
            .spawn((
                Position { z: -100.0 }, // 100 meters below ice
                Mass(1000.0),
                Buoyancy(1500.0), // Buoyancy > Mass
                AnchorState::Snapped,
                Velocity(0.0),
            ))
            .id();

        app.update();

        // Object should move upwards (z increases towards 0.0)
        let pos = app.world().get::<Position>(module).unwrap();
        assert!(pos.z > -100.0);
    }

    #[test]
    fn test_intact_anchor_prevents_floating() {
        let mut app = App::new();
        app.add_systems(Update, buoyancy_movement_system);

        let module = app
            .world_mut()
            .spawn((
                Position { z: -100.0 },
                Mass(1000.0),
                Buoyancy(1500.0),
                AnchorState::Intact, // Anchor holding it down
                Velocity(0.0),
            ))
            .id();

        app.update();

        // Object should not move
        let pos = app.world().get::<Position>(module).unwrap();
        assert_eq!(pos.z, -100.0);
    }

    #[test]
    fn test_collision_with_ice_crust() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (buoyancy_movement_system, ice_collision_system).chain(),
        );

        let module = app
            .world_mut()
            .spawn((
                Position { z: -1.0 }, // Right below ice
                Mass(1000.0),
                Buoyancy(5000.0), // High buoyancy = fast ascent
                Velocity(10.0),
                AnchorState::Snapped,
                StructuralIntegrity(100.0),
            ))
            .id();

        app.update();

        // Module hits z = 0.0, takes collision damage based on velocity/buoyancy
        let pos = app.world().get::<Position>(module).unwrap();
        let health = app.world().get::<StructuralIntegrity>(module).unwrap();

        assert_eq!(pos.z, 0.0); // Stopped at ceiling
        assert!(health.0 < 100.0); // Took damage
    }
}
