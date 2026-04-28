use bevy_ecs::prelude::*;

#[derive(PartialEq, Eq, Debug)]
pub enum Alignment {
    Peaceful,
    Hostile,
}

#[derive(Component)]
pub struct Culture {
    pub alignment: Alignment,
}

#[derive(Resource, Default)]
pub struct CulturalInfluenceGrid {
    pub total_pressure: f32,
}

use crate::layer1::economy::resources::ColonyResources;

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
        assert!(influence.total_pressure > 100.0, "High art and luxury should generate significant cultural pressure");
    }
}
