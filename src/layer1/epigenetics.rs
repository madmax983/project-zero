use bevy::prelude::*;

use crate::layer1::pop::{Pop, PopBorn};
use crate::layer1::stress::StressTracker;

#[derive(Component)]
pub struct StarvationTrauma;

#[derive(Component)]
pub struct HoarderTrait;

#[derive(Component)]
pub struct OffspringOf(pub Entity);

#[allow(clippy::type_complexity)]
pub fn apply_epigenetic_trauma_system(
    mut commands: Commands,
    mut pop_born_events: EventReader<PopBorn>,
    child_query: Query<&OffspringOf, (With<Pop>, Without<HoarderTrait>)>,
    parent_query: Query<(&StressTracker, Option<&StarvationTrauma>), With<Pop>>,
) {
    for event in pop_born_events.read() {
        if let Ok(offspring) = child_query.get(event.entity) {
            if let Ok((stress, starvation_trauma)) = parent_query.get(offspring.0) {
                if stress.accumulated_stress >= 80.0 && starvation_trauma.is_some() {
                    commands.entity(event.entity).insert(HoarderTrait);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offspring_inherits_trauma_as_hoarder_trait() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<PopBorn>();
        app.add_systems(Update, apply_epigenetic_trauma_system);

        let parent = app
            .world_mut()
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 90.0,
                },
                StarvationTrauma,
            ))
            .id();

        let child = app.world_mut().spawn((Pop, OffspringOf(parent))).id();

        app.world_mut().send_event(PopBorn {
            entity: child,
            name: "Child".to_string(),
            tick: 1,
            source: "Test".to_string(),
        });

        // Act
        app.update();

        // Assert
        // Child should have developed the Hoarder trait due to parent's StarvationTrauma and high stress
        assert!(app.world().get::<HoarderTrait>(child).is_some());
    }

    #[test]
    fn test_offspring_without_traumatized_parent_does_not_inherit() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<PopBorn>();
        app.add_systems(Update, apply_epigenetic_trauma_system);

        let parent = app
            .world_mut()
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 10.0,
                }, // Low stress, no trauma
            ))
            .id();

        let child = app.world_mut().spawn((Pop, OffspringOf(parent))).id();

        app.world_mut().send_event(PopBorn {
            entity: child,
            name: "Child".to_string(),
            tick: 1,
            source: "Test".to_string(),
        });

        // Act
        app.update();

        // Assert
        // Child should NOT have the Hoarder trait
        assert!(app.world().get::<HoarderTrait>(child).is_none());
    }
}
