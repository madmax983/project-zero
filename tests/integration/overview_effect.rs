use bevy::prelude::*;
use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::morale::Morale;
use scale::layer1::pop::Pop;
use scale::layer1::resources::ColonyResources;
use scale::layer1::skills::{SkillType, Skills};
use scale::layer1::traits::{Trait, Traits};
use scale::layer1::observatory::Observatory;
use scale::layer1::overview_effect::{ObserveEvent, overview_effect_system};
use scale::layer1::integration::observatory_overview_bridge_system;

#[test]
fn test_observatory_triggers_overview_effect() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<Events<ObserveEvent>>();
    app.insert_resource(ColonyResources::default());

    app.add_systems(Update, (
        observatory_overview_bridge_system,
        overview_effect_system,
    ).chain());

    let observatory = app.world_mut().spawn(Observatory::default()).id();

    let pop = app.world_mut().spawn((
        Pop,
        Morale::default(),
        Skills::default(),
        Traits(std::collections::HashSet::from([Trait::Optimist])),
        AssignedTo {
            entity: observatory,
            assignment_type: AssignmentType::ObservatoryWorker,
        },
    )).id();

    // Run the app a few times to ensure the bridge system triggers
    for _ in 0..500 {
        app.update();
    }

    let skills = app.world().get::<Skills>(pop).unwrap();
    assert!(skills.get_xp(SkillType::Crafting) > 0.0, "Pop should gain XP from Overview Effect");

    let morale = app.world().get::<Morale>(pop).unwrap();
    assert!(morale.modifiers.iter().any(|m| m.label == "Inspired"), "Optimist Pop should be Inspired");
}
