//! The Cultural Influence Grid.
//!
//! This module manages the total cultural pressure exerted by a colony.
//! Cultural influence acts as a soft power metric, primarily driven by the accumulation
//! of art and luxury goods. Over time, base pressure naturally decays, requiring
//! constant production to maintain dominance.

use bevy_ecs::prelude::*;

/// The alignment of a specific [`Culture`].
#[derive(PartialEq, Eq, Debug)]
pub enum Alignment {
    Peaceful,
    Hostile,
}

/// A marker component indicating an entity belongs to a culture.
#[derive(Component)]
pub struct Culture {
    pub alignment: Alignment,
}

/// Tracks the global cultural pressure exerted by the colony.
///
/// # Examples
/// ```rust
/// use scale::layer1::culture::cultural_influence::CulturalInfluenceGrid;
///
/// let mut grid = CulturalInfluenceGrid::default();
/// grid.total_pressure = 100.0;
///
/// // Simulating decay
/// grid.total_pressure *= 0.95;
/// assert_eq!(grid.total_pressure, 95.0);
/// ```
#[derive(Resource, Default)]
pub struct CulturalInfluenceGrid {
    pub total_pressure: f32,
}

use crate::layer1::economy::resources::ColonyResources;

/// Calculates the new cultural pressure based on current resources.
///
/// This system applies a flat 5% decay to the existing [`CulturalInfluenceGrid::total_pressure`],
/// and then adds new pressure derived from [`ColonyResources::art`] and [`ColonyResources::luxury`].
///
/// # Examples
/// ```rust
/// use bevy_app::prelude::*;
/// use scale::layer1::economy::resources::ColonyResources;
/// use scale::layer1::culture::cultural_influence::{CulturalInfluenceGrid, calculate_cultural_pressure_system};
///
/// let mut app = App::new();
/// app.insert_resource(ColonyResources { art: 10.0, luxury: 5.0, ..Default::default() });
/// app.insert_resource(CulturalInfluenceGrid { total_pressure: 100.0 });
///
/// app.add_systems(Update, calculate_cultural_pressure_system);
/// app.update();
///
/// let influence = app.world().get_resource::<CulturalInfluenceGrid>().unwrap();
/// // 100.0 * 0.95 = 95.0
/// // New pressure: (10.0 * 1.5) + 5.0 = 20.0
/// // Total: 95.0 + 20.0 = 115.0
/// assert_eq!(influence.total_pressure, 115.0);
/// ```
pub fn calculate_cultural_pressure_system(
    resources: Option<Res<ColonyResources>>,
    influence: Option<ResMut<CulturalInfluenceGrid>>,
) {
    if let (Some(res), Some(mut inf)) = (resources, influence) {
        // Decay to represent fading trends
        inf.total_pressure *= 0.95;
        // Formula: (Art * 1.5) + Luxury = added Pressure
        inf.total_pressure += (res.art * 1.5) + res.luxury;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.insert_resource(CulturalInfluenceGrid::default());
        app.add_systems(Update, calculate_cultural_pressure_system);
        app
    }

    #[test]
    fn test_high_art_luxury_generates_cultural_pressure() {
        let mut app = setup_app();

        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        resources.art = 500.0;
        resources.luxury = 300.0;

        app.update();

        // Assert the cultural influence grid or score has increased
        let influence = app.world().get_resource::<CulturalInfluenceGrid>().unwrap();
        assert!(
            influence.total_pressure > 100.0,
            "High art and luxury should generate significant cultural pressure"
        );
    }

    #[test]
    fn test_calculate_cultural_pressure_system_decays_over_time() {
        let mut app = setup_app();

        // Ensure 0 new art/luxury so only decay happens
        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        resources.art = 0.0;
        resources.luxury = 0.0;

        let mut influence = app.world_mut().resource_mut::<CulturalInfluenceGrid>();
        influence.total_pressure = 100.0;

        app.update();

        let influence = app.world().get_resource::<CulturalInfluenceGrid>().unwrap();
        assert!((influence.total_pressure - 95.0).abs() < f32::EPSILON, "Pressure should decay by 5% when no new art/luxury is generated");
    }

    #[test]
    fn test_calculate_cultural_pressure_system_missing_resources() {
        // Run with no resources, shouldn't panic
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, calculate_cultural_pressure_system);
        app.update(); // Should just skip processing
    }
}
