use crate::layer1::morale::Morale;
use crate::layer1::pop::PopDied;
use crate::layer1::resources::ColonyResources;
use crate::layer1::unrest::{Unrest, UnrestModifier};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct NobleScion {
    pub allowance: f32,
}

pub fn income_system(
    mut resources: ResMut<ColonyResources>,
    query: Query<(&NobleScion, Option<&Morale>)>,
) {
    for (scion, morale) in query.iter() {
        let multiplier = morale.map_or(1.0, |m| m.value);
        resources.add_credits(scion.allowance * multiplier);
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
    fn test_noble_allowance_scales_with_morale() {
        use crate::layer1::morale::Morale;
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let mut morale = Morale::default();
        morale.value = 0.5;

        // Spawn Noble
        world.spawn((
            Pop,
            Traits(std::collections::HashSet::from([Trait::Noble])),
            NobleScion { allowance: 100.0 },
            morale,
        ));

        // Run monthly tick (mocked)
        let mut schedule = Schedule::default();
        schedule.add_systems(income_system);
        schedule.run(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.credits, 50.0);
    }

    #[test]
    fn test_noble_refuses_work() {
        use crate::layer1::utility_eval_types::{PopEvalData, UtilityAIBuffer, WorldContext};
        use crate::layer1::needs::Needs;
        use crate::layer1::map::GridPosition;
        use crate::layer1::utility_types::PopAction;
        use crate::layer1::utility_ai::evaluate_single_pop;
        use crate::layer1::day_night::DayNightCycle;
        use crate::layer1::taboo::TabooState;
        use crate::layer1::zone::ZoneGrid;

        let buffer = UtilityAIBuffer::default();
        let resources = ColonyResources::default();
        let cycle = DayNightCycle::default();
        let taboo = TabooState::default();
        let zone_grid = ZoneGrid::new(1, 1);
        let context = WorldContext {
            resources: &resources,
            cycle: &cycle,
            taboo: &taboo,
            factions: None,
            zone_grid: &zone_grid,
            temperature_grid: None,
        };

        let traits = Traits(std::collections::HashSet::from([Trait::Noble]));

        let data = PopEvalData {
            entity: Entity::PLACEHOLDER,
            pos: GridPosition { x: 0, y: 0 },
            needs: Needs::default(),
            weights: crate::layer1::utility_types::UtilityWeights::default(),
            action: PopAction::default(),
            equipment: None,
            carrying: None,
            carrying_item: None,
            carrying_item_type: None,
            mental_state: None,
            drafted: None,
            faction_member: None,
            penal_labor: None,
            breakdown: None,
            traits: Some(traits),
            stress: 0.0,
            hobby_type: None,
            chemical_state: None,
            is_memetic_carrier: false,
            health: None,
            job: None,
            insulation: 0.0,
        };

        let (action_type, _, _) = evaluate_single_pop(&buffer, &data, &context);

        assert_ne!(action_type, crate::layer1::utility_types::ActionType::Work);
        assert_ne!(action_type, crate::layer1::utility_types::ActionType::Repair);
        assert_ne!(action_type, crate::layer1::utility_types::ActionType::FetchTool);
        assert_ne!(action_type, crate::layer1::utility_types::ActionType::BuryCorpse);
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
