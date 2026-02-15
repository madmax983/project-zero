#![allow(clippy::cast_precision_loss)]
use bevy_ecs::prelude::*;
use rand::Rng;

use crate::layer1::beauty::BeautyGrid;
use crate::layer1::building::Building;
use crate::layer1::factions::{FactionMember, FactionState, Factions};
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::{Job, JobType, Pop};

/// Graffiti content type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraffitiType {
    /// Created by unhappy pops. Reduces beauty.
    Tag,
    /// Created by artists/happy pops. Increases beauty.
    Mural,
    /// Created by loyal faction members. Slight beauty increase.
    Propaganda,
}

/// Component representing graffiti on a building/wall.
#[derive(Component, Debug, Clone)]
pub struct Graffiti {
    /// The type of graffiti.
    pub content: GraffitiType,
    /// Time remaining until it fades (in ticks).
    pub decay: f32,
    /// The beauty modifier value.
    pub beauty_mod: f32,
}

/// System to create graffiti based on pop state.
pub fn graffiti_creation_system(
    mut commands: Commands,
    mut pops: Query<(&GridPosition, &Needs, Option<&Job>, Option<&FactionMember>), With<Pop>>,
    buildings: Query<(Entity, &GridPosition), (With<Building>, Without<Graffiti>)>,
    factions: Res<Factions>,
) {
    let mut rng = rand::thread_rng();

    for (pop_pos, needs, job, faction_member) in &mut pops {
        // 1% chance per tick to express oneself
        if rng.gen_bool(0.01) {
            let mut graffiti_type = None;

            // Determine type
            if needs.morale() < 0.2 {
                graffiti_type = Some(GraffitiType::Tag);
            } else if let Some(j) = job {
                if j.job_type == JobType::Artist && needs.morale() > 0.8 {
                    graffiti_type = Some(GraffitiType::Mural);
                }
            }

            if graffiti_type.is_none() {
                if let Some(fm) = faction_member {
                    if let Some(fid) = fm.faction_id {
                        if let Some(faction_data) = factions.get(fid) {
                            if faction_data.state == FactionState::Loyal && needs.morale() > 0.5 {
                                graffiti_type = Some(GraffitiType::Propaganda);
                            }
                        }
                    }
                }
            }

            if let Some(g_type) = graffiti_type {
                // Find adjacent building
                // Naive: check all buildings (optimization needed later)
                // Better: Check buildings within distance 1
                for (b_entity, b_pos) in &buildings {
                    if (pop_pos.x - b_pos.x).abs() + (pop_pos.y - b_pos.y).abs() <= 1 {
                        // Found a target!
                        let (decay, beauty) = match g_type {
                            GraffitiType::Tag => (200.0, -2.0),
                            GraffitiType::Mural => (500.0, 5.0),
                            GraffitiType::Propaganda => (300.0, 1.0),
                        };

                        commands.entity(b_entity).insert(Graffiti {
                            content: g_type,
                            decay,
                            beauty_mod: beauty,
                        });
                        break; // Only one graffiti per tick per pop
                    }
                }
            }
        }
    }
}

/// System to decay graffiti over time.
pub fn graffiti_decay_system(mut commands: Commands, mut query: Query<(Entity, &mut Graffiti)>) {
    for (entity, mut graffiti) in &mut query {
        graffiti.decay -= 1.0;
        if graffiti.decay <= 0.0 {
            commands.entity(entity).remove::<Graffiti>();
        }
    }
}

/// System to apply graffiti beauty effects to the grid.
/// This must run AFTER `update_beauty_grid_system`.
pub fn apply_graffiti_beauty_system(
    mut grid: ResMut<BeautyGrid>,
    query: Query<(&GridPosition, &Graffiti)>,
) {
    for (pos, graffiti) in &query {
        if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
            let current = grid.get(x, y);
            grid.set(x, y, current + graffiti.beauty_mod);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_graffiti_decay() {
        let mut world = World::new();
        let entity = world
            .spawn(Graffiti {
                content: GraffitiType::Tag,
                decay: 1.0,
                beauty_mod: -2.0,
            })
            .id();

        world.run_system_once(graffiti_decay_system).unwrap();

        assert!(world.get::<Graffiti>(entity).is_none());
    }

    #[test]
    fn test_apply_graffiti_beauty() {
        let mut world = World::new();
        let mut grid = BeautyGrid::new(10, 10);
        grid.set(0, 0, 10.0);
        world.insert_resource(grid);

        world.spawn((
            Graffiti {
                content: GraffitiType::Tag,
                decay: 100.0,
                beauty_mod: -5.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        world.run_system_once(apply_graffiti_beauty_system).unwrap();

        let grid = world.resource::<BeautyGrid>();
        assert_eq!(grid.get(0, 0), 5.0); // 10 - 5
    }
}
