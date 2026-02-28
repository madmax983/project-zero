use crate::layer1::beauty::BeautySource;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct HoloProjector {
    pub active_beauty: f32,
    pub radius: f32,
    pub is_active: bool,
}

#[derive(Event)]
pub struct HologramFailureEvent {
    pub position: GridPosition,
    pub radius: f32,
}

pub fn update_holograms_system(
    mut query: Query<(
        &mut HoloProjector,
        &mut BeautySource,
        &PowerConsumer,
        &GridPosition,
    )>,
    mut events: EventWriter<HologramFailureEvent>,
) {
    for (mut holo, mut beauty_source, power, pos) in query.iter_mut() {
        // Check if powered
        let powered = power.active;

        if powered && !holo.is_active {
            // Turning ON
            holo.is_active = true;
            beauty_source.value = holo.active_beauty;
            beauty_source.radius = holo.radius;
        } else if !powered && holo.is_active {
            // Turning OFF (Failure)
            holo.is_active = false;
            beauty_source.value = 0.0;
            // Radius stays for now or could reset, but value 0.0 effectively disables it in beauty system

            // Trigger shock
            events.send(HologramFailureEvent {
                position: *pos,
                radius: holo.radius,
            });
        }
    }
}

pub fn apply_disillusionment_system(
    mut events: EventReader<HologramFailureEvent>,
    mut pops: Query<(&GridPosition, &mut crate::layer1::morale::Morale)>,
) {
    for event in events.read() {
        for (pop_pos, mut morale) in pops.iter_mut() {
            // Simple distance check
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let dist = pop_pos.distance_chebyshev(event.position);

            #[allow(clippy::cast_sign_loss)]
            if dist <= event.radius as u32 {
                morale.add_modifier(crate::layer1::morale::MoodModifier {
                    label: "Disillusionment".to_string(),
                    value: -0.2,
                    duration: 50,
                });
            }
        }
    }
}
