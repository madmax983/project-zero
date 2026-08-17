use bevy_ecs::prelude::*;
use std::collections::HashSet;
use crate::layer1::culture::funeral::Corpse;
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::pop::Pop as Soul;

#[derive(Component)]
pub struct Recyclable {
    pub biomass_yield: f32,
}

#[derive(Component)]
pub struct BiomassRecycler {
    pub active: bool,
}

#[derive(Event)]
pub struct RecycleEvent {
    pub target: Entity,
    pub processor: Entity,
}

#[derive(Component, Clone)]
pub struct BiomassRation {
    pub has_grief_taint: bool,
    pub nutritional_value: f32,
}

#[derive(Component)]
pub struct DigestionQueue(pub Vec<BiomassRation>);

#[derive(Resource)]
pub struct PsychologicalConfig {
    pub biomass_taint_stress_penalty: f32,
}

impl Default for PsychologicalConfig {
    fn default() -> Self {
        Self {
            biomass_taint_stress_penalty: 50.0,
        }
    }
}

pub fn process_corpse_recycling_system(
    mut commands: Commands,
    mut events: EventReader<RecycleEvent>,
    corpse_query: Query<&Recyclable, With<Corpse>>,
) {
    let mut processed = HashSet::new();
    for event in events.read() {
        if !processed.insert(event.target) {
            continue;
        }
        if let Ok(recyclable) = corpse_query.get(event.target) {
            commands.entity(event.target).despawn();
            commands.spawn(BiomassRation {
                has_grief_taint: true,
                nutritional_value: recyclable.biomass_yield,
            });
        }
    }
}

pub fn consume_rations_system(
    mut souls_query: Query<(Entity, &mut StressTracker, &mut DigestionQueue), With<Soul>>,
    config: Option<Res<PsychologicalConfig>>,
) {
    let penalty = config.map(|c| c.biomass_taint_stress_penalty).unwrap_or(50.0);
    for (_entity, mut stress, mut queue) in souls_query.iter_mut() {
        if !queue.0.is_empty() {
            let ration = queue.0.remove(0);
            if ration.has_grief_taint {
                stress.accumulated_stress += penalty;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_corpse_recycling_produces_tainted_biomass() {
        let mut world = World::new();

        let corpse = world.spawn((
            Corpse { name: "Test".to_string(), decay: 0.0 },
            Recyclable { biomass_yield: 10.0 },
        )).id();

        let recycler = world.spawn((
            BiomassRecycler { active: true },
        )).id();

        world.init_resource::<Events<RecycleEvent>>();
        let mut events = world.resource_mut::<Events<RecycleEvent>>();
        events.send(RecycleEvent { target: corpse, processor: recycler });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_corpse_recycling_system);
        schedule.run(&mut world);

        let query = world.query::<&BiomassRation>().iter(&world).collect::<Vec<_>>();
        assert_eq!(query.len(), 1, "Should produce one batch of Biomass");
        assert!(query[0].has_grief_taint, "Biomass produced from a Corpse MUST have the grief taint");
    }

    #[test]
    fn test_consuming_tainted_biomass_increases_stress() {
        let mut world = World::new();

        let soul = world.spawn((
            Soul,
            StressTracker { accumulated_stress: 0.0 },
            DigestionQueue(vec![BiomassRation { has_grief_taint: true, nutritional_value: 10.0 }]),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(consume_rations_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(soul).unwrap();
        assert!(stress.accumulated_stress > 0.0, "Consuming tainted biomass must increase the Soul's Stress");
    }

    #[test]
    fn test_recycling_multiple_events_same_corpse() {
        let mut world = World::new();

        let corpse = world.spawn((
            Corpse { name: "Test".to_string(), decay: 0.0 },
            Recyclable { biomass_yield: 10.0 },
        )).id();

        let recycler = world.spawn((
            BiomassRecycler { active: true },
        )).id();

        world.init_resource::<Events<RecycleEvent>>();
        let mut events = world.resource_mut::<Events<RecycleEvent>>();
        events.send(RecycleEvent { target: corpse, processor: recycler });
        events.send(RecycleEvent { target: corpse, processor: recycler }); // Duplicate event

        let mut schedule = Schedule::default();
        schedule.add_systems(process_corpse_recycling_system);
        schedule.run(&mut world);

        let query = world.query::<&BiomassRation>().iter(&world).collect::<Vec<_>>();
        assert_eq!(query.len(), 1, "Should produce only one batch of Biomass despite duplicate events");
    }
}
