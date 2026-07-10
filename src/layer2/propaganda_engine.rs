use crate::layer1::core::chronicle::AddChronicleEvent;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PropagandaEngine {
    pub active: bool,
    pub bluff_level: u32,
}

#[derive(Resource, Default)]
pub struct DiplomaticWeight {
    pub value: u32,
}

#[derive(Resource, Default)]
pub struct InspectorEvent {
    pub triggered: bool,
}

pub fn update_propaganda_system(
    mut engine: ResMut<PropagandaEngine>,
    mut weight: ResMut<DiplomaticWeight>,
    mut inspector: ResMut<InspectorEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    if engine.active {
        // Simple buff logic: Add the bluff level to the weight (assuming base weight is 100 for this test)
        weight.value = 100 + engine.bluff_level;

        if inspector.triggered {
            // Bluff called! Catastrophic loss of face.
            weight.value = 0;
            engine.active = false;
            engine.bluff_level = 0;
            inspector.triggered = false; // Reset event

            chronicle_events.send(AddChronicleEvent {
                text: "The Propaganda Engine's bluff was called by an inspector. Catastrophic loss of face!".to_string(),
                importance: crate::layer1::core::chronicle::EventImportance::Major,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, update_propaganda_system);
        app.insert_resource(PropagandaEngine {
            active: false,
            bluff_level: 0,
        });
        app.insert_resource(DiplomaticWeight { value: 100 });
        app.init_resource::<InspectorEvent>();
        if !app.world().contains_resource::<Events<AddChronicleEvent>>() {
            app.add_event::<AddChronicleEvent>();
        }
        app
    }

    #[test]
    fn test_propaganda_increases_weight() {
        let mut app = setup_app();

        // Activate propaganda
        app.world_mut().resource_mut::<PropagandaEngine>().active = true;
        app.world_mut()
            .resource_mut::<PropagandaEngine>()
            .bluff_level = 50;

        app.update();

        let weight = app.world().resource::<DiplomaticWeight>();
        assert!(
            weight.value > 100,
            "Diplomatic weight should increase when the propaganda engine is active."
        );
    }

    #[test]
    fn test_inspector_calls_bluff() {
        let mut app = setup_app();

        // Active bluff
        app.world_mut().resource_mut::<PropagandaEngine>().active = true;
        app.world_mut()
            .resource_mut::<PropagandaEngine>()
            .bluff_level = 100;

        app.update();

        // Trigger inspector
        app.world_mut().resource_mut::<InspectorEvent>().triggered = true;

        app.update();

        let weight = app.world().resource::<DiplomaticWeight>();
        let engine = app.world().resource::<PropagandaEngine>();
        assert_eq!(
            weight.value, 0,
            "Diplomatic weight should plummet to zero when a bluff is called."
        );
        assert!(!engine.active, "The bluff should end.");
    }
}
