# 737 - The Monuments of Failure

## 1. Overview
We build on the bones of the past. A ruin is just a free wall. Destroyed buildings leave "Ruin" tiles that provide partial cover, storage, or materials. They lower Beauty but increase "History". You can refurbish them cheaper than building new, but they retain "Scars" (lower max HP). This creates a tension between rebuilding with a clean slate (higher cost) versus adapting the ruins (scars/history).

## 2. Dependencies
- Layer 1 Building construction and destruction systems.
- Beauty and History stat tracking for tiles/areas.
- HP management for structures.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_destroyed_building_spawns_ruin_with_history_and_negative_beauty() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<BuildingDestroyedEvent>();
        app.add_systems(Update, handle_building_destruction_system);

        let building = app.world_mut().spawn((
            Building { max_hp: 100.0, current_hp: 0.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Act
        app.world_mut().send_event(BuildingDestroyedEvent { entity: building, pos: GridPosition { x: 5, y: 5 } });
        app.update();

        // Assert
        let mut found_ruin = false;
        for (ruin, pos, beauty, history) in app.world_mut().query::<(&Ruin, &GridPosition, &BeautyModifier, &HistoryModifier)>().iter(app.world()) {
            if pos.x == 5 && pos.y == 5 {
                found_ruin = true;
                assert!(beauty.value < 0.0, "Ruins should lower beauty");
                assert!(history.value > 0.0, "Ruins should increase history");
            }
        }
        assert!(found_ruin, "A ruin should have been spawned at the destroyed building's position");
    }

    #[test]
    fn test_refurbish_ruin_applies_scar_penalty() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<RefurbishRuinEvent>();
        app.add_systems(Update, handle_refurbish_ruin_system);

        let ruin = app.world_mut().spawn((
            Ruin,
            GridPosition { x: 1, y: 1 },
        )).id();

        // Act
        app.world_mut().send_event(RefurbishRuinEvent { ruin_entity: ruin, target_building_type: BuildingType::Cafeteria });
        app.update();

        // Assert
        let mut found_refurbished = false;
        for (building, pos, scars) in app.world_mut().query::<(&Building, &GridPosition, &Scars)>().iter(app.world()) {
            if pos.x == 1 && pos.y == 1 {
                found_refurbished = true;
                assert!(scars.max_hp_penalty > 0.0, "Refurbished buildings should have a max HP penalty");
                assert!(building.max_hp < 100.0, "Max HP should be lower than a fresh building"); // Assuming base 100
            }
        }
        assert!(found_refurbished, "The ruin should have been replaced by a refurbished building");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Building {
    pub max_hp: f32,
    pub current_hp: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Ruin;

#[derive(Component)]
pub struct BeautyModifier {
    pub value: f32,
}

#[derive(Component)]
pub struct HistoryModifier {
    pub value: f32,
}

#[derive(Component)]
pub struct Scars {
    pub max_hp_penalty: f32,
}

#[derive(Event)]
pub struct BuildingDestroyedEvent {
    pub entity: Entity,
    pub pos: GridPosition,
}

#[derive(Event)]
pub struct RefurbishRuinEvent {
    pub ruin_entity: Entity,
    pub target_building_type: BuildingType,
}

#[derive(Clone, Copy)]
pub enum BuildingType {
    Cafeteria,
    // Other types...
}

pub fn handle_building_destruction_system(
    mut commands: Commands,
    mut events: EventReader<BuildingDestroyedEvent>,
) {
    for event in events.read() {
        commands.entity(event.entity).despawn();
        commands.spawn((
            Ruin,
            event.pos,
            BeautyModifier { value: -10.0 },
            HistoryModifier { value: 10.0 },
        ));
    }
}

pub fn handle_refurbish_ruin_system(
    mut commands: Commands,
    mut events: EventReader<RefurbishRuinEvent>,
    query: Query<&GridPosition, With<Ruin>>,
) {
    for event in events.read() {
        if let Ok(pos) = query.get(event.ruin_entity) {
            commands.entity(event.ruin_entity).despawn();
            commands.spawn((
                Building { max_hp: 80.0, current_hp: 80.0 }, // Base is 100, penalty is 20
                *pos,
                Scars { max_hp_penalty: 20.0 },
            ));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `handle_building_destruction_system` should ideally be triggered by a health-monitoring system that despawns the building and emits the event, or just hooks directly into the death logic.
- Building properties (base max HP, cost) should be data-driven. The `Scars` penalty should be a percentage of the base max HP rather than a hardcoded subtraction.
- Cost reduction for refurbishing needs to be implemented in the construction logic, reading the presence of a `Ruin` at the target position.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Destroying a building spawns a Ruin with Beauty penalties and History bonuses.
- [ ] Refurbishing a Ruin creates a new Building with reduced Max HP (`Scars`).

## 7. Technical Guidance
- The history modifier should tie into the Morale or Culture systems, potentially offsetting the Beauty loss for certain Pops who value history.
- Ensure Ruins block normal construction unless the player explicitly chooses "Refurbish" or clears the Ruin first.

## 8. Questions
*Builder: add questions here if spec is unclear.*
