use crate::layer1::husbandry::Tame;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Lithovore;

pub fn simulate_lithovore_metabolism(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Tame), With<Lithovore>>,
) {
    for (entity, mut tame) in query.iter_mut() {
        tame.hunger += 1.0; // Base metabolic rate per tick

        if tame.hunger >= 100.0 {
            // Starvation causes them to go feral
            commands.entity(entity).remove::<Tame>();
            // If they don't have Traits yet, this will insert one
            commands
                .entity(entity)
                .insert(Traits(std::collections::HashSet::from([Trait::Feral])));
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn lithovore_eating_system(
    mut commands: Commands,
    mut grid: Option<ResMut<TerrainGrid>>,
    mut resources: Option<ResMut<ColonyResources>>,
    mut query: Query<
        (
            Entity,
            Option<&mut Tame>,
            Option<&mut Traits>,
            &GridPosition,
        ),
        With<Lithovore>,
    >,
    mut buildings: Query<(
        Entity,
        &GridPosition,
        &mut crate::layer1::structure::Structure,
    )>,
) {
    for (_entity, mut tame_opt, traits_opt, pos) in query.iter_mut() {
        let is_feral = traits_opt
            .as_ref()
            .is_some_and(|t| t.0.contains(&Trait::Feral));

        let mut hunger = 0.0;
        if let Some(tame) = tame_opt.as_mut() {
            hunger = tame.hunger;
        } else if is_feral {
            // Need a way to track feral hunger if Tame is removed.
            // In the RED phase, we used a separate Hunger component.
            // Let's assume Feral creatures eat automatically if they are feral,
            // or we use a separate Hunger component for ferals.
            // For now, if they are feral, they just constantly eat.
            hunger = 100.0;
        }

        if hunger < 5.0 {
            continue; // Not hungry enough to eat
        }

        // Find adjacent cells
        let adjacent = [
            (pos.x - 1, pos.y),
            (pos.x + 1, pos.y),
            (pos.x, pos.y - 1),
            (pos.x, pos.y + 1),
        ];

        let mut ate = false;

        if is_feral {
            // Feral eats buildings
            for (building_ent, b_pos, mut structure) in buildings.iter_mut() {
                if adjacent.contains(&(b_pos.x, b_pos.y)) {
                    structure.current_hp -= 10.0;
                    ate = true;
                    // If building destroyed, we'd despawn it here, but skipping for minimal pass
                    if structure.current_hp <= 0.0 {
                        commands.entity(building_ent).despawn();
                    }
                    break; // Eat one building per tick
                }
            }
        } else {
            // Tame eats rock
            if let Some(grid) = grid.as_mut() {
                for &(ax, ay) in &adjacent {
                    #[allow(clippy::cast_sign_loss)]
                    if ax >= 0
                        && ay >= 0
                        && grid.get(ax as usize, ay as usize) == Some(TerrainType::Rock)
                    {
                        grid.set(ax as usize, ay as usize, TerrainType::Dirt);

                        if let Some(resources) = resources.as_mut() {
                            // Assume we are adding stone, ignoring max capacity limits here
                            resources.stone += 1.0;
                        }
                        ate = true;
                        break;
                    }
                }
            }
        }

        if ate {
            if let Some(tame) = tame_opt.as_mut() {
                tame.hunger -= 20.0;
                if tame.hunger < 0.0 {
                    tame.hunger = 0.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::husbandry::Tame;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::traits::{Trait, Traits};
    use bevy_app::{App, Update};

    #[test]
    fn test_starving_lithovore_goes_feral() {
        let mut app = App::new();

        let _lithovore = app
            .world_mut()
            .spawn((
                Lithovore,
                Tame {
                    hunger: 95.0,
                    produce_timer: 0,
                },
            ))
            .id();

        app.add_systems(Update, simulate_lithovore_metabolism);

        // Advance time enough to trigger starvation (5 ticks)
        for _ in 0..5 {
            app.update();
        }

        let is_feral = app
            .world()
            .entity(lithovore)
            .get::<Traits>()
            .is_some_and(|t| t.0.contains(&Trait::Feral));
        assert!(
            is_feral,
            "Starving lithovore should lose Tame and become Feral"
        );
        assert!(
            !app.world().entity(lithovore).contains::<Tame>(),
            "Starving lithovore should no longer be Tame"
        );
    }

    #[test]
    fn test_lithovore_eats_rock_and_produces_stone_block() {
        // Arrange: A lithovore adjacent to a Rock tile
        let mut app = App::new();
        let res = ColonyResources::default();
        let initial_stone = res.stone; // ColonyResources::default() starts with 5.0 stone
        app.insert_resource(res);

        // Set up grid
        let mut grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        grid.set(1, 1, TerrainType::Rock);
        app.insert_resource(grid);

        let _lithovore = app
            .world_mut()
            .spawn((
                Lithovore,
                Tame {
                    hunger: 50.0,
                    produce_timer: 0,
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        app.add_systems(Update, lithovore_eating_system);
        app.update();

        // Assert: Rock tile becomes Dirt
        let grid = app.world().resource::<TerrainGrid>();
        assert_eq!(grid.get(1, 1), Some(TerrainType::Dirt));

        // Assert: A Stone item is added to inventory
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.stone, initial_stone + 1.0);

        // Assert: Hunger decreases
        let tame = app.world().get::<Tame>(lithovore).unwrap();
        assert!(tame.hunger < 50.0);
    }

    #[test]
    fn test_feral_lithovore_eats_buildings() {
        // Arrange: A Feral lithovore adjacent to a BuildingWall
        let mut app = App::new();

        let building = app
            .world_mut()
            .spawn((
                crate::layer1::building::Building {
                    building_type: crate::layer1::building::BuildingType::Wall,
                },
                GridPosition { x: 1, y: 1 },
                crate::layer1::structure::Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        let _lithovore = app
            .world_mut()
            .spawn((
                Lithovore,
                Traits(std::collections::HashSet::from([Trait::Feral])),
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        app.add_systems(Update, lithovore_eating_system);
        app.update();

        // Assert: Building takes damage
        let structure = app
            .world()
            .get::<crate::layer1::structure::Structure>(building)
            .unwrap();
        assert!(structure.current_hp < 100.0);
    }
}
