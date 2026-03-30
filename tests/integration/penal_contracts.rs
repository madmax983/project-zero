use bevy_app::{App, Update};
use scale::layer1::justice::Inmate;
use scale::layer1::pop::Pop;
use scale::layer2::trade::penal_contracts::PrisonerOf;
use scale::layer1::integration::sync_prisoner_to_inmate_system;

#[test]
fn test_sync_prisoner_to_inmate() {
    let mut app = App::new();

    app.add_systems(Update, sync_prisoner_to_inmate_system);

    let faction = app.world_mut().spawn_empty().id();
    let prisoner = app.world_mut().spawn((Pop, PrisonerOf(faction))).id();

    app.update();

    let inmate = app.world().get::<Inmate>(prisoner);
    assert!(
        inmate.is_some(),
        "Pop with PrisonerOf should be assigned an Inmate component"
    );
    assert!(
        inmate.unwrap().sentence_ticks > 0,
        "Inmate sentence_ticks should be initialized > 0"
    );
}

#[test]
fn test_sync_prisoner_cleanup() {
    let mut app = App::new();

    app.add_systems(Update, sync_prisoner_to_inmate_system);

    let faction = app.world_mut().spawn_empty().id();
    let prisoner = app.world_mut().spawn((Pop, PrisonerOf(faction))).id();

    app.update();

    let inmate = app.world().get::<Inmate>(prisoner);
    assert!(
        inmate.is_some(),
        "Pop with PrisonerOf should be assigned an Inmate component"
    );

    // Simulate contract ending/expiring
    app.world_mut().entity_mut(prisoner).remove::<PrisonerOf>();

    app.update();

    let inmate_after = app.world().get::<Inmate>(prisoner);
    assert!(
        inmate_after.is_none(),
        "Pop should lose Inmate component when PrisonerOf is removed"
    );
}
