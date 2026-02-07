//! Shared world setup used by all entry points (native, headless, WASM).

use bevy_ecs::prelude::*;

use crate::gpu::context::GpuContext;
use crate::layer1::{
    BuildMode, BuildingTracker, Chronicle, ChronicleUiState, ColonyMemory, ColonyResources,
    DesignationMode, NamedLocations, NotificationQueue, OccupiedTiles, SeasonState, TechState,
    UtilityConfig, Viewport, generate_terrain, initial_chronicle_event, initial_naming_system,
    spawn_initial_anomalies, spawn_initial_pops,
};
use crate::shared::colony::ColonyName;
use crate::shared::input::InputContextStack;
use crate::shared::log::MessageLog;
use crate::shared::narrative::NarrativeGenerator;
use crate::shared::selection::Selection;
use crate::shared::state::GameState;
use crate::shared::time::SimulationTime;
use crate::shared::world_history::generate_world_history;
use crate::ui::map::RenderCache;

/// Ensures the Bevy task pools are initialized (required for `par_iter_mut`).
///
/// Safe to call multiple times — uses `get_or_init` internally.
pub fn init_task_pools() {
    bevy_tasks::ComputeTaskPool::get_or_init(bevy_tasks::TaskPool::default);
}

/// Create and initialize a new game world with all resources.
#[must_use]
pub fn setup_world() -> World {
    init_task_pools();
    let mut world = World::new();
    world.insert_resource(GameState::default());
    world.insert_resource(MenuState::default());
    world.insert_resource(generate_terrain(80, 50));
    world.insert_resource(Viewport::default());
    world.insert_resource(SimulationTime::default());
    world.insert_resource(BuildMode::default());
    world.insert_resource(DesignationMode::default());
    world.insert_resource(OccupiedTiles::default());
    world.insert_resource(ColonyResources::default());
    world.insert_resource(InputContextStack::default());
    world.insert_resource(MessageLog::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(ChronicleUiState::default());
    world.insert_resource(NotificationQueue::default());
    world.insert_resource(BuildingTracker::default());
    world.insert_resource(Selection::default());
    world.insert_resource(RenderCache::default());
    world.insert_resource(UtilityConfig::default());
    world.insert_resource(ColonyMemory::default());
    world.insert_resource(SeasonState::default());
    world.insert_resource(NamedLocations::default());
    world.insert_resource(TechState::default());
    world.insert_resource(crate::layer1::beauty::BeautyGrid::new(80, 50));

    // Initialize GPU compute context (non-fatal if no GPU available)
    match pollster::block_on(GpuContext::new()) {
        Ok(ctx) => {
            world.insert_resource(ctx);
        }
        Err(e) => {
            eprintln!("GPU init failed ({e}), falling back to CPU evaluate");
        }
    }

    let generator = NarrativeGenerator::from_embedded();
    let colony_name = generator.generate_star_name();
    world.insert_resource(generator);
    world.insert_resource(ColonyName { name: colony_name });
    generate_world_history(&mut world);

    spawn_initial_pops(&mut world);
    spawn_initial_anomalies(&mut world, 5);
    initial_naming_system(&mut world);
    initial_chronicle_event(&mut world);

    world
}

use crate::shared::menu::MenuState;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::{Pop, TerrainGrid};
    use crate::shared::colony::ColonyName;
    use crate::shared::narrative::NarrativeGenerator;

    #[test]
    fn test_setup_world_creates_resources() {
        let world = setup_world();

        assert!(world.contains_resource::<GameState>());
        assert!(world.contains_resource::<SimulationTime>());
        assert!(world.contains_resource::<TerrainGrid>());
        assert!(world.contains_resource::<ColonyResources>());
        assert!(world.contains_resource::<Chronicle>());
        assert!(world.contains_resource::<Selection>());
        assert!(world.contains_resource::<NotificationQueue>());
        assert!(world.contains_resource::<RenderCache>());
        assert!(world.contains_resource::<UtilityConfig>());
        assert!(world.contains_resource::<SeasonState>());
        assert!(world.contains_resource::<NamedLocations>());
        assert!(world.contains_resource::<TechState>());
        assert!(world.contains_resource::<NarrativeGenerator>());
        assert!(world.contains_resource::<ColonyName>());
    }

    #[test]
    fn test_setup_world_spawns_pops() {
        let mut world = setup_world();
        let pop_count = world.query::<&Pop>().iter(&world).count();
        assert!(pop_count > 0, "Should have spawned initial pops");
    }

    #[test]
    fn test_setup_world_has_chronicle_event() {
        let world = setup_world();
        let chronicle = world.resource::<Chronicle>();
        assert!(
            !chronicle.events.is_empty(),
            "Should have initial chronicle event"
        );
    }

    #[test]
    fn test_setup_world_default_state_is_main_menu() {
        let world = setup_world();
        assert_eq!(*world.resource::<GameState>(), GameState::MainMenu);
    }
}
