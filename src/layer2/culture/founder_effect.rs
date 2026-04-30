use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::psychology::traits::Trait;

#[derive(Component)]
pub struct ColonyShip {
    pub crew_traits: HashMap<Trait, u32>,
}

#[derive(Component)]
pub struct FoundColonyAction {
    pub target_system: u32,
}

#[derive(Component)]
pub struct ColonyCulture {
    pub dominant_trait: Trait,
    pub trait_distribution: HashMap<Trait, u32>,
}

pub fn found_colony_system(
    mut commands: Commands,
    query: Query<(Entity, &ColonyShip, &FoundColonyAction)>,
) {
    for (entity, ship, _) in query.iter() {
        let dominant_trait = ship.crew_traits
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(t, _)| *t)
            .unwrap_or(Trait::HardWorker); // Fallback

        // Spawn the new colony with inherited culture
        commands.spawn((
            ColonyCulture {
                dominant_trait,
                trait_distribution: ship.crew_traits.clone(),
            },
        ));

        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::psychology::traits::Trait;

    #[test]
    fn test_colony_inherits_founder_traits() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, found_colony_system);

        let mut ship_traits = HashMap::new();
        ship_traits.insert(Trait::Volatile, 10);
        ship_traits.insert(Trait::Compassionate, 1);

        let ship_entity = app.world_mut().spawn((
            ColonyShip {
                crew_traits: ship_traits.clone(),
            },
            FoundColonyAction { target_system: 42 },
        )).id();

        // Act
        app.update();

        // Assert
        let mut found_colony = false;
        let mut query = app.world_mut().query::<&ColonyCulture>();
        for colony in query.iter(app.world()) {
            assert_eq!(colony.dominant_trait, Trait::Volatile);
            found_colony = true;
        }
        assert!(found_colony);
        assert!(app.world().get_entity(ship_entity).is_err()); // Ship despawned
    }
}
