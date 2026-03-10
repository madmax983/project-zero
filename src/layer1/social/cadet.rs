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
