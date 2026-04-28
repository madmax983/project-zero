# 1246: The Ghost Harvest

## 1. Overview
**Layer:** 1
**Fantasy:** You rely on agriculture, but the plants have begun mimicking the dead.
**Mechanic:** When pops die near agricultural zones, the crops occasionally grow in the shape of the deceased. These "Ghost Crops" yield massive food but drastically reduce the mood of relatives or friends who harvest/eat them.
**Emergence:** A mass casualty event leads to a bumper crop next season, saving the colony from starvation but causing a wave of severe depression and religious cult formation.
**Tension:** Harvest the disturbing bounty to survive, or burn the fields to preserve sanity?

## 2. Dependencies
- Layer 1 Farming / Agricultural zones
- Pop Death Events
- Pop Needs (Mood/Stress)
- Pop Relationships (Relatives/Friends)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ghost_crop_spawns_after_death_near_farm() {
        let mut app = App::new();
        app.add_systems(Update, process_pop_deaths_near_farms);

        // Spawn a farm
        let farm_pos = GridPosition { x: 5, y: 5 };
        app.world.spawn((Building { type_: BuildingType::Farm }, farm_pos));

        // Add a pop death event near the farm
        app.world.insert_resource(Events::<PopDeathEvent>::default());
        let mut events = app.world.get_resource_mut::<Events<PopDeathEvent>>().unwrap();
        events.send(PopDeathEvent {
            position: GridPosition { x: 6, y: 5 },
            deceased_id: Entity::from_raw(1)
        });

        app.update();

        // Check if a GhostCrop component was spawned at the farm
        let mut ghost_crop_query = app.world.query::<&GhostCrop>();
        let mut found = false;
        for crop in ghost_crop_query.iter(&app.world) {
            if crop.mimicked_pop == Entity::from_raw(1) {
                found = true;
            }
        }
        assert!(found, "A GhostCrop must spawn when a pop dies near a farm");
    }

    #[test]
    fn test_eating_ghost_crop_reduces_mood_of_relatives() {
        let mut app = App::new();
        app.add_systems(Update, consume_ghost_food_system);

        let deceased_id = Entity::from_raw(1);
        let relative_id = app.world.spawn((Pop, Mood { value: 100.0 })).id();

        // Add relationship
        app.world.spawn(Relationship {
            pop1: relative_id,
            pop2: deceased_id,
            relationship_type: RelationshipType::Relative,
        });

        // Add a consume event for the Ghost Crop
        app.world.insert_resource(Events::<ConsumeFoodEvent>::default());
        let mut events = app.world.get_resource_mut::<Events<ConsumeFoodEvent>>().unwrap();
        events.send(ConsumeFoodEvent {
            consumer: relative_id,
            food_type: FoodType::GhostCrop(deceased_id),
            amount: 1.0,
        });

        app.update();

        // The relative's mood should drop drastically
        let mood = app.world.get::<Mood>(relative_id).unwrap();
        assert!(mood.value < 100.0, "Consuming a Ghost Crop of a relative must drastically reduce mood");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct GhostCrop {
    pub mimicked_pop: Entity,
}

#[derive(Event)]
pub struct PopDeathEvent {
    pub position: GridPosition,
    pub deceased_id: Entity,
}

#[derive(Event)]
pub struct ConsumeFoodEvent {
    pub consumer: Entity,
    pub food_type: FoodType,
    pub amount: f32,
}

pub enum FoodType {
    Standard,
    GhostCrop(Entity),
}

pub fn process_pop_deaths_near_farms(
    mut death_events: EventReader<PopDeathEvent>,
    mut commands: Commands,
    farms: Query<(Entity, &GridPosition), With<FarmBuilding>>,
) {
    for event in death_events.read() {
        for (farm_entity, farm_pos) in farms.iter() {
            // Simple distance check (e.g., adjacent)
            if (event.position.x - farm_pos.x).abs() <= 1 && (event.position.y - farm_pos.y).abs() <= 1 {
                commands.entity(farm_entity).insert(GhostCrop {
                    mimicked_pop: event.deceased_id,
                });
            }
        }
    }
}

pub fn consume_ghost_food_system(
    mut consume_events: EventReader<ConsumeFoodEvent>,
    mut pops: Query<&mut Mood, With<Pop>>,
    relationships: Query<&Relationship>,
) {
    for event in consume_events.read() {
        if let FoodType::GhostCrop(mimicked_id) = event.food_type {
            if let Ok(mut mood) = pops.get_mut(event.consumer) {
                // Check if they are related to the mimicked pop
                let mut is_relative = false;
                for rel in relationships.iter() {
                    if (rel.pop1 == event.consumer && rel.pop2 == mimicked_id) ||
                       (rel.pop2 == event.consumer && rel.pop1 == mimicked_id) {
                        if rel.relationship_type == RelationshipType::Relative {
                            is_relative = true;
                        }
                    }
                }

                if is_relative {
                    mood.value -= 50.0; // Drastic mood reduction
                }
            }
        }
    }
}

// Dummy structs for compilation
#[derive(Component)]
pub struct Building { pub type_: BuildingType }
pub enum BuildingType { Farm }
#[derive(Component)]
pub struct FarmBuilding;
#[derive(Component, Clone, Copy)]
pub struct GridPosition { pub x: i32, pub y: i32 }
#[derive(Component)]
pub struct Pop;
#[derive(Component)]
pub struct Mood { pub value: f32 }
#[derive(Component)]
pub struct Relationship {
    pub pop1: Entity,
    pub pop2: Entity,
    pub relationship_type: RelationshipType,
}
#[derive(PartialEq)]
pub enum RelationshipType { Relative, Friend, Enemy }
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Hook `process_pop_deaths_near_farms` to the actual `PopDeathEvent` in the existing lifecycle system.
- **Yield Boosting:** Add logic to the farming harvest system so that crops with `GhostCrop` yield double or triple the standard food output.
- **Harvesting UI:** Provide an action or designation for the player to "Burn Ghost Crops" to prevent pops from eating them, sacrificing the high yield to save their sanity.

## 6. Acceptance Criteria
- [ ] A `GhostCrop` component spawns when a Pop dies near a Farm.
- [ ] Consuming Ghost Crops drops the `Mood` of related Pops significantly.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.
- [ ] 0 clippy warnings.

## 7. Technical Guidance
- If `PopDeathEvent` already exists, extend it or create a listener rather than redefining it.
- Make sure that when a `GhostCrop` is harvested, the resulting food item retains the identity of the mimicked Pop (e.g., in its inventory data).

## 8. Questions
*Builder: Add any questions here.*
