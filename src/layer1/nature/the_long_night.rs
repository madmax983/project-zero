use crate::layer1::agriculture::farm::Farm;
use crate::layer1::biology::health::Health;
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::energy::PowerSource;
use crate::layer1::lighting::{AmbientLight, LightSource};
use crate::layer1::map::GridPosition;
use crate::layer1::nature::solar::SolarPower;
use crate::layer1::nature::temperature::TemperatureGrid;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct LongNightEvent {
    pub is_active: bool,
    pub original_ambient: f32,
    pub duration_remaining: i32,
}

#[derive(Event)]
pub struct StartLongNightEvent {
    pub duration_ticks: i32,
}

pub fn start_long_night(
    temp_grid: Option<Res<TemperatureGrid>>,
    mut events: EventReader<StartLongNightEvent>,
    mut long_night: ResMut<LongNightEvent>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        if !long_night.is_active {
            chronicle.send(AddChronicleEvent {
                text: "The Long Night Begins. Darkness covers the colony.".to_string(),
                importance: EventImportance::Legendary,
            });
        }
        long_night.is_active = true;
        long_night.duration_remaining = event.duration_ticks;
        if let Some(grid) = temp_grid.as_ref() {
            long_night.original_ambient = grid.ambient;
        }
    }
}

pub fn process_long_night_effects(
    mut long_night: ResMut<LongNightEvent>,
    mut solar_sources: Query<(&mut PowerSource, &SolarPower)>,
    mut temp_grid: Option<ResMut<TemperatureGrid>>,
    mut farms: Query<(&mut Health, &GridPosition), With<Farm>>,
    lights: Query<(&LightSource, &GridPosition)>,
    mut ambient: Option<ResMut<AmbientLight>>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    if long_night.is_active {
        long_night.duration_remaining -= 1;
        if long_night.duration_remaining <= 0 {
            long_night.is_active = false;
            chronicle.send(AddChronicleEvent {
                text: "The Long Night has ended. The sun returns.".to_string(),
                importance: EventImportance::Legendary,
            });
        }

        if let Some(a) = ambient.as_deref_mut() {
            a.level = 0.0;
        }

        for (mut source, _) in solar_sources.iter_mut() {
            source.output = 0.0;
        }

        if let Some(grid) = temp_grid.as_deref_mut() {
            grid.ambient = -20.0;
        }

        for (mut health, farm_pos) in farms.iter_mut() {
            let mut has_light = false;
            for (light, light_pos) in lights.iter() {
                let dx = farm_pos.x - light_pos.x;
                let dy = farm_pos.y - light_pos.y;
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= light.radius {
                    has_light = true;
                    break;
                }
            }
            if !has_light {
                health.take_damage(1.0);
            }
        }
    } else {
        if let Some(grid) = temp_grid.as_deref_mut() {
            if grid.ambient < 20.0 {
                grid.ambient += 0.5;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use bevy::prelude::*;

    #[test]
    fn test_long_night_event_disables_solar_power() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (start_long_night, process_long_night_effects).chain(),
        );

        let solar_entity = app
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
            .init_resource::<Events<StartLongNightEvent>>();
        app.world_mut().init_resource::<Events<AddChronicleEvent>>();
        app.world_mut().insert_resource(LongNightEvent::default());

        app.world_mut().send_event(StartLongNightEvent {
            duration_ticks: 10_000,
        });
        app.update();

        let event_active = app.world().resource::<LongNightEvent>();
        assert!(event_active.is_active);

        let power = app.world().get::<PowerSource>(solar_entity).unwrap();
        assert_eq!(power.output, 0.0);
    }

    #[test]
    fn test_long_night_plummets_global_temperature() {
        let mut app = App::new();
        app.add_systems(Update, process_long_night_effects);

        let mut grid = TemperatureGrid::new(10, 10, 20.0);
        grid.ambient = 20.0;
        app.world_mut().init_resource::<Events<AddChronicleEvent>>();
        app.insert_resource(grid);
        app.insert_resource(LongNightEvent {
            is_active: true,
            duration_remaining: 5000,
            original_ambient: 20.0,
        });

        app.update();

        let temp = app.world().resource::<TemperatureGrid>();
        assert!(temp.ambient < 5.0);
    }

    #[test]
    fn test_crops_die_during_long_night_without_light() {
        let mut app = App::new();
        app.add_systems(Update, process_long_night_effects);

        let crop_id = app
            .world_mut()
            .spawn((
                Farm::default(),
                Health::default(),
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        app.world_mut().init_resource::<Events<AddChronicleEvent>>();
        app.insert_resource(LongNightEvent {
            is_active: true,
            duration_remaining: 5000,
            original_ambient: 20.0,
        });

        app.update();

        let health = app.world().get::<Health>(crop_id).unwrap();
        assert!(health.current < 100.0);
    }
}
