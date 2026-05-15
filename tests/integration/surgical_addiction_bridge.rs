#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::biology::addiction::{check_self_surgery_system, SurgicalAddiction};
    use scale::layer1::core::chronicle::AddChronicleEvent;
    use scale::layer1::health::Health;
    use scale::layer1::inventory::{Inventory, InventoryItem};
    use scale::layer1::items::ItemType;
    use scale::layer1::pop::Pop;

    #[test]
    fn test_self_surgery_emits_chronicle_event() {
        let mut app = bevy_app::App::new();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(bevy_app::Update, check_self_surgery_system);

        let mut inventory = Inventory::default();
        let _ = inventory.try_add(InventoryItem {
            item_type: ItemType::Scrap,
            entity: None,
        });

        app.world_mut().spawn((
            Pop,
            SurgicalAddiction {
                craving: 0.0,
                decay_rate: 1.0,
            },
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            inventory,
        ));

        app.update();

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let ev = reader.read(events).next();

        assert!(ev.is_some(), "AddChronicleEvent should be emitted");
        assert!(ev.unwrap().text.contains("self-surgery using scrap metal"));
    }
}
