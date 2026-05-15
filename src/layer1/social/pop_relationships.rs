use crate::layer1::actions::AssignedTo;
use crate::layer1::social::morale::Morale;
use crate::layer1::social::{AffinityChange, Relationships};
use bevy::prelude::*;

#[derive(Event)]
pub struct ShiftEndEvent;

pub fn update_workplace_relationships_system(
    mut events: EventReader<ShiftEndEvent>,
    query: Query<(Entity, &AssignedTo)>,
    mut affinity_events: EventWriter<AffinityChange>,
) {
    for _ in events.read() {
        // Collect all workers and their locations
        let mut location_map = std::collections::HashMap::new();
        for (entity, assigned_to) in query.iter() {
            location_map
                .entry(assigned_to.entity)
                .or_insert_with(Vec::new)
                .push(entity);
        }

        // Build relationships based on shared locations
        for (_, workers) in location_map.iter() {
            if workers.len() > 1 {
                for &worker in workers {
                    for &peer in workers {
                        if worker != peer {
                            affinity_events.send(AffinityChange {
                                source: worker,
                                target: peer,
                                amount: 1.0,
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn calculate_relationship_mood_buff_system(
    time: Res<Time>,
    mut query: Query<(&Relationships, &mut Morale)>,
) {
    for (rels, mut morale) in query.iter_mut() {
        let mut total_buff = 0.0;
        for (_, &score) in rels.affinities.iter() {
            if score.0 >= 50.0 {
                total_buff += 0.5; // Friendly buff
            } else if score.0 <= -50.0 {
                total_buff -= 0.5; // Rivalry debuff
            }
        }

        let new_value = morale.value + (total_buff * time.delta_secs());
        morale.value = new_value.clamp(0.0, 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::AssignmentType;
    use crate::layer1::entities::pop::Pop;

    #[test]
    fn test_pops_develop_relationships_working_together() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ShiftEndEvent>();
        app.add_event::<AffinityChange>();
        app.add_systems(Update, update_workplace_relationships_system);

        let _entity1 = app
            .world_mut()
            .spawn((
                Pop,
                AssignedTo {
                    entity: Entity::from_raw(1),
                    assignment_type: AssignmentType::FarmWorker,
                },
                Relationships::default(),
            ))
            .id();

        let _entity2 = app
            .world_mut()
            .spawn((
                Pop,
                AssignedTo {
                    entity: Entity::from_raw(1),
                    assignment_type: AssignmentType::FarmWorker,
                }, // Same location
                Relationships::default(),
            ))
            .id();

        // Act
        app.world_mut().send_event(ShiftEndEvent);
        app.update();

        // Assert
        let events = app.world().resource::<Events<AffinityChange>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();

        assert_eq!(emitted.len(), 2); // 1->2 and 2->1
        assert_eq!(emitted[0].amount, 1.0);
    }

    #[test]
    fn test_high_relationships_boost_mood() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, calculate_relationship_mood_buff_system);
        app.insert_resource(Time::<()>::default());
        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(std::time::Duration::from_secs(1));

        let entity2 = app.world_mut().spawn(Pop).id();

        let bonds = Relationships::with_affinity(entity2, 50.0, 0);

        let entity1 = app
            .world_mut()
            .spawn((
                Pop,
                bonds,
                Morale {
                    value: 50.0,
                    ..Default::default()
                },
            ))
            .id();

        // Act
        app.update();

        // Assert
        let mood = app.world().get::<Morale>(entity1).unwrap();
        assert!(mood.value > 50.0);
    }
}
