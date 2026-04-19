# 1102 - The Cult of the First Ship

## 1. Overview
"The Cult of the First Ship" feature introduces a "Holy Site" mechanic for the original landing ship module. Pops develop an emergent cultural attachment to the physical landing site. Modifying or destroying this site causes massive unrest, while maintaining it provides a strong cultural anchor, creating tension between optimal city layout and cultural heritage.

## 2. Dependencies
- Layer 1 core simulation (Buildings, Pops, Needs/Morale)
- Cultural/Social systems (Unrest, Cultural Anchors)
- Building destruction/relocation mechanics

## 3. RED Phase: Tests First

```rust
// tests/integration/cult_of_first_ship.rs

#[test]
fn test_first_ship_grants_cultural_anchor() {
    // Arrange: Create a colony with a `FirstShip` building entity.
    // Act: Advance the simulation tick for morale/cultural calculations.
    // Assert: Pops near the `FirstShip` or within the colony receive a positive `CulturalAnchor` buff to their morale.
}

#[test]
fn test_destroying_first_ship_causes_unrest() {
    // Arrange: Create a colony with a `FirstShip` and stable Pop morale.
    // Act: Destroy the `FirstShip` entity.
    // Assert: A massive `HolySiteDestroyed` unrest penalty is applied to all Pops.
}

#[test]
fn test_relocating_first_ship_causes_unrest() {
    // Arrange: Create a colony with a `FirstShip`.
    // Act: Relocate the `FirstShip` to a new grid position.
    // Assert: A `HolySiteDesecrated` unrest penalty is applied (perhaps less severe than total destruction, but still significant).
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/culture.rs

#[derive(Component)]
pub struct FirstShip {
    pub is_intact: bool,
}

#[derive(Component)]
pub struct CulturalAnchor {
    pub morale_bonus: f32,
}

#[derive(Component)]
pub struct HolySiteUnrest {
    pub duration: f32,
    pub severity: f32,
}

// System to apply the cultural anchor buff
pub fn first_ship_morale_system(
    first_ship_query: Query<&FirstShip>,
    mut pops: Query<&mut Needs>,
) {
    if first_ship_query.iter().any(|ship| ship.is_intact) {
        for mut needs in pops.iter_mut() {
            // Provide a small passive boost to social/morale needs
            needs.social = (needs.social + 0.1).min(100.0);
        }
    }
}

// System to handle the destruction event
pub fn first_ship_destruction_system(
    mut events: EventReader<BuildingDestroyedEvent>,
    first_ship_query: Query<&FirstShip>,
    mut commands: Commands,
    pops: Query<Entity, With<Population>>,
) {
    for event in events.read() {
        if first_ship_query.get(event.entity).is_ok() {
            // The first ship was destroyed! Apply severe unrest to all Pops.
            for pop_entity in pops.iter() {
                commands.entity(pop_entity).insert(HolySiteUnrest {
                    duration: 100.0,
                    severity: -50.0,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Memories**: The destruction of the First Ship should generate a specific `Memory` for all alive Pops, ensuring the grudge lasts for their lifetime, rather than just a temporary `HolySiteUnrest` component.
- **Relocation Logic**: Implement specific handling for "moving" the building vs "destroying" it, potentially allowing a very expensive "Sacred Relocation" ritual to avoid the unrest penalty.
- **Visuals/UI**: The `FirstShip` should have a distinct map icon or aura indicator so players know not to bulldoze it accidentally.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new cultural systems.
- [ ] The presence of the `FirstShip` grants a measurable morale/social benefit.
- [ ] Destroying the `FirstShip` triggers a severe, measurable unrest penalty across the colony.

## 7. Technical Guidance
- **Entity Setup**: Ensure the initial colony generation script correctly tags the central landing module with the `FirstShip` component.
- **Event Handling**: Rely on existing `BuildingDestroyedEvent` or similar infrastructure rather than polling the existence of the entity every frame.

## 8. Questions
*Builder: add questions here if spec is unclear.*
