use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use bevy::prelude::*;

#[derive(Component)]
pub struct HeavyIndustry {
    pub active: bool,
}

#[derive(Component, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Component)]
pub struct Smog {
    pub density: f32,
}

pub fn generate_smog_system(
    mut commands: Commands,
    query: Query<(&HeavyIndustry, Option<&Position>, Option<&GridPosition>)>,
) {
    for (industry, pos, grid_pos) in query.iter() {
        if industry.active {
            let (x, y, z) = if let Some(p) = pos {
                (p.x, p.y, p.z)
            } else if let Some(gp) = grid_pos {
                (gp.x, gp.y, 0)
            } else {
                (0, 0, 0)
            };
            commands.spawn((
                Smog { density: 0.1 },
                Position { x, y, z: z - 1 }, // Simplistic: just spawn below
            ));
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn apply_smog_effects_system(
    mut pops: Query<(&mut Needs, Option<&Position>, Option<&GridPosition>), With<Pop>>,
    smogs: Query<(&Smog, &Position)>,
) {
    for (mut needs, pop_pos, pop_grid_pos) in pops.iter_mut() {
        let (px, py, pz) = if let Some(p) = pop_pos {
            (p.x, p.y, p.z)
        } else if let Some(gp) = pop_grid_pos {
            (gp.x, gp.y, 0)
        } else {
            (0, 0, 0)
        };

        for (smog, smog_pos) in smogs.iter() {
            if px == smog_pos.x && py == smog_pos.y && pz == smog_pos.z {
                needs.oxygen -= smog.density;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smog_generation() {
        let mut app = App::new();
        app.add_systems(Update, generate_smog_system);

        let _industry = app
            .world_mut()
            .spawn((
                HeavyIndustry { active: true },
                Position { x: 5, y: 5, z: -1 },
            ))
            .id();

        app.update();

        // Ensure a smog entity was generated at or below the industry
        let mut smog_query = app.world_mut().query::<(&Smog, &Position)>();
        let mut found_smog = false;
        for (_, pos) in smog_query.iter(app.world()) {
            if pos.z <= -1 {
                found_smog = true;
            }
        }
        assert!(
            found_smog,
            "Smog should be generated at or below active heavy industry"
        );
    }

    #[test]
    fn test_smog_suffocates_pops() {
        let mut app = App::new();
        app.add_systems(Update, apply_smog_effects_system);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    oxygen: 100.0,
                    ..default()
                },
                Position { x: 5, y: 5, z: -2 },
            ))
            .id();

        app.world_mut()
            .spawn((Smog { density: 1.0 }, Position { x: 5, y: 5, z: -2 }));

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(
            needs.oxygen < 100.0,
            "Smog should decrease oxygen for Pops in the same position"
        );
    }
}
