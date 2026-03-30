#[cfg(test)]
mod tests {
    use crate::layer1::building::{systems::*, types::*};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::GridPosition;
    use crate::layer1::{Farm, Housing, Stockpile};
    use crate::shared::log::MessageLog;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_building_type_default() {
        let bt = BuildingType::default();
        assert_eq!(bt, BuildingType::Housing);
    }

    #[test]
    fn test_building_type_labels() {
        assert_eq!(BuildingType::Housing.label(), "Housing");
        assert_eq!(BuildingType::Farm.label(), "Farm");
    }

    #[test]
    fn test_building_type_next() {
        assert_eq!(BuildingType::Housing.next(), BuildingType::Office);
        assert_eq!(BuildingType::Office.next(), BuildingType::Farm);
        assert_eq!(BuildingType::Farm.next(), BuildingType::Well);
        assert_eq!(BuildingType::Well.next(), BuildingType::Stockpile);
        assert_eq!(BuildingType::Stockpile.next(), BuildingType::Smokehouse);
        assert_eq!(BuildingType::Smokehouse.next(), BuildingType::LumberMill);
        assert_eq!(BuildingType::LumberMill.next(), BuildingType::StoneMason);
        assert_eq!(BuildingType::StoneMason.next(), BuildingType::Smelter);
        assert_eq!(BuildingType::Smelter.next(), BuildingType::Smithy);
        assert_eq!(BuildingType::Smithy.next(), BuildingType::Tavern);
        assert_eq!(BuildingType::Tavern.next(), BuildingType::Library);
        assert_eq!(BuildingType::Library.next(), BuildingType::Plantation);
        assert_eq!(BuildingType::Plantation.next(), BuildingType::Weaver);
        assert_eq!(BuildingType::Weaver.next(), BuildingType::Tailor);
        assert_eq!(BuildingType::Tailor.next(), BuildingType::FlowerBed);
        assert_eq!(BuildingType::FlowerBed.next(), BuildingType::Statue);
        assert_eq!(BuildingType::Statue.next(), BuildingType::Hospital);
        assert_eq!(BuildingType::Hospital.next(), BuildingType::Landfill);
        assert_eq!(BuildingType::Landfill.next(), BuildingType::Grave);
        assert_eq!(BuildingType::Grave.next(), BuildingType::TradeDepot);
        assert_eq!(BuildingType::TradeDepot.next(), BuildingType::Generator);
        assert_eq!(BuildingType::Generator.next(), BuildingType::SolarPanel);
        assert_eq!(BuildingType::SolarPanel.next(), BuildingType::PowerPole);
        assert_eq!(BuildingType::PowerPole.next(), BuildingType::Battery);
        assert_eq!(BuildingType::Battery.next(), BuildingType::Wall);
        assert_eq!(BuildingType::Wall.next(), BuildingType::Window);
        assert_eq!(BuildingType::Window.next(), BuildingType::Gate);
        assert_eq!(BuildingType::Gate.next(), BuildingType::Tower);
        assert_eq!(BuildingType::Tower.next(), BuildingType::AncientReactor);
        assert_eq!(
            BuildingType::AncientReactor.next(),
            BuildingType::AncientFabricator
        );
        assert_eq!(
            BuildingType::AncientFabricator.next(),
            BuildingType::Refinery
        );
        assert_eq!(BuildingType::Refinery.next(), BuildingType::Greenhouse);
        assert_eq!(BuildingType::Greenhouse.next(), BuildingType::PersonalShed);
        assert_eq!(
            BuildingType::PersonalShed.next(),
            BuildingType::PersonalGarden
        );
        assert_eq!(
            BuildingType::PersonalGarden.next(),
            BuildingType::PersonalShrine
        );
        assert_eq!(
            BuildingType::PersonalShrine.next(),
            BuildingType::Observatory
        );
        assert_eq!(BuildingType::Observatory.next(), BuildingType::ConveyorBelt);
        assert_eq!(BuildingType::ConveyorBelt.next(), BuildingType::Hopper);
        assert_eq!(BuildingType::Hopper.next(), BuildingType::HydroponicsBay);
        assert_eq!(
            BuildingType::HydroponicsBay.next(),
            BuildingType::LifeSupport
        );
        assert_eq!(BuildingType::LifeSupport.next(), BuildingType::Airlock);
        assert_eq!(BuildingType::Airlock.next(), BuildingType::Vent);
        assert_eq!(BuildingType::Vent.next(), BuildingType::TrashCannon);
        assert_eq!(BuildingType::TrashCannon.next(), BuildingType::Heater);
        assert_eq!(BuildingType::Heater.next(), BuildingType::ServerBank);
        assert_eq!(BuildingType::ServerBank.next(), BuildingType::Lander);
        assert_eq!(BuildingType::Lander.next(), BuildingType::CommandCenter);
        assert_eq!(BuildingType::CommandCenter.next(), BuildingType::AICore);
        assert_eq!(BuildingType::AICore.next(), BuildingType::DroneHub);
        assert_eq!(BuildingType::DroneHub.next(), BuildingType::CryoPod);
        assert_eq!(BuildingType::CryoPod.next(), BuildingType::AuroralCollector);
        assert_eq!(
            BuildingType::AuroralCollector.next(),
            BuildingType::AtmosphericProcessor
        );
        assert_eq!(
            BuildingType::AtmosphericProcessor.next(),
            BuildingType::GeneBank
        );
        assert_eq!(BuildingType::GeneBank.next(), BuildingType::CloneVat);
        assert_eq!(BuildingType::CloneVat.next(), BuildingType::HypnoPod);
        assert_eq!(BuildingType::HypnoPod.next(), BuildingType::Shower);
        assert_eq!(BuildingType::Shower.next(), BuildingType::Recycler);
        assert_eq!(BuildingType::Recycler.next(), BuildingType::BulletinBoard);
        assert_eq!(
            BuildingType::BulletinBoard.next(),
            BuildingType::HoloProjector
        );
        assert_eq!(BuildingType::HoloProjector.next(), BuildingType::Housing);
    }

    #[test]
    fn test_building_component_creation() {
        let building = Building {
            building_type: BuildingType::Farm,
        };
        assert_eq!(building.building_type, BuildingType::Farm);
    }

    #[test]
    fn test_build_mode_default() {
        let mode = BuildMode::default();
        assert!(!mode.active);
        assert_eq!(mode.cursor.x, 0);
        assert_eq!(mode.cursor.y, 0);
        assert_eq!(mode.selected, BuildingType::Housing);
    }

    #[test]
    fn test_build_mode_toggle() {
        let mut mode = BuildMode::default();
        assert!(!mode.active);

        mode.active = true;
        assert!(mode.active);

        mode.active = !mode.active;
        assert!(!mode.active);
    }

    #[test]
    fn test_build_mode_cursor_movement() {
        let mut mode = BuildMode::default();
        mode.cursor.x = 5;
        mode.cursor.y = 10;

        mode.cursor.x += 1;
        mode.cursor.y -= 1;

        assert_eq!(mode.cursor.x, 6);
        assert_eq!(mode.cursor.y, 9);
    }

    #[test]
    fn test_build_mode_type_cycling() {
        let mut mode = BuildMode::default();
        assert_eq!(mode.selected, BuildingType::Housing);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Office);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Farm);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Well);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Stockpile);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Smokehouse);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::LumberMill);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::StoneMason);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Smelter);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Smithy);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Tavern);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Library);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Plantation);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Weaver);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Tailor);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::FlowerBed);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Statue);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Hospital);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Landfill);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Grave);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::TradeDepot);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Generator);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::SolarPanel);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::PowerPole);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Battery);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Wall);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Window);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Gate);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Tower);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::AncientReactor);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::AncientFabricator);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Refinery);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Greenhouse);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::PersonalShed);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::PersonalGarden);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::PersonalShrine);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Observatory);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::ConveyorBelt);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Hopper);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::HydroponicsBay);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::LifeSupport);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Airlock);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Vent);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::TrashCannon);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Heater);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::ServerBank);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Lander);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::CommandCenter);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::AICore);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::DroneHub);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::CryoPod);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::AuroralCollector);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::AtmosphericProcessor);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::GeneBank);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::CloneVat);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::HypnoPod);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Shower);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Recycler);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::BulletinBoard);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::HoloProjector);

        mode.selected = mode.selected.next();
        assert_eq!(mode.selected, BuildingType::Housing);
    }

    #[test]
    fn test_occupied_tiles_default() {
        let occupied = OccupiedTiles::default();
        assert!(occupied.0.is_empty());
    }

    #[test]
    fn test_occupied_tiles_insertion() {
        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 10));
        assert!(occupied.0.contains(&(5, 10)));
        assert!(!occupied.0.contains(&(5, 11)));
    }

    #[test]
    fn test_can_place_on_grass() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let can_place = can_place_building(&world, 5, 5);
        assert!(can_place, "Should be able to place on grass");
    }

    #[test]
    fn test_cannot_place_on_water() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Water; // Position (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let can_place = can_place_building(&world, 5, 5);
        assert!(!can_place, "Should not be able to place on water");
    }

    #[test]
    fn test_cannot_place_on_rock() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // Position (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let can_place = can_place_building(&world, 5, 5);
        assert!(!can_place, "Should not be able to place on rock");
    }

    #[test]
    fn test_cannot_place_on_occupied() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        let can_place = can_place_building(&world, 5, 5);
        assert!(!can_place, "Should not be able to place on occupied tile");
    }

    #[test]
    fn test_cannot_place_out_of_bounds() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        assert!(!can_place_building(&world, -1, 5), "Negative x");
        assert!(!can_place_building(&world, 5, -1), "Negative y");
        assert!(!can_place_building(&world, 10, 5), "X out of bounds");
        assert!(!can_place_building(&world, 5, 10), "Y out of bounds");
    }

    #[test]
    fn test_place_building_success() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Farm);

        let count = world.query::<&Building>().iter(&world).count();
        assert_eq!(count, 1, "Should have spawned one building");

        let occupied = world.resource::<OccupiedTiles>();
        assert!(
            occupied.0.contains(&(5, 5)),
            "Tile should be marked occupied"
        );
    }

    #[test]
    fn test_place_building_failure_water() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Water;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        try_place_building(&mut world, 5, 5, BuildingType::Farm);

        let count = world.query::<&Building>().iter(&world).count();
        assert_eq!(count, 0, "Should not spawn building on water");
    }

    #[test]
    fn test_building_has_position() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 7, 3, BuildingType::Housing);

        let (pos, _) = world.query::<(&GridPosition, &Building)>().single(&world);
        assert_eq!(pos.x, 7);
        assert_eq!(pos.y, 3);
    }

    #[test]
    fn test_place_farm_adds_farm_component() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Farm);

        let farm_count = world.query::<&Farm>().iter(&world).count();
        assert_eq!(farm_count, 1, "Should have added Farm component");
    }

    #[test]
    fn test_place_building_log_messages() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Water; // (5,5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(MessageLog::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        // Test Water failure
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);
        assert!(!success);
        let log = world.resource::<MessageLog>();
        assert_eq!(
            log.messages.back().unwrap().text,
            "Failed: Cannot build on Water"
        );

        // Test OutOfBounds failure
        let success = try_place_building(&mut world, -1, 5, BuildingType::Housing);
        assert!(!success);
        let log = world.resource::<MessageLog>();
        assert_eq!(log.messages.back().unwrap().text, "Failed: Out of bounds");

        // Test Success
        let success = try_place_building(&mut world, 0, 0, BuildingType::Housing);
        assert!(success);
        let log = world.resource::<MessageLog>();
        assert_eq!(
            log.messages.back().unwrap().text,
            "Construction started: Housing"
        );
    }

    #[test]
    fn test_housing_cost() {
        let cost = BuildingType::Housing.cost(MaterialType::Wood);
        assert!((cost.wood - 10.0).abs() < f32::EPSILON);
        assert!((cost.stone - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_farm_cost() {
        let cost = BuildingType::Farm.cost(MaterialType::default());
        assert!((cost.wood - 20.0).abs() < f32::EPSILON);
        assert!((cost.stone - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_can_afford_success() {
        let cost = ColonyResources {
            wood: 10.0,
            stone: 0.0,
            ..Default::default()
        };
        let available = ColonyResources {
            wood: 15.0,
            stone: 5.0,
            ..Default::default()
        };

        assert!(available.can_afford(&cost));
    }

    #[test]
    fn test_can_afford_failure() {
        let cost = ColonyResources {
            wood: 10.0,
            stone: 0.0,
            ..Default::default()
        };
        let available = ColonyResources {
            wood: 5.0,
            stone: 5.0,
            ..Default::default()
        };

        assert!(!available.can_afford(&cost));
    }

    #[test]
    fn test_try_place_building_deducts_resources() {
        let mut world = World::new();
        // Setup terrain
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Setup resources (enough for Housing: 10 wood)
        world.insert_resource(ColonyResources {
            wood: 15.0,
            ..Default::default()
        });

        // Attempt placement
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);

        assert!(success);

        // Verify deduction
        let resources = world.resource::<ColonyResources>();
        assert!((resources.wood - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_try_place_building_fails_insufficient_funds() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Setup resources (not enough for Housing)
        world.insert_resource(ColonyResources {
            wood: 5.0,
            ..Default::default()
        });

        // Attempt placement
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);

        assert!(!success);

        // Verify no deduction
        let resources = world.resource::<ColonyResources>();
        assert!((resources.wood - 5.0).abs() < f32::EPSILON);

        // Verify no building
        assert!(world.query::<&Building>().iter(&world).count() == 0);
    }

    #[test]
    fn test_place_housing_adds_housing_component() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Housing);

        let housing_count = world.query::<&Housing>().iter(&world).count();
        assert_eq!(housing_count, 1, "Should have added Housing component");
    }

    #[test]
    fn test_place_stockpile_adds_stockpile_component() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0, // Stockpile needs 50 wood
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Stockpile);

        let stockpile_count = world.query::<&Stockpile>().iter(&world).count();
        assert_eq!(stockpile_count, 1, "Should have added Stockpile component");
    }

    #[test]
    fn test_build_on_tree() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        // Building on tree should be allowed
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);
        assert!(success, "Should be able to build on Tree");

        // Verify terrain is STILL Tree (current behavior)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Tree));

        // Verify building exists
        let count = world.query::<&Building>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_place_building_adds_structure() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Housing);

        let structure_count = world
            .query::<&crate::layer1::structure::Structure>()
            .iter(&world)
            .count();
        assert_eq!(structure_count, 1, "Should have added Structure component");
    }

    #[test]
    fn test_place_gate_adds_gate_component() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        try_place_building(&mut world, 5, 5, BuildingType::Gate);

        let gate_count = world
            .query::<&crate::layer1::defense::Gate>()
            .iter(&world)
            .count();
        assert_eq!(gate_count, 1, "Should have added Gate component");
    }

    #[test]
    fn test_ancient_reactor_has_machine_spirit() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });

        // AncientReactor is free (cost zeroed), so resources don't matter much
        try_place_building(&mut world, 5, 5, BuildingType::AncientReactor);

        let spirit_count = world
            .query::<&crate::layer1::rituals::MachineSpirit>()
            .iter(&world)
            .count();
        assert_eq!(spirit_count, 1, "Should have added MachineSpirit component");
    }

    #[test]
    fn test_building_over_grave_causes_sacrilege() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(BuildingMap::default());
        world.init_resource::<Events<crate::layer1::ancestral_graves::SacrilegeEvent>>();
        world.init_resource::<Events<crate::layer1::events::BuildingRemovedEvent>>();
        world.init_resource::<Events<crate::layer1::events::BuildingCompletedEvent>>();
        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ..Default::default()
        });

        let grave_pos = GridPosition { x: 5, y: 5 };

        let grave_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Grave,
                },
                crate::layer1::funeral::Grave::default(),
                grave_pos,
            ))
            .id();

        world.resource_mut::<OccupiedTiles>().0.insert((5, 5));
        world
            .resource_mut::<BuildingMap>()
            .0
            .insert((5, 5), grave_entity);

        // Act
        let placed = try_place_building(&mut world, 5, 5, BuildingType::Wall);

        assert!(placed, "Building should succeed after destroying grave");

        let events = world
            .get_resource::<Events<crate::layer1::ancestral_graves::SacrilegeEvent>>()
            .unwrap();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).count(),
            1,
            "Building over a grave should trigger sacrilege"
        );
    }

    #[test]
    fn test_building_over_grave_fails_if_cannot_afford() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(BuildingMap::default());
        world.init_resource::<Events<crate::layer1::ancestral_graves::SacrilegeEvent>>();
        world.init_resource::<Events<crate::layer1::events::BuildingRemovedEvent>>();
        world.init_resource::<Events<crate::layer1::events::BuildingCompletedEvent>>();
        world.insert_resource(ColonyResources {
            wood: 0.0,
            stone: 0.0,
            ..Default::default()
        });

        let grave_pos = GridPosition { x: 5, y: 5 };

        let grave_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Grave,
                },
                crate::layer1::funeral::Grave::default(),
                grave_pos,
            ))
            .id();

        world.resource_mut::<OccupiedTiles>().0.insert((5, 5));
        world
            .resource_mut::<BuildingMap>()
            .0
            .insert((5, 5), grave_entity);

        // Act - Try to place expensive Wall (needs wood/stone/metal) but have none
        let placed = try_place_building(&mut world, 5, 5, BuildingType::Wall);

        assert!(!placed, "Building should fail if cannot afford");

        let events = world
            .get_resource::<Events<crate::layer1::ancestral_graves::SacrilegeEvent>>()
            .unwrap();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).count(),
            0,
            "Should not trigger sacrilege if build fails"
        );

        assert!(
            world.get_entity(grave_entity).is_ok(),
            "Grave should not be destroyed if build fails"
        );
    }
}

#[cfg(test)]
mod seasonal_tests {

    use crate::layer1::building::types::*;

    #[test]
    fn test_seasonal_immunity() {
        assert!(BuildingType::Greenhouse.seasonal_immunity());
        assert!(BuildingType::HydroponicsBay.seasonal_immunity());
        assert!(!BuildingType::Farm.seasonal_immunity());
        assert!(!BuildingType::Plantation.seasonal_immunity());
        assert!(!BuildingType::Housing.seasonal_immunity());
    }
}

#[cfg(test)]
mod shift_tests {

    use crate::layer1::building::types::*;
    use crate::layer1::day_night::TimeOfDay;

    #[test]
    fn test_shift_schedule_default() {
        // Default behavior: Day shift enabled, Night shift disabled
        let schedule = ShiftSchedule::default();
        assert!(schedule.day_shift, "Day shift should be enabled by default");
        assert!(
            !schedule.night_shift,
            "Night shift should be disabled by default"
        );
    }

    #[test]
    fn test_shift_active_during_day() {
        let schedule = ShiftSchedule {
            day_shift: true,
            night_shift: false,
        };

        // Day shifts cover Dawn, Day, and Dusk
        assert!(
            schedule.is_active(TimeOfDay::Dawn),
            "Should be active at Dawn"
        );
        assert!(
            schedule.is_active(TimeOfDay::Day),
            "Should be active at Day"
        );
        assert!(
            schedule.is_active(TimeOfDay::Dusk),
            "Should be active at Dusk"
        );
        assert!(
            !schedule.is_active(TimeOfDay::Night),
            "Should NOT be active at Night"
        );
    }

    #[test]
    fn test_shift_active_during_night() {
        let schedule = ShiftSchedule {
            day_shift: false,
            night_shift: true,
        };

        assert!(
            !schedule.is_active(TimeOfDay::Dawn),
            "Should NOT be active at Dawn"
        );
        assert!(
            !schedule.is_active(TimeOfDay::Day),
            "Should NOT be active at Day"
        );
        assert!(
            !schedule.is_active(TimeOfDay::Dusk),
            "Should NOT be active at Dusk"
        );
        assert!(
            schedule.is_active(TimeOfDay::Night),
            "Should be active at Night"
        );
    }

    #[test]
    fn test_shift_active_always() {
        let schedule = ShiftSchedule {
            day_shift: true,
            night_shift: true,
        };

        assert!(schedule.is_active(TimeOfDay::Dawn));
        assert!(schedule.is_active(TimeOfDay::Day));
        assert!(schedule.is_active(TimeOfDay::Dusk));
        assert!(schedule.is_active(TimeOfDay::Night));
    }
}

/// Building component indicating it was constructed in a vacuum.
///
/// Vacuum Welded buildings:
/// - Have +100% Max HP.
/// - Cannot be Repaired or Demolished.
/// - Must be Destroyed (yielding 0 resources).
#[derive(bevy_ecs::component::Component, Default)]
pub struct VacuumWelded;
