use crate::layer1::propaganda::PropagandaBroadcastEvent;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Empire {
    pub name: String,
    pub is_authoritarian: bool,
}

#[derive(Component)]
pub struct DiplomaticRelation {
    pub relation: f32, // -100.0 to 100.0
}

/// A system that catches Propaganda broadcasts and penalizes relations with non-authoritarian empires.
pub fn propaganda_diplomatic_fallout_system(
    mut events: EventReader<PropagandaBroadcastEvent>,
    mut query: Query<(&Empire, &mut DiplomaticRelation)>,
) {
    let broadcast_count = events.read().count();
    if broadcast_count == 0 {
        return;
    }

    for (empire, mut relation) in query.iter_mut() {
        if !empire.is_authoritarian {
            // Apply negative modifier per broadcast (e.g., -5.0)
            relation.relation = (relation.relation - (5.0 * broadcast_count as f32)).max(-100.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::propaganda::{PropagandaBroadcastEvent, PropagandaMonolith};

    #[test]
    fn test_monolith_triggers_diplomatic_penalty() {
        // Arrange: active Propaganda Monolith on Layer 1, neighboring Empire on Layer 3.
        let mut world = World::new();
        world.init_resource::<Events<PropagandaBroadcastEvent>>();

        let monolith = world.spawn(PropagandaMonolith).id();

        let neighbor = world
            .spawn((
                Empire {
                    name: "Democratic Neighbors".to_string(),
                    is_authoritarian: false,
                },
                DiplomaticRelation { relation: 50.0 },
            ))
            .id();

        // Act: advance simulation.
        let mut schedule = Schedule::default();
        schedule.add_systems(propaganda_diplomatic_fallout_system);

        let mut events = world.resource_mut::<Events<PropagandaBroadcastEvent>>();
        events.send(PropagandaBroadcastEvent { entity: monolith });

        schedule.run(&mut world);

        // Assert: Neighboring empire's diplomatic relation with player decreases due to "Cultural Warfare".
        let relation = world.get::<DiplomaticRelation>(neighbor).unwrap();
        assert!(relation.relation < 50.0);
    }
}
