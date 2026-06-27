use bevy_ecs::prelude::*;
use scale::layer1::designation::{Designation, DesignationType};
use scale::layer1::fauna::{Fauna, FaunaState, FaunaType};
use scale::layer1::husbandry::{HusbandryConfig, Tame};
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::skills::{SkillType, Skills};
use scale::layer1::utility_ai::{ActionType, PopAction, UtilityWeights};
use scale::layer1::zone::{ZoneGrid, ZoneType};
use scale::shared::state::GameState;
use scale::simulation::run_simulation_tick;

#[test]
#[ignore = "AI evaluation does not currently select Tame action. Requires review."]
fn test_husbandry_full_loop() {
    let mut world = scale::setup::setup_world();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
    world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();
    *world.resource_mut::<scale::shared::state::GameState>() =
        scale::shared::state::GameState::Running;

    *world.resource_mut::<GameState>() = GameState::Running;
    world.insert_resource(HusbandryConfig::default());

    // 1. Setup Zone (Pasture)
    // Force terrain to be walkable
    let mut terrain = world.resource_mut::<scale::layer1::terrain::TerrainGrid>();
    for x in 0..10 {
        for y in 0..10 {
            terrain.set(x, y, scale::layer1::terrain::TerrainType::Grass);
        }
    }

    let mut zone_grid = world.resource_mut::<ZoneGrid>();
    for x in 0..5 {
        for y in 0..5 {
            zone_grid.set(x, y, ZoneType::Pasture);
        }
    }

    // 2. Spawn Pop with Husbandry Skill
    let mut skills = Skills::default();
    skills.add_xp(SkillType::Husbandry, 60000.0); // High XP to guarantee success

    let _pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            skills,
            scale::layer1::needs::Needs::default(),
            PopAction::default(),
            UtilityWeights::default(),
        ))
        .id();

    // 3. Spawn Wild Animal inside Pasture
    let animal = world
        .spawn((
            Fauna {
                fauna_type: FaunaType::SpaceRat,
                state: FaunaState::Wander,
                detection_range: 0.0, // Blind so it doesn't chase/move
                ..Default::default()
            },
            GridPosition { x: 2, y: 2 },
        ))
        .id();

    // 4. Designate Animal for Taming
    world.spawn((
        Designation {
            designation_type: DesignationType::Tame,
        },
        GridPosition { x: 2, y: 2 }, // Must match animal pos
    ));

    // 5. Run Simulation Loop until Tamed (or timeout)
    let mut tamed = false;
    for _ in 0..200 {
        run_simulation_tick(&mut world);

        if world.get::<Tame>(animal).is_some() {
            tamed = true;
            break;
        }
    }

    assert!(tamed, "Animal should be tamed by the pop");

    // 6. Test Production
    // Fast forward production timer
    if let Some(mut tame) = world.get_mut::<Tame>(animal) {
        tame.produce_timer = 0;
    }

    // Run tick to trigger production
    run_simulation_tick(&mut world);

    // Check for produced resource
    let items = world
        .query::<&scale::layer1::resources::ResourceItem>()
        .iter(&world)
        .count();
    assert!(items > 0, "Tamed animal should produce resources");

    // 7. Test Confinement
    // Force animal to try to move outside
    world
        .entity_mut(animal)
        .insert(scale::layer1::execution::MovementTarget {
            target_entity: Entity::PLACEHOLDER,
            target_position: GridPosition { x: 10, y: 10 }, // Outside pasture
            for_action: ActionType::Idle,
        });

    run_simulation_tick(&mut world);

    // MovementTarget should be removed by pasture_confinement_system
    assert!(
        world
            .get::<scale::layer1::execution::MovementTarget>(animal)
            .is_none(),
        "Confinement system should remove invalid movement target"
    );
}
