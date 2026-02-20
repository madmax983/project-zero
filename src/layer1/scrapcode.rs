use bevy_ecs::prelude::*;

/// Resource tracking the Scrapcode infection.
#[derive(Resource, Debug, Clone)]
pub struct Scrapcode {
    /// Whether the infection is currently active.
    pub active: bool,
    /// Cost multiplier (e.g., 1.5 for 50% increase).
    pub severity: f32,
    /// Duration of the infection in ticks.
    pub duration: u32,
}

impl Default for Scrapcode {
    fn default() -> Self {
        Self {
            active: false,
            severity: 1.0, // Default to 1.0 (no change) to avoid 0.0 cost bugs!
            duration: 0,
        }
    }
}

/// Purges the scrapcode infection, resetting it to a dormant state.
pub fn perform_purge(world: &mut World) {
    if let Some(mut scrapcode) = world.get_resource_mut::<Scrapcode>() {
        scrapcode.active = false;
        scrapcode.severity = 1.0;
        scrapcode.duration = 0;

        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
             log.add("Scrapcode Purged!");
        }
    }
}

/// System to decay Scrapcode duration over time.
pub fn scrapcode_decay_system(mut scrapcode: ResMut<Scrapcode>) {
    if scrapcode.active && scrapcode.duration > 0 {
        scrapcode.duration = scrapcode.duration.saturating_sub(1);
        if scrapcode.duration == 0 {
            scrapcode.active = false;
            scrapcode.severity = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{BuildingType, try_place_building, MaterialType};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup Terrain
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        // Setup Resources (Rich)
        world.insert_resource(ColonyResources {
            wood: 1000.0,
            stone: 1000.0,
            metal: 1000.0,
            ..Default::default()
        });
        // Setup OccupiedTiles
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());
        // Setup BuildMode
        world.insert_resource(crate::layer1::building::BuildMode::default());
        // Setup MessageLog
        world.insert_resource(crate::shared::log::MessageLog::default());

        world
    }

    #[test]
    fn test_scrapcode_increases_building_cost() {
        let mut world = setup_world();

        // Activate Scrapcode
        world.insert_resource(Scrapcode {
            active: true,
            severity: 1.5, // 50% cost increase
            ..Default::default()
        });

        // Building Type: Housing
        let building = BuildingType::Housing;
        let normal_cost = building.cost(MaterialType::Wood).wood;

        // Attempt placement
        // Note: try_place_building subtracts cost. We check how much was subtracted.
        let initial_wood = world.resource::<ColonyResources>().wood;
        let success = try_place_building(&mut world, 5, 5, building);
        assert!(success);

        let final_wood = world.resource::<ColonyResources>().wood;
        let cost_paid = initial_wood - final_wood;

        // With 1.5 severity, cost should be increased
        // Note: Use a tolerance or integer math if needed, but for MVP float check:
        let expected_cost = (normal_cost * 1.5).ceil();
        assert!((cost_paid - expected_cost).abs() < f32::EPSILON, "Scrapcode should increase cost by 50%. Paid: {}, Expected: {}", cost_paid, expected_cost);
    }

    #[test]
    fn test_purge_action_removes_scrapcode() {
        let mut world = setup_world();
        world.insert_resource(Scrapcode { active: true, severity: 1.0, duration: 100 });

        // Run Purge System (simulating action completion)
        perform_purge(&mut world);

        let scrapcode = world.resource::<Scrapcode>();
        assert!(!scrapcode.active, "Purge should deactivate Scrapcode");
        assert_eq!(scrapcode.severity, 1.0); // Reset to baseline or 0?
    }

    #[test]
    fn test_scrapcode_decay_system() {
        let mut world = setup_world();
        world.insert_resource(Scrapcode {
            active: true,
            severity: 1.5,
            duration: 1,
        });

        // Run decay logic directly (since we can't easily run system function with ResMut directly outside world context easily, or use system scheduler)
        // Actually, we can schedule it.
        let mut schedule = Schedule::default();
        schedule.add_systems(super::scrapcode_decay_system);
        schedule.run(&mut world);

        let scrapcode = world.resource::<Scrapcode>();
        assert!(!scrapcode.active, "Scrapcode should decay and deactivate");
        assert_eq!(scrapcode.duration, 0);
        assert_eq!(scrapcode.severity, 1.0);
    }
}
