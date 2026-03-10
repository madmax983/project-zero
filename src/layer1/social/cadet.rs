use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::unrest::{Unrest, UnrestModifier};
use crate::layer1::pop::PopDied;
use crate::layer1::needs::Needs;

#[derive(Component, Debug)]
pub struct NobleScion {
    pub allowance: f32,
}

use crate::shared::time::SimulationTime;
use crate::layer1::balance::TICKS_PER_YEAR;

pub fn income_system(
    mut resources: ResMut<ColonyResources>,
    query: Query<(&NobleScion, Option<&Needs>)>,
    time: Res<SimulationTime>,
) {
    // Check if it's a new month. 1 Year = 1000 ticks. 1 Month = ~83 ticks.
    let month_ticks = TICKS_PER_YEAR / 12;
    if time.tick > 0 && time.tick % month_ticks == 0 {
        for (scion, needs_opt) in query.iter() {
            // Refactor: Happiness Scaling
            // Unhappy nobles write home complaining, reducing the payment.
            let happiness_factor = needs_opt.map_or(1.0, |n| n.leisure);
            resources.credits += scion.allowance * happiness_factor;
        }
    }
}

pub fn death_consequence_system(
    mut events: EventReader<PopDied>,
    query: Query<&NobleScion>,
    mut unrest: ResMut<Unrest>,
) {
    for event in events.read() {
        if query.get(event.entity).is_ok() {
            // A noble has died! Huge unrest penalty.
            unrest.modifiers.push(UnrestModifier {
                value: 0.5, // +50% Unrest
                #[allow(clippy::cast_possible_truncation)]
                duration: (TICKS_PER_YEAR * 2) as u32, // Lasts a long time
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, PopDied};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::social::cadet::{NobleScion, income_system, death_consequence_system};
    use crate::layer1::unrest::Unrest;
    use crate::shared::time::SimulationTime;
    use crate::layer1::balance::TICKS_PER_YEAR;

    #[test]
    fn test_noble_allowance_income() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SimulationTime { tick: TICKS_PER_YEAR / 12, speed: crate::shared::time::SimSpeed::Normal });

        // Spawn Noble
        world.spawn((
            Pop,
            Traits(std::collections::HashSet::from([Trait::Noble])),
            NobleScion { allowance: 100.0 },
        ));

        // Run monthly tick
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
        world.insert_resource(Unrest {
            level: 0.0,
            modifiers: vec![],
        });

        let noble = world.spawn((
            Pop,
            NobleScion { allowance: 100.0 },
        )).id();

        // Kill them
        world.resource_mut::<Events<PopDied>>().send(PopDied {
            entity: noble,
            name: "Duke FancyPants".to_string(),
            tick: 1,
            reason: "Unknown".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(death_consequence_system);
        schedule.run(&mut world);

        let unrest = world.resource::<Unrest>();
        assert!(!unrest.modifiers.is_empty(), "Should add an unrest modifier");
        assert!(unrest.modifiers[0].value > 0.0, "Modifier should be positive (increase unrest)");
    }
}
