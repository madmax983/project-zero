# 237: Photophobic Resources

## Overview

Introduces **Photophobic Resources** (specifically **Shadow Crystals**) that decay rapidly when exposed to light. These resources are valuable but require careful handling: mining must be done in darkness, and storage must be shielded from light.

This mechanic creates a tension between safety (light prevents monster spawns/stress) and profit (darkness preserves crystals). It also enables emergent disasters where a hull breach or power failure lets light into a storage bay, destroying the stockpile.

## Dependencies

- `018` — Mining Resources (to spawn them)
- `053` — Lighting System (to check light levels)
- `022` — Resource Stockpiles (storage behavior)
- `222` — Paperwork Physicality (precedent for physical items not in ColonyResources)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/photophobic_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::map::{GridPosition, LightMap};
    use crate::layer1::photophobic::{Photophobic, photophobic_decay_system};
    use crate::layer1::inventory::Inventory;

    #[test]
    fn test_photophobic_component_initialization() {
        let p = Photophobic {
            decay_rate: 10.0,
            current_hp: 100.0,
            max_hp: 100.0,
        };
        assert_eq!(p.current_hp, 100.0);
    }

    #[test]
    fn test_decay_in_light() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0); // Full light
        world.insert_resource(light_map);

        let item = world.spawn((
            Item { item_type: ItemType::ShadowCrystal },
            Photophobic {
                decay_rate: 10.0,
                current_hp: 100.0,
                max_hp: 100.0,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(photophobic_decay_system);
        schedule.run(&mut world);

        let p = world.get::<Photophobic>(item).unwrap();
        assert!(p.current_hp < 100.0, "Should decay in light");
        // Expected: 100 - (1.0 * 10.0) = 90
        assert!((p.current_hp - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_no_decay_in_darkness() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 0.0); // Darkness
        world.insert_resource(light_map);

        let item = world.spawn((
            Item { item_type: ItemType::ShadowCrystal },
            Photophobic {
                decay_rate: 10.0,
                current_hp: 100.0,
                max_hp: 100.0,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(photophobic_decay_system);
        schedule.run(&mut world);

        let p = world.get::<Photophobic>(item).unwrap();
        assert_eq!(p.current_hp, 100.0, "Should not decay in darkness");
    }

    #[test]
    fn test_decay_in_inventory() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0); // Light at storage location
        world.insert_resource(light_map);

        // Storage Building (e.g. Stockpile)
        let storage = world.spawn(GridPosition { x: 5, y: 5 }).id();

        // Item inside inventory (no GridPosition, has Parent?)
        // Assuming inventory logic attaches item as child or just referenced
        // For this test, we'll assume the system checks Parent's position
        let item = world.spawn((
            Item { item_type: ItemType::ShadowCrystal },
            Photophobic {
                decay_rate: 10.0,
                current_hp: 100.0,
                max_hp: 100.0,
            },
            Parent(storage), // Bevy hierarchy
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(photophobic_decay_system);
        schedule.run(&mut world);

        let p = world.get::<Photophobic>(item).unwrap();
        assert!(p.current_hp < 100.0, "Should decay if storage is lit");
    }

    #[test]
    fn test_destruction_at_zero_hp() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0);
        world.insert_resource(light_map);

        let item = world.spawn((
            Item { item_type: ItemType::ShadowCrystal },
            Photophobic {
                decay_rate: 100.0, // Instant kill
                current_hp: 10.0,
                max_hp: 100.0,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(photophobic_decay_system);
        schedule.run(&mut world);

        assert!(world.get_entity(item).is_none(), "Item should be destroyed");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components (`src/layer1/photophobic.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::{GridPosition, LightMap};

#[derive(Component, Debug, Clone)]
pub struct Photophobic {
    pub decay_rate: f32, // HP lost per tick at max light
    pub current_hp: f32,
    pub max_hp: f32,
}

pub fn photophobic_decay_system(
    mut commands: Commands,
    light_map: Res<LightMap>,
    mut query: Query<(Entity, &mut Photophobic, Option<&GridPosition>, Option<&Parent>)>,
    parents: Query<&GridPosition, Without<Photophobic>>, // To find parent position
) {
    for (entity, mut photo, pos, parent) in query.iter_mut() {
        // Determine position
        let target_pos = if let Some(p) = pos {
            Some(*p)
        } else if let Some(p) = parent {
            if let Ok(parent_pos) = parents.get(p.get()) {
                Some(*parent_pos)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(p) = target_pos {
            let light_level = light_map.get(p.x as u32, p.y as u32);
            if light_level > 0.0 {
                let decay = photo.decay_rate * light_level;
                photo.current_hp -= decay;

                if photo.current_hp <= 0.0 {
                    commands.entity(entity).despawn_recursive();
                    // Optional: Spawn "Dust" or "Fizz" effect here
                }
            }
        }
    }
}
```

### 2. Update `ItemType` (`src/layer1/items.rs`)

```rust
pub enum ItemType {
    // ... existing
    ShadowCrystal,
}
```

### 3. Integration (`src/simulation.rs`)

Register `photophobic_decay_system` in `Layer1SystemSet::Simulation`.

## REFACTOR Phase: Quality & Design

- **Optimization**: The query checks every photophobic item every tick. If we have thousands, this might be slow. Consider checking every N ticks or using a spatial hash for light updates.
- **Feedback**: Add a visual effect (particle system) when decay happens so the player knows *why* their crystals are vanishing.
- **Stockpiles**: Ensure `Stockpile` logic supports storing specific `ItemType` entities physically. If stockpiles convert items to abstract `ColonyResources`, this mechanic breaks. Use `Inventory` component on stockpiles for `ShadowCrystal` specifically.

## Acceptance Criteria

- [ ] `Photophobic` component exists and tracks HP.
- [ ] Items decay proportionally to light level (Full light = fast decay, Dim light = slow decay).
- [ ] Items inside containers (Inventory) check the container's position for light.
- [ ] Items reach 0 HP and are despawned.
- [ ] `ShadowCrystal` is added to `ItemType`.

## Technical Guidance

- Use `Parent` query to handle items in inventory. This assumes items in inventory are children in the ECS hierarchy. If they are just referenced by ID in a `Vec<Entity>`, the logic needs to change to query the holder.
- Ensure `LightMap` is updated before this system runs.
- `ShadowCrystal` should NOT be added to `ColonyResources` struct to force physical management.
