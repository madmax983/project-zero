use crate::layer1::health::Dead;
use crate::layer1::inventory::Inventory;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct InheritanceEvent {
    pub deceased: Entity,
    pub items: Vec<crate::layer1::inventory::InventoryItem>,
}

#[derive(Event)]
pub struct OverrideWillEvent {
    pub affected_pops: Vec<Entity>,
}

#[allow(clippy::type_complexity)]
pub fn process_spiteful_will_system(
    query: Query<(Entity, &Traits, &Inventory), (With<Pop>, Added<Dead>)>,
    mut inheritance_events: EventWriter<InheritanceEvent>,
) {
    for (entity, traits, inventory) in query.iter() {
        if traits.has(Trait::Spiteful) && !inventory.items.is_empty() {
            inheritance_events.send(InheritanceEvent {
                deceased: entity,
                items: inventory.items.clone(),
            });
        }
    }
}

pub fn process_override_will_system(
    mut override_events: EventReader<OverrideWillEvent>,
    mut query: Query<&mut Morale, With<Pop>>,
) {
    for event in override_events.read() {
        for entity in &event.affected_pops {
            if let Ok(mut morale) = query.get_mut(*entity) {
                morale.modifiers.push(MoodModifier {
                    label: "Will Overridden".to_string(),
                    value: -0.2,
                    duration: 100,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::health::Dead;
    use crate::layer1::inventory::{Inventory, InventoryItem};
    use crate::layer1::items::ItemType;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};
    use bevy::prelude::*;


    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<InheritanceEvent>();
        app.add_event::<OverrideWillEvent>();
        app.add_systems(
            Update,
            (process_spiteful_will_system, process_override_will_system),
        );
        app
    }

    #[test]
    fn test_spiteful_will_triggers_on_death() {
        let mut app = setup_app();

        // Simulate dying by adding Dead component in an already spawned entity, to trigger Added<Dead> correctly.
        let dead_pop = app
            .world_mut()
            .spawn((
                Pop,
                Traits(1 << (Trait::Spiteful as u8)),
                Inventory {
                    items: vec![InventoryItem {
                        item_type: ItemType::Tool,
                        entity: None,
                    }],
                    capacity: 20,
                },
            ))
            .id();

        // Add Dead component to trigger the event
        app.world_mut().entity_mut(dead_pop).insert(Dead);

        app.update();

        let events = app.world().resource::<Events<InheritanceEvent>>();
        let mut reader = events.get_cursor();
        assert!(
            reader.read(events).len() > 0,
            "Inheritance event should be triggered for spiteful pop"
        );
    }

    #[test]
    fn test_confiscating_loot_causes_unrest() {
        let mut app = setup_app();

        let heir = app.world_mut().spawn((Pop, Morale::default())).id();

        // Simulate the player overriding the will
        app.world_mut()
            .resource_mut::<Events<OverrideWillEvent>>()
            .send(OverrideWillEvent {
                affected_pops: vec![heir],
            });

        app.update();

        let morale = app.world().get::<Morale>(heir).unwrap();
        assert!(
            morale
                .modifiers
                .iter()
                .any(|m| m.label == "Will Overridden"),
            "Heir should be upset by override"
        );
    }
}
