use bevy::prelude::*;

#[derive(Component)]
pub struct Fleet {
    pub base_speed: f32,
    pub current_speed: f32,
}

#[derive(Component)]
pub struct SolarSails {
    pub deployed: bool,
    pub efficiency: f32,
}

#[derive(Component)]
pub struct CurrentSector {
    pub sector_entity: Entity,
}

#[derive(Component)]
pub struct Sector;

#[derive(Component)]
pub struct StellarWeather {
    pub wind_strength: f32,
    pub flare_active: bool,
}

#[derive(Event)]
pub struct FleetDamagedEvent {
    pub fleet: Entity,
    pub amount: f32,
}

pub fn apply_stellar_weather_effects(
    mut fleets: Query<(Entity, &mut Fleet, &SolarSails, &CurrentSector)>,
    sectors: Query<&StellarWeather, With<Sector>>,
    mut damage_events: EventWriter<FleetDamagedEvent>,
) {
    for (fleet_entity, mut fleet, sails, current_sector) in fleets.iter_mut() {
        if let Ok(weather) = sectors.get(current_sector.sector_entity) {
            if sails.deployed {
                fleet.current_speed = fleet.base_speed + (weather.wind_strength * sails.efficiency);
                if weather.flare_active {
                    damage_events.send(FleetDamagedEvent {
                        fleet: fleet_entity,
                        amount: 50.0,
                    });
                }
            } else {
                fleet.current_speed = fleet.base_speed;
            }
        } else {
            fleet.current_speed = fleet.base_speed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solar_sails_boost_speed_in_favorable_wind() {
        let mut app = App::new();
        app.add_event::<FleetDamagedEvent>();
        app.add_systems(Update, apply_stellar_weather_effects);

        let sector_entity = app
            .world_mut()
            .spawn((
                Sector,
                StellarWeather {
                    wind_strength: 5.0,
                    flare_active: false,
                },
            ))
            .id();

        let fleet_entity = app
            .world_mut()
            .spawn((
                Fleet {
                    base_speed: 10.0,
                    current_speed: 10.0,
                },
                SolarSails {
                    deployed: true,
                    efficiency: 2.0,
                },
                CurrentSector { sector_entity },
            ))
            .id();

        app.update();

        let fleet = app.world().get::<Fleet>(fleet_entity).unwrap();
        assert!(
            fleet.current_speed > fleet.base_speed,
            "Speed should be boosted by favorable wind"
        );
        assert_eq!(fleet.current_speed, 10.0 + (5.0 * 2.0));
    }

    #[test]
    fn test_solar_flare_damages_deployed_sails() {
        let mut app = App::new();
        app.add_event::<FleetDamagedEvent>();
        app.add_systems(Update, apply_stellar_weather_effects);

        let sector_entity = app
            .world_mut()
            .spawn((
                Sector,
                StellarWeather {
                    wind_strength: 5.0,
                    flare_active: true,
                },
            ))
            .id();

        let fleet_entity = app
            .world_mut()
            .spawn((
                Fleet {
                    base_speed: 10.0,
                    current_speed: 10.0,
                },
                SolarSails {
                    deployed: true,
                    efficiency: 2.0,
                },
                CurrentSector { sector_entity },
            ))
            .id();

        app.update();

        let damage_events = app.world().resource::<Events<FleetDamagedEvent>>();
        let mut reader = damage_events.get_cursor();
        assert!(
            reader.read(damage_events).any(|e| e.fleet == fleet_entity),
            "Fleet should be damaged by flare when sails are deployed"
        );
    }

    #[test]
    fn test_solar_sails_speed_resets_in_clear_sector() {
        let mut app = App::new();
        app.add_event::<FleetDamagedEvent>();
        app.add_systems(Update, apply_stellar_weather_effects);

        let sector_entity = app.world_mut().spawn(Sector).id();

        let fleet_entity = app
            .world_mut()
            .spawn((
                Fleet {
                    base_speed: 10.0,
                    current_speed: 20.0, // Pre-existing weather boost
                },
                SolarSails {
                    deployed: true,
                    efficiency: 2.0,
                },
                CurrentSector { sector_entity },
            ))
            .id();

        app.update();

        let fleet = app.world().get::<Fleet>(fleet_entity).unwrap();
        assert_eq!(
            fleet.current_speed, fleet.base_speed,
            "Speed should be reset to base speed when no weather is present"
        );
    }
}
