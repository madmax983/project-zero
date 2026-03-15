use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct GlobalFloraHealth {
    pub total_health: f32,
}

#[derive(Event)]
pub struct FloraDamagedEvent {
    pub damage_amount: f32,
}

pub fn sync_empathic_network_system(mut query: Query<(&Traits, &mut StressTracker)>) {
    let mut total_stress = 0.0;
    let mut count = 0;

    // Calculate average
    for (traits, stress) in query.iter() {
        if traits.has(Trait::EmpathicLink) {
            total_stress += stress.accumulated_stress;
            count += 1;
        }
    }

    if count == 0 {
        return;
    }
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
#[path = "biosphere_empathy_tests.rs"]
mod biosphere_empathy_tests;
