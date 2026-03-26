# Sub-Orbital Harpoon Logistics

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** Forget delicate dropships. You deliver bulk goods from orbit by literally shooting them into the planet's crust with massive kinetic harpoons.
**Mechanic:** A cheap, high-throughput method of moving resources from Layer 2 orbital stations to Layer 1 colonies. Massive payload spikes are fired into the ground. They cause minor localized earthquakes and destroy the tile they land on, but instantly deliver thousands of tons of raw materials.

## 2. Dependencies
- `102-orbital-drop-logistics.md`
- `153-geological-instability.md`

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_harpoon_impact_delivers_resources() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_harpoon_impact_system);

        let impact_event = app.world.spawn((
            HarpoonImpact {
                position: Vec2::new(10.0, 10.0),
                resource_type: ResourceType::Metal,
                amount: 1000,
            },
        )).id();

        // Act
        app.update();

        // Assert
        // Check if resource crate/pile was created at the impact site
        let mut found_resource = false;
        for (pos, resource) in app.world.query::<(&Transform, &ResourcePile)>().iter(&app.world) {
            if pos.translation.x == 10.0 && pos.translation.y == 10.0 {
                assert_eq!(resource.amount, 1000);
                assert_eq!(resource.resource_type, ResourceType::Metal);
                found_resource = true;
            }
        }
        assert!(found_resource);
        assert!(app.world.get_entity(impact_event).is_none()); // Event consumed
    }

    #[test]
    fn test_harpoon_impact_destroys_tile_and_causes_quake() {
        // Arrange
        let mut app = App::new();
        app.add_event::<QuakeEvent>();
        app.add_systems(Update, process_harpoon_impact_system);

        // Setup terrain
        app.world.spawn((
            Tile { position: Vec2::new(10.0, 10.0) },
            Terrain::Grass,
        ));

        app.world.spawn((
            HarpoonImpact {
                position: Vec2::new(10.0, 10.0),
                resource_type: ResourceType::Metal,
                amount: 1000,
            },
        ));

        // Act
        app.update();

        // Assert
        let quake_events = app.world.resource::<Events<QuakeEvent>>();
        assert!(quake_events.get_reader().len(&quake_events) > 0);

        // Verify terrain changed to Crater
        let mut found_crater = false;
        for (pos, terrain) in app.world.query::<(&Tile, &Terrain)>().iter(&app.world) {
            if pos.position.x == 10.0 && pos.position.y == 10.0 {
                assert_eq!(*terrain, Terrain::Crater);
                found_crater = true;
            }
        }
        assert!(found_crater);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum ResourceType {
    Metal,
    Stone,
}

#[derive(Component)]
pub struct ResourcePile {
    pub resource_type: ResourceType,
    pub amount: u32,
}

#[derive(Component)]
pub struct HarpoonImpact {
    pub position: Vec2,
    pub resource_type: ResourceType,
    pub amount: u32,
}

#[derive(Component)]
pub struct Tile {
    pub position: Vec2,
}

#[derive(Component, PartialEq, Debug)]
pub enum Terrain {
    Grass,
    Crater,
}

#[derive(Event)]
pub struct QuakeEvent {
    pub epicenter: Vec2,
    pub magnitude: f32,
}

pub fn process_harpoon_impact_system(
    mut commands: Commands,
    impacts: Query<(Entity, &HarpoonImpact)>,
    mut tiles: Query<(&Tile, &mut Terrain)>,
    mut quake_events: EventWriter<QuakeEvent>,
) {
    for (entity, impact) in impacts.iter() {
        // Spawn resources
        commands.spawn((
            ResourcePile {
                resource_type: impact.resource_type.clone(),
                amount: impact.amount,
            },
            Transform::from_xyz(impact.position.x, impact.position.y, 0.0),
        ));

        // Destroy tile
        for (tile, mut terrain) in tiles.iter_mut() {
            if tile.position == impact.position {
                *terrain = Terrain::Crater;
            }
        }

        // Trigger quake
        quake_events.send(QuakeEvent {
            epicenter: impact.position,
            magnitude: 5.0, // Arbitrary standard harpoon magnitude
        });

        // Cleanup impact event
        commands.entity(entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Impact processing should verify building destruction on the target tile.
- Re-use the existing `SeismicEvent` or `QuakeEvent` from `153-geological-instability.md`.
- `HarpoonImpact` should probably be an `Event` rather than a spawned entity component, to align with Bevy's event-driven architecture for transient occurrences.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Harpoon delivery creates resources, turns the tile into a crater, and triggers a quake.

## 7. Technical Guidance
- Remember to properly destroy any buildings located at the exact impact `Vec2`.
- Accuracy mechanics should be implemented in Layer 2—the ground layer just receives the `HarpoonImpact` event at its final landing coordinate.

## 8. Questions
*Builder: add questions here if spec is unclear.*
