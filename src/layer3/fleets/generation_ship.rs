//! Generation Ships
//!
//! Simulation of massive, slow arks carrying populations across centuries.
//! Whole societies are born, live, and die aboard these vessels before reaching their destination.

use crate::layer1::culture::{Alignment, Culture};
use bevy::prelude::*;

#[derive(Component)]
pub struct GenerationShip;

#[derive(Component, Default)]
pub struct TransitConditions {
    pub food_scarcity: bool,
    pub mechanical_failures: u32,
}

#[derive(Event)]
pub struct ColonyFoundedEvent {
    pub source_ship: Entity,
    pub target_planet: Entity,
}

#[derive(Component)]
pub struct TransitDrift {
    pub hostility_score: f32,
}

pub fn simulate_transit_drift_system(
    mut ships: Query<(&TransitConditions, &mut TransitDrift), With<GenerationShip>>,
) {
    for (conditions, mut drift) in ships.iter_mut() {
        if conditions.food_scarcity {
            drift.hostility_score += 10.0;
        }
        drift.hostility_score += conditions.mechanical_failures as f32 * 5.0;
    }
}

pub fn apply_drift_on_foundation_system(
    mut commands: Commands,
    mut events: EventReader<ColonyFoundedEvent>,
    ships: Query<&TransitDrift, With<GenerationShip>>,
) {
    for event in events.read() {
        if let Ok(drift) = ships.get(event.source_ship) {
            let alignment = if drift.hostility_score > 50.0 {
                Alignment::Hostile
            } else {
                Alignment::Peaceful
            };

            commands
                .entity(event.target_planet)
                .insert(Culture { alignment });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::culture::{Alignment, Culture};

    #[test]
    fn test_transit_drift_accumulation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, simulate_transit_drift_system);

        let ship_id = app
            .world_mut()
            .spawn((
                GenerationShip,
                TransitConditions {
                    food_scarcity: true,
                    mechanical_failures: 2,
                },
                TransitDrift {
                    hostility_score: 0.0,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert
        // Hostility score should increase due to poor conditions
        let drift = app.world().get::<TransitDrift>(ship_id).unwrap();
        assert!(
            drift.hostility_score > 0.0,
            "Drift hostility should increase under poor conditions."
        );
    }

    #[test]
    fn test_colony_foundation_with_drift() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ColonyFoundedEvent>();
        app.add_systems(Update, apply_drift_on_foundation_system);

        let ship_id = app
            .world_mut()
            .spawn((
                GenerationShip,
                TransitDrift {
                    hostility_score: 100.0,
                }, // High hostility
            ))
            .id();

        let target_planet_id = app.world_mut().spawn_empty().id();

        // Act
        app.world_mut().send_event(ColonyFoundedEvent {
            source_ship: ship_id,
            target_planet: target_planet_id,
        });
        app.update();

        // Assert
        // Target planet should now have a hostile culture
        let culture = app.world().get::<Culture>(target_planet_id).unwrap();
        assert_eq!(
            culture.alignment,
            Alignment::Hostile,
            "High drift should result in hostile alignment."
        );
    }
}
