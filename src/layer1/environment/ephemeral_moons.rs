use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
use crate::layer1::energy::PowerSource;
use crate::layer1::solar::SolarPower;
use bevy_ecs::prelude::*;
use bevy_time::Time;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MoonType {
    Bright,
    Dark,
    Gravitational,
}

#[derive(Event)]
pub struct MoonCapturedEvent {
    pub moon_type: MoonType,
    pub duration_days: f32,
}

#[derive(Event)]
pub struct MoonEjectedEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct EphemeralMoon {
    pub moon_type: MoonType,
    pub days_remaining: f32,
}

pub fn handle_moon_capture_system(
    mut commands: Commands,
    mut events: EventReader<MoonCapturedEvent>,
) {
    for event in events.read() {
        commands.spawn(EphemeralMoon {
            moon_type: event.moon_type,
            days_remaining: event.duration_days,
        });
    }
}

pub fn apply_moon_modifiers_system(
    moons: Query<&EphemeralMoon>,
    env_opt: Option<Res<DayNightCycle>>,
    mut panels: Query<(&mut PowerSource, &SolarPower)>,
) {
    if let Some(env) = env_opt {
        let mut bright_moon_present = false;
        for moon in moons.iter() {
            if moon.moon_type == MoonType::Bright {
                bright_moon_present = true;
                break;
            }
        }

        if env.time_of_day == TimeOfDay::Night && bright_moon_present {
            // Give it 50% boost over base
            for (mut source, panel) in panels.iter_mut() {
                source.output += panel.base_output * 0.5;
            }
        }
    }
}

pub fn decay_ephemeral_moons_system(
    mut commands: Commands,
    time: Res<Time>,
    mut moons: Query<(Entity, &mut EphemeralMoon)>,
    mut eject_events: EventWriter<MoonEjectedEvent>,
) {
    // Assuming 1 day = 100 seconds in game time for simplicity.
    // Builder should use actual constant.
    let day_delta = time.delta_secs() / 100.0;

    for (entity, mut moon) in moons.iter_mut() {
        moon.days_remaining -= day_delta;
        if moon.days_remaining <= 0.0 {
            eject_events.send(MoonEjectedEvent { entity });
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::energy::PowerSource;
    use crate::layer1::solar::SolarPower;
    use bevy_app::{App, Update};
    use bevy_time::Time;

    #[test]
    fn test_moon_capture_creates_ephemeral_moon_entity() {
        let mut app = App::new();
        app.add_event::<MoonCapturedEvent>();
        app.add_systems(Update, handle_moon_capture_system);

        app.world_mut().send_event(MoonCapturedEvent {
            moon_type: MoonType::Bright,
            duration_days: 30.0,
        });
        app.update();

        let mut query = app.world_mut().query::<&EphemeralMoon>();
        let moon_query = query.iter(app.world()).next();
        assert!(moon_query.is_some());
        assert_eq!(moon_query.unwrap().days_remaining, 30.0);
    }

    #[test]
    fn test_bright_moon_boosts_night_solar_power() {
        let mut app = App::new();
        app.add_systems(Update, apply_moon_modifiers_system);

        app.world_mut().spawn(EphemeralMoon {
            moon_type: MoonType::Bright,
            days_remaining: 10.0,
        });

        let solar_panel = app
            .world_mut()
            .spawn((
                SolarPower { base_output: 10.0 },
                PowerSource {
                    output: 0.0,
                    active: true,
                },
            ))
            .id();

        let env = DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        };
        app.insert_resource(env);

        app.update();

        let source = app.world().get::<PowerSource>(solar_panel).unwrap();
        // Base is 10.0, Night output normally 0.0. With boost, output is + 5.0 (50% boost)
        assert!(source.output > 0.0);
    }

    #[test]
    fn test_moon_ejected_after_duration() {
        let mut app = App::new();
        app.insert_resource(bevy_time::Time::<bevy_time::Real>::default());
        app.insert_resource(bevy_time::Time::<bevy_time::Virtual>::default());
        app.insert_resource(bevy_time::Time::<()>::default());
        app.add_event::<MoonEjectedEvent>();
        app.add_systems(Update, decay_ephemeral_moons_system);

        let moon = app
            .world_mut()
            .spawn(EphemeralMoon {
                moon_type: MoonType::Bright,
                days_remaining: 1.0,
            })
            .id();

        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(101));
        app.update();

        assert!(app.world().get::<EphemeralMoon>(moon).is_none());
        let events = app.world().resource::<Events<MoonEjectedEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).next().is_some());
    }
}
