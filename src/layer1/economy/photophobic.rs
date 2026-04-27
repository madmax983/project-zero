use bevy_ecs::prelude::*;

/// A resource that degrades when exposed to light.
#[derive(Component)]
pub struct PhotophobicResource {
    /// Current amount of the resource.
    pub amount: f32,
    /// Rate at which the resource degrades per unit of light intensity.
    pub degradation_rate: f32,
}

/// Tracking component for an entity's light exposure.
#[derive(Component)]
pub struct LightLevel {
    /// Intensity of light exposure (0.0 to 1.0+).
    pub intensity: f32,
}

/// System that updates the LightLevel of photophobic resources based on their GridPosition and the global LightMap.
pub fn update_photophobic_light_level_system(
    light_map: Res<crate::layer1::lighting::LightMap>,
    mut query: Query<
        (&mut LightLevel, &crate::layer1::map::GridPosition),
        With<PhotophobicResource>,
    >,
) {
    for (mut light_level, pos) in query.iter_mut() {
        if pos.x >= 0 && pos.y >= 0 {
            light_level.intensity = light_map.get(pos.x as u32, pos.y as u32);
        } else {
            light_level.intensity = 0.0;
        }
    }
}

/// System that degrades photophobic resources when exposed to light.
pub fn photophobic_degradation_system(mut query: Query<(&mut PhotophobicResource, &LightLevel)>) {
    for (mut resource, light) in query.iter_mut() {
        if light.intensity > 0.0 {
            resource.amount -= resource.degradation_rate * light.intensity;
            if resource.amount < 0.0 {
                resource.amount = 0.0;
            }
        }
    }
}

/// System that causes pops working in darkness to gain stress faster.
pub fn mining_in_dark_stress_system(
    mut query: Query<(
        &crate::layer1::pop::Pop,
        &crate::layer1::map::GridPosition,
        &mut crate::layer1::stress::StressTracker,
        &crate::layer1::utility_types::PopAction,
    )>,
    light_map: Res<crate::layer1::lighting::LightMap>,
) {
    for (_pop, pos, mut stress, action) in query.iter_mut() {
        if action.current == crate::layer1::utility_types::ActionType::Work {
            let light_level = light_map.get(pos.x as u32, pos.y as u32);
            if light_level < 0.1 {
                stress.accumulated_stress += 5.0; // Significantly faster than normal
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::lighting::LightMap;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::utility_types::{ActionType, PopAction};
    use bevy_app::{App, Update};

    #[test]
    fn test_shadow_glass_degrades_in_light() {
        let mut app = App::new();
        app.add_systems(Update, photophobic_degradation_system);

        let resource = app
            .world_mut()
            .spawn((
                PhotophobicResource {
                    amount: 10.0,
                    degradation_rate: 1.0,
                },
                LightLevel { intensity: 1.0 },
            ))
            .id();

        app.update();

        let res = app.world().get::<PhotophobicResource>(resource).unwrap();
        assert!(res.amount < 10.0, "The resource amount decreases over time");
    }

    #[test]
    fn test_shadow_glass_stable_in_darkness() {
        let mut app = App::new();
        app.add_systems(Update, photophobic_degradation_system);

        let resource = app
            .world_mut()
            .spawn((
                PhotophobicResource {
                    amount: 10.0,
                    degradation_rate: 1.0,
                },
                LightLevel { intensity: 0.0 },
            ))
            .id();

        app.update();

        let res = app.world().get::<PhotophobicResource>(resource).unwrap();
        assert_eq!(res.amount, 10.0, "The resource amount does not decrease");
    }

    #[test]
    fn test_pops_mining_in_dark_gain_stress() {
        let mut app = App::new();
        app.add_systems(Update, mining_in_dark_stress_system);

        let mut light_map = LightMap::new(10, 10);
        // Darkness
        light_map.set(5, 5, 0.0);
        app.insert_resource(light_map);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                StressTracker {
                    accumulated_stress: 0.0,
                },
                PopAction {
                    current: ActionType::Work,
                    current_utility: 1.0,
                    ticks_committed: 1,
                },
            ))
            .id();

        app.update();

        let stress = app.world().get::<StressTracker>(pop).unwrap();
        assert!(
            stress.accumulated_stress > 0.0,
            "Pop's stress increases significantly faster than normal"
        );
    }
}
