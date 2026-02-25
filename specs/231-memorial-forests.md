# 231 Memorial Forests

## 1. Overview

"A graveyard that breathes. Your ancestors are not stone; they are the shade you sit under."

This feature introduces a cultural and ecological alternative to simple corpse disposal. Instead of a grave or cremation, a Pop can be "planted" with a sapling. The resulting tree becomes a `MemorialTree`, retaining the identity of the deceased. These trees grow faster when "tended" by relatives and serve as high-value beauty/morale objects. However, cutting them down (intentionally or accidentally) causes severe "Desecration" stress to the colony, especially to relatives.

This adds tension between sustainable lumber (harvesting the dead) and sacred memory (preserving the grove).

## 2. Dependencies

- [x] `019` Forestry System (Base tree mechanics)
- [x] `047` Pop Relationships (Relatives tracking)
- [x] `031` Pop Morale (Stress/Mood effects)
- [x] `221` Organic Recycling (Alternative corpse use)

## 3. RED Phase: Tests First

These tests define the required behavior. They must be written in `tests/layer1/memorial_forests.rs` and fail initially.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_burial_creates_memorial_tree() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, burial_system);

        let corpse = app.world.spawn((
            Corpse { name: "Miner Bob".to_string(), ..default() },
            GridPosition { x: 5, y: 5 },
        )).id();

        let sapling_item = app.world.spawn(ItemType::Sapling).id();

        // Act: Trigger burial action
        app.world.spawn(BurialAction {
            target_corpse: corpse,
            target_position: GridPosition { x: 10, y: 10 },
            used_item: sapling_item,
        });
        app.update();

        // Assert
        // Corpse should be despawned (consumed)
        assert!(app.world.get_entity(corpse).is_none());

        // Memorial Tree should exist at target
        let tree_query = app.world.query::<(&MemorialTree, &GridPosition)>();
        let (memorial, pos) = tree_query.single(&app.world);

        assert_eq!(memorial.deceased_name, "Miner Bob");
        assert_eq!(*pos, GridPosition { x: 10, y: 10 });
    }

    #[test]
    fn test_memorial_tree_growth_bonus() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, (memorial_growth_system, tending_system));

        let tree = app.world.spawn((
            MemorialTree { deceased_name: "Bob".to_string(), tended_recently: false },
            Tree { growth_progress: 0.0, ..default() },
        )).id();

        // Act 1: Normal growth
        app.update();
        let progress_1 = app.world.get::<Tree>(tree).unwrap().growth_progress;

        // Act 2: Tending action
        app.world.spawn(TendingAction { target_tree: tree });
        app.update(); // Apply tending
        app.update(); // Apply growth

        let tree_comp = app.world.get::<Tree>(tree).unwrap();
        let memorial = app.world.get::<MemorialTree>(tree).unwrap();

        // Assert
        assert!(memorial.tended_recently);
        assert!(tree_comp.growth_progress > progress_1 * 1.5, "Tended trees should grow significantly faster");
    }

    #[test]
    fn test_desecration_stress() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, desecration_system);

        let deceased_id = Entity::from_raw(123); // Mock ID
        let relative = app.world.spawn((
            Pop { name: "Alice".to_string(), ..default() },
            Relationships { relatives: vec![deceased_id], ..default() },
            Mood { stress: 0.0, ..default() },
        )).id();

        let unrelated = app.world.spawn((
            Pop { name: "Stranger".to_string(), ..default() },
            Relationships { relatives: vec![], ..default() },
            Mood { stress: 0.0, ..default() },
        )).id();

        let memorial_tree = app.world.spawn((
            MemorialTree { deceased_name: "Bob".to_string(), original_entity_id: Some(deceased_id), ..default() },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Act: Destroy the tree
        app.world.send_event(TreeFelledEvent { entity: memorial_tree });
        app.update();

        // Assert
        let alice_mood = app.world.get::<Mood>(relative).unwrap();
        let stranger_mood = app.world.get::<Mood>(unrelated).unwrap();

        assert!(alice_mood.stress > 50.0, "Relative should suffer massive stress");
        assert!(stranger_mood.stress > 10.0, "Colony should suffer minor stress from desecration");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/memorial.rs

#[derive(Component, Default)]
pub struct MemorialTree {
    pub deceased_name: String,
    pub original_entity_id: Option<Entity>, // To link with relationships
    pub tended_recently: bool,
}

#[derive(Event)]
pub struct TreeFelledEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct TendingAction {
    pub target_tree: Entity,
}

pub fn burial_system(
    mut commands: Commands,
    query: Query<(Entity, &BurialAction)>,
    mut corpse_query: Query<&Corpse>,
) {
    for (action_entity, action) in query.iter() {
        if let Ok(corpse) = corpse_query.get(action.target_corpse) {
            // Spawn Memorial Tree
            commands.spawn((
                MemorialTree {
                    deceased_name: corpse.name.clone(),
                    original_entity_id: Some(action.target_corpse), // Preserving ID might be tricky if entity despawns, usually store UUID or similar
                    tended_recently: false,
                },
                Tree { growth_progress: 0.1, ..default() }, // Starts as sapling
                action.target_position,
                // Visuals...
            ));

            // Despawn corpse and action
            commands.entity(action.target_corpse).despawn();
            commands.entity(action_entity).despawn();
        }
    }
}

pub fn desecration_system(
    mut events: EventReader<TreeFelledEvent>,
    tree_query: Query<&MemorialTree>,
    mut pop_query: Query<(&Relationships, &mut Mood)>,
) {
    for event in events.read() {
        if let Ok(memorial) = tree_query.get(event.entity) {
            for (rels, mut mood) in pop_query.iter_mut() {
                if let Some(deceased_id) = memorial.original_entity_id {
                    if rels.relatives.contains(&deceased_id) {
                        mood.stress += 50.0;
                        // Log "Desecration" thought
                    } else {
                        mood.stress += 5.0; // General unease
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Relationship Persistence**: Since Entities are generational, relying on `Entity` ID for dead pops is risky if IDs are reused. Refactor `Relationships` to use a `PersonId` (UUID) or similar persistent identifier that the `MemorialTree` can also store.
- **Tending AI**: Add a `TendMemorial` utility AI action. Relatives should have a high weight for this action.
- **Forestry Integration**: Ensure `MemorialTree` also has the standard `Tree` component so it interacts with the `Forestry` system (can be designated for chopping).
- **UI**: The Inspector should show "This is the resting place of [Name]" when selecting the tree.

## 6. Acceptance Criteria

- [ ] `burial_system` correctly consumes a Corpse and Sapling to create a `MemorialTree`.
- [ ] `MemorialTree` retains the name of the deceased.
- [ ] `tending_system` significantly boosts growth rate of `MemorialTree`.
- [ ] Cutting down a `MemorialTree` triggers `TreeFelledEvent`.
- [ ] `desecration_system` applies high stress to relatives and low stress to others upon felling.
- [ ] Test coverage > 85%.

## 7. Technical Guidance

- Use the existing `Tree` component from `src/layer1/forestry.rs` to ensure the memorial behaves like a tree (visuals, physics).
- Add `MemorialTree` as a marker component.
- In `forestry.rs` or wherever `chop_tree` logic exists, ensure it emits `TreeFelledEvent` if the target has `MemorialTree`.
- For the `Relationships` check, if `PersonId` doesn't exist yet, simply comment the need for it in the TODO or use the `Entity` for now (assuming strictly increasing IDs or handle reuse carefully).

## 8. Questions

- *Builder: Should Memorial Trees die of old age?*
  - *Architect: Yes, but they should last longer than normal trees. When they die naturally, they leave a "Stump" that still functions as a memorial but with lower beauty.*
