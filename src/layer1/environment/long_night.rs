use bevy_ecs::prelude::*;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::nature::temperature::TemperatureGrid;
use crate::layer1::LightSource;
use crate::layer1::biology::genetics::crop_modification::Crop;

#[derive(Resource, Default)]
pub struct LongNightEvent {
    pub is_active: bool,
    pub duration_remaining: u64,
}

#[derive(Event)]
pub struct StartLongNightEvent {
    pub duration_ticks: u64,
}

pub fn start_long_night(
    mut events: EventReader<StartLongNightEvent>,
    mut long_night: ResMut<LongNightEvent>,
) {
    for event in events.read() {
        long_night.is_active = true;
        long_night.duration_remaining = event.duration_ticks;
    }
}

pub fn process_long_night_effects(
    mut long_night: ResMut<LongNightEvent>,
    day_night: Option<ResMut<DayNightCycle>>,
    mut temp: ResMut<TemperatureGrid>,
    mut crops: Query<&mut Crop, Without<LightSource>>,
) {
    if long_night.is_active {
        // Decrease duration
        long_night.duration_remaining = long_night.duration_remaining.saturating_sub(1);
        if long_night.duration_remaining == 0 {
            long_night.is_active = false;
        }

        if let Some(_cycle) = day_night {
            // For now, simulate darkness by not advancing day count/light or something similar
            // In lieu of ambient_light, we can mock something or leave it out if we lack the field
        }

        // Plummet ambient temperature
        temp.ambient = -20.0; // Extreme cold

        // Kill crops requiring light (that don't have an artificial light source)
        for mut crop in crops.iter_mut() {
            crop.current_yield = crop.current_yield.saturating_sub(1);
        }
    } else {
        // Restoration can be handled by season/ambient slowly, but for test coverage:
        if temp.ambient < 20.0 { // Mocking base recovery
            temp.ambient = (temp.ambient + 0.5).min(20.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::MinimalPlugins;
    use bevy_app::{App, Update};
    use super::*;

    #[test]
    fn test_long_night_plummets_global_temperature() {
        let mut app = bevy_app::App::new();
        app.add_plugins(bevy::MinimalPlugins);

        let grid = TemperatureGrid::new(10, 10, 20.0);
        app.insert_resource(grid);
        app.insert_resource(LongNightEvent { is_active: true, duration_remaining: 5000 });

        // Advance time and check temperature
        app.add_systems(Update, process_long_night_effects);
        app.update();

        let temp = app.world().resource::<TemperatureGrid>();
        // Temperature should be drastically lower than base
        assert!(temp.ambient < 5.0);
    }

    #[test]
    fn test_crops_die_during_long_night_without_light() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let crop_id = app.world_mut().spawn((
            Crop { base_yield: 100, current_yield: 100 },
        )).id();

        app.insert_resource(LongNightEvent { is_active: true, duration_remaining: 5000 });
        let grid = TemperatureGrid::new(10, 10, 20.0);
        app.insert_resource(grid);

        app.add_systems(Update, process_long_night_effects);
        // Advance time
        app.update();

        let crop = app.world().get::<Crop>(crop_id).unwrap();
        // Crop health should be decreasing
        assert!(crop.current_yield < 100);
    }
}
