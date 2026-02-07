use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::shared::narrative::NarrativeGenerator;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Stores named locations on the world map.
#[derive(Resource, Default, Debug)]
pub struct NamedLocations {
    /// A map from grid coordinates (x, y) to location names.
    pub names: HashMap<(i32, i32), String>,
}

impl NamedLocations {
    /// Name a specific coordinate. Overwrites existing names.
    pub fn add(&mut self, x: i32, y: i32, name: String) {
        self.names.insert((x, y), name);
    }

    /// Get the name of a location, if it exists.
    #[must_use]
    pub fn get(&self, x: i32, y: i32) -> Option<&String> {
        self.names.get(&(x, y))
    }
}

/// Names the starting location based on the first pop found.
pub fn initial_naming_system(world: &mut World) {
    let mut query = world.query_filtered::<&GridPosition, With<Pop>>();
    let start_pos = query.iter(world).next().map(|pos| (pos.x, pos.y));

    if let Some((x, y)) = start_pos {
        let name = world
            .resource::<NarrativeGenerator>()
            .get_random_fragment("LANDING_NAME")
            .cloned()
            .unwrap_or_else(|| "Landing Site".to_string());

        let mut locations = world.resource_mut::<NamedLocations>();
        locations.add(x, y, name.clone());

        let tick = world.resource::<SimulationTime>().tick;
        let mut chronicle = world.resource_mut::<Chronicle>();
        chronicle.add_event(
            tick,
            format!("We name this place {name}. Here we begin."),
            EventImportance::Standard,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::Chronicle;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::shared::narrative::NarrativeGenerator;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_named_locations_resource_default() {
        let locations = NamedLocations::default();
        assert!(locations.names.is_empty());
    }

    #[test]
    fn test_add_and_get_location_name() {
        let mut locations = NamedLocations::default();

        locations.add(10, 20, "Landing Site".to_string());

        assert_eq!(locations.get(10, 20), Some(&"Landing Site".to_string()));
    }

    #[test]
    fn test_get_nonexistent_location() {
        let locations = NamedLocations::default();
        assert_eq!(locations.get(5, 5), None);
    }

    #[test]
    fn test_overwrite_location_name() {
        let mut locations = NamedLocations::default();

        locations.add(10, 20, "Old Name".to_string());
        locations.add(10, 20, "New Name".to_string());

        assert_eq!(locations.get(10, 20), Some(&"New Name".to_string()));
    }

    #[test]
    fn test_initial_naming_system_creates_landing_site() {
        let mut world = World::new();
        world.insert_resource(NamedLocations::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(NarrativeGenerator::from_embedded());

        // Spawn a pop to define the landing site
        world.spawn((Pop, GridPosition { x: 40, y: 25 }));

        // Run system
        initial_naming_system(&mut world);

        let locations = world.resource::<NamedLocations>();
        let name = locations.get(40, 25);
        assert!(name.is_some(), "Should have a named location");
        assert!(
            !name.unwrap().is_empty(),
            "Location name should not be empty"
        );
    }

    #[test]
    fn test_initial_naming_logs_chronicle_event() {
        let mut world = World::new();
        world.insert_resource(NamedLocations::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(NarrativeGenerator::from_embedded());

        world.spawn((Pop, GridPosition { x: 40, y: 25 }));

        initial_naming_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert!(!chronicle.events.is_empty());
        assert!(!chronicle.events[0].text.is_empty());
    }

    #[test]
    fn test_get_location_name_at_viewport_center() {
        let mut locations = NamedLocations::default();
        locations.add(10, 10, "Center City".to_string());

        // Viewport (0,0) to (80,50)
        // Center of viewport (assuming 80x50 rendering) is roughly x+40, y+25
        // But for this test, let's just test the helper function logic directly
        // if we define a helper for "get focused name"

        // Here we simulate checking the center coordinate logic
        // If viewport is (0,0), and we assume a view of say 20x20 for the test
        // the test hardcoded check:
        // "Center of viewport (assuming 80x50 rendering) is roughly x+40, y+25"
        // Wait, the test in the spec was slightly ambiguous about what logic checks what.
        // It simply asserted: assert_eq!(locations.get(10, 10), Some(&"Center City".to_string()));
        // This just tests `get` again, not the viewport logic.
        // But since I'm implementing the spec test verbatim:

        assert_eq!(locations.get(10, 10), Some(&"Center City".to_string()));
    }
}
