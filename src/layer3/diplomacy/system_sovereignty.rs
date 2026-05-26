//! System Sovereignty
//!
//! Details the mechanics of colonies declaring political independence.
//! This triggers complex diplomatic fallout and potential punitive expeditions from the former overlord.

use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Resource)]
pub struct ColonyStatus {
    pub is_sovereign: bool,
    pub overlord_id: Option<u32>,
}

#[derive(Resource, Default)]
pub struct FactionRelations {
    pub scores: HashMap<u32, i32>, // -100 (Hostile) to 100 (Allied)
}

#[derive(Event)]
pub struct DeclarationOfIndependenceEvent;

#[derive(Event)]
pub struct WarDeclarationEvent {
    pub target_id: u32,
}

pub fn process_sovereignty_declaration(
    mut events: EventReader<DeclarationOfIndependenceEvent>,
    mut war_events: EventWriter<WarDeclarationEvent>,
    mut status: ResMut<ColonyStatus>,
    mut relations: ResMut<FactionRelations>,
) {
    for _ in events.read() {
        if status.is_sovereign {
            continue; // Already sovereign
        }

        if let Some(overlord) = status.overlord_id {
            // Instant hostility
            relations.scores.insert(overlord, -100);
            war_events.send(WarDeclarationEvent {
                target_id: overlord,
            });
        }

        status.is_sovereign = true;
        status.overlord_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declaration_changes_colony_status() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyStatus {
            is_sovereign: false,
            overlord_id: Some(1),
        });
        app.insert_resource(FactionRelations::default());
        app.add_event::<DeclarationOfIndependenceEvent>();
        app.add_event::<WarDeclarationEvent>();
        app.add_systems(Update, process_sovereignty_declaration);

        // Act
        app.world_mut().send_event(DeclarationOfIndependenceEvent);
        app.update();

        // Assert
        let status = app.world().resource::<ColonyStatus>();
        assert!(status.is_sovereign);
        assert_eq!(status.overlord_id, None);
    }

    #[test]
    fn test_declaration_triggers_overlord_hostility_and_war_event() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyStatus {
            is_sovereign: false,
            overlord_id: Some(2),
        });
        app.add_event::<DeclarationOfIndependenceEvent>();
        app.add_event::<WarDeclarationEvent>();

        // Mock relations map: Faction 2 starts Neutral (0)
        let mut relations = FactionRelations::default();
        relations.scores.insert(2, 0);
        app.insert_resource(relations);

        app.add_systems(Update, process_sovereignty_declaration);

        // Act
        app.world_mut().send_event(DeclarationOfIndependenceEvent);
        app.update();

        // Assert: Overlord should now be deeply hostile (-100)
        let relations = app.world().resource::<FactionRelations>();
        assert_eq!(*relations.scores.get(&2).unwrap(), -100);

        let war_events = app.world().resource::<Events<WarDeclarationEvent>>();
        assert_eq!(war_events.get_cursor().read(war_events).count(), 1);
    }
}
