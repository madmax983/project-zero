use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::map::GridPosition;

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
pub struct GlassStructure;

#[derive(Component)]
pub struct Shattered;

pub fn sonic_suppression_system(
    q_turrets: Query<(&SonicTurret, &GridPosition)>,
    mut q_pops: Query<(&GridPosition, &mut Nausea), With<Pop>>,
    mut commands: Commands,
    q_glass: Query<(Entity, &GridPosition), With<GlassStructure>>,
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
        for (entity, glass_pos) in q_glass.iter() {
            let dx = turret_pos.x as f32 - glass_pos.x as f32;
            let dy = turret_pos.y as f32 - glass_pos.y as f32;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= turret.range {
                commands.entity(entity).insert(Shattered);
            }
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
            SonicTurret { range: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = app.world_mut().spawn((
            Pop,
            Nausea { level: 0.0 },
            GridPosition { x: 5, y: 0 },
        )).id();

        let glass = app.world_mut().spawn((
            GlassStructure,
            GridPosition { x: 8, y: 0 },
        )).id();

        app.update();

        let pop_nausea = app.world().get::<Nausea>(pop).unwrap().level;
        assert!(pop_nausea > 0.0, "Pop in range should gain nausea");

        assert!(app.world().get::<Shattered>(glass).is_some(), "Glass in range should be shattered");
    }

    #[test]
    fn test_inactive_turret_has_no_effect() {
        let mut app = App::new();
        app.add_systems(Update, sonic_suppression_system);

        app.world_mut().spawn((
            SonicTurret { range: 10.0, active: false },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = app.world_mut().spawn((
            Pop,
            Nausea { level: 0.0 },
            GridPosition { x: 5, y: 0 },
        )).id();

        let glass = app.world_mut().spawn((
            GlassStructure,
            GridPosition { x: 8, y: 0 },
        )).id();

        app.update();

        let pop_nausea = app.world().get::<Nausea>(pop).unwrap().level;
        assert_eq!(pop_nausea, 0.0, "Inactive turret should not cause nausea");

        assert!(app.world().get::<Shattered>(glass).is_none(), "Inactive turret should not shatter glass");
    }

    #[test]
    fn test_entities_out_of_range_are_unaffected() {
        let mut app = App::new();
        app.add_systems(Update, sonic_suppression_system);

        app.world_mut().spawn((
            SonicTurret { range: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = app.world_mut().spawn((
            Pop,
            Nausea { level: 0.0 },
            GridPosition { x: 15, y: 0 },
        )).id();

        let glass = app.world_mut().spawn((
            GlassStructure,
            GridPosition { x: 20, y: 0 },
        )).id();

        app.update();

        let pop_nausea = app.world().get::<Nausea>(pop).unwrap().level;
        assert_eq!(pop_nausea, 0.0, "Pop out of range should not gain nausea");

        assert!(app.world().get::<Shattered>(glass).is_none(), "Glass out of range should not be shattered");
    }
}
