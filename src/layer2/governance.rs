use bevy_ecs::prelude::*;

use crate::layer1::traits::{Trait, Traits};

#[derive(Component, Debug, Clone)]
pub struct Governor {
    pub pop_entity: Entity,
    pub assigned_at: u64,
}

#[derive(Component, Debug, Clone)]
pub struct GovernorStats {
    pub loyalty: f32,
    pub ambition: f32,
    pub corruption: f32,
}

#[derive(Component, Debug, Clone)]
pub struct PlanetProduction {
    pub base_throughput: f32,
    pub current_throughput: f32,
}

#[derive(Component, Debug, Clone)]
pub struct ProsperityRating(pub f32);

#[derive(Event, Debug, Clone)]
pub struct RebellionEvent {
    pub planet_entity: Entity,
}

pub fn assign_governor(world: &mut World, planet: Entity, pop: Entity) {
    // Basic assignment logic for Spec 241 dependency
    // In a real implementation (Spec 209), this might check eligibility, remove old governor, etc.
    world.entity_mut(planet).insert(Governor {
        pop_entity: pop,
        assigned_at: 0, // Placeholder
    });

    // Add stats to pop so it can be tracked
    if world.get::<GovernorStats>(pop).is_none() {
        world.entity_mut(pop).insert(GovernorStats {
            loyalty: 100.0,
            ambition: 0.0,
            corruption: 0.0,
        });
    }
}

pub fn apply_governor_effects_system(
    mut query: Query<(&Governor, &mut PlanetProduction)>,
    pop_query: Query<&Traits>,
) {
    for (governor, mut production) in query.iter_mut() {
        production.current_throughput = production.base_throughput;
        if let Ok(traits) = pop_query.get(governor.pop_entity) {
            if traits.has(Trait::LogisticsExpert) {
                production.current_throughput *= 1.2; // 20% boost
            }
        }
    }
}

pub fn update_governor_ambition_system(
    query: Query<(&Governor, &ProsperityRating)>,
    mut stats_query: Query<&mut GovernorStats>,
) {
    for (governor, prosperity) in query.iter() {
        if let Ok(mut stats) = stats_query.get_mut(governor.pop_entity) {
            stats.ambition += prosperity.0 * 0.01; // Scale arbitrarily for MVP
        }
    }
}

pub fn check_governor_rebellion_system(
    query: Query<(Entity, &Governor)>,
    mut stats_query: Query<&mut GovernorStats>,
    mut events: EventWriter<RebellionEvent>,
) {
    for (planet_entity, governor) in query.iter() {
        if let Ok(mut stats) = stats_query.get_mut(governor.pop_entity) {
            if stats.ambition >= 100.0 {
                events.send(RebellionEvent { planet_entity });
                stats.ambition = 0.0; // Reset ambition after rebellion to avoid spam
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::Trait;
    use crate::layer2::generation::Planet;
    use bevy_app::Update;

    use crate::layer1::traits::Traits;

    #[test]
    fn test_assign_governor_applies_bonuses() {
        let mut world = World::new();
        // Arrange: Create a Planet and a Pop with a "Logistics Expert" trait. Assign Pop as Governor.
        let mut traits = Traits::default();
        traits.add(Trait::LogisticsExpert);
        let pop = world.spawn((Pop, traits)).id();
        let planet = world
            .spawn((
                Planet,
                PlanetProduction {
                    base_throughput: 1.0,
                    current_throughput: 1.0,
                },
                Governor {
                    pop_entity: pop,
                    assigned_at: 0,
                },
            ))
            .id();

        // Act: Advance simulation.
        let mut schedule = Schedule::new(Update);
        schedule.add_systems(apply_governor_effects_system);
        schedule.run(&mut world);

        // Assert: The Planet receives a bonus to logistics/production throughput.
        let production = world.get::<PlanetProduction>(planet).unwrap();
        assert!(
            production.current_throughput > production.base_throughput,
            "Governor trait should boost production throughput."
        );
    }

    #[test]
    fn test_governor_accumulates_ambition_over_time() {
        let mut world = World::new();
        // Arrange: Assign a Governor to a highly prosperous planet.
        let pop = world
            .spawn((
                Pop,
                GovernorStats {
                    loyalty: 100.0,
                    ambition: 0.0,
                    corruption: 0.0,
                },
            ))
            .id();
        let _planet = world
            .spawn((
                Planet,
                ProsperityRating(10.0),
                Governor {
                    pop_entity: pop,
                    assigned_at: 0,
                },
            ))
            .id();

        // Act: Advance simulation by a long period.
        let mut schedule = Schedule::new(Update);
        schedule.add_systems(update_governor_ambition_system);
        schedule.run(&mut world);

        // Assert: The Governor's "Ambition" stat increases.
        let stats = world.get::<GovernorStats>(pop).unwrap();
        assert!(
            stats.ambition > 0.0,
            "Governor of a prosperous planet should accumulate ambition."
        );
    }

    #[test]
    fn test_high_ambition_triggers_rebellion_event() {
        let mut world = World::new();
        // Arrange: Set a Governor's Ambition to maximum.
        let pop = world
            .spawn((
                Pop,
                GovernorStats {
                    loyalty: 100.0,
                    ambition: 100.0,
                    corruption: 0.0,
                },
            ))
            .id();
        let planet = world
            .spawn((
                Planet,
                Governor {
                    pop_entity: pop,
                    assigned_at: 0,
                },
            ))
            .id();

        // Add event reader
        world.init_resource::<Events<RebellionEvent>>();

        // Act: Advance simulation.
        let mut schedule = Schedule::new(Update);
        schedule.add_systems(check_governor_rebellion_system);
        schedule.run(&mut world);

        // Assert: A "Rebellion" or "Secession" event is generated for that planet.
        let events = world.resource::<Events<RebellionEvent>>();
        let mut reader = events.get_cursor();
        assert!(
            reader.read(events).any(|e| e.planet_entity == planet),
            "High ambition governor should trigger a RebellionEvent."
        );
    }
}
