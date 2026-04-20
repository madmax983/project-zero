use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SubLightTourist {
    pub wealth: u32,
    pub target_event_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tourist_creation() {
        let mut app = bevy_ecs::world::World::new();

        let tourist_entity = app.spawn((
            SubLightTourist {
                wealth: 5000,
                target_event_id: "Supernova_A".to_string(),
            },
        )).id();

        let tourist = app.get::<SubLightTourist>(tourist_entity).unwrap();
        assert_eq!(tourist.wealth, 5000);
        assert_eq!(tourist.target_event_id, "Supernova_A");
    }
}
