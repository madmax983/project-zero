//! Brain Drain Migration
//!
//! Emigration of highly intelligent pops to neighboring factions when their living standards
//! or freedom are significantly higher than the home faction.
use crate::layer1::entities::pop::Pop;
use bevy::prelude::*;

#[derive(Component)]
pub struct Intelligence {
    pub value: u32,
}

#[derive(Component)]
pub struct Freedom {
    pub value: u32,
}

#[derive(Component)]
pub struct LivingStandard {
    pub value: u32,
}

#[derive(Resource)]
pub struct NeighborStats {
    pub freedom: u32,
    pub living_standard: u32,
}

#[derive(Resource)]
pub struct BrainDrainThresholds {
    pub intelligence_required: u32,
    pub freedom_difference: u32,
    pub living_standard_difference: u32,
}

impl Default for BrainDrainThresholds {
    fn default() -> Self {
        Self {
            intelligence_required: 100,
            freedom_difference: 20,
            living_standard_difference: 20,
        }
    }
}

#[derive(Event)]
pub struct EmigrationEvent {
    pub pop_entity: Entity,
}

/// Processes the emigration of highly intelligent pops to neighboring factions.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::entities::pop::Pop;
/// use scale::layer3::diplomacy::brain_drain::{check_brain_drain_migration, Intelligence, Freedom, LivingStandard, EmigrationEvent, NeighborStats};
/// let mut app = App::new();
/// app.add_event::<EmigrationEvent>();
/// app.add_systems(Update, check_brain_drain_migration);
/// let entity = app.world_mut().spawn((Pop, Intelligence { value: 100 }, Freedom { value: 20 }, LivingStandard { value: 30 })).id();
/// app.world_mut().insert_resource(NeighborStats { freedom: 80, living_standard: 90 });
/// app.update();
/// let events = app.world().resource::<Events<EmigrationEvent>>();
/// let mut cursor = events.get_cursor();
/// assert_eq!(cursor.read(events).count(), 1);
/// ```
pub fn check_brain_drain_migration(
    mut commands: Commands,
    query: Query<(Entity, &Intelligence, &Freedom, &LivingStandard), With<Pop>>,
    neighbor_stats: Option<Res<NeighborStats>>,
    thresholds_opt: Option<Res<BrainDrainThresholds>>,
    mut emigration_events: EventWriter<EmigrationEvent>,
) {
    if let Some(neighbor) = neighbor_stats {
        let default_thresholds = BrainDrainThresholds::default();
        let thresholds = if let Some(t) = thresholds_opt.as_ref() {
            t.as_ref()
        } else {
            &default_thresholds
        };

        for (entity, intelligence, freedom, living_standard) in query.iter() {
            if intelligence.value >= thresholds.intelligence_required
                && neighbor.freedom > freedom.value + thresholds.freedom_difference
                && neighbor.living_standard
                    > living_standard.value + thresholds.living_standard_difference
            {
                emigration_events.send(EmigrationEvent { pop_entity: entity });
                commands.entity(entity).despawn();
            }
        }
    }
}

pub struct BrainDrainMigrationPlugin;

impl Plugin for BrainDrainMigrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BrainDrainThresholds>();
        app.add_event::<EmigrationEvent>();
        app.add_systems(Update, check_brain_drain_migration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn assert_emigrated(mut reader: EventReader<EmigrationEvent>) {
        assert_eq!(reader.read().count(), 1);
    }

    fn assert_not_emigrated(mut reader: EventReader<EmigrationEvent>) {
        assert_eq!(reader.read().count(), 0);
    }

    #[test]
    fn test_pop_emigrates_when_neighbor_has_better_standards() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Intelligence { value: 100 },
                Freedom { value: 20 },
                LivingStandard { value: 30 },
                Pop,
            ))
            .id();

        world.insert_resource(NeighborStats {
            freedom: 80,
            living_standard: 90,
        });

        world.init_resource::<Events<EmigrationEvent>>();

        let _ = world.run_system_once(check_brain_drain_migration);

        let _ = world.run_system_once(assert_emigrated);
        assert!(world.get_entity(entity).is_err());
    }

    #[test]
    fn test_pop_stays_when_neighbor_has_lower_standards() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Intelligence { value: 100 },
                Freedom { value: 90 },
                LivingStandard { value: 90 },
                Pop,
            ))
            .id();

        world.insert_resource(NeighborStats {
            freedom: 40,
            living_standard: 50,
        });

        world.init_resource::<Events<EmigrationEvent>>();

        let _ = world.run_system_once(check_brain_drain_migration);

        let _ = world.run_system_once(assert_not_emigrated);
        assert!(world.get_entity(entity).is_ok());
    }

    #[test]
    fn test_brain_drain_migration_plugin() {
        let mut app = App::new();
        app.add_plugins(BrainDrainMigrationPlugin);
        assert!(app.world().get_resource::<BrainDrainThresholds>().is_some());
    }

    #[test]
    fn test_default_thresholds_used_if_no_resource() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Intelligence { value: 100 },
                Freedom { value: 20 },
                LivingStandard { value: 30 },
                Pop,
            ))
            .id();

        world.insert_resource(NeighborStats {
            freedom: 80,
            living_standard: 90,
        });

        world.init_resource::<Events<EmigrationEvent>>();
        // Ensure no BrainDrainThresholds resource
        world.remove_resource::<BrainDrainThresholds>();

        let _ = world.run_system_once(check_brain_drain_migration);

        let _ = world.run_system_once(assert_emigrated);
        assert!(world.get_entity(entity).is_err());
    }

    #[test]
    fn test_custom_thresholds_used() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Intelligence { value: 100 },
                Freedom { value: 20 },
                LivingStandard { value: 30 },
                Pop,
            ))
            .id();

        world.insert_resource(NeighborStats {
            freedom: 80,
            living_standard: 90,
        });

        world.insert_resource(BrainDrainThresholds {
            intelligence_required: 150, // Should not emigrate because it's only 100
            freedom_difference: 20,
            living_standard_difference: 20,
        });

        world.init_resource::<Events<EmigrationEvent>>();

        let _ = world.run_system_once(check_brain_drain_migration);

        let _ = world.run_system_once(assert_not_emigrated);
        assert!(world.get_entity(entity).is_ok());
    }
}
