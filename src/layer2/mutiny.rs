use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
#[cfg(test)]
use crate::layer2::fleet::Fleet;
use crate::layer2::fleet::FleetFaction;
use bevy_ecs::prelude::*;

/// Tracks the morale of a fleet's crew.
#[derive(Component)]
pub struct CrewMorale {
    pub value: f32,
}

/// Indicates whether a fleet has active supply lines (e.g., food).
#[derive(Component)]
pub struct SupplyLines {
    pub food_supplied: bool,
}

/// Tag component added to fleets that have mutinied.
#[derive(Component)]
pub struct Mutinied;

/// System to decay fleet morale if supply lines are not met.
pub fn decay_fleet_morale(mut fleets: Query<(&mut CrewMorale, &SupplyLines)>) {
    for (mut morale, supply) in fleets.iter_mut() {
        if !supply.food_supplied {
            morale.value -= 10.0;
        }
    }
}

/// System to evaluate if a fleet should mutiny based on morale.
pub fn evaluate_fleet_mutiny(
    mut commands: Commands,
    mut fleets: Query<(Entity, &mut FleetFaction, &CrewMorale), Without<Mutinied>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for (entity, mut faction, morale) in fleets.iter_mut() {
        if morale.value <= 0.0 {
            *faction = FleetFaction::Pirate;
            commands.entity(entity).insert(Mutinied);
            chronicle_events.send(AddChronicleEvent {
                text: "A fleet has mutinied and turned to piracy!".to_string(),
                importance: EventImportance::Major,
            ..Default::default()});
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_loses_morale_when_unpaid_or_starving() {
        // Arrange
        let mut app = bevy_app::App::new();
        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Player,
                CrewMorale { value: 100.0 },
                SupplyLines {
                    food_supplied: false,
                },
            ))
            .id();

        // Act
        app.add_systems(bevy_app::Update, decay_fleet_morale);
        app.update();

        // Assert
        let morale = app.world().get::<CrewMorale>(fleet).unwrap();
        assert!(morale.value < 100.0, "Morale should decay without supplies");
    }

    #[test]
    fn test_fleet_mutinies_at_zero_morale() {
        // Arrange
        let mut app = bevy_app::App::new();
        app.init_resource::<Events<AddChronicleEvent>>();

        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Player,
                CrewMorale { value: 0.0 }, // Critically low
            ))
            .id();

        // Act
        app.add_systems(bevy_app::Update, evaluate_fleet_mutiny);
        app.update();

        // Assert
        let fleet_faction = app.world().get::<FleetFaction>(fleet).unwrap();
        assert_eq!(
            *fleet_faction,
            FleetFaction::Pirate,
            "Fleet should switch to pirate faction upon mutiny"
        );
        assert!(
            app.world().get::<Mutinied>(fleet).is_some(),
            "Fleet should be tagged as mutinied"
        );

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();
        assert_eq!(emitted.len(), 1, "Should emit a chronicle event on mutiny");
    }
}
