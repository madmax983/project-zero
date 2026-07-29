use bevy_ecs::prelude::*;
use bevy::math::Vec2;

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
        for mut nav in query.iter_mut() {
            // Minimal scramble logic, e.g., invert or randomize. For GREEN, just invert.
            nav.heading = -nav.heading;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PoleFlipEvent>();
        app.insert_resource(MagneticField { active: true, radiation_level: 0.0 });
        app.add_systems(Update, (handle_pole_flip_system, navigation_scramble_system).chain());
        app
    }

    #[test]
    fn test_magnetic_reversal_starts() {
        let mut app = setup_app();

        // Trigger event
        app.world_mut().send_event(PoleFlipEvent);
        app.update();

        let field = app.world().resource::<MagneticField>();
        assert!(!field.active);
        assert!(field.radiation_level > 0.0);
    }

    #[test]
    fn test_magnetic_reversal_scrambles_navigation() {
        let mut app = setup_app();

        // Setup entity with navigation component
        let entity = app.world_mut().spawn(NavigationComponent { heading: 90.0, target: Vec2::new(10.0, 10.0) }).id();
        app.world_mut().insert_resource(MagneticField { active: false, radiation_level: 5.0 });

        app.update();

        let nav = app.world().get::<NavigationComponent>(entity).unwrap();
        assert_ne!(nav.heading, 90.0); // Heading should be scrambled
    }
}
