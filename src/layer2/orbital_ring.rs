use bevy::prelude::*;

#[derive(Resource)]
pub struct OrbitalRing {
    pub active: bool,
    pub rotation_angle: f32, // Simplified representation of the ring's position
}

#[derive(Resource, Default)]
pub struct ShadowBand {
    pub active: bool,
    pub current_angle: f32,
}

pub fn update_shadow_band_system(
    ring_opt: Option<ResMut<OrbitalRing>>,
    shadow_opt: Option<ResMut<ShadowBand>>,
) {
    if let (Some(mut ring), Some(mut shadow)) = (ring_opt, shadow_opt) {
        if ring.active {
            ring.rotation_angle += 0.01; // Arbitrary rotation speed
            shadow.active = true;
            shadow.current_angle = ring.rotation_angle;
        } else {
            shadow.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, update_shadow_band_system);
        app.insert_resource(OrbitalRing {
            active: true,
            rotation_angle: 0.0,
        });
        app.init_resource::<ShadowBand>();
        app
    }

    #[test]
    fn test_shadow_band_rotates() {
        let mut app = setup_app();
        app.update(); // Tick 1

        let ring = app.world().resource::<OrbitalRing>();
        assert!(
            ring.rotation_angle > 0.0,
            "The orbital ring should rotate over time."
        );
    }

    #[test]
    fn test_shadow_band_affects_lighting() {
        let mut app = setup_app();
        app.update();

        let shadow = app.world().resource::<ShadowBand>();
        assert!(
            shadow.active,
            "The shadow band should be active when the ring is built."
        );
    }

    #[test]
    fn test_shadow_band_inactive_when_ring_inactive() {
        let mut app = setup_app();

        // Disable the ring
        let mut ring = app.world_mut().resource_mut::<OrbitalRing>();
        ring.active = false;

        app.update();

        let shadow = app.world().resource::<ShadowBand>();
        assert!(
            !shadow.active,
            "The shadow band should be inactive when the ring is inactive."
        );
    }
}
