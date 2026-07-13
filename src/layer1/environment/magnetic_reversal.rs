use bevy::math::Vec2;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Resource)]
pub struct MagneticField {
    pub active: bool,
    pub radiation_level: f32,
}

#[derive(Event)]
pub struct PoleFlipEvent;

pub fn handle_pole_flip_system(
    mut events: EventReader<PoleFlipEvent>,
    mut field: ResMut<MagneticField>,
) {
    for _ in events.read() {
        field.active = false;
        field.radiation_level = 10.0; // Arbitrary spike
    }
}

#[derive(Component)]
pub struct NavigationComponent {
    pub heading: f32,
    pub target: Vec2,
}

pub fn navigation_scramble_system(
    field: Res<MagneticField>,
    mut query: Query<&mut NavigationComponent>,
) {
    if !field.active {
        // Determinism can be improved later by a PRNG resource, but minimal impl uses thread_rng
        let mut rng = rand::thread_rng();
        for mut nav in query.iter_mut() {
            // Apply continuous random noise to heading to scramble navigation
            nav.heading += rng.gen_range(-10.0..10.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PoleFlipEvent>();
        app.add_systems(
            Update,
            (handle_pole_flip_system, navigation_scramble_system),
        );
        app
    }

    #[test]
    fn test_magnetic_reversal_starts() {
        let mut app = setup_app();
        app.insert_resource(MagneticField {
            active: true,
            radiation_level: 0.0,
        });

        app.world_mut().send_event(PoleFlipEvent);
        app.update();

        let field = app.world().resource::<MagneticField>();
        assert!(!field.active);
        assert!(field.radiation_level > 0.0);
    }

    #[test]
    fn test_magnetic_reversal_scrambles_navigation() {
        let mut app = setup_app();
        let entity = app
            .world_mut()
            .spawn(NavigationComponent {
                heading: 90.0,
                target: Vec2::new(10.0, 10.0),
            })
            .id();
        app.insert_resource(MagneticField {
            active: false,
            radiation_level: 5.0,
        });

        app.update();

        let nav = app.world().get::<NavigationComponent>(entity).unwrap();
        assert_ne!(nav.heading, 90.0); // Heading should be scrambled
    }
}
