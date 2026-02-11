use bevy_ecs::prelude::*;
use crate::platform::input::{GameKeyCode, GameKeyEvent};

/// What is currently selected by the player.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SelectionTarget {
    /// Nothing selected.
    #[default]
    None,
    /// A tile at grid position (x, y).
    Tile(i32, i32),
    /// An entity (pop, building, etc.).
    Entity(Entity),
}

/// Tracks the current player selection.
#[derive(Resource, Default, Debug)]
pub struct Selection {
    target: SelectionTarget,
}

impl Selection {
    /// Create a new selection resource.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            target: SelectionTarget::None,
        }
    }

    /// Get the current selection target.
    #[must_use]
    pub const fn target(&self) -> SelectionTarget {
        self.target
    }

    /// Select a tile at grid coordinates.
    pub const fn select_tile(&mut self, x: i32, y: i32) {
        self.target = SelectionTarget::Tile(x, y);
    }

    /// Select an entity.
    pub const fn select_entity(&mut self, entity: Entity) {
        self.target = SelectionTarget::Entity(entity);
    }

    /// Clear the selection.
    pub const fn clear(&mut self) {
        self.target = SelectionTarget::None;
    }

    /// Check if anything is selected.
    #[must_use]
    pub const fn is_selected(&self) -> bool {
        !matches!(self.target, SelectionTarget::None)
    }
}

/// Handle selection input in normal mode.
pub fn handle_selection_input(world: &mut World, key: GameKeyEvent) {
    if key.code == GameKeyCode::Esc {
        // Clear selection
        world.resource_mut::<Selection>().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_default_none() {
        let selection = Selection::default();
        assert_eq!(selection.target(), SelectionTarget::None);
    }

    #[test]
    fn test_selection_tile() {
        let mut selection = Selection::default();
        selection.select_tile(10, 5);

        assert_eq!(selection.target(), SelectionTarget::Tile(10, 5));
    }

    #[test]
    fn test_selection_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let mut selection = Selection::default();
        selection.select_entity(entity);

        assert_eq!(selection.target(), SelectionTarget::Entity(entity));
    }

    #[test]
    fn test_selection_clear() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let mut selection = Selection::default();
        selection.select_entity(entity);
        assert!(selection.is_selected());

        selection.clear();
        assert!(!selection.is_selected());
        assert_eq!(selection.target(), SelectionTarget::None);
    }

    #[test]
    fn test_selection_is_selected() {
        let mut selection = Selection::default();
        assert!(!selection.is_selected());

        selection.select_tile(0, 0);
        assert!(selection.is_selected());

        selection.clear();
        assert!(!selection.is_selected());
    }

    #[test]
    fn test_selection_get_tile() {
        let mut selection = Selection::default();
        selection.select_tile(10, 20);

        if let SelectionTarget::Tile(x, y) = selection.target() {
            assert_eq!(x, 10);
            assert_eq!(y, 20);
        } else {
            panic!("Expected tile selection");
        }
    }

    #[test]
    fn test_selection_get_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let mut selection = Selection::default();
        selection.select_entity(entity);

        if let SelectionTarget::Entity(e) = selection.target() {
            assert_eq!(e, entity);
        } else {
            panic!("Expected entity selection");
        }
    }

    #[test]
    fn test_selection_overwrite() {
        let mut selection = Selection::default();
        selection.select_tile(5, 5);
        assert!(selection.is_selected());

        // Selecting new target should overwrite
        selection.select_tile(10, 10);
        assert_eq!(selection.target(), SelectionTarget::Tile(10, 10));
    }

    #[test]
    fn test_selection_target_is_copy() {
        let target1 = SelectionTarget::Tile(5, 5);
        let target2 = target1; // Should copy
        assert_eq!(target1, target2);
    }
}
