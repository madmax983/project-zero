#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::biology::health::Dead;
    use scale::layer1::core::integration::pet_death_bridge_system;
    use scale::layer1::culture::memorial_revolt::{ColonyPet, PetDeathEvent};

    #[test]
    fn test_pet_death_emits_event() {
        let mut app = App::new();
        app.add_event::<PetDeathEvent>();

        let owner = app.world_mut().spawn_empty().id();
        let pet = app.world_mut().spawn(ColonyPet { owner }).id();

        app.add_systems(Update, pet_death_bridge_system);

        // Initial update
        app.update();

        // Kill the pet
        app.world_mut().entity_mut(pet).insert(Dead);

        app.update();

        let events = app.world().resource::<Events<PetDeathEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();

        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].pet_entity, pet);
        assert_eq!(emitted[0].owner_entity, owner);
    }
}
