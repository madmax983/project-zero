use bevy_ecs::prelude::*;

#[derive(Clone, Copy)]
pub struct NegativeEvent {
    pub severity: f32,
    pub time: f32,
}

#[derive(Component, Default)]
pub struct NegativeEventHistory {
    pub events: Vec<NegativeEvent>,
}

#[derive(Component)]
pub struct Cursed;

#[derive(Component)]
pub struct Efficiency(pub f32);

pub fn evaluate_architectural_superstition(
    mut commands: Commands,
    query: Query<(Entity, &NegativeEventHistory), Without<Cursed>>,
) {
    for (entity, history) in query.iter() {
        // Simple threshold: 3 or more negative events
        if history.events.len() >= 3 {
            let total_severity: f32 = history.events.iter().map(|e| e.severity).sum();
            if total_severity >= 10.0 {
                commands.entity(entity).insert(Cursed);
            }
        }
    }
}

use crate::layer1::psychology::StressTracker;

pub fn apply_cursed_penalties(
    mut query: Query<(&mut Efficiency, &mut StressTracker), Added<Cursed>>,
) {
    for (mut efficiency, mut stress) in query.iter_mut() {
        // Significantly drop efficiency
        efficiency.0 *= 0.5;
        // Increase stress (lower mood equivalent)
        stress.accumulated_stress = (stress.accumulated_stress + 20.0).min(100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::Building;
    use crate::layer1::architecture::BuildingType;
    use crate::layer1::map::GridPosition;
    use bevy::prelude::*;

    #[test]
    fn test_building_becomes_cursed_after_multiple_negative_events() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_architectural_superstition);

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 10, y: 10 },
                NegativeEventHistory::default(),
            ))
            .id();

        // Arrange: Add multiple negative events near the building
        let mut history = app
            .world_mut()
            .get_mut::<NegativeEventHistory>(building)
            .unwrap();
        history.events.push(NegativeEvent {
            severity: 5.0,
            time: 0.0,
        });
        history.events.push(NegativeEvent {
            severity: 6.0,
            time: 1.0,
        });
        history.events.push(NegativeEvent {
            severity: 4.0,
            time: 2.0,
        });

        // Act
        app.update();

        // Assert: The building should now have the Cursed component
        assert!(app.world().get::<Cursed>(building).is_some());
    }

    #[test]
    fn test_cursed_building_reduces_efficiency_and_mood() {
        use crate::layer1::psychology::StressTracker;
        let mut app = App::new();
        app.add_systems(Update, apply_cursed_penalties);

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Cursed,
                Efficiency(1.0),
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert: Efficiency should be reduced and stress increased
        let efficiency = app.world().get::<Efficiency>(building).unwrap();
        assert!(efficiency.0 < 1.0);
        let stress = app.world().get::<StressTracker>(building).unwrap();
        assert!(stress.accumulated_stress > 0.0);
    }
}
