# 840: Soil Liquefaction

## 1. Overview
During seismic events, specific unstable terrain types (like Sand, Mud, or Riverbanks) temporarily lose their structural integrity, undergoing liquefaction. Buildings built on these terrain types take massive structural damage or are completely swallowed by the earth, while Pops caught in the liquefied zones can drown or suffer fatal injuries. This creates tension between choosing cheap, flat land with high risk versus uneven, hard bedrock with high safety.

## 2. Dependencies
- Base ECS with `Entity`, `Query`, `Commands`, and `Res`.
- A grid/terrain system (`TerrainType`, `Tile`).
- An event system for disasters (`SeismicEvent`).
- Building and Pop entities with `Structure`, `Health`, and `Position` components.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_soil_liquefaction_damages_buildings() {
        let mut app = App::new();
        app.add_systems(Update, soil_liquefaction_system);
        app.add_event::<SeismicEvent>();

        // Arrange: Unstable terrain and building
        let terrain_entity = app.world_mut().spawn((
            Position { x: 5, y: 5 },
            Terrain { terrain_type: TerrainType::Sand },
        )).id();

        let building_entity = app.world_mut().spawn((
            Position { x: 5, y: 5 },
            Structure { max_integrity: 100, current_integrity: 100 },
        )).id();

        // Act: Trigger seismic event
        app.world_mut().send_event(SeismicEvent { magnitude: 5.0 });
        app.update();

        // Assert: Building takes damage
        let building = app.world().get::<Structure>(building_entity).unwrap();
        assert!(building.current_integrity < 100, "Building on sand should take damage during an earthquake");
    }

    #[test]
    fn test_soil_liquefaction_drowns_pops() {
        let mut app = App::new();
        app.add_systems(Update, soil_liquefaction_system);
        app.add_event::<SeismicEvent>();

        // Arrange: Pop on mud
        let terrain_entity = app.world_mut().spawn((
            Position { x: 2, y: 2 },
            Terrain { terrain_type: TerrainType::Mud },
        )).id();

        let pop_entity = app.world_mut().spawn((
            Position { x: 2, y: 2 },
            Health { current: 50, max: 50 },
            Pop,
        )).id();

        // Act
        app.world_mut().send_event(SeismicEvent { magnitude: 7.5 });
        app.update();

        // Assert: Pop is dead or takes severe damage
        let health = app.world().get::<Health>(pop_entity).unwrap();
        assert!(health.current < 50, "Pop should take drowning damage on mud during an earthquake");
    }

    #[test]
    fn test_bedrock_is_safe_from_liquefaction() {
        let mut app = App::new();
        app.add_systems(Update, soil_liquefaction_system);
        app.add_event::<SeismicEvent>();

        // Arrange: Bedrock terrain and building
        let terrain_entity = app.world_mut().spawn((
            Position { x: 10, y: 10 },
            Terrain { terrain_type: TerrainType::Bedrock },
        )).id();

        let building_entity = app.world_mut().spawn((
            Position { x: 10, y: 10 },
            Structure { max_integrity: 100, current_integrity: 100 },
        )).id();

        // Act
        app.world_mut().send_event(SeismicEvent { magnitude: 8.0 });
        app.update();

        // Assert: Building is unharmed
        let building = app.world().get::<Structure>(building_entity).unwrap();
        assert_eq!(building.current_integrity, 100, "Building on bedrock should not be affected by liquefaction");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Clone, PartialEq)]
pub enum TerrainType {
    Sand,
    Mud,
    Riverbank,
    Bedrock,
    Dirt,
}

#[derive(Component)]
pub struct Terrain {
    pub terrain_type: TerrainType,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Structure {
    pub max_integrity: u32,
    pub current_integrity: u32,
}

#[derive(Component)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Event)]
pub struct SeismicEvent {
    pub magnitude: f32,
}

pub fn soil_liquefaction_system(
    mut events: EventReader<SeismicEvent>,
    terrain_query: Query<(&Position, &Terrain)>,
    mut structures: Query<(&Position, &mut Structure)>,
    mut pops: Query<(&Position, &mut Health), With<Pop>>,
) {
    for event in events.read() {
        if event.magnitude > 4.0 {
            let mut unstable_positions = std::collections::HashSet::new();

            for (pos, terrain) in terrain_query.iter() {
                if matches!(terrain.terrain_type, TerrainType::Sand | TerrainType::Mud | TerrainType::Riverbank) {
                    unstable_positions.insert(*pos);
                }
            }

            for (pos, mut structure) in structures.iter_mut() {
                if unstable_positions.contains(pos) {
                    let damage = (event.magnitude * 10.0) as u32;
                    structure.current_integrity = structure.current_integrity.saturating_sub(damage);
                }
            }

            for (pos, mut health) in pops.iter_mut() {
                if unstable_positions.contains(pos) {
                    let damage = (event.magnitude * 5.0) as u32;
                    health.current = health.current.saturating_sub(damage);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Queries:** Replace `HashSet` generation with a spatial hash map or grid resource lookup to avoid iterating all terrain tiles on every seismic event.
- **System Param:** Group the `Query` and `EventReader` into a `SystemParam` if the signature grows.
- **Event Splitting:** Instead of direct damage application, it might be better to emit `DamageEvent`s so a centralized damage resolution system can handle modifiers, death triggers, and effects.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Buildings on Sand/Mud/Riverbanks take damage during earthquakes.
- [ ] Pops on unstable terrain suffer damage (drowning/sinking).

## 7. Technical Guidance
- **Terrain Abstraction:** Bevy spatial indices will be crucial when checking what terrain type a position has. Do not iterate over the entire map.
- **Liquefaction Modifiers:** Consider letting building foundations (a possible new component/tech) mitigate liquefaction damage.

## 8. Questions
*Builder: add questions here if spec is unclear.*
