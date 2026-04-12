use bevy_ecs::prelude::*;
use rand::Rng;

use crate::layer1::energy::PowerConsumer;
use crate::layer1::logistics::mass_driver::BombardmentEvent;
use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;
use crate::layer1::map::GridPosition;
use crate::layer2::debris::OrbitalDebris;
use crate::layer2::fleet::{Fleet, InOrbit};

/// A building component that winches down orbital debris or entities.
#[derive(Component)]
pub struct GravityHarpoon {
    /// The Layer 2 entity currently hooked.
    pub target_entity: Option<Entity>,
    /// Progress of the winching operation (0.0 to 100.0).
    pub winch_progress: f32,
}

impl Default for GravityHarpoon {
    fn default() -> Self {
        Self {
            target_entity: None,
            winch_progress: 0.0,
        }
    }
}

/// Winch execution system for the Gravity Harpoon.
/// - Checks if the harpoon is powered.
/// - If powered and has a target, increments progress.
/// - On 100% progress, triggers drop event.
/// - Has a chance to snap the cable based on target's size/mass.
pub fn winch_execution_system(
    mut commands: Commands,
    mut harpoon_query: Query<(&mut GravityHarpoon, &PowerConsumer, &GridPosition)>,
    mut debris_query: Query<&mut OrbitalDebris>,
    fleet_query: Query<&InOrbit, With<Fleet>>,
    mut drop_writer: EventWriter<OrbitalDropEvent>,
    mut bomb_writer: EventWriter<BombardmentEvent>,
) {
    let mut rng = rand::thread_rng();

    for (mut harpoon, power, pos) in harpoon_query.iter_mut() {
        if !power.active || harpoon.target_entity.is_none() {
            continue;
        }

        let target = harpoon.target_entity.unwrap();

        harpoon.winch_progress += 10.0;

        let mut mass = 0.0;

        if let Ok(debris) = debris_query.get(target) {
            mass += debris.amount;
        }
        if fleet_query.get(target).is_ok() {
            mass += 5.0;
        }

        let snap_chance = 0.05 + (mass * 0.1).clamp(0.0, 0.9);

        if rng.gen_bool(snap_chance as f64) {
            if let Ok(mut debris) = debris_query.get_mut(target) {
                debris.amount = (debris.amount - 0.5).max(0.0);
            } else {
                commands.entity(target).despawn();
            }

            bomb_writer.send(BombardmentEvent {
                target: Entity::PLACEHOLDER,
                kinetic_energy: (mass * 100.0).max(100.0) as u32,
            });

            harpoon.target_entity = None;
            harpoon.winch_progress = 0.0;
            continue;
        }

        if harpoon.winch_progress >= 100.0 {
            if let Ok(mut debris) = debris_query.get_mut(target) {
                debris.amount = (debris.amount - 0.5).max(0.0);
            } else {
                commands.entity(target).despawn();
            }

            drop_writer.send(OrbitalDropEvent {
                target: *pos,
                items: vec![crate::layer1::items::ItemType::Scrap; 5],
                scatter_radius: 2,
            });

            harpoon.target_entity = None;
            harpoon.winch_progress = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::logistics::mass_driver::BombardmentEvent;
    use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;
    use crate::layer1::map::GridPosition;
    use crate::layer2::debris::OrbitalDebris;
    use bevy::prelude::*;

    #[test]
    fn test_gravity_harpoon_winching_debris() {
        let mut app = App::new();
        app.add_systems(Update, winch_execution_system);
        app.add_event::<OrbitalDropEvent>();
        app.add_event::<BombardmentEvent>();

        let target_entity = app.world_mut().spawn(OrbitalDebris { amount: 0.0 }).id();

        let harpoon_entity = app
            .world_mut()
            .spawn((
                GravityHarpoon {
                    target_entity: Some(target_entity),
                    winch_progress: 90.0,
                },
                PowerConsumer {
                    demand: 100.0,
                    active: true,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut success = false;
        for _ in 0..100 {
            app.update();
            let harpoon = app.world().get::<GravityHarpoon>(harpoon_entity).unwrap();
            if harpoon.target_entity.is_none() {
                let drops = app.world().resource::<Events<OrbitalDropEvent>>();
                let bombs = app.world().resource::<Events<BombardmentEvent>>();

                if drops.len() > 0 {
                    success = true;
                    break;
                } else if bombs.len() > 0 {
                    app.world_mut()
                        .resource_mut::<Events<BombardmentEvent>>()
                        .clear();
                    app.world_mut()
                        .get_mut::<GravityHarpoon>(harpoon_entity)
                        .unwrap()
                        .target_entity = Some(target_entity);
                    app.world_mut()
                        .get_mut::<GravityHarpoon>(harpoon_entity)
                        .unwrap()
                        .winch_progress = 90.0;
                }
            }
        }

        assert!(
            success,
            "The debris should descend to the Layer 1 Drop Zone successfully"
        );
    }

    #[test]
    fn test_gravity_harpoon_cable_snap() {
        let mut app = App::new();
        app.add_systems(Update, winch_execution_system);
        app.add_event::<OrbitalDropEvent>();
        app.add_event::<BombardmentEvent>();

        let target_entity = app.world_mut().spawn(OrbitalDebris { amount: 100.0 }).id();

        let harpoon_entity = app
            .world_mut()
            .spawn((
                GravityHarpoon {
                    target_entity: Some(target_entity),
                    winch_progress: 0.0,
                },
                PowerConsumer {
                    demand: 100.0,
                    active: true,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut snapped = false;
        for _ in 0..10 {
            app.update();
            let harpoon = app.world().get::<GravityHarpoon>(harpoon_entity).unwrap();
            if harpoon.target_entity.is_none() {
                let bombs = app.world().resource::<Events<BombardmentEvent>>();
                if bombs.len() > 0 {
                    snapped = true;
                    break;
                }
            }
        }

        assert!(
            snapped,
            "The cable snaps, dealing AOE damage via bombardment"
        );
    }

    #[test]
    fn test_gravity_harpoon_power_drain() {
        let mut app = App::new();
        app.add_systems(Update, winch_execution_system);
        app.add_event::<OrbitalDropEvent>();
        app.add_event::<BombardmentEvent>();

        let target_entity = app.world_mut().spawn(OrbitalDebris { amount: 0.0 }).id();

        let harpoon_entity = app
            .world_mut()
            .spawn((
                GravityHarpoon {
                    target_entity: Some(target_entity),
                    winch_progress: 0.0,
                },
                PowerConsumer {
                    demand: 100.0,
                    active: false,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        app.update();

        let harpoon = app.world().get::<GravityHarpoon>(harpoon_entity).unwrap();
        assert_eq!(
            harpoon.winch_progress, 0.0,
            "Unpowered harpoon should not winch"
        );

        app.world_mut()
            .get_mut::<PowerConsumer>(harpoon_entity)
            .unwrap()
            .active = true;
        app.update();

        let harpoon2 = app.world().get::<GravityHarpoon>(harpoon_entity).unwrap();
        assert!(
            harpoon2.winch_progress > 0.0 || harpoon2.target_entity.is_none(),
            "Powered harpoon should winch"
        );
    }
}
