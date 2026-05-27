use bevy_ecs::prelude::*;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::core::chronicle::{Chronicle, EventImportance};
use crate::shared::time::SimulationTime;
use crate::layer1::nature::seasons::SeasonState;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct WeatherFront {
    pub weather_type: WeatherType,
    pub distance_km: f32, // Positive = approaching, 0 = active/passing
    pub speed_km_h: f32,  // Speed of approach
    pub duration_h: f32,  // Time remaining once active
}

#[derive(Resource, Default)]
pub struct WeatherFronts {
    pub active_fronts: Vec<WeatherFront>,
}

pub fn update_weather_fronts_system(
    mut fronts: ResMut<WeatherFronts>,
    mut weather: ResMut<WeatherState>,
    mut chronicle: ResMut<Chronicle>,
    time: Res<SimulationTime>,
) {
    let time_tick = time.tick;
    let dt_h = 0.1f32;

    let mut messages = Vec::new();
    let mut active_weather_override = None;

    fronts.active_fronts.retain_mut(|front| {
        if front.distance_km > 0.0 {
            front.distance_km -= front.speed_km_h * dt_h;
            if front.distance_km <= 0.0 {
                front.distance_km = 0.0;
                messages.push((
                    format!("A {:?} front has hit the colony!", front.weather_type),
                    EventImportance::Major
                ));
            }
            true
        } else {
            front.duration_h -= dt_h;
            if front.duration_h > 0.0 {
                active_weather_override = Some(front.weather_type);
                true
            } else {
                messages.push((
                    format!("The {:?} front has passed.", front.weather_type),
                    EventImportance::Standard
                ));
                false
            }
        }
    });

    for (msg, importance) in messages {
        chronicle.add_event(time_tick, msg, importance);
    }

    if let Some(new_type) = active_weather_override {
        weather.current_weather = new_type;
        weather.duration_remaining = 10;
    }
}

pub fn spawn_weather_fronts_system(
    mut fronts: ResMut<WeatherFronts>,
    _season: Res<SeasonState>,
) {
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.005) {
        let weather_type = if rng.gen_bool(0.5) {
            WeatherType::Storm
        } else {
            WeatherType::Rain
        };
        fronts.active_fronts.push(WeatherFront {
            weather_type,
            distance_km: 2000.0,
            speed_km_h: 50.0,
            duration_h: rng.gen_range(24.0..72.0),
        });
    }
}
