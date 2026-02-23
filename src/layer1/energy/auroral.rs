//! System for Auroral Collector power generation.
use bevy_ecs::prelude::*;
use crate::layer1::weather::{WeatherState, WeatherType};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::PowerSource;
use crate::layer1::lighting::LightSource;

/// Updates the power output and light intensity of Auroral Collectors based on weather.
pub fn update_auroral_output_system(
    weather: Res<WeatherState>,
    mut query: Query<(&mut PowerSource, &mut LightSource, &Building)>,
) {
    let is_storm = weather.current_weather == WeatherType::MagneticStorm;
    let target_output = if is_storm { 50.0 } else { 0.0 };
    let target_intensity = if is_storm { 1.0 } else { 0.0 };

    for (mut source, mut light, building) in &mut query {
        if building.building_type == BuildingType::AuroralCollector {
            // Update Power
            if (source.output - target_output).abs() > f32::EPSILON {
                source.output = target_output;
            }

            // Update Visuals
            if (light.intensity - target_intensity).abs() > f32::EPSILON {
                light.intensity = target_intensity;
            }
        }
    }
}
