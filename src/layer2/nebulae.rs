use bevy::prelude::*;
use crate::layer2::fleet::{Fleet, FleetHealth as Hull};
use crate::layer2::sensor_ambiguity::Sensors;
use crate::layer2::shielding::OrbitalShield as Shields;
use crate::layer2::navigation::stellar_weather::Fleet as MovementSpeed;

#[cfg(test)]
use std::time::Duration;

#[derive(Component, Debug, Clone, PartialEq)]
pub enum Nebula {
    IonCloud,
    DustCloud,
    ProtoplanetaryDisk,
}

#[derive(Component, Debug, Clone)]
pub struct SpatialVolume {
    pub radius: f32,
}

impl SpatialVolume {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

#[allow(clippy::type_complexity)]
pub struct NebulaPlugin;

impl Plugin for NebulaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, nebula_effects_system);
    }
}

#[allow(clippy::type_complexity)]
pub fn nebula_effects_system(
    time: Res<Time>,
    nebulae: Query<(&Nebula, &SpatialVolume, &Transform)>,
    mut fleets: Query<(
        &Transform,
        Option<&mut Sensors>,
        Option<&mut Shields>,
        Option<&mut MovementSpeed>,
        Option<&mut Hull>,
    ), With<Fleet>>,
) {
    for (fleet_transform, maybe_sensors, maybe_shields, maybe_speed, maybe_hull) in
        fleets.iter_mut()
    {
        let mut in_ion_cloud = false;
        let mut in_dust_cloud = false;
        let mut in_protoplanetary_disk = false;

        for (nebula, volume, nebula_transform) in nebulae.iter() {
            let dist = fleet_transform
                .translation
                .distance(nebula_transform.translation);
            if dist <= volume.radius {
                match nebula {
                    Nebula::IonCloud => in_ion_cloud = true,
                    Nebula::DustCloud => in_dust_cloud = true,
                    Nebula::ProtoplanetaryDisk => in_protoplanetary_disk = true,
                }
            }
        }

        // Apply Ion Cloud effects
        if let Some(mut sensors) = maybe_sensors {
            if in_ion_cloud {
                if sensors.range != 0.0 {
                    sensors.range = 0.0;
                }
            } else if (sensors.range - sensors.base_range).abs() > f32::EPSILON {
                sensors.range = sensors.base_range;
            }
        }
        if let Some(mut shields) = maybe_shields {
            if shields.disabled != in_ion_cloud {
                shields.disabled = in_ion_cloud;
            }
        }

        // Apply Dust Cloud effects
        if let Some(mut speed) = maybe_speed {
            if in_dust_cloud {
                let new_speed = speed.base_speed * 0.5;
                if (speed.current_speed - new_speed).abs() > f32::EPSILON {
                    speed.current_speed = new_speed; // Reduce speed by 50%
                }
            } else if (speed.current_speed - speed.base_speed).abs() > f32::EPSILON {
                speed.current_speed = speed.base_speed;
            }
        }

        // Apply Protoplanetary Disk effects
        if in_protoplanetary_disk {
            if let Some(mut hull) = maybe_hull {
                hull.current -= 10.0 * time.delta_secs(); // Damage over time
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nebula_ion_cloud_disables_sensors() {
        let mut app = bevy::app::App::new();
        app.init_resource::<Time>();
        app.add_systems(Update, nebula_effects_system);

        // Arrange: Spawn an Ion Cloud Nebula region and a Fleet with active sensors
        let _nebula = app.world_mut().spawn((Nebula::IonCloud, SpatialVolume::new(10.0), Transform::default())).id();
        let fleet = app.world_mut().spawn((Fleet, Transform::default(), Sensors::new(100.0), Shields::default())).id();

        // Act: Move the Fleet into the Ion Cloud region
        app.world_mut().entity_mut(fleet).get_mut::<Transform>().unwrap().translation = Vec3::ZERO;
        app.update();

        // Assert: Verify the Fleet's sensor range is reduced to 0 and shields are disabled
        let sensors = app.world().get::<Sensors>(fleet).unwrap();
        let shields = app.world().get::<Shields>(fleet).unwrap();
        assert_eq!(sensors.range, 0.0);
        assert!(shields.is_disabled());
    }

    #[test]
    fn test_nebula_dust_cloud_slows_movement() {
        let mut app = bevy::app::App::new();
        app.init_resource::<Time>();
        app.add_systems(Update, nebula_effects_system);

        // Arrange: Spawn a Dust Cloud Nebula region and a Fleet
        let _nebula = app.world_mut().spawn((Nebula::DustCloud, SpatialVolume::new(10.0), Transform::default())).id();
        let fleet = app.world_mut().spawn((Fleet, Transform::default(), MovementSpeed { base_speed: 10.0, current_speed: 10.0 })).id();

        // Act: Move the Fleet through the Dust Cloud
        app.world_mut().entity_mut(fleet).get_mut::<Transform>().unwrap().translation = Vec3::ZERO;
        app.update();

        // Assert: Verify the Fleet's movement speed is reduced while inside the cloud
        let speed = app.world().get::<MovementSpeed>(fleet).unwrap();
        assert!(speed.current_speed < speed.base_speed);
    }

    #[test]
    fn test_nebula_protoplanetary_disk_hull_damage() {
        let mut app = bevy::app::App::new();
        app.init_resource::<Time>();
        app.add_systems(Update, nebula_effects_system);

        // Arrange: Spawn a Protoplanetary Disk Nebula region and a Fleet
        let _nebula = app.world_mut().spawn((Nebula::ProtoplanetaryDisk, SpatialVolume::new(10.0), Transform::default())).id();
        let fleet = app.world_mut().spawn((Fleet, Transform::default(), Hull { max: 100.0, current: 100.0 })).id();

        // Act: Advance time while the Fleet is in the Disk
        app.world_mut().entity_mut(fleet).get_mut::<Transform>().unwrap().translation = Vec3::ZERO;
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs(1));
        app.update();

        // Assert: Verify the Fleet's hull takes incremental damage over time
        let hull = app.world().get::<Hull>(fleet).unwrap();
        assert!(hull.current < hull.max);
    }
}
