use bevy_ecs::prelude::*;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::stress::StressTracker;

#[derive(Resource, Default)]
pub struct GlobalFloraHealth {
    pub total_health: f32,
}

#[derive(Event, Default)]
pub struct FloraDamagedEvent {
    pub damage_amount: f32,
}

pub fn sync_empathic_network_system(
    mut query: Query<(&Traits, &mut StressTracker)>,
) {
    let mut total_stress = 0.0;
    let mut count = 0;

    // Calculate average
    for (traits, stress) in query.iter() {
        if traits.has(Trait::EmpathicLink) {
            total_stress += stress.accumulated_stress;
            count += 1;
        }
    }

    if count == 0 { return; }
    let average_stress = total_stress / count as f32;

    // Apply pull towards average
    for (traits, mut stress) in query.iter_mut() {
        if traits.has(Trait::EmpathicLink) {
            // Pull 10% towards the average per tick
            stress.accumulated_stress += (average_stress - stress.accumulated_stress) * 0.1;
        }
    }
}

pub fn handle_flora_damage_empathy_system(
    mut events: EventReader<FloraDamagedEvent>,
    mut query: Query<(&Traits, &mut StressTracker)>,
) {
    let mut total_damage = 0.0;
    for event in events.read() {
        total_damage += event.damage_amount;
    }

    if total_damage > 0.0 {
        for (traits, mut stress) in query.iter_mut() {
            if traits.has(Trait::EmpathicLink) {
                // Flat stress penalty based on flora damage
                stress.accumulated_stress += total_damage * 0.5;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::traits::{Trait, Traits};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(GlobalFloraHealth {
            total_health: 100.0,
        });
        world.init_resource::<Events<FloraDamagedEvent>>();
        world
    }

    #[test]
    fn test_empathic_pops_sync_stress() {
        let mut world = setup_world();

        let pop1 = world
            .spawn((
                Pop,
                Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
                StressTracker {
                    accumulated_stress: 80.0,
                    ..Default::default()
                },
            ))
            .id();

        let pop2 = world
            .spawn((
                Pop,
                Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
                StressTracker {
                    accumulated_stress: 20.0,
                    ..Default::default()
                },
            ))
            .id();

        // Run the system multiple times to simulate the slow pull towards average
        for _ in 0..20 {
            let _ = world.run_system_once(sync_empathic_network_system);
        }

        // Stress should equalize towards the average (50)
        let s1 = world.get::<StressTracker>(pop1).unwrap().accumulated_stress;
        let s2 = world.get::<StressTracker>(pop2).unwrap().accumulated_stress;

        assert!((s1 - 50.0).abs() < 10.0, "s1 is {}", s1);
        assert!((s2 - 50.0).abs() < 10.0, "s2 is {}", s2);
    }

    #[test]
    fn test_flora_damage_spikes_stress() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
                StressTracker {
                    accumulated_stress: 10.0,
                    ..Default::default()
                },
            ))
            .id();

        world.send_event(FloraDamagedEvent {
            damage_amount: 50.0,
        });
        let _ = world
            .run_system_once(crate::layer1::systems::update_event_buffer::<FloraDamagedEvent>);

        let _ = world.run_system_once(handle_flora_damage_empathy_system);

        let stress = world.get::<StressTracker>(pop).unwrap().accumulated_stress;
        // Stress should spike heavily
        assert!(stress > 10.0, "stress is {}", stress);
    }
}
