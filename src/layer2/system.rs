use bevy_ecs::prelude::*;
use ratatui::style::Color;

/// Defines the current view mode of the game.
#[derive(Resource, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    /// The Colony View (Layer 1).
    #[default]
    Colony,
    /// The System View (Layer 2).
    System,
}

/// Represents a celestial body in the system view (e.g., Planet, Moon).
#[derive(Component, Debug, Clone)]
pub struct OrbitalBody {
    /// The display name of the body.
    pub name: String,
    /// The radius of the body (affects rendering size).
    pub radius: f32,
    /// The color of the body.
    pub color: Color,
    /// The character used to represent the body.
    pub char: char,
}

/// Defines the orbit of a celestial body.
#[derive(Component, Debug, Clone)]
pub struct Orbit {
    /// The parent entity this body orbits around.
    pub parent: Entity,
    /// The radius of the orbit.
    pub radius: f32,
    /// The orbital speed (radians per tick).
    pub speed: f32,
    /// The current angle in the orbit (radians).
    pub angle: f32,
}

/// Resource holding system-level map data.
#[derive(Resource, Default)]
pub struct SystemMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::input::{GameKeyCode, GameKeyEvent};
    use crate::shared::input::{InputContext, InputContextStack};
    use crate::shared::state::GameState;
    // InputRouter might not be public or available directly. Let's check imports.
    // route_input is a function in crate::shared::input.
    use crate::shared::input::route_input;

    fn key_event(code: GameKeyCode) -> GameKeyEvent {
        GameKeyEvent::new(code)
    }

    #[test]
    fn test_view_mode_resource() {
        let mut world = World::new();
        world.insert_resource(ViewMode::default());

        assert_eq!(*world.resource::<ViewMode>(), ViewMode::Colony);
    }

    #[test]
    fn test_orbital_body_component() {
        let mut world = World::new();
        let entity = world
            .spawn(OrbitalBody {
                name: "Planet".to_string(),
                radius: 10.0,
                color: Color::Blue,
                char: 'O',
            })
            .id();

        let body = world.get::<OrbitalBody>(entity).unwrap();
        assert_eq!(body.name, "Planet");
        assert_eq!(body.char, 'O');
    }

    #[test]
    fn test_orbit_component() {
        let mut world = World::new();
        let sun = world.spawn_empty().id();
        let planet = world
            .spawn(Orbit {
                parent: sun,
                radius: 100.0,
                speed: 0.1,
                angle: 0.0,
            })
            .id();

        let orbit = world.get::<Orbit>(planet).unwrap();
        assert_eq!(orbit.parent, sun);
        assert!((orbit.radius - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_toggle_view_mode_input() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        world.insert_resource(ViewMode::Colony);
        world.insert_resource(crate::layer2::visibility::SystemVisibility::Full);

        // Setup Input Stack
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);

        // Press Tab to switch to System View
        // Note: route_input is the function to call.
        route_input(&mut world, key_event(GameKeyCode::Tab));

        assert_eq!(
            *world.resource::<ViewMode>(),
            ViewMode::System,
            "Tab should switch to System view"
        );

        // Press Tab to switch back to Colony View
        route_input(&mut world, key_event(GameKeyCode::Tab));
        assert_eq!(
            *world.resource::<ViewMode>(),
            ViewMode::Colony,
            "Tab should switch back to Colony view"
        );
    }
}
