#[cfg(test)]
mod tests {
    use crate::layer1::anomalies::{Anomaly, AnomalyType};
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::{mine_rock, ColonyResources, MiningProgress};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_mine_rock_spawns_anomaly_probabilistically() {
        let mut world = World::new();
        // Setup grid
        world.insert_resource(TerrainGrid {
            width: 100,
            height: 100,
            tiles: vec![TerrainType::Rock; 10000],
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::shared::log::MessageLog::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(100, 100));

        // We need to run mine_rock many times to trigger the probability (e.g. 5%)
        // Or refactor mine_rock to accept a seed/config.
        // For this test, we run 1000 times.
        // We use unique positions so we don't hit previously mined dirt tiles.

        let mut anomaly_spawned = false;

        for i in 0..1000 {
            let entity = world
                .spawn((
                    Designation {
                        designation_type: DesignationType::Mine,
                    },
                    MiningProgress {
                        current: 9.0,
                        max: 10.0,
                    },
                    GridPosition {
                        x: i % 100,
                        y: i / 100,
                    },
                ))
                .id();

            // Complete mining
            mine_rock(&mut world, entity, 1.0);

            // Check if Anomaly exists at this position
            // Note: mine_rock despawns the designation. Anomaly is spawned at the same GridPosition.
            // We need to query for Anomaly entities.
            // However, GridPosition is a component on the Anomaly entity.
            let count = world
                .query::<(&Anomaly, &GridPosition)>()
                .iter(&world)
                .count();
            if count > 0 {
                anomaly_spawned = true;
                break;
            }
        }

        assert!(
            anomaly_spawned,
            "Should have spawned at least one anomaly in 1000 mining attempts"
        );
    }

    #[test]
    fn test_spawned_anomaly_has_valid_type() {
        // Setup world to FORCE spawn if possible, or check the one that spawned.
        // If we can't force it, we rely on the loop above or a mock.
        // Assuming we rely on the loop:

        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 100,
            height: 100,
            tiles: vec![TerrainType::Rock; 10000],
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::shared::log::MessageLog::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(100, 100));

        // Mine until anomaly
        let mut anomaly_entity = None;
        for i in 0..1000 {
            let entity = world
                .spawn((
                    Designation {
                        designation_type: DesignationType::Mine,
                    },
                    MiningProgress {
                        current: 10.0,
                        max: 10.0,
                    }, // Instant complete
                    GridPosition {
                        x: i % 100,
                        y: i / 100,
                    },
                ))
                .id();

            mine_rock(&mut world, entity, 10.0);

            if let Some((e, _)) = world.query::<(Entity, &Anomaly)>().iter(&world).next() {
                anomaly_entity = Some(e);
                break;
            }
        }

        if let Some(e) = anomaly_entity {
            let anomaly = world.get::<Anomaly>(e).unwrap();
            // Should be a valid type (Ruins, Geode, etc)
            assert!(matches!(
                anomaly.anomaly_type,
                AnomalyType::Ruins | AnomalyType::Geode | AnomalyType::StrangeFlora
            ));
        } else {
            // It's possible (though unlikely) that 100 tries didn't spawn one.
            // But if the previous test passes, this one should too eventually.
            // To be safe, we might want to panic or warn, but for TDD strictness, panic is fine if we expect it to work.
            panic!("Failed to spawn anomaly for type check in 1000 attempts");
        }
    }
}
