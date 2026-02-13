use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::social::{Relationships, SocialBuff};
use scale::setup::setup_world;

#[test]
fn test_proximity_social_buff_applied() {
    // 1. Setup World
    let mut world = setup_world();

    // 2. Spawn Friend Pops (High Affinity)
    // Pop 1
    let pop1 = world
        .spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            Relationships::default(),
        ))
        .id();

    // Pop 2 (Close to Pop 1)
    let pop2 = world
        .spawn((
            Pop,
            GridPosition { x: 10, y: 11 },
            Relationships::default(),
        ))
        .id();

    // Set mutual high affinity (> 20.0)
    world.entity_mut(pop1).get_mut::<Relationships>().unwrap().set_affinity(pop2, 50.0);
    world.entity_mut(pop2).get_mut::<Relationships>().unwrap().set_affinity(pop1, 50.0);

    // 3. Run Simulation Tick
    scale::simulation::run_simulation_tick(&mut world);

    // 4. Assert SocialBuff on Pop 2
    // Pop 2 is near Friend (Pop 1) -> Should have positive buff
    let buff2 = world.get::<SocialBuff>(pop2);
    assert!(buff2.is_some(), "Pop 2 should have SocialBuff from friend nearby");
    assert!(buff2.unwrap().value > 0.0, "Pop 2 should have positive buff");
}

#[test]
fn test_proximity_social_debuff_applied() {
    let mut world = setup_world();

    let pop1 = world
        .spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            Relationships::default(),
        ))
        .id();

    let pop2 = world
        .spawn((
            Pop,
            GridPosition { x: 10, y: 11 },
            Relationships::default(),
        ))
        .id();

    // Set enemy affinity (< -20.0)
    world.entity_mut(pop1).get_mut::<Relationships>().unwrap().set_affinity(pop2, -50.0);

    scale::simulation::run_simulation_tick(&mut world);

    // Pop 1 sees Pop 2 as enemy -> Negative buff
    let buff1 = world.get::<SocialBuff>(pop1);
    assert!(buff1.is_some(), "Pop 1 should have SocialBuff from enemy nearby");
    assert!(buff1.unwrap().value < 0.0, "Pop 1 should have negative buff");
}
