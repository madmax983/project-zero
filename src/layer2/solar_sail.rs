use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::energy::PowerSource;
use crate::layer1::nature::temperature::TemperatureGrid;
use crate::layer2::system::Orbit;
use bevy::app::{App, Plugin, Update};
use bevy_ecs::prelude::*;

pub struct SolarSailPlugin;

impl Plugin for SolarSailPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SolarMigrationState>()
            .add_systems(Update, (check_sail_fleet_proximity, apply_solar_sail_effects));
    }
}

/// Indicates a fleet is a massive Solar Sail nomad fleet
#[derive(Component)]
pub struct SolarSailFleet;

/// Tracks the state of the Solar Migration event
#[derive(Resource)]
pub struct SolarMigrationState {
    pub is_active: bool,
    pub intensity_multiplier: f32,
}

impl Default for SolarMigrationState {
    fn default() -> Self {
        Self {
            is_active: false,
            intensity_multiplier: 1.0,
        }
    }
}

/// Checks the distance between Solar Sail Fleets and the main planet to trigger the event.
pub fn check_sail_fleet_proximity(
    mut state: ResMut<SolarMigrationState>,
    // In our system, let's assume body with "Planet" or a main planet target exists.
    // For simplicity, we just look for any fleet with SolarSailFleet and use its Orbit radius
    fleet_query: Query<&Orbit, With<SolarSailFleet>>,
) {
    let mut active = false;
    let mut intensity = 1.0;

    for orbit in fleet_query.iter() {
        // If a solar sail fleet is close to the star/planet (small radius)
        if orbit.radius < 50.0 {
            active = true;
            intensity = 1.0 + (50.0 - orbit.radius) * 0.1;
        }
    }

    state.is_active = active;
    state.intensity_multiplier = intensity.max(1.0);
}

/// Applies the effects of intense reflected sunlight: boosts solar power, increases global heat.
pub fn apply_solar_sail_effects(
    state: Res<SolarMigrationState>,
    mut power_query: Query<(&Building, &mut PowerSource)>,
    mut temp_grid: Option<ResMut<TemperatureGrid>>,
    time: Res<bevy::time::Time>,
) {
    // Determine the boost multiplier
    let multiplier = if state.is_active {
        state.intensity_multiplier
    } else {
        1.0
    };

    // Apply to all Solar Panels
    for (building, mut source) in power_query.iter_mut() {
        if building.building_type == BuildingType::SolarPanel {
            // Assume 10.0 is the base output for Solar Panels as defined in architecture::building
            source.output = 10.0 * multiplier;
        }
    }

    // Apply heat to the whole map if active
    if state.is_active {
        if let Some(ref mut grid) = temp_grid {
            // Apply a global heat influx
            let extra_heat = 1.0 * state.intensity_multiplier * time.delta_secs();
            let width = grid.width;
            let height = grid.height;
            for x in 0..width {
                for y in 0..height {
                    let current = grid.get(x, y);
                    grid.set(x, y, current + extra_heat);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::system::Orbit;
    use bevy::prelude::*;
    use bevy::time::Time;

    #[test]
    fn test_solar_sail_transit_starts_event() {
        let mut app = App::new();
        app.add_systems(Update, check_sail_fleet_proximity);
        app.insert_resource(SolarMigrationState::default());

        let parent = app.world_mut().spawn_empty().id();

        // Spawn fleet nearby (radius < 50)
        app.world_mut().spawn((
            SolarSailFleet,
            Orbit {
                parent,
                radius: 20.0,
                angle: 0.0,
                speed: 0.1,
            },
        ));

        app.update();

        let state = app.world().resource::<SolarMigrationState>();
        assert!(state.is_active, "Migration event should be active.");
        assert!(
            state.intensity_multiplier > 1.0,
            "Sunlight intensity should be multiplied."
        );
    }

    #[test]
    fn test_solar_sail_increases_solar_power_and_heat() {
        let mut app = App::new();
        app.insert_resource(SolarMigrationState {
            is_active: true,
            intensity_multiplier: 3.0,
        });
        app.insert_resource(TemperatureGrid::new(10, 10, 20.0));
        app.insert_resource(Time::<()>::default());

        // Tick time by 1 second
        let mut time = app.world_mut().resource_mut::<Time>();
        let mut fixed = bevy::time::Time::<()>::default();
        fixed.advance_by(std::time::Duration::from_secs(1));
        *time = fixed;

        app.add_systems(Update, apply_solar_sail_effects);

        let solar_panel = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::SolarPanel,
                },
                PowerSource {
                    output: 10.0,
                    active: true,
                },
            ))
            .id();

        app.update();

        let power = app.world().get::<PowerSource>(solar_panel).unwrap().output;
        assert_eq!(power, 30.0, "Power output should be multiplied.");

        let temp = app.world().resource::<TemperatureGrid>().get(5, 5);
        assert!(temp > 20.0, "Surface temperature should increase.");
    }
}
