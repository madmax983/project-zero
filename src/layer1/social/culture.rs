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

pub fn calculate_cultural_pressure_system(
    resources: Res<crate::layer1::economy::resources::ColonyResources>,
    mut influence: ResMut<CulturalInfluenceGrid>,
    time: Option<Res<bevy::prelude::Time>>,
) {
    // Formula: (Art * 1.5) + Luxury = Target Pressure
    let target_pressure = (resources.art * 1.5) + resources.luxury;

    // Decay logic as part of REFACTOR
    // Interpolate towards the target. In the real simulation, ticks happen over time.
    // If we have time, use delta_secs, else a default small tick value
    let dt = time.map(|t| t.delta_secs()).unwrap_or(1.0);
    let decay_rate = 0.05 * dt;

    influence.total_pressure =
        influence.total_pressure * (1.0 - decay_rate) + target_pressure * decay_rate;
}
