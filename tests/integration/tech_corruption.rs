#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::combat::{AttackProperties, CombatState};
    use scale::layer1::farm::{produce_food_system, Farm};
    use scale::layer1::fauna::{Fauna, FaunaType};
    use scale::layer1::health::Health;
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::refining::process_refining_system;
    use scale::layer1::resources::{ColonyResources, RefiningProgress, ResourceType};
    use scale::layer1::skills::Skills;
    use scale::layer1::tech::{Tech, TechState, TechStatus};
    use scale::layer1::turret::{turret_fire_system, Turret};
    use scale::layer1::utility_ai::{ActionType, PopAction};
    use scale::shared::time::SimulationTime;

    fn setup_world() -> World {
        scale::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(scale::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![scale::layer1::terrain::TerrainType::Grass; 100],
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(TechState::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(scale::layer1::map::ScreenShake::default());
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world.insert_resource(scale::shared::log::MessageLog::default());
        world.insert_resource(scale::layer1::seasons::SeasonState::default());
        world.insert_resource(scale::layer1::factions::Factions::default());
        world.init_resource::<Events<scale::layer1::eureka::EurekaEvent>>();

        // Needed for turret particle spawn
        world.insert_resource(scale::layer1::beauty::BeautyGrid::new(10, 10)); // For map updates if any

        world
    }

    #[test]
    fn smelter_stops_when_corrupted() {
        let mut world = setup_world();

        // Setup Resources
        world.insert_resource(ColonyResources {
            ore: 10.0,
            wood: 10.0,
            metal: 0.0,
            ..Default::default()
        });

        // Unlock Tech
        let mut tech_state = TechState {
            total_capacity: 100.0,
            ..Default::default()
        };
        tech_state.unlock(Tech::MetalWorking);
        world.insert_resource(tech_state);

        // Spawn Smelter
        let smelter = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                GridPosition { x: 5, y: 5 },
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
            ))
            .id();

        // Spawn Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Skills::default(),
            PopAction {
                current: ActionType::Refine,
                current_utility: 0.5,
                ticks_committed: 1,
            },
        ));

        // 1. Verify it works when Active
        process_refining_system(&mut world);
        let progress = world.get::<RefiningProgress>(smelter).unwrap();
        assert!(
            progress.current > 0.0,
            "Smelter should work when tech is active"
        );

        // Reset Progress
        world.entity_mut(smelter).insert(RefiningProgress {
            current: 0.0,
            max: 10.0,
        });

        // 2. Corrupt Tech
        let mut tech_state = world.resource_mut::<TechState>();
        tech_state
            .techs
            .insert(Tech::MetalWorking, TechStatus::Corrupted);

        // 3. Verify it stops working
        process_refining_system(&mut world);
        let progress = world.get::<RefiningProgress>(smelter).unwrap();
        assert_eq!(
            progress.current, 0.0,
            "Smelter should NOT work when tech is corrupted"
        );
    }

    #[test]
    fn hydroponics_stops_when_corrupted() {
        let mut world = setup_world();

        // Setup Resources (Give water!)
        world.resource_mut::<ColonyResources>().water = 100.0;

        // Unlock Tech
        let mut tech_state = TechState {
            total_capacity: 100.0,
            ..Default::default()
        };
        tech_state.unlock(Tech::Hydroponics);
        world.insert_resource(tech_state);

        // Spawn Hydroponics Bay (requires power, so we simulate power consumer active)
        let _farm = world
            .spawn((
                Farm::default(),
                Building {
                    building_type: BuildingType::HydroponicsBay,
                },
                GridPosition { x: 5, y: 5 },
                scale::layer1::energy::PowerConsumer {
                    demand: 5.0,
                    active: true, // Powered
                },
            ))
            .id();

        // Spawn Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        // 1. Verify it works when Active
        world.run_system_once(produce_food_system).unwrap();
        let res = world.resource::<ColonyResources>();
        assert!(
            res.food > 0.0,
            "Hydroponics should produce food when tech is active"
        );

        // Reset Food
        world.resource_mut::<ColonyResources>().food = 0.0;

        // 2. Corrupt Tech
        let mut tech_state = world.resource_mut::<TechState>();
        tech_state
            .techs
            .insert(Tech::Hydroponics, TechStatus::Corrupted);

        // 3. Verify it stops working
        world.run_system_once(produce_food_system).unwrap();
        let res = world.resource::<ColonyResources>();
        assert_eq!(
            res.food, 0.0,
            "Hydroponics should NOT produce food when tech is corrupted"
        );
    }

    #[test]
    fn trash_cannon_stops_when_corrupted() {
        let mut world = setup_world();

        // Unlock Tech
        let mut tech_state = TechState {
            total_capacity: 100.0,
            ..Default::default()
        };
        tech_state.unlock(Tech::Militia);
        world.insert_resource(tech_state);

        // Add Ammo
        world.resource_mut::<ColonyResources>().waste = 10.0;

        // Spawn Trash Cannon
        world.spawn((
            Building {
                building_type: BuildingType::TrashCannon,
            },
            Turret {
                attack: AttackProperties {
                    damage: 10.0,
                    range: 5.0,
                    cooldown: 0,
                    accuracy: 1.0,
                },
                ammo_cost: 1.0,
                ammo_type: ResourceType::Waste,
            },
            GridPosition { x: 0, y: 0 },
            CombatState::default(),
        ));

        // Spawn Enemy
        let enemy = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    ..Default::default()
                },
                GridPosition { x: 2, y: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // 1. Verify it fires when Active
        turret_fire_system(&mut world);
        let health = world.get::<Health>(enemy).unwrap();
        assert!(
            health.current < 100.0,
            "Turret should fire when tech is active"
        );

        // Reset Enemy Health
        world.entity_mut(enemy).insert(Health {
            current: 100.0,
            max: 100.0,
        });

        // Reset Cooldown
        for mut state in world.query::<&mut CombatState>().iter_mut(&mut world) {
            state.cooldown = 0;
        }

        // 2. Corrupt Tech
        let mut tech_state = world.resource_mut::<TechState>();
        tech_state
            .techs
            .insert(Tech::Militia, TechStatus::Corrupted);

        // 3. Verify it stops firing
        turret_fire_system(&mut world);
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(
            health.current, 100.0,
            "Turret should NOT fire when tech is corrupted"
        );
    }
}
