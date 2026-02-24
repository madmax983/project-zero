use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{MiningProgress, ResourceItem, ResourceType};
use crate::shared::log::MessageLog;

#[derive(Component, Debug, Clone)]
#[allow(missing_docs)]
pub struct OrbitalEvent {
    pub target: GridPosition,
    pub damage: f32,
    pub heat: f32,
}

#[derive(Component, Debug, Clone)]
#[allow(missing_docs)]
pub struct ImpactSite {
    pub scrap_amount: f32,
    pub harvest_difficulty: f32,
}

/// Mines scrap from an impact site.
pub fn mine_scrap(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // 1. Get position and verify ImpactSite
    let (pos, site_entity) = {
        let pos = if let Some(p) = world.get::<GridPosition>(designation_entity) {
            *p
        } else {
            return;
        };

        // Find ImpactSite at pos
        let mut target = None;
        let mut query = world.query::<(Entity, &ImpactSite, &GridPosition)>();
        for (e, _, p) in query.iter(world) {
            if *p == pos {
                target = Some(e);
                break;
            }
        }
        (pos, target)
    };

    let Some(site_entity) = site_entity else { return };

    // 2. Update Progress (on designation)
    // Ensure MiningProgress exists
    if world.get::<MiningProgress>(designation_entity).is_none() {
        world.entity_mut(designation_entity).insert(MiningProgress { current: 0.0, max: 20.0 }); // 20 work for scrap
    }

    let completed = if let Some(mut progress) = world.get_mut::<MiningProgress>(designation_entity) {
        progress.current += work_amount;
        progress.current >= progress.max
    } else { false };

    // 3. Completion
    if completed {
        // Get scrap yield
        let yield_amount = world.get::<ImpactSite>(site_entity).map_or(1.0, |s| s.scrap_amount);

        // Despawn ImpactSite
        world.despawn(site_entity);

        // Spawn Scrap Item
        world.spawn((
            ResourceItem {
                resource_type: ResourceType::Scrap,
                amount: yield_amount,
            },
            pos,
        ));

        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add("Salvaged Scrap from Impact Site.");
        }

        // Remove designation
        world.despawn(designation_entity);
    }
}

#[allow(missing_docs, clippy::cast_sign_loss)]
pub fn impact_system(world: &mut World) {
    let mut events = Vec::new();
    let mut query = world.query::<(Entity, &OrbitalEvent)>();
    for (entity, event) in query.iter(world) {
        events.push((entity, event.target, event.damage, event.heat));
    }

    for (event_entity, target, damage, heat) in events {
        // 1. Destroy/Damage Buildings
        let mut destroyed = Vec::new();
        {
            let mut buildings = world.query::<(Entity, &GridPosition, &mut crate::layer1::structure::Structure)>();
            for (b_entity, pos, mut structure) in buildings.iter_mut(world) {
                if *pos == target {
                    structure.current_hp -= damage;
                    if structure.current_hp <= 0.0 {
                        destroyed.push(b_entity);
                    }
                }
            }
        }
        for e in destroyed {
            world.despawn(e);
        }

        // 2. Deform Terrain
        if let Some(mut grid) = world.get_resource_mut::<crate::layer1::terrain::TerrainGrid>() {
            // Need to handle bounds check or assume grid.set does it (it does)
            // But target.x might be negative (GridPosition is i32).
            if target.x >= 0 && target.y >= 0 {
                grid.set(
                    target.x as usize,
                    target.y as usize,
                    crate::layer1::terrain::TerrainType::Rock,
                );
            }
        }

        // 3. Add Heat
        if let Some(mut temp) = world.get_resource_mut::<crate::layer1::temperature::TemperatureGrid>() {
            temp.add(target.x, target.y, heat);
        }

        // 4. Spawn Scrap (ImpactSite)
        world.spawn((
            ImpactSite { scrap_amount: 50.0, harvest_difficulty: 1.0 },
            target
        ));

        // 5. Cleanup event
        world.despawn(event_entity);
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::orbital_crossfire::{OrbitalEvent, ImpactSite, impact_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::temperature::TemperatureGrid;
    use crate::layer1::building::{Building, BuildingType};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Grass; 100] });
        world.insert_resource(TemperatureGrid::new(10, 10, 20.0));
        world
    }

    #[test]
    fn test_impact_destroys_building() {
        let mut world = setup_world();

        // Spawn a building at (5,5)
        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 5, y: 5 },
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        // Trigger Impact Event at (5,5)
        world.spawn(OrbitalEvent {
            target: GridPosition { x: 5, y: 5 },
            damage: 500.0,
            heat: 1000.0
        });

        // Run system
        impact_system(&mut world);

        // Building should be gone
        assert!(world.get_entity(building).is_err());
    }

    #[test]
    fn test_impact_creates_crater() {
        let mut world = setup_world();
        let mut grid = world.resource_mut::<TerrainGrid>();
        grid.set(5, 5, TerrainType::Grass);

        // Trigger Impact
        world.spawn(OrbitalEvent {
            target: GridPosition { x: 5, y: 5 },
            damage: 100.0,
            heat: 100.0
        });

        impact_system(&mut world);

        let grid = world.resource::<TerrainGrid>();
        // Should be converted to Rock (placeholder for Crater)
        assert_eq!(grid.get(5, 5).unwrap(), TerrainType::Rock);
    }

    #[test]
    fn test_impact_generates_heat() {
        let mut world = setup_world();

        world.spawn(OrbitalEvent {
            target: GridPosition { x: 5, y: 5 },
            damage: 100.0,
            heat: 500.0
        });

        impact_system(&mut world);

        let temp = world.resource::<TemperatureGrid>();
        assert!(temp.get(5, 5) >= 500.0);
    }

    #[test]
    fn test_impact_spawns_harvestable_scrap() {
        let mut world = setup_world();

        world.spawn(OrbitalEvent {
            target: GridPosition { x: 5, y: 5 },
            damage: 100.0,
            heat: 100.0
        });

        impact_system(&mut world);

        // Check for ImpactSite entity with Scrap
        let (_, site, pos) = world.query::<(Entity, &ImpactSite, &GridPosition)>().single(&world);

        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
        assert!(site.scrap_amount > 0.0);
    }

    #[test]
    fn test_mine_scrap() {
        let mut world = setup_world();
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Spawn Impact Site
        world.spawn((
            ImpactSite { scrap_amount: 20.0, harvest_difficulty: 1.0 },
            GridPosition { x: 5, y: 5 }
        ));

        // Spawn Designation
        let designation = world.spawn((
            crate::layer1::designation::Designation { designation_type: crate::layer1::designation::DesignationType::Mine },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Mine
        super::mine_scrap(&mut world, designation, 20.0);

        // ImpactSite should be gone
        let mut site_query = world.query::<&ImpactSite>();
        assert_eq!(site_query.iter(&world).count(), 0);

        // Scrap Item should be spawned
        let mut item_query = world.query::<&crate::layer1::resources::ResourceItem>();
        let item = item_query.iter(&world).next().unwrap();
        assert_eq!(item.resource_type, crate::layer1::resources::ResourceType::Scrap);

        // Designation should be gone
        assert!(world.get_entity(designation).is_err());
    }
}
