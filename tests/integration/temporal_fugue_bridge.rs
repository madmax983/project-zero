use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::day_night::DayNightCycle;
use scale::layer1::execution::general_work::calculate_work_amount;
use scale::layer1::map::GridPosition;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;
use scale::layer1::resources::ColonyResources;
use scale::layer1::skills::{SkillType, Skills};
use scale::layer1::taboo::TabooState;
use scale::layer1::temporal_fugue::{
    evaluate_fugue_state_system, fugue_completion_system, TemporalFugue,
};
use scale::layer1::utility_types::{ActionType, PopAction, UtilityConfig};
use scale::layer1::{Designation, DesignationType};
use scale::shared::time::SimulationTime;

#[test]
fn test_highly_skilled_pop_enters_temporal_fugue() {
    let mut app = bevy_app::App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, evaluate_fugue_state_system);

    let mut skills = Skills::default();
    skills.add_xp(SkillType::Mining, 900_000.0); // Enough for level 90+

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            skills,
            PopAction {
                current: ActionType::Work,
                current_utility: 1.0,
                ticks_committed: 10,
            },
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<TemporalFugue>(pop).is_some(),
        "Highly skilled Pops on work jobs should enter Temporal Fugue."
    );
}

#[test]
fn test_temporal_fugue_multiplies_work_speed() {
    let mut app = bevy_app::App::new();

    let mut skills = Skills::default();
    skills.add_xp(SkillType::Mining, 900_000.0); // Enough for level 90+

    let pop = app.world_mut().spawn((Pop, skills, TemporalFugue)).id();

    let work_amount =
        calculate_work_amount(app.world(), pop, DesignationType::Mine, None, 0.5, 1.0, 1.0);

    // Default WORK_PER_TICK is 1.0. The fugue modifier is 3.0. Morale efficiency at 0.5 is ~0.5.
    // So it should be around 1.5. A normal pop would be around 0.5.
    // The key is that `temporal_fugue_modifier` of 3.0 is applied.
    assert!(
        work_amount > 1.0,
        "Fugue state should significantly increase work amount."
    );
}

#[test]
fn test_pop_in_fugue_ignores_needs_until_task_completion() {
    scale::setup::init_task_pools();
    let mut app = bevy_app::App::new();
    app.add_event::<AddChronicleEvent>();
    app.init_resource::<UtilityConfig>();
    app.init_resource::<SimulationTime>();
    app.init_resource::<ColonyResources>();
    app.init_resource::<DayNightCycle>();
    app.init_resource::<TabooState>();
    app.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10));

    // Add the AI system
    app.add_systems(
        bevy_app::Update,
        scale::layer1::mind::evaluate_actions_system,
    );

    let mut skills = Skills::default();
    skills.add_xp(SkillType::Mining, 900_000.0); // Enough for level 90+

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs {
                hunger: 0.0,
                rest: 0.0,
                leisure: 0.0,
                hygiene: 0.0,
            }, // Starving, should normally eat
            scale::layer1::utility_types::UtilityWeights::default(),
            PopAction {
                current: ActionType::Work,
                current_utility: 1.0,
                ticks_committed: 10,
            },
            TemporalFugue,
            skills,
        ))
        .id();

    // Spawn a work designation so the pop has something to do and doesn't fall through
    app.world_mut().spawn((
        GridPosition { x: 1, y: 0 },
        Designation {
            designation_type: DesignationType::Mine,
        },
    ));

    // With 0 hunger, normal utility AI would force SatisfyHunger.
    app.update();

    let action = app.world().get::<PopAction>(pop).unwrap();
    assert_eq!(
        action.current,
        ActionType::Work,
        "Pop in fugue should not drop tasks to fulfill needs."
    );

    // Complete the task (ActionType != Work)
    app.world_mut().get_mut::<PopAction>(pop).unwrap().current = ActionType::Idle;

    app.add_systems(bevy_app::Update, fugue_completion_system);
    app.update();

    assert!(
        app.world().get::<TemporalFugue>(pop).is_none(),
        "Fugue state should end when task completes."
    );
}
