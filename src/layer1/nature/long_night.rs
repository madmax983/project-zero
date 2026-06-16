use crate::layer1::energy::PowerSource;
use crate::layer1::nature::solar::SolarPower;
use crate::layer1::nature::temperature::TemperatureGrid;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct LongNightEvent {
    pub is_active: bool,
    pub duration_remaining: i32,
}

#[derive(Event)]
pub struct StartLongNightEvent {
    pub duration_ticks: i32,
}

#[derive(Component)]
pub struct LightDependentCrop {
    pub health: f32,
    pub requires_light: bool,
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
    mut grid: ResMut<TemperatureGrid>,
    mut power_sources: Query<&mut PowerSource, With<SolarPower>>,
    mut crops: Query<&mut LightDependentCrop>,
) {
    if long_night.is_active {
        // Decrease duration
        long_night.duration_remaining -= 1;
        if long_night.duration_remaining <= 0 {
            long_night.is_active = false;
        }

        // Disable solar power
        for mut source in power_sources.iter_mut() {
            source.output = 0.0;
        }

        // Plummet ambient temperature
        grid.ambient = -20.0; // Extreme cold

        // Kill crops requiring light
        for mut crop in crops.iter_mut() {
            if crop.requires_light {
                crop.health -= 1.0;
                if crop.health < 0.0 {
                    crop.health = 0.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_long_night_event_disables_solar_power() {
        let mut app = bevy::app::App::new();

        let panel_entity = app
            .world_mut()
            .spawn((
                PowerSource {
                    output: 100.0,
                    active: true,
                },
                SolarPower { base_output: 100.0 },
            ))
            .id();
        app.world_mut()
            .insert_resource(TemperatureGrid::new(10, 10, 20.0));

        // Trigger The Long Night
        app.world_mut().insert_resource(LongNightEvent {
            is_active: true,
            duration_remaining: 10_000,
        });

        // Assume system updates power
        app.world_mut()
            .run_system_once(process_long_night_effects)
            .unwrap();

        let source = app.world().get::<PowerSource>(panel_entity).unwrap();
        // Solar power should be effectively 0
        assert_eq!(source.output, 0.0);
    }

    #[test]
    fn test_long_night_plummets_global_temperature() {
        let mut app = bevy::app::App::new();

        app.world_mut()
            .insert_resource(TemperatureGrid::new(10, 10, 20.0));
        app.world_mut().insert_resource(LongNightEvent {
            is_active: true,
            duration_remaining: 5000,
        });

        // Assume system updates temperature
        app.world_mut()
            .run_system_once(process_long_night_effects)
            .unwrap();

        let grid = app.world().resource::<TemperatureGrid>();
        // Temperature should be drastically lower than base
        assert!(grid.ambient < 5.0);
    }

    #[test]
    fn test_crops_die_during_long_night_without_light() {
        let mut app = bevy::app::App::new();

        let crop_id = app
            .world_mut()
            .spawn((LightDependentCrop {
                health: 100.0,
                requires_light: true,
            },))
            .id();

        app.world_mut()
            .insert_resource(TemperatureGrid::new(10, 10, 20.0));
        app.world_mut().insert_resource(LongNightEvent {
            is_active: true,
            duration_remaining: 5000,
        });

        // Advance time
        app.world_mut()
            .run_system_once(process_long_night_effects)
            .unwrap();

        let crop = app.world().get::<LightDependentCrop>(crop_id).unwrap();
        // Crop health should be decreasing
        assert!(crop.health < 100.0);
    }

}
