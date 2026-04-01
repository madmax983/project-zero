use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale {
    pub value: f32,
}

#[derive(Component)]
pub struct SpacecraftAccess {
    pub available: bool,
}

#[derive(Component)]
pub struct AsteroidNode;

#[derive(Component)]
pub struct HermitOutpost {
    pub stolen_goods: f32,
}

#[derive(Component)]
pub struct Deserted;

#[derive(Component)]
pub struct TradeShip {
    pub cargo: f32,
}

#[derive(Event)]
pub struct PopDesertedEvent {
    pub entity: Entity,
}

#[allow(clippy::needless_pass_by_value)]
pub fn process_hermit_desertions(
    mut commands: Commands,
    query_pops: Query<(Entity, &Morale, &SpacecraftAccess), Without<Deserted>>,
    query_asteroids: Query<Entity, (With<AsteroidNode>, Without<HermitOutpost>)>,
    mut events: EventWriter<PopDesertedEvent>,
) {
    let mut available_asteroids: Vec<Entity> = query_asteroids.iter().collect();

    for (pop_entity, morale, access) in query_pops.iter() {
        if morale.value < 20.0 && access.available {
            if let Some(asteroid_entity) = available_asteroids.pop() {
                // Mark Pop as deserted
                commands.entity(pop_entity).insert(Deserted);

                // Create an outpost on the asteroid
                commands.entity(asteroid_entity).insert(HermitOutpost { stolen_goods: 0.0 });

                events.send(PopDesertedEvent { entity: pop_entity });
            }
        }
    }
}

pub fn hermit_theft_system(
    mut query_ships: Query<(&mut TradeShip, &Transform)>,
    mut query_outposts: Query<(&mut HermitOutpost, &Transform)>,
) {
    let theft_radius = 50.0;

    for (mut ship, ship_transform) in query_ships.iter_mut() {
        for (mut outpost, outpost_transform) in query_outposts.iter_mut() {
            let distance = ship_transform.translation.distance(outpost_transform.translation);

            let theft_amount = ship.cargo * 0.01; // Steal 1% of cargo

            if distance < theft_radius && ship.cargo >= theft_amount {
                ship.cargo -= theft_amount;
                outpost.stolen_goods += theft_amount;
                // Once stolen by one outpost, we move on (prevents double dipping per frame for simplicity)
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_deserts_to_become_hermit() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopDesertedEvent>();
        app.add_systems(Update, process_hermit_desertions);

        // A Pop with critically low morale and spacecraft access
        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 10.0 }, // Very low
            SpacecraftAccess { available: true },
        )).id();

        // An available asteroid node
        let asteroid = app.world_mut().spawn((
            AsteroidNode,
            Transform::from_xyz(100.0, 50.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        let hermit_state = app.world().get::<HermitOutpost>(asteroid);
        assert!(hermit_state.is_some(), "Asteroid should now host a Hermit Outpost");

        let pop_state = app.world().get::<Deserted>(pop);
        assert!(pop_state.is_some(), "Pop should be marked as Deserted");
    }

    #[test]
    fn test_hermit_outpost_steals_from_trade_route() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, hermit_theft_system);

        let hermit_outpost = app.world_mut().spawn((
            HermitOutpost { stolen_goods: 0.0 },
            Transform::from_xyz(100.0, 50.0, 0.0),
        )).id();

        let trade_ship = app.world_mut().spawn((
            TradeShip { cargo: 1000.0 },
            Transform::from_xyz(105.0, 50.0, 0.0), // Very close
        )).id();

        // Act
        app.update();

        // Assert
        let updated_ship = app.world().get::<TradeShip>(trade_ship).unwrap();
        let updated_outpost = app.world().get::<HermitOutpost>(hermit_outpost).unwrap();

        assert!(updated_ship.cargo < 1000.0, "Trade ship should have lost cargo to the hermits");
        assert!(updated_outpost.stolen_goods > 0.0, "Hermits should have accumulated stolen goods");
    }

    #[test]
    fn test_high_morale_pop_does_not_desert() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopDesertedEvent>();
        app.add_systems(Update, process_hermit_desertions);

        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 90.0 }, // High
            SpacecraftAccess { available: true },
        )).id();

        let asteroid = app.world_mut().spawn((
            AsteroidNode,
        )).id();

        // Act
        app.update();

        // Assert
        let hermit_state = app.world().get::<HermitOutpost>(asteroid);
        assert!(hermit_state.is_none(), "Asteroid should NOT host an outpost");
        assert!(app.world().get::<Deserted>(pop).is_none(), "Happy Pop should not desert");
    }
}
