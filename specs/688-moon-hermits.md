# 688 - Moon Hermits

## 1. Overview
**Layer:** 2
**Fantasy:** Pops who reject society and escape to live in hollowed-out rocks, creating unpredictable micro-factions on the fringe.
**Mechanic:** Individual Pops or small families with low Morale and access to basic spacecraft can desert the colony. They land on random uninhabitable asteroids in Layer 2, setting up hidden "Hermit" nodes. They survive by stealing trace amounts of power or resources from passing trade ships.

## 2. Dependencies
- `Pop` and `Morale` components (Layer 1)
- `Asteroid` node component (Layer 2)
- `TradeRoute` system (Layer 2)
- `Ship/Spacecraft` access logic

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_pop_deserts_to_become_hermit() {
        // Arrange
        let mut app = App::new();
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

        let mut trade_ship = app.world_mut().spawn((
            TradeShip { cargo: 1000.0 },
            Transform::from_xyz(105.0, 50.0, 0.0), // Very close
        ));

        // Act
        app.update();

        // Assert
        let updated_ship = app.world().get::<TradeShip>(trade_ship.id()).unwrap();
        let updated_outpost = app.world().get::<HermitOutpost>(hermit_outpost).unwrap();

        assert!(updated_ship.cargo < 1000.0, "Trade ship should have lost cargo to the hermits");
        assert!(updated_outpost.stolen_goods > 0.0, "Hermits should have accumulated stolen goods");
    }

    #[test]
    fn test_high_morale_pop_does_not_desert() {
        // Arrange
        let mut app = App::new();
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
```

## 4. GREEN Phase: Minimal Implementation

```rust
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

pub fn process_hermit_desertions(
    mut commands: Commands,
    query_pops: Query<(Entity, &Morale, &SpacecraftAccess), Without<Deserted>>,
    query_asteroids: Query<Entity, (With<AsteroidNode>, Without<HermitOutpost>)>,
) {
    let mut available_asteroids: Vec<Entity> = query_asteroids.iter().collect();

    for (pop_entity, morale, access) in query_pops.iter() {
        if morale.value < 20.0 && access.available {
            if let Some(asteroid_entity) = available_asteroids.pop() {
                // Mark Pop as deserted
                commands.entity(pop_entity).insert(Deserted);

                // Create an outpost on the asteroid
                commands.entity(asteroid_entity).insert(HermitOutpost { stolen_goods: 0.0 });
            }
        }
    }
}

pub fn hermit_theft_system(
    mut query_ships: Query<(&mut TradeShip, &Transform)>,
    mut query_outposts: Query<(&mut HermitOutpost, &Transform)>,
) {
    let theft_radius = 50.0;
    let theft_amount = 10.0; // Flat amount for simplicity in GREEN phase

    for (mut ship, ship_transform) in query_ships.iter_mut() {
        for (mut outpost, outpost_transform) in query_outposts.iter_mut() {
            let distance = ship_transform.translation.distance(outpost_transform.translation);

            if distance < theft_radius && ship.cargo >= theft_amount {
                ship.cargo -= theft_amount;
                outpost.stolen_goods += theft_amount;
                // Once stolen by one outpost, we move on (prevents double dipping per frame for simplicity)
                break;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Theft Logic**: Rather than a flat theft amount, steal a percentage (e.g., 1%) of the cargo so it scales with larger trade ships.
- **Desertion Event**: When a pop deserts, fire a `PopDesertedEvent` to alert the UI and possibly trigger news events.
- **Asteroid Selection**: Right now, they pick the first available asteroid in the query array. Consider finding the closest, or furthest, uninhabitable rock.
- **Resource Depletion Limits**: If the hermits steal too much, the player needs a way to find them. Add a tracking mechanic based on accumulated stolen goods.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops with low morale and spacecraft access successfully become `Deserted` and convert `AsteroidNode`s to `HermitOutpost`s.
- [ ] Trade ships passing close to a `HermitOutpost` have cargo stolen and transferred to the outpost.

## 7. Technical Guidance
- **Cross-Layer Sync**: Pop desertion happens at Layer 1, but the `HermitOutpost` and trade ship interactions are firmly Layer 2 map logic. Ensure coordinate systems or node attachments bridge correctly.
- **Performance**: `hermit_theft_system` currently checks every ship against every outpost. If there are many outposts and ships, this could get slow. Consider spatial partitioning or attaching the theft logic to trade route node traversal instead of every frame distance checks.
- **Visibility**: The `HermitOutpost` should initially be invisible or disguised to the player until a sufficient amount of goods are stolen or a scan action is taken.

## 8. Questions
*Builder: add questions here if spec is unclear.*
