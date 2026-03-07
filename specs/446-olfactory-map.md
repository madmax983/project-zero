# Specification: 446 - Olfactory Map

## 1. Overview
The **Olfactory Map** feature introduces smell as a dynamic, physical element in the colony. Tiles will emit and track "Scent" values that diffuse and mix over time. This creates a new layer of colony planning where the placement of industrial (dirty/smelly) buildings versus amenities (pleasant/fragrant) must be balanced to maintain Pops' mood and health, directly addressing the "industrial efficiency vs. air quality" tension.

## 2. Dependencies
- `010` Chronicle System
- `016` Utility AI System
- `064` Room Quality
- Base Bevy ECS and Grid/Tilemap architecture

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::grid::{GridPos, GridMap};
    use scale::layer1::mood::{Mood, MoodModifier};

    #[test]
    fn test_scent_emission_and_diffusion() {
        // Arrange: Setup world with grid and scent tracking
        let mut app = App::new();
        app.add_plugins(ScentPlugin);

        let mut grid = GridMap::new(10, 10);
        app.world.insert_resource(grid);

        // Act: Add a strong foul scent source (e.g., Waste Dump)
        let source_pos = GridPos::new(5, 5);
        app.world.spawn((
            ScentEmitter { scent_type: ScentType::Foul, strength: 10.0 },
            source_pos,
        ));

        // Tick system to allow diffusion
        app.update();

        // Assert: Center tile has highest foul scent
        let scent_map = app.world.resource::<ScentMap>();
        let center_scent = scent_map.get_scent(source_pos);
        assert!(center_scent.foul > 5.0, "Center should have high foul scent");

        // Assert: Adjacent tile has diffused scent
        let adjacent_pos = GridPos::new(5, 6);
        let adjacent_scent = scent_map.get_scent(adjacent_pos);
        assert!(adjacent_scent.foul > 0.0 && adjacent_scent.foul < center_scent.foul,
                "Scent should diffuse outward");
    }

    #[test]
    fn test_pleasant_scent_mood_buff() {
        let mut app = App::new();
        app.add_plugins(ScentPlugin);

        let pop_pos = GridPos::new(2, 2);
        let pop_entity = app.world.spawn((
            Mood::default(),
            pop_pos,
            Pop,
        )).id();

        // Spawn pleasant scent
        app.world.spawn((
            ScentEmitter { scent_type: ScentType::Pleasant, strength: 5.0 },
            pop_pos,
        ));

        app.update();

        // Assert: Pop gains a positive mood modifier from the pleasant scent
        let mood = app.world.get::<Mood>(pop_entity).unwrap();
        assert!(mood.modifiers.iter().any(|m| matches!(m, MoodModifier::PleasantScent)),
                "Pop should receive pleasant scent buff");
    }

    #[test]
    fn test_foul_scent_mood_debuff() {
        let mut app = App::new();
        app.add_plugins(ScentPlugin);

        let pop_pos = GridPos::new(3, 3);
        let pop_entity = app.world.spawn((
            Mood::default(),
            pop_pos,
            Pop,
        )).id();

        // Spawn foul scent
        app.world.spawn((
            ScentEmitter { scent_type: ScentType::Foul, strength: 8.0 },
            pop_pos,
        ));

        app.update();

        // Assert: Pop gains a negative mood modifier from the foul scent
        let mood = app.world.get::<Mood>(pop_entity).unwrap();
        assert!(mood.modifiers.iter().any(|m| matches!(m, MoodModifier::FoulScent)),
                "Pop should receive foul scent debuff");
    }

    #[test]
    fn test_scent_mixing_and_overpowering() {
        let mut app = App::new();
        app.add_plugins(ScentPlugin);
        app.world.insert_resource(GridMap::new(10, 10));

        let pos = GridPos::new(4, 4);

        // Add strong foul and weak pleasant scent to same area
        app.world.spawn((ScentEmitter { scent_type: ScentType::Foul, strength: 10.0 }, pos));
        app.world.spawn((ScentEmitter { scent_type: ScentType::Pleasant, strength: 2.0 }, pos));

        app.update();

        let scent_map = app.world.resource::<ScentMap>();
        let tile_scent = scent_map.get_scent(pos);

        // Assert: Foul should dominate
        assert!(tile_scent.foul > tile_scent.pleasant, "Foul scent should overpower pleasant");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct ScentEmitter {
    pub scent_type: ScentType,
    pub strength: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScentType {
    Pleasant,
    Foul,
}

#[derive(Clone, Default)]
pub struct TileScent {
    pub pleasant: f32,
    pub foul: f32,
}

#[derive(Resource, Default)]
pub struct ScentMap {
    // Map of GridPos to TileScent
    // Simple implementation could just be a HashMap or 2D vec
}

impl ScentMap {
    pub fn get_scent(&self, pos: GridPos) -> TileScent {
        // Return scent at position
        TileScent::default()
    }
}

pub fn scent_diffusion_system(
    mut scent_map: ResMut<ScentMap>,
    emitters: Query<(&ScentEmitter, &GridPos)>,
) {
    // Basic diffusion logic: zero out map, add emitter strengths, diffuse to adjacent
}

pub fn scent_mood_system(
    scent_map: Res<ScentMap>,
    mut pops: Query<(&GridPos, &mut Mood)>,
) {
    // Check scent at Pop's position and apply appropriate MoodModifier
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance Optimization:** Instead of recalculating the entire grid's scent map every tick, use an event-driven approach where scent is only recalculated when emitters are added/removed or at a fixed slower interval (e.g., once every few seconds).
- **Data Structure:** The `ScentMap` should be a flat `Vec` aligned with the existing `GridMap` rather than a `HashMap` to ensure memory locality and cache-friendly iteration.
- **Wall Occlusion:** Scent diffusion should eventually take walls into account. Walls should block or heavily dampen scent spread.
- **Decay:** Scent should decay over time if the emitter is removed.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Foul scents correctly lower nearby Pops' mood.
- [ ] Pleasant scents correctly raise nearby Pops' mood.

## 7. Technical Guidance
- Integrate the `scent_diffusion_system` into a slow-tick schedule to avoid tanking performance.
- When applying mood modifiers, ensure they don't stack infinitely. Use a discrete `MoodModifier::FoulScent` state rather than stacking raw numerical penalties.
- Consider clamping maximum scent values to prevent ridiculous accumulation in confined spaces.

## 8. Questions
*Builder: add questions here if spec is unclear.*
