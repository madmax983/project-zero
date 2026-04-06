# Specification: Diplomatic White Elephants (Feature 812)

## 1. Overview
The **Diplomatic White Elephants** feature bridges Layer 3 (Galactic Diplomacy) and Layer 1 (Colony Management). Allied factions will occasionally send "Gifts" that are materialized on the colony map. While these objects provide massive diplomatic prestige and relationship buffs, they carry severe local detriments—they may be radioactive, attract vermin, or consume immense amounts of power/food (e.g., a "Sacred Beast"). Removing, moving, or destroying the gift triggers a severe diplomatic incident. This creates a tension between maintaining high-level galactic standing and managing local resource/safety constraints.

## 2. Dependencies
- `Layer 3 Diplomacy System`
- `Layer 1 Structure/Placement System`
- `Resource Consumption Loop`
- `Tile Properties System`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_white_elephant_grants_diplomatic_buff() {
        let mut app = App::new();
        app.add_systems(Update, apply_diplomatic_buff_system);

        // Arrange
        let faction_entity = app.world_mut().spawn(Faction { prestige: 50 }).id();
        app.world_mut().spawn(DiplomaticGift { faction: faction_entity, prestige_bonus: 50 });

        // Act
        app.update();

        // Assert
        let faction = app.world().get::<Faction>(faction_entity).unwrap();
        assert_eq!(faction.prestige, 100, "The presence of the White Elephant should grant diplomatic prestige");
    }

    #[test]
    fn test_white_elephant_consumes_local_resources() {
        let mut app = App::new();
        app.add_systems(Update, consume_local_resources_system);

        // Arrange
        let colony_storage = app.world_mut().spawn(ColonyStorage { food: 100 }).id();
        app.world_mut().spawn(DiplomaticGift { faction: Entity::PLACEHOLDER, prestige_bonus: 0 })
            .insert(MassiveConsumer { food_per_tick: 50 });
        app.world_mut().insert_resource(GlobalStorageRef(colony_storage));

        // Act
        app.update();

        // Assert
        let storage = app.world().get::<ColonyStorage>(colony_storage).unwrap();
        assert_eq!(storage.food, 50, "The White Elephant should consume massive local resources");
    }

    #[test]
    fn test_destroying_white_elephant_causes_incident() {
        let mut app = App::new();
        app.add_systems(Update, detect_gift_destruction_system);

        // Arrange
        let faction_entity = app.world_mut().spawn(Faction { prestige: 100 }).id();
        let gift = app.world_mut().spawn(DiplomaticGift { faction: faction_entity, prestige_bonus: 50 }).id();

        // Act: Destroy the gift
        app.world_mut().entity_mut(gift).despawn();
        app.update(); // Process despawn and our system

        // Assert
        assert!(!app.world().resource::<Events<DiplomaticIncidentEvent>>().is_empty(), "Destroying the gift should trigger a DiplomaticIncidentEvent");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Faction {
    pub prestige: i32,
}

#[derive(Component)]
pub struct DiplomaticGift {
    pub faction: Entity,
    pub prestige_bonus: i32,
}

#[derive(Component)]
pub struct MassiveConsumer {
    pub food_per_tick: u32,
}

#[derive(Component)]
pub struct ColonyStorage {
    pub food: u32,
}

#[derive(Resource)]
pub struct GlobalStorageRef(pub Entity);

#[derive(Event)]
pub struct DiplomaticIncidentEvent {
    pub faction: Entity,
}

pub fn apply_diplomatic_buff_system(
    mut faction_query: Query<&mut Faction>,
    gift_query: Query<&DiplomaticGift, Added<DiplomaticGift>>
) {
    for gift in gift_query.iter() {
        if let Ok(mut faction) = faction_query.get_mut(gift.faction) {
            faction.prestige += gift.prestige_bonus;
        }
    }
}

pub fn consume_local_resources_system(
    mut storage_query: Query<&mut ColonyStorage>,
    consumer_query: Query<&MassiveConsumer, With<DiplomaticGift>>,
    storage_ref: Option<Res<GlobalStorageRef>>
) {
    if let Some(storage_entity) = storage_ref {
        if let Ok(mut storage) = storage_query.get_mut(storage_entity.0) {
            for consumer in consumer_query.iter() {
                if storage.food >= consumer.food_per_tick {
                    storage.food -= consumer.food_per_tick;
                }
            }
        }
    }
}

pub fn detect_gift_destruction_system(
    mut removed_gifts: RemovedComponents<DiplomaticGift>,
    // In a full implementation we'd need a way to track which faction the removed gift belonged to.
    // For this minimal green phase, we'll assume a global singleton faction or track it differently.
    mut events: EventWriter<DiplomaticIncidentEvent>
) {
    for _ in removed_gifts.read() {
        events.send(DiplomaticIncidentEvent { faction: Entity::PLACEHOLDER });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Tracking faction on despawn:** `RemovedComponents` only yields the `Entity` ID, not its previous component data. Refactor `detect_gift_destruction_system` to use Bevy's observers (`OnRemove` triggers in newer Bevy versions) or maintain a separate `HashMap<Entity, Entity>` tracking table to map the gift back to its parent faction when destroyed.
- **Dynamic Detriments:** Replace the hardcoded `MassiveConsumer` with an array or trait defining different detriments (`RadioactiveEmitter`, `VerminAttractor`, `PowerDrain`) so different factions can send thematic gifts.
- **Graceful Rejection:** Implement a mechanic where a high enough diplomatic skill allows the player to politely decline the gift before it lands on Layer 1.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The gift successfully bridges Layer 1 penalties and Layer 3 buffs.
- [ ] Removing the entity correctly fires the `DiplomaticIncidentEvent`.

## 7. Technical Guidance
- When testing despawns with Bevy `Commands`, remember the `app.update()` required to flush the command queue before asserting the event.
- Ensure `app.world().init_resource::<Events<DiplomaticIncidentEvent>>()` is called during integration tests, or else `EventWriter` insertion will panic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
