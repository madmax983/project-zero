use bevy::prelude::*;

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
        field.radiation_level = 10.0;
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
            nav.heading = -nav.heading;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_magnetic_reversal_starts() {
        let mut app = App::new();
        app.add_event::<PoleFlipEvent>();
        app.add_systems(Update, handle_pole_flip_system);

        app.world_mut().insert_resource(MagneticField { active: true, radiation_level: 0.0 });

        app.world_mut().send_event(PoleFlipEvent);
        let _ = app.world_mut().run_system_once(handle_pole_flip_system);

        let field = app.world().resource::<MagneticField>();
        assert!(!field.active);
        assert!(field.radiation_level > 0.0);
    }

    #[test]
    fn test_magnetic_reversal_scrambles_navigation() {
        let mut app = App::new();
        app.add_systems(Update, navigation_scramble_system);

        let entity = app.world_mut().spawn(NavigationComponent { heading: 90.0, target: Vec2::new(10.0, 10.0) }).id();
        app.world_mut().insert_resource(MagneticField { active: false, radiation_level: 5.0 });

        let _ = app.world_mut().run_system_once(navigation_scramble_system);

        let nav = app.world().get::<NavigationComponent>(entity).unwrap();
        assert_ne!(nav.heading, 90.0);
    }
}
