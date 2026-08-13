use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::magnetic_reversal_chronicle_bridge;
use scale::layer1::physics::magnetic_reversal::PoleFlipEvent;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnetic_reversal_chronicle_bridge() {
        let mut app = App::new();
        app.add_event::<PoleFlipEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, magnetic_reversal_chronicle_bridge);

        app.world_mut().send_event(PoleFlipEvent);
        app.update();

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut cursor = events.get_cursor();
        let mut found = false;
        for ev in cursor.read(events) {
            if ev.text.contains("magnetic poles flipped") && ev.importance == EventImportance::Major
            {
                found = true;
            }
        }
        assert!(found, "Chronicle event should be emitted on PoleFlipEvent");
    }
}
