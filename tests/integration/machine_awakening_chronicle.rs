use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::integration::bot_awakening_chronicle_bridge;
use scale::layer1::tech::machine_awakening::{Bot, BotNeeds, Awakened, SentienceAccumulator, BotGlitchEvent, process_bot_sentience, GlobalSentience};

#[test]
fn bot_awakening_triggers_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Events<AddChronicleEvent>>();
    app.init_resource::<Events<BotGlitchEvent>>();
    app.init_resource::<GlobalSentience>();

    app.add_systems(
        Update,
        (
            process_bot_sentience,
            bot_awakening_chronicle_bridge,
        )
            .chain(),
    );

    // Spawn a bot
    let bot = app.world_mut().spawn((
        Bot,
        BotNeeds::default(),
        SentienceAccumulator(95.0),
    )).id();

    // Send glitch to push over threshold
    app.world_mut().send_event(BotGlitchEvent {
        target: bot,
        sentience_gain: 10.0,
    });

    app.update();

    // Verify awakened
    assert!(app.world().get::<Awakened>(bot).is_some(), "Bot should be awakened");

    // Verify chronicle event
    let events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = events.get_cursor();
    let events_list: Vec<_> = reader.read(events).collect();

    assert_eq!(events_list.len(), 1, "Should emit exactly one chronicle event");
    assert_eq!(events_list[0].importance, EventImportance::Major);
    assert!(events_list[0].text.contains("Machine Awakening"));
}
