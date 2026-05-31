use crate::layer1::biology::biocompatibility::ExposedToEnvironment;
use crate::layer1::GridPosition;
use rand::Rng;
use crate::layer1::building::Building;
use crate::layer1::entities::pop::Pop;
use crate::layer1::health::Health;
use crate::layer1::nature::radioactive::RadiationSickness;
use bevy_ecs::prelude::*;
use bevy_time::{Time, Timer, TimerMode};

#[derive(Component)]
pub struct Electronic;

#[derive(Component)]
pub struct Shielded;

#[derive(Component)]
pub struct FlareIsotope {
    pub value: f32,
    pub timer: Timer,
}

#[derive(Event)]
pub struct SolarFlareEvent;

type SolarFlareEmpQuery<'world, 'state, 'a> =
    Query<'world, 'state, &'a mut Health, (With<Building>, With<Electronic>, Without<Shielded>)>;

pub fn solar_flare_emp_system(
    mut events: EventReader<SolarFlareEvent>,
    mut buildings: SolarFlareEmpQuery,
) {
    for _ in events.read() {
        for mut health in buildings.iter_mut() {
            health.current -= 50.0;
        }
    }
}

pub fn solar_flare_radiation_system(
    mut commands: Commands,
    mut events: EventReader<SolarFlareEvent>,
    exposed_pops: Query<Entity, (With<Pop>, With<ExposedToEnvironment>)>,
) {
    for _ in events.read() {
        for entity in exposed_pops.iter() {
            commands
                .entity(entity)
                .insert(RadiationSickness { severity: 50.0 });
        }
    }
}

pub fn spawn_flare_isotopes_system(
    mut commands: Commands,
    mut events: EventReader<SolarFlareEvent>,
) {
    for _ in events.read() {
        // Spawn 10 random isotopes for testing
        for _ in 0..10 {
            let mut rng = rand::thread_rng();
            commands.spawn((FlareIsotope {
                value: 100.0,
                timer: Timer::from_seconds(120.0, TimerMode::Once),
            }, GridPosition { x: rng.gen_range(0..100), y: rng.gen_range(0..100) }));
        }
    }
}

pub fn decay_flare_isotopes_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FlareIsotope)>,
) {
    for (entity, mut isotope) in query.iter_mut() {
        isotope.timer.tick(time.delta());
        if isotope.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_solar_flare_event_damages_unshielded_buildings() {
        let mut app = App::new();
        app.add_event::<SolarFlareEvent>();
        app.add_systems(Update, solar_flare_emp_system);

        let building_unshielded = app
            .world_mut()
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::BuildingType::Housing,
                    ..Default::default()
                },
                Electronic,
                Health {
                    current: 100.0,
                    max: 100.0,
                    ..Default::default()
                },
            ))
            .id();

        let building_shielded = app
            .world_mut()
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::BuildingType::Housing,
                    ..Default::default()
                },
                Electronic,
                Shielded,
                Health {
                    current: 100.0,
                    max: 100.0,
                    ..Default::default()
                },
            ))
            .id();

        app.world_mut().send_event(SolarFlareEvent);
        app.update();

        assert_eq!(
            app.world()
                .get::<Health>(building_unshielded)
                .unwrap()
                .current,
            50.0
        );
        assert_eq!(
            app.world()
                .get::<Health>(building_shielded)
                .unwrap()
                .current,
            100.0
        );
    }

    #[test]
    fn test_solar_flare_event_causes_radiation_sickness() {
        let mut app = App::new();
        app.add_event::<SolarFlareEvent>();
        app.add_systems(Update, solar_flare_radiation_system);

        let pop_exposed = app.world_mut().spawn((Pop, ExposedToEnvironment)).id();
        let pop_sheltered = app.world_mut().spawn(Pop).id();

        app.world_mut().send_event(SolarFlareEvent);
        app.update();

        assert!(app.world().get::<RadiationSickness>(pop_exposed).is_some());
        assert!(app
            .world()
            .get::<RadiationSickness>(pop_sheltered)
            .is_none());
    }

    #[test]
    fn test_flare_isotopes_spawn_and_decay() {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.add_event::<SolarFlareEvent>();
        app.add_systems(
            Update,
            (spawn_flare_isotopes_system, decay_flare_isotopes_system),
        );

        app.world_mut().send_event(SolarFlareEvent);
        app.update();

        let mut isotope_count = 0;
        for _ in app.world_mut().query::<&FlareIsotope>().iter(app.world()) {
            isotope_count += 1;
        }
        assert_eq!(isotope_count, 10);

        // advance time
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(121));
        app.update();

        let mut isotope_count = 0;
        for _ in app.world_mut().query::<&FlareIsotope>().iter(app.world()) {
            isotope_count += 1;
        }
        assert_eq!(isotope_count, 0);
    }
}
