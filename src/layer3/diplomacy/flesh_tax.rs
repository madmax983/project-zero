use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::memory::{Memories, MemoryType};
use crate::layer2::bombardment::BombardmentEvent;
use bevy::prelude::*;

#[derive(Event)]
pub struct FleshTaxPaymentEvent {
    pub tribute_pops: Vec<Entity>,
}

#[derive(Event)]
pub struct FleshTaxFailedEvent {}

pub fn process_flesh_tax_payment(
    mut commands: Commands,
    mut events: EventReader<FleshTaxPaymentEvent>,
    mut query: Query<(Entity, Option<&mut Memories>), With<Pop>>,
    time: Res<Time>,
) {
    for event in events.read() {
        for pop_entity in &event.tribute_pops {
            commands.entity(*pop_entity).despawn();
        }

        for (entity, memory_opt) in query.iter_mut() {
            if !event.tribute_pops.contains(&entity) {
                let current_tick = time.elapsed_secs() as u64; // Fallback tick
                if let Some(mut memory) = memory_opt {
                    memory.add(MemoryType::FleshTaxTrauma, current_tick);
                } else {
                    let mut new_memory = Memories::default();
                    new_memory.add(MemoryType::FleshTaxTrauma, current_tick);
                    commands.entity(entity).insert(new_memory);
                }
            }
        }
    }
}

pub fn process_flesh_tax_failure(
    mut events: EventReader<FleshTaxFailedEvent>,
    mut bombardment_writer: EventWriter<BombardmentEvent>,
) {
    for _event in events.read() {
        bombardment_writer.send(BombardmentEvent {
            target: Vec2::ZERO,
            damage: 9999.0,
            scatter_radius: 100.0,
            blast_radius: 10.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::memory::{Memories, MemoryType};
    use crate::layer2::bombardment::BombardmentEvent;

    #[test]
    fn test_flesh_tax_tribute_removes_pops() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin);
        app.add_event::<FleshTaxPaymentEvent>();
        app.add_systems(Update, process_flesh_tax_payment);

        let pop1 = app
            .world_mut()
            .spawn((Pop {}, Name::new("Sacrifice 1")))
            .id();
        let pop2 = app.world_mut().spawn((Pop {}, Name::new("Survivor"))).id();

        // Trigger a flesh tax payment
        app.world_mut().send_event(FleshTaxPaymentEvent {
            tribute_pops: vec![pop1],
        });

        app.update();
        app.update();

        assert!(
            app.world().get_entity(pop1).is_err(),
            "Pop 1 should be despawned as tribute"
        );
        assert!(app.world().get_entity(pop2).is_ok(), "Pop 2 should survive");
    }

    #[test]
    fn test_flesh_tax_survivors_receive_scarred_memory() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin);
        app.add_event::<FleshTaxPaymentEvent>();
        app.add_systems(Update, process_flesh_tax_payment);

        let pop1 = app
            .world_mut()
            .spawn((Pop {}, Name::new("Sacrifice 1")))
            .id();
        let pop2 = app.world_mut().spawn((Pop {}, Name::new("Survivor"))).id();

        app.world_mut().send_event(FleshTaxPaymentEvent {
            tribute_pops: vec![pop1],
        });

        app.update();
        app.update();

        let memories = app.world().get::<Memories>(pop2);
        assert!(memories.is_some());
        let has_trauma = memories
            .unwrap()
            .items
            .iter()
            .any(|m| m.memory_type == MemoryType::FleshTaxTrauma);
        assert!(has_trauma, "Survivor should have trauma memory");
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
        assert_eq!(
            bombardment_events.get_cursor().len(bombardment_events),
            1,
            "Failure should trigger an orbital bombardment"
        );
    }
}
