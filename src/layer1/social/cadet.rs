use crate::layer1::pop::PopDied;
use crate::layer1::resources::ColonyResources;
use crate::layer1::unrest::{Unrest, UnrestModifier};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct NobleScion {
    pub allowance: f32,
}

pub fn income_system(mut resources: ResMut<ColonyResources>, query: Query<&NobleScion>) {
    for scion in query.iter() {
        resources.add_credits(scion.allowance);
    }
}

pub fn death_consequence_system(
    mut events: EventReader<PopDied>,
    query: Query<&NobleScion>,
    mut unrest: ResMut<Unrest>,
) {
    for event in events.read() {
        if query.get(event.entity).is_ok() {
            // Noble died! Penalize unrest heavily.
            unrest.modifiers.push(UnrestModifier {
                value: 0.5,
                duration: 1000,
            });
            unrest.level = (unrest.level + 0.5).clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::pop::{Pop, PopDied};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::social::cadet::{death_consequence_system, income_system, NobleScion};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::unrest::Unrest;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_noble_allowance_income() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Noble
        world.spawn((
            Pop,
            Traits(std::collections::HashSet::from([Trait::Noble])),
            NobleScion { allowance: 100.0 },
        ));

        // Run monthly tick (mocked)
        let mut schedule = Schedule::default();
        schedule.add_systems(income_system);
        schedule.run(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.credits, 100.0);
    }

    #[test]
    fn test_noble_refuses_work() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::farm::Farm;
        use crate::layer1::map::GridPosition;
        use crate::layer1::needs::Needs;
        use crate::layer1::utility_ai::evaluate_actions_system;
        use crate::layer1::utility_types::{ActionType, PopAction, UtilityConfig, UtilityWeights};
        use crate::shared::time::SimulationTime;

        crate::setup::init_task_pools();
        let mut world = World::new();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));

        // Spawn a hungry Noble pop
        let pop = world
            .spawn((
                Pop,
                Traits(std::collections::HashSet::from([Trait::Noble])),
                NobleScion { allowance: 100.0 },
                GridPosition { x: 0, y: 0 },
                Needs {
                    hunger: 0.1,
                    rest: 0.8,
                    leisure: 0.8,
                    hygiene: 0.8,
                }, // Very hungry!
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.2,
                    ticks_committed: 10,
                },
            ))
            .id();

        // Spawn a Farm
        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 3, y: 0 },
            Farm::default(),
        ));

        // The Noble is extremely hungry. But they refuse to work, even at a farm.
        // Or wait: Does ActionType::Farm mean *working* at a farm?
        // Yes. Farming is work. Noble should refuse it, meaning they might rather
        // stay idle or find ready-to-eat food (ActionType::SatisfyHunger).
        // Let's test they don't pick `ActionType::Farm` (which is the work action).

        // Actually, to make them really want to work, let's make them well-fed and give them no other good options.
        let mut needs = world.get_mut::<Needs>(pop).unwrap();
        needs.hunger = 0.9;
        needs.rest = 0.9;
        needs.leisure = 0.9;

        evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        // A regular Pop would choose ActionType::Farm here.
        // A Noble should NOT choose ActionType::Farm.
        assert_ne!(
            action.current,
            ActionType::Farm,
            "Nobles should refuse to work at a farm"
        );
        assert_ne!(
            action.current,
            ActionType::Work,
            "Nobles should refuse general work"
        );
    }

    #[test]
    fn test_noble_death_penalizes_relations() {
        let mut world = World::new();
        world.insert_resource(Events::<PopDied>::default());
        world.insert_resource(Unrest::default());

        let noble = world.spawn((Pop, NobleScion { allowance: 100.0 })).id();

        // Kill them
        world.send_event(PopDied {
            entity: noble,
            name: "Noble Guy".to_string(),
            tick: 100,
            reason: "Mock death".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(death_consequence_system);
        schedule.run(&mut world);

        // Check consequences
        let unrest = world.resource::<Unrest>();
        assert!(!unrest.modifiers.is_empty(), "Should add a modifier");
        assert!(unrest.modifiers[0].value > 0.0);
    }
}
