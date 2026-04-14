use bevy_ecs::prelude::*;

use crate::layer1::energy::PowerSource;
use crate::layer1::temperature::TemperatureGrid;

use crate::layer1::solar::SolarPower;
use crate::shared::time::SimulationTime;
use rand::Rng;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryPhase {
    #[default]
    Normal,
    DoubleNoon,
    TheLongDay,
}

#[derive(Resource, Default)]
pub struct BinaryStarSystem {
    pub star1_intensity: f32,
    pub star2_intensity: f32,
    pub current_phase: BinaryPhase,
    pub heat_wave_chance: f32,
}

#[derive(Component)]
pub struct BinaryCrop {
    pub growth: f32,
    pub health: f32,
    pub needs_rest: bool,
}

#[derive(Event)]
pub struct HeatWaveEvent;

pub fn binary_star_fluctuating_solar_power_system(
    binary_system: Res<BinaryStarSystem>,
    mut query: Query<(&mut PowerSource, &SolarPower)>,
) {
    let multiplier = match binary_system.current_phase {
        BinaryPhase::DoubleNoon => 2.0,
        BinaryPhase::TheLongDay => 1.5,
        BinaryPhase::Normal => 1.0,
    };

    for (mut source, solar) in &mut query {
        source.output = solar.base_output
            * multiplier
            * binary_system.star1_intensity
            * binary_system.star2_intensity;
    }
}

pub fn binary_star_crop_withering_system(
    binary_system: Res<BinaryStarSystem>,
    mut query: Query<&mut BinaryCrop>,
) {
    if binary_system.current_phase == BinaryPhase::TheLongDay {
        for mut crop in &mut query {
            if crop.needs_rest {
                crop.health -= 10.0;
                // No growth during TheLongDay
            }
        }
    }
}

pub fn binary_star_heat_waves_system(
    time: Res<SimulationTime>,
    binary_system: Res<BinaryStarSystem>,
    mut events: EventWriter<HeatWaveEvent>,
    mut grid: ResMut<TemperatureGrid>,
) {
    let mut rng = rand::thread_rng();
    if time.tick % 100 == 0 && rng.gen::<f32>() < binary_system.heat_wave_chance {
        events.send(HeatWaveEvent);
        // Increase temp across the grid
        for i in 0..grid.values.len() {
            grid.values[i] += 5.0;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::time::Duration;

    #[test]
    fn test_binary_star_fluctuating_solar_power() {
        let mut app = bevy::app::App::new();
        app.add_systems(
            bevy::app::Update,
            binary_star_fluctuating_solar_power_system,
        );

        // Arrange: Generate a Binary Star system and place a Solar Panel on Layer 1
        let panel = app
            .world_mut()
            .spawn((
                PowerSource {
                    output: 0.0,
                    ..Default::default()
                },
                SolarPower { base_output: 10.0 },
            ))
            .id();
        let binary_system = BinaryStarSystem {
            star1_intensity: 1.0,
            star2_intensity: 1.0,
            current_phase: BinaryPhase::DoubleNoon,
            ..Default::default()
        };
        app.insert_resource(binary_system);

        // Act: Advance time through the complex dual-orbit day cycle
        app.update();

        // Assert: Verify solar power output spikes during "Double Noon" and drops normally otherwise
        let updated_panel = app.world().get::<PowerSource>(panel).unwrap();
        let solar = app.world().get::<SolarPower>(panel).unwrap();
        assert!(updated_panel.output > solar.base_output * 1.5);
    }

    #[test]
    fn test_binary_star_crop_withering_without_rest() {
        let mut app = bevy::app::App::new();
        app.insert_resource(bevy::time::Time::<bevy::time::Real>::default());
        app.add_systems(bevy::app::Update, binary_star_crop_withering_system);

        // Arrange: Generate a Binary Star system causing "The Long Day" and plant crops
        let crop = app
            .world_mut()
            .spawn(BinaryCrop {
                growth: 0.0,
                health: 100.0,
                needs_rest: true,
            })
            .id();
        let binary_system = BinaryStarSystem {
            current_phase: BinaryPhase::TheLongDay,
            ..Default::default()
        };
        app.insert_resource(binary_system);

        // Act: Advance time without building blackout protection
        app.world_mut()
            .resource_mut::<bevy::time::Time<bevy::time::Real>>()
            .advance_by(Duration::from_secs(100));
        app.update();

        // Assert: Verify crops fail to progress or wither due to lack of a dark rest cycle
        let updated_crop = app.world().get::<BinaryCrop>(crop).unwrap();
        assert!(updated_crop.health < 100.0);
        assert_eq!(updated_crop.growth, 0.0);
    }

    #[test]
    fn test_binary_star_heat_waves() {
        let mut app = bevy::app::App::new();
        app.world_mut().init_resource::<Events<HeatWaveEvent>>();
        app.add_systems(bevy::app::Update, binary_star_heat_waves_system);

        // Arrange: Generate a Binary Star system
        let binary_system = BinaryStarSystem {
            current_phase: BinaryPhase::DoubleNoon,
            heat_wave_chance: 1.0,
            ..Default::default()
        };
        app.insert_resource(binary_system);
        app.insert_resource(TemperatureGrid::new(10, 10, 20.0));
        app.insert_resource(SimulationTime { tick: 100, ..Default::default() });

        // Act: Advance time
        app.update();

        // Assert: Verify random heat wave events trigger, raising local temperature grids on Layer 1
        let events = app.world().resource::<Events<HeatWaveEvent>>();
        assert!(!events.is_empty());
        let updated_grid = app.world().resource::<TemperatureGrid>();
        // Since it's a grid, we'll check average temp
        let sum: f32 = updated_grid.values.iter().sum();
        let avg = sum / (updated_grid.width * updated_grid.height) as f32;
        assert!(avg > 20.0);
    }
}
