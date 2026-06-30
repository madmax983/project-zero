use crate::layer1::architecture::Structure;
use crate::layer1::environment::volatile::ExplosionEvent;
use crate::layer1::resources::MiningEvent;
use crate::layer1::shields::DamageEvent;
use bevy_ecs::prelude::*;

#[derive(Resource, Debug)]
pub struct TectonicStress {
    pub current: f32,
    pub threshold: f32,
    pub dissipation_rate: f32,
}

impl Default for TectonicStress {
    fn default() -> Self {
        Self {
            current: 0.0,
            threshold: 100.0,
            dissipation_rate: 0.1,
        }
    }
}

#[derive(Event)]
pub struct MegaQuakeEvent;

pub fn update_stress_system(
    mut stress: ResMut<TectonicStress>,
    mut mining: EventReader<MiningEvent>,
    mut explosions: EventReader<ExplosionEvent>,
) {
    // 1. Add Stress
    for _ in mining.read() {
        stress.current += 0.5; // Tuning value
    }
    for event in explosions.read() {
        stress.current += event.damage * 0.1;
    }

    // 2. Dissipate
    stress.current = (stress.current - stress.dissipation_rate).max(0.0);
}

pub fn apply_mega_quake_damage_system(
    mut events: EventReader<MegaQuakeEvent>,
    mut damage_writer: EventWriter<DamageEvent>,
    structures: Query<Entity, With<Structure>>,
) {
    for _ in events.read() {
        for entity in structures.iter() {
            damage_writer.send(DamageEvent {
                target: entity,
                amount: 100.0,
                velocity: 50.0,
            });
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct ReliefQuakeEvent {
    pub amount: f32,
}

pub fn trigger_relief_quake_system(
    mut stress: ResMut<TectonicStress>,
    mut events: EventReader<ReliefQuakeEvent>,
) {
    for event in events.read() {
        stress.current = (stress.current - event.amount).max(0.0);
    }
}

pub fn check_quake_system(
    mut stress: ResMut<TectonicStress>,
    mut quake_writer: EventWriter<MegaQuakeEvent>,
    mut geo_events: EventWriter<crate::layer1::geology::GeologicalEvent>,
) {
    if stress.current >= stress.threshold {
        quake_writer.send(MegaQuakeEvent);
        // Dispatch actual damage using GeologicalEvent
        geo_events.send(crate::layer1::geology::GeologicalEvent {
            center: crate::layer1::map::GridPosition { x: 50, y: 50 }, // Approximation since map size is not directly here, or we can just send multiple.
            magnitude: 15.0,                                           // MegaQuake is big
        });
        stress.current = 0.0; // Reset
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use crate::layer1::environment::volatile::ExplosionEvent;
    use crate::layer1::geology::tectonic::{
        apply_mega_quake_damage_system, check_quake_system, update_stress_system, MegaQuakeEvent,
        TectonicStress,
    };
    use crate::layer1::resources::MiningEvent;
    use crate::layer1::shields::DamageEvent;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_mining_increases_stress() {
        let mut world = World::new();
        world.insert_resource(TectonicStress::default());
        world.init_resource::<Events<MiningEvent>>();
        world.init_resource::<Events<ExplosionEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_stress_system);

        // Send mining event
        world.send_event(MiningEvent { amount: 10.0 });

        schedule.run(&mut world);

        let stress = world.resource::<TectonicStress>();
        assert!(stress.current > 0.0);
    }

    #[test]
    fn test_stress_dissipation() {
        let mut world = World::new();
        world.insert_resource(TectonicStress {
            current: 50.0,
            dissipation_rate: 1.0,
            ..Default::default()
        });
        world.init_resource::<Events<MiningEvent>>();
        world.init_resource::<Events<ExplosionEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_stress_system);

        schedule.run(&mut world);

        let stress = world.resource::<TectonicStress>();
        assert_eq!(stress.current, 49.0);
    }

    #[test]
    fn test_mega_quake_trigger() {
        let mut world = World::new();
        world.insert_resource(TectonicStress {
            current: 100.0,
            threshold: 100.0,
            ..Default::default()
        });
        world.init_resource::<Events<MegaQuakeEvent>>();
        world.init_resource::<Events<crate::layer1::geology::GeologicalEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_quake_system);

        schedule.run(&mut world);

        let events = world.resource::<Events<MegaQuakeEvent>>();
        assert!(!events.is_empty());

        let geo_events = world.resource::<Events<crate::layer1::geology::GeologicalEvent>>();
        assert!(!geo_events.is_empty());

        let stress = world.resource::<TectonicStress>();
        assert_eq!(stress.current, 0.0); // Reset after quake
    }

    #[test]
    fn test_mega_quake_damages_structures() {
        let mut world = World::new();
        world.init_resource::<Events<MegaQuakeEvent>>();
        world.init_resource::<Events<DamageEvent>>();

        let structure_entity = world
            .spawn(crate::layer1::architecture::Structure {
                current_hp: 100.0,
                max_hp: 100.0,
            })
            .id();

        world.send_event(MegaQuakeEvent);

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_mega_quake_damage_system);
        schedule.run(&mut world);

        let damage_events = world.resource::<Events<DamageEvent>>();
        let mut reader = damage_events.get_cursor();
        let emitted: Vec<_> = reader.read(damage_events).collect();

        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].target, structure_entity);
        assert_eq!(emitted[0].amount, 100.0);
    }

    #[test]
    fn test_relief_quake_reduces_stress() {
        let mut world = World::new();
        world.insert_resource(TectonicStress {
            current: 50.0,
            ..Default::default()
        });
        world.init_resource::<Events<crate::layer1::geology::tectonic::ReliefQuakeEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::geology::tectonic::trigger_relief_quake_system);

        world.send_event(crate::layer1::geology::tectonic::ReliefQuakeEvent { amount: 20.0 });
        schedule.run(&mut world);

        let stress = world.resource::<TectonicStress>();
        assert_eq!(stress.current, 30.0);
    }

    #[test]
    fn test_relief_quake_cannot_reduce_stress_below_zero() {
        let mut world = World::new();
        world.insert_resource(TectonicStress {
            current: 10.0,
            ..Default::default()
        });
        world.init_resource::<Events<crate::layer1::geology::tectonic::ReliefQuakeEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::geology::tectonic::trigger_relief_quake_system);

        world.send_event(crate::layer1::geology::tectonic::ReliefQuakeEvent { amount: 20.0 });
        schedule.run(&mut world);

        let stress = world.resource::<TectonicStress>();
        assert_eq!(stress.current, 0.0);
    }
}
