use crate::layer1::architecture::structure::Structure;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Pop, Speed};
use crate::layer1::psychology::stress::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SonicTurret {
    pub range: f32,
    pub active: bool,
}

#[derive(Component, Default)]
pub struct Nausea {
    pub level: f32,
}

#[derive(Component)]
pub struct Glass;

pub fn sonic_suppression_system(
    q_turrets: Query<(&SonicTurret, &GridPosition)>,
    mut q_pops: Query<(&GridPosition, &mut Nausea), With<Pop>>,
    mut q_glass: Query<(&GridPosition, &mut Structure), With<Glass>>,
) {
    for (turret, turret_pos) in q_turrets.iter() {
        if !turret.active {
            continue;
        }
        for (pop_pos, mut nausea) in q_pops.iter_mut() {
            let dx = turret_pos.x as f32 - pop_pos.x as f32;
            let dy = turret_pos.y as f32 - pop_pos.y as f32;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= turret.range {
                nausea.level += 10.0;
            }
        }
        for (glass_pos, mut structure) in q_glass.iter_mut() {
            let dx = turret_pos.x as f32 - glass_pos.x as f32;
            let dy = turret_pos.y as f32 - glass_pos.y as f32;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= turret.range {
                structure.current_hp = 0.0;
            }
        }
    }
}

pub fn apply_nausea_effects_system(
    mut query: Query<(&mut Nausea, Option<&mut StressTracker>, Option<&mut Speed>)>,
) {
    for (mut nausea, mut stress_tracker_opt, mut speed_opt) in query.iter_mut() {
        if nausea.level > 0.0 {
            if let Some(stress_tracker) = stress_tracker_opt.as_deref_mut() {
                stress_tracker.accumulated_stress += nausea.level * 0.1;
            }

            if let Some(speed) = speed_opt.as_deref_mut() {
                let debuff = (1.0 - (nausea.level / 100.0)).clamp(0.1, 1.0);
                speed.current *= debuff;
            }

            nausea.level = (nausea.level - 1.0).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_sonic_suppression_effects() {
        let mut app = App::new();
        app.add_systems(Update, sonic_suppression_system);

        app.world_mut().spawn((
            SonicTurret {
                range: 10.0,
                active: true,
            },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = app
            .world_mut()
            .spawn((Pop, Nausea { level: 0.0 }, GridPosition { x: 5, y: 0 }))
            .id();

        let glass = app
            .world_mut()
            .spawn((
                Glass,
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 8, y: 0 },
            ))
            .id();

        app.update();

        let pop_nausea = app.world().get::<Nausea>(pop).unwrap().level;
        assert!(pop_nausea > 0.0, "Pop in range should gain nausea");

        let glass_hp = app.world().get::<Structure>(glass).unwrap().current_hp;
        assert_eq!(glass_hp, 0.0, "Glass in range should be shattered (hp 0)");
    }

    #[test]
    fn test_inactive_turret_has_no_effect() {
        let mut app = App::new();
        app.add_systems(Update, sonic_suppression_system);

        app.world_mut().spawn((
            SonicTurret {
                range: 10.0,
                active: false,
            },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = app
            .world_mut()
            .spawn((Pop, Nausea { level: 0.0 }, GridPosition { x: 5, y: 0 }))
            .id();

        let glass = app
            .world_mut()
            .spawn((
                Glass,
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 8, y: 0 },
            ))
            .id();

        app.update();

        let pop_nausea = app.world().get::<Nausea>(pop).unwrap().level;
        assert_eq!(pop_nausea, 0.0, "Inactive turret should not cause nausea");

        let glass_hp = app.world().get::<Structure>(glass).unwrap().current_hp;
        assert_eq!(glass_hp, 100.0, "Inactive turret should not shatter glass");
    }

    #[test]
    fn test_apply_nausea_effects_system() {
        let mut app = App::new();
        app.add_systems(Update, apply_nausea_effects_system);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Nausea { level: 20.0 },
                StressTracker {
                    accumulated_stress: 0.0,
                },
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        app.update();

        let nausea = app.world().get::<Nausea>(pop).unwrap().level;
        assert_eq!(nausea, 19.0, "Nausea should decay by 1.0");

        let stress = app
            .world()
            .get::<StressTracker>(pop)
            .unwrap()
            .accumulated_stress;
        assert_eq!(stress, 2.0, "Stress should increase by nausea * 0.1");

        let speed = app.world().get::<Speed>(pop).unwrap().current;
        assert_eq!(
            speed, 0.8,
            "Speed should be reduced by nausea debuff (1.0 - 20/100 = 0.8)"
        );
    }

    #[test]
    fn test_entities_out_of_range_are_unaffected() {
        let mut app = App::new();
        app.add_systems(Update, sonic_suppression_system);

        app.world_mut().spawn((
            SonicTurret {
                range: 10.0,
                active: true,
            },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = app
            .world_mut()
            .spawn((Pop, Nausea { level: 0.0 }, GridPosition { x: 15, y: 0 }))
            .id();

        let glass = app
            .world_mut()
            .spawn((
                Glass,
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                GridPosition { x: 20, y: 0 },
            ))
            .id();

        app.update();

        let pop_nausea = app.world().get::<Nausea>(pop).unwrap().level;
        assert_eq!(pop_nausea, 0.0, "Pop out of range should not gain nausea");

        let glass_hp = app.world().get::<Structure>(glass).unwrap().current_hp;
        assert_eq!(
            glass_hp, 100.0,
            "Glass out of range should not be shattered"
        );
    }
}
