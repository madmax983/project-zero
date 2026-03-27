# 656: Heirloom Items

## 1. Overview
**Layer:** 1
**Fantasy:** "This rifle belonged to the Founder. It never misses."
**Mechanic:** Tools and weapons track their "History" (kills, crafts, years owned). High-history items gain unique names and stat buffs. They persist after the owner dies.
**Emergence:** A desperate fight to recover a "Legendary" medical kit from a burning building, not for the resources, but for the +20% Heal Rate it grants.
**Tension:** Burry the hero with their gear (respect) or strip it for the next recruit (utility)?

## 2. Dependencies
- Layer 1 Items/Inventory System (`Inventory`, `Item`, `Weapon`, `Tool`)
- Layer 1 Combat/Crafting System (events that increase history: `EntityKilledEvent`, `ItemCraftedEvent`)
- Layer 1 Pop Death/Burial System (`PopDeathEvent`, `Corpse`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_item_accumulates_history_on_kill() {
        let mut app = App::new();
        app.add_systems(Update, process_item_history_on_kill);
        app.add_event::<EntityKilledEvent>();

        let weapon_entity = app.world_mut().spawn(ItemHistory { kills: 0, ..default() }).id();
        let killer_entity = app.world_mut().spawn(Inventory { items: vec![weapon_entity] }).id();

        app.world_mut().resource_mut::<Events<EntityKilledEvent>>().send(EntityKilledEvent { killer: killer_entity });

        app.update();

        let history = app.world().get::<ItemHistory>(weapon_entity).unwrap();
        assert_eq!(history.kills, 1, "Item should gain history when owner gets a kill");
    }

    #[test]
    fn test_high_history_item_becomes_heirloom() {
        let mut app = App::new();
        app.add_systems(Update, check_heirloom_status);

        let item = app.world_mut().spawn((
            Item { name: "Rifle".to_string() },
            ItemHistory { kills: 100, years_owned: 5, crafts: 0 },
        )).id();

        app.update();

        let heirloom = app.world().get::<Heirloom>(item);
        assert!(heirloom.is_some(), "Item with 100 kills should become an Heirloom");
        let item_data = app.world().get::<Item>(item).unwrap();
        assert_eq!(item_data.name, "Legendary Rifle", "Heirloom item should gain a unique prefix");
    }

    #[test]
    fn test_heirloom_persists_on_death() {
        let mut app = App::new();
        app.add_systems(Update, process_pop_death_inventory_drop);
        app.add_event::<PopDeathEvent>();

        let heirloom_item = app.world_mut().spawn((Item::default(), Heirloom)).id();
        let pop = app.world_mut().spawn(Inventory { items: vec![heirloom_item] }).id();

        app.world_mut().resource_mut::<Events<PopDeathEvent>>().send(PopDeathEvent { entity: pop });

        app.update();

        let pop_inventory = app.world().get::<Inventory>(pop);
        assert!(pop_inventory.is_none() || pop_inventory.unwrap().items.is_empty(), "Dead pop should drop inventory");

        // Assert heirloom is spawned in world (has Transform/GlobalTransform)
        let heirloom_transform = app.world().get::<Transform>(heirloom_item);
        assert!(heirloom_transform.is_some(), "Heirloom should drop into the world with a Transform");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Item {
    pub name: String,
}

#[derive(Component, Default)]
pub struct ItemHistory {
    pub kills: u32,
    pub crafts: u32,
    pub years_owned: u32,
}

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<Entity>,
}

#[derive(Component)]
pub struct Heirloom;

#[derive(Event)]
pub struct EntityKilledEvent {
    pub killer: Entity,
}

#[derive(Event)]
pub struct PopDeathEvent {
    pub entity: Entity,
}

pub fn process_item_history_on_kill(
    mut events: EventReader<EntityKilledEvent>,
    inventory_query: Query<&Inventory>,
    mut history_query: Query<&mut ItemHistory>,
) {
    for event in events.read() {
        if let Ok(inventory) = inventory_query.get(event.killer) {
            for item_entity in &inventory.items {
                if let Ok(mut history) = history_query.get_mut(*item_entity) {
                    history.kills += 1;
                }
            }
        }
    }
}

pub fn check_heirloom_status(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Item, &ItemHistory), Without<Heirloom>>,
) {
    for (entity, mut item, history) in query.iter_mut() {
        if history.kills >= 100 {
            commands.entity(entity).insert(Heirloom);
            item.name = format!("Legendary {}", item.name);
        }
    }
}

pub fn process_pop_death_inventory_drop(
    mut commands: Commands,
    mut events: EventReader<PopDeathEvent>,
    mut inventory_query: Query<&mut Inventory>,
    heirloom_query: Query<&Heirloom>,
) {
    for event in events.read() {
        if let Ok(mut inventory) = inventory_query.get_mut(event.entity) {
            for item in inventory.items.drain(..) {
                if heirloom_query.contains(item) {
                    // Drop heirloom into world
                    commands.entity(item).insert(Transform::default()).insert(GlobalTransform::default());
                }
            }
            // Other items might be destroyed or generic dropped, minimal impl focuses on Heirloom
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded `100` kills for heirloom status. This should be a configurable threshold in a Bevy resource (`HeirloomConfig`).
- **Logic Improvement**: Renaming items blindly (`"Legendary " + old_name`) can lead to `"Legendary Legendary Rifle"` if not careful. Check existing prefixes or use a dedicated `Prefix` component.
- **System Integration**: The drop logic assumes generic Transforms. It needs to align with the actual map/tile coordinate system used in Layer 1.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Items gain history on kill/craft events.
- [ ] Items reaching the threshold gain the `Heirloom` component and are renamed.
- [ ] Heirloom items drop into the world upon Pop death rather than being despawned.

## 7. Technical Guidance
- `ItemHistory` should be attached to individual Item entities, not the generic Item type resource.
- Ensure only equipped weapons/tools gain history, not everything in the inventory.
- Be careful with `commands.entity(item).insert(Transform)`—make sure the drop location matches the dead Pop's last known location.

## 8. Questions
*Builder: add questions here if spec is unclear.*
