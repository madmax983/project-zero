#!/usr/bin/perl
use strict;
use warnings;

my $file = 'src/layer1/designation.rs';
open my $in, '<', $file or die "Cannot open $file: $!";
my $content = do { local $/; <$in> };
close $in;

my $new_try_designate_area = <<'REPLACE_END';
fn clamp_area_dimensions(x1: i32, y1: i32, x2: i32, y2: i32) -> (i32, i32, i32, i32) {
    const MAX_DIMENSION: i32 = 50;

    let min_x_raw = x1.min(x2);
    let max_x_raw = x1.max(x2);
    let min_y_raw = y1.min(y2);
    let max_y_raw = y1.max(y2);

    let width = (max_x_raw - min_x_raw).min(MAX_DIMENSION);
    let height = (max_y_raw - min_y_raw).min(MAX_DIMENSION);

    let min_x = min_x_raw;
    let max_x = min_x_raw + width;
    let min_y = min_y_raw;
    let max_y = min_y_raw + height;

    (min_x, max_x, min_y, max_y)
}

fn get_valid_targets_for_tool(world: &World, tool: DesignationType) -> Option<HashSet<(i32, i32)>> {
    match tool {
        DesignationType::Tame => Some(
            world
                .query::<(
                    Entity,
                    &GridPosition,
                    &crate::layer1::fauna::Fauna,
                    Option<&crate::layer1::husbandry::Tame>,
                )>()
                .iter(world)
                .filter_map(|(_, pos, _, tame)| {
                    if tame.is_none() {
                        Some((pos.x, pos.y))
                    } else {
                        None
                    }
                })
                .collect(),
        ),
        DesignationType::ClearFlora => Some(
            world
                .query::<(&GridPosition, &crate::layer1::flora::Flora)>()
                .iter(world)
                .map(|(pos, _)| (pos.x, pos.y))
                .collect(),
        ),
        DesignationType::Cannibalize => Some(
            world
                .query::<(&GridPosition, &crate::layer1::building::Building)>()
                .iter(world)
                .filter_map(|(pos, b)| {
                    if b.building_type == crate::layer1::building::BuildingType::Lander {
                        Some((pos.x, pos.y))
                    } else {
                        None
                    }
                })
                .collect(),
        ),
        DesignationType::CollectSample => {
            let mut targets = HashSet::new();
            for (pos, _) in world
                .query::<(&GridPosition, &crate::layer1::flora::Flora)>()
                .iter(world)
            {
                targets.insert((pos.x, pos.y));
            }
            for (pos, _) in world
                .query::<(&GridPosition, &crate::layer1::fauna::Fauna)>()
                .iter(world)
            {
                targets.insert((pos.x, pos.y));
            }
            Some(targets)
        }
        _ => None,
    }
}

/// Designate all eligible tiles in a rectangle. Returns count of successful designations.
///
/// The rectangle is defined by two corners `(x1, y1)` and `(x2, y2)`. Corners can be
/// given in any order; the function normalizes to min/max internally.
///
/// # Examples
///
/// ```
/// use scale::layer1::designation::{try_designate_area, DesignationType};
/// use scale::layer1::terrain::{TerrainGrid, TerrainType};
/// use scale::layer1::building::OccupiedTiles;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
/// let mut tiles = vec![TerrainType::Grass; 100];
/// tiles[55] = TerrainType::Rock; // (5,5)
/// tiles[56] = TerrainType::Rock; // (6,5)
/// world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
/// world.insert_resource(OccupiedTiles::default());
///
/// assert_eq!(try_designate_area(&mut world, 5, 5, 6, 5, DesignationType::Mine), 2);
/// ```
pub fn try_designate_area(
    world: &mut World,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    tool: DesignationType,
) -> u32 {
    let (min_x, max_x, min_y, max_y) = clamp_area_dimensions(x1, y1, x2, y2);

    let existing_designations: HashSet<(i32, i32)> = world
        .query::<(&Designation, &GridPosition)>()
        .iter(world)
        .map(|(_, pos)| (pos.x, pos.y))
        .collect();

    let valid_targets = get_valid_targets_for_tool(world, tool);
    let mut to_spawn = Vec::new();

    {
        let terrain_grid = world.get_resource::<TerrainGrid>();
        let occupied_tiles = world.get_resource::<OccupiedTiles>();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x < 0 || y < 0 {
                    continue;
                }

                #[allow(clippy::cast_sign_loss)]
                if terrain_grid
                    .is_some_and(|grid| (x as usize) >= grid.width || (y as usize) >= grid.height)
                {
                    continue;
                }

                if existing_designations.contains(&(x, y)) {
                    continue;
                }

                #[allow(clippy::cast_sign_loss)]
                let is_valid = match tool {
                    DesignationType::Mine => terrain_grid
                        .is_some_and(|g| g.get(x as usize, y as usize) == Some(TerrainType::Rock)),
                    DesignationType::Demolish => {
                        occupied_tiles.is_some_and(|o| o.0.contains(&(x, y)))
                    }
                    DesignationType::Chop => terrain_grid
                        .is_some_and(|g| g.get(x as usize, y as usize) == Some(TerrainType::Tree)),
                    DesignationType::Repair | DesignationType::JuryRig => {
                        occupied_tiles.is_some_and(|o| o.0.contains(&(x, y)))
                    }
                    DesignationType::SetZone(_) => true,
                    DesignationType::Tame
                    | DesignationType::ClearFlora
                    | DesignationType::Cannibalize
                    | DesignationType::CollectSample => valid_targets
                        .as_ref()
                        .is_some_and(|targets| targets.contains(&(x, y))),
                    DesignationType::Destroy => {
                        occupied_tiles.is_some_and(|o| o.0.contains(&(x, y)))
                    }
                };

                if is_valid {
                    to_spawn.push((x, y));
                }
            }
        }
    }

    let mut count = 0;
    for (x, y) in to_spawn {
        world.spawn((
            Designation {
                designation_type: tool,
            },
            GridPosition { x, y },
        ));
        count += 1;
    }

    count
}
REPLACE_END

$content =~ s/\#\[allow\(clippy::too_many_lines\)\]\n\s*pub fn try_designate_area.*?count\n\}/$new_try_designate_area/ms
  or die "Could not find try_designate_area to replace";

open my $out, '>', $file or die "Cannot open $file for writing: $!";
print $out $content;
close $out;
