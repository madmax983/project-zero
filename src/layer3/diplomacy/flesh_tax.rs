use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::memory::{Memories, MemoryType};
use crate::layer2::bombardment::BombardmentEvent;

#[derive(Event)]
pub struct FleshTaxPaymentEvent {
    pub tribute_pops: Vec<Entity>,
}

#[derive(Event)]
pub struct FleshTaxFailedEvent {}

use crate::shared::time::SimulationTime;

pub fn process_flesh_tax_payment(
    mut commands: Commands,
    mut events: EventReader<FleshTaxPaymentEvent>,
    mut query: Query<(Entity, Option<&mut Memories>), With<Pop>>,
    time: Option<Res<SimulationTime>>,
) {
    for event in events.read() {
        for pop_entity in &event.tribute_pops {
            if let Some(mut entity_commands) = commands.get_entity(*pop_entity) {
                entity_commands.despawn();
            }
        }

        let tick = time.as_ref().map_or(0, |t| t.tick);

        for (entity, memory_opt) in query.iter_mut() {
            if !event.tribute_pops.contains(&entity) {
                if let Some(mut memory) = memory_opt {
                    memory.add(MemoryType::FleshTaxTrauma, tick);
                } else {
                    let mut new_memories = Memories::default();
                    new_memories.add(MemoryType::FleshTaxTrauma, tick);
                    commands.entity(entity).insert(new_memories);
                }
            }
        }
    }
}

use crate::layer1::map::GridPosition;

pub fn process_flesh_tax_failure(
    mut events: EventReader<FleshTaxFailedEvent>,
    mut bombardment_writer: EventWriter<BombardmentEvent>,
    pop_query: Query<&GridPosition, With<Pop>>,
) {
    for _event in events.read() {
        // Find a fallback target, like a pop's position, or center of map
        let target_pos = if let Some(pos) = pop_query.iter().next() {
            Vec2::new(pos.x as f32, pos.y as f32)
        } else {
            // Fallback if no pops are found
            Vec2::new(50.0, 50.0)
        };

        bombardment_writer.send(BombardmentEvent {
            target: target_pos,
            damage: 200.0,
            scatter_radius: 10.0,
            blast_radius: 5.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::memory::{MemoryType, Memories};
    use crate::layer2::bombardment::BombardmentEvent;

    #[test]
    fn test_flesh_tax_tribute_removes_pops() {
        let mut app = App::new();
        app.add_event::<FleshTaxPaymentEvent>();
        app.add_systems(Update, process_flesh_tax_payment);

        let pop1 = app.world_mut().spawn((Pop, Name::new("Sacrifice 1"))).id();
        let pop2 = app.world_mut().spawn((Pop, Name::new("Survivor"))).id();

        // Trigger a flesh tax payment
        app.world_mut().send_event(FleshTaxPaymentEvent {
            tribute_pops: vec![pop1],
        });

        app.update();

        assert!(app.world().get_entity(pop1).is_err(), "Pop 1 should be despawned as tribute");
        assert!(app.world().get_entity(pop2).is_ok(), "Pop 2 should survive");
    }

    #[test]
    fn test_flesh_tax_survivors_receive_scarred_memory() {
        let mut app = App::new();
        app.add_event::<FleshTaxPaymentEvent>();
        app.add_systems(Update, process_flesh_tax_payment);
        app.world_mut().init_resource::<Time>();

        let pop1 = app.world_mut().spawn((Pop, Name::new("Sacrifice 1"))).id();
        let pop2 = app.world_mut().spawn((Pop, Name::new("Survivor"))).id();

        app.world_mut().send_event(FleshTaxPaymentEvent {
            tribute_pops: vec![pop1],
        });

        app.update();

        let memory = app.world().get::<Memories>(pop2);
        assert!(memory.is_some());
        let memory_types: Vec<MemoryType> = memory.unwrap().items.iter().map(|m| m.memory_type).collect();
        assert!(memory_types.contains(&MemoryType::FleshTaxTrauma), "Survivor should have trauma memory");
    }

    #[test]
    fn test_flesh_tax_failure_triggers_orbital_bombardment() {
        let mut app = App::new();
        app.add_event::<BombardmentEvent>();
        app.add_event::<FleshTaxFailedEvent>();
        app.add_systems(Update, process_flesh_tax_failure);

        app.world_mut().send_event(FleshTaxFailedEvent {});

        app.update();

        let bombardment_events = app.world().resource::<Events<BombardmentEvent>>();
        assert_eq!(bombardment_events.get_cursor().len(&bombardment_events), 1, "Failure should trigger an orbital bombardment");
    }
}
