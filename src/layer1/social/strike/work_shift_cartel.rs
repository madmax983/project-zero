use bevy::prelude::*;
use std::collections::HashSet;

/// Component indicating this pop is part of a shift cartel for a specific sector.
#[derive(Component)]
pub struct ShiftCartelMember {
    pub sector_id: u32,
}

/// Indicates the sector and productivity of a worker.
#[derive(Component)]
pub struct Worker {
    pub sector_id: u32,
    pub base_productivity: f32,
}

/// A dynamic multiplier applied to the pop's base work output.
#[derive(Component)]
pub struct ProductivityModifier {
    pub value: f32,
}

/// System that applies sabotage penalties to non-cartel members within the same sector.
pub fn apply_cartel_sabotage_system(
    cartel_query: Query<&ShiftCartelMember>,
    mut worker_query: Query<(Entity, &Worker, &mut ProductivityModifier)>,
) {
    let mut cartel_sectors = HashSet::new();

    for member in cartel_query.iter() {
        cartel_sectors.insert(member.sector_id);
    }

    // Sabotage non-members in cartel sectors
    for (entity, worker, mut modifier) in worker_query.iter_mut() {
        if cartel_sectors.contains(&worker.sector_id) {
            // Check if they are a member
            if cartel_query.get(entity).is_err() {
                // Apply sabotage penalty
                modifier.value = 0.5;
            } else {
                modifier.value = 1.0;
            }
        } else {
            modifier.value = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cartel_sabotages_non_members() {
        let mut app = App::new();
        app.add_systems(Update, apply_cartel_sabotage_system);

        // Cartel member
        app.world_mut().spawn((ShiftCartelMember { sector_id: 1 },));

        // Non-cartel worker in same sector
        let victim = app
            .world_mut()
            .spawn((
                Worker {
                    sector_id: 1,
                    base_productivity: 1.0,
                },
                ProductivityModifier { value: 1.0 },
            ))
            .id();

        app.update();

        let modifier = app.world().get::<ProductivityModifier>(victim).unwrap();
        assert!(modifier.value < 1.0);
    }

    #[test]
    fn test_cartel_does_not_sabotage_members() {
        let mut app = App::new();
        app.add_systems(Update, apply_cartel_sabotage_system);

        // Cartel member who is also a worker
        let member = app.world_mut().spawn((
            ShiftCartelMember { sector_id: 1 },
            Worker { sector_id: 1, base_productivity: 1.0 },
            ProductivityModifier { value: 1.0 },
        )).id();

        app.update();

        let modifier = app.world().get::<ProductivityModifier>(member).unwrap();
        assert_eq!(modifier.value, 1.0);
    }

    #[test]
    fn test_cartel_does_not_sabotage_other_sectors() {
        let mut app = App::new();
        app.add_systems(Update, apply_cartel_sabotage_system);

        // Cartel member in sector 1
        app.world_mut().spawn((
            ShiftCartelMember { sector_id: 1 },
        ));

        // Non-cartel worker in sector 2
        let untouched = app.world_mut().spawn((
            Worker { sector_id: 2, base_productivity: 1.0 },
            ProductivityModifier { value: 1.0 },
        )).id();

        app.update();

        let modifier = app.world().get::<ProductivityModifier>(untouched).unwrap();
        assert_eq!(modifier.value, 1.0);
    }
}
