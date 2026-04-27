# Protest Crowds

## 1. Overview
**Layer:** 1
**Fantasy:** The physical weight of dissent.
**Mechanic:** Unhappy Factions don't just complain; they physically gather in "Mobs" that block tiles (Halls, Airlocks, Power Plants). They refuse to move until demands are met or force is used.

## 2. Dependencies
- Layer 1 Pathfinding & Grid System
- Faction Morale System
- Utility AI (for pops joining mobs)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_mob_formation_blocks_tile() {
        // Arrange
        let mut app = App::new();
        app.add_event::<FormMobEvent>();
        let target_tile = UVec2::new(5, 5);
        let mut grid = TerrainGrid::new(10, 10);
        app.insert_resource(grid);

        let faction = app.world_mut().spawn(Faction { morale: 10.0 }).id(); // Low morale

        // Act
        app.world_mut().send_event(FormMobEvent {
            location: target_tile,
            faction,
        });
        app.update();

        // Assert
        let grid = app.world().resource::<TerrainGrid>();
        assert!(!grid.is_passable(target_tile), "Mob should block the target tile");

        let mobs = app.world().query::<&Mob>().iter(app.world()).count();
        assert_eq!(mobs, 1);
    }

    #[test]
    fn test_mob_dispersion() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DisperseMobEvent>();
        let target_tile = UVec2::new(5, 5);
        let mut grid = TerrainGrid::new(10, 10);
        grid.set_passable(target_tile, false); // Manually block for test
        app.insert_resource(grid);

        let mob_entity = app.world_mut().spawn(Mob { location: target_tile }).id();

        // Act
        app.world_mut().send_event(DisperseMobEvent {
            mob: mob_entity,
        });
        app.update();

        // Assert
        let grid = app.world().resource::<TerrainGrid>();
        assert!(grid.is_passable(target_tile), "Tile should be passable after mob disperses");
        assert!(app.world().get_entity(mob_entity).is_err(), "Mob entity should be despawned");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use glam::UVec2;

#[derive(Component)]
pub struct Faction {
    pub morale: f32,
}

#[derive(Component)]
pub struct Mob {
    pub location: UVec2,
}

#[derive(Resource)]
pub struct TerrainGrid {
    // simplified for testing
    passability: std::collections::HashSet<UVec2>,
}

impl TerrainGrid {
    pub fn new(_width: u32, _height: u32) -> Self {
        Self { passability: std::collections::HashSet::new() }
    }
    pub fn set_passable(&mut self, pos: UVec2, passable: bool) {
        if passable {
            self.passability.insert(pos);
        } else {
            self.passability.remove(&pos);
        }
    }
    pub fn is_passable(&self, pos: UVec2) -> bool {
        self.passability.contains(&pos)
    }
}

#[derive(Event)]
pub struct FormMobEvent {
    pub location: UVec2,
    pub faction: Entity,
}

#[derive(Event)]
pub struct DisperseMobEvent {
    pub mob: Entity,
}

pub fn form_mob_system(
    mut events: EventReader<FormMobEvent>,
    mut grid: ResMut<TerrainGrid>,
    mut commands: Commands,
) {
    for event in events.read() {
        grid.set_passable(event.location, false);
        commands.spawn(Mob { location: event.location });
    }
}

pub fn disperse_mob_system(
    mut events: EventReader<DisperseMobEvent>,
    mut grid: ResMut<TerrainGrid>,
    query: Query<&Mob>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mob) = query.get(event.mob) {
            grid.set_passable(mob.location, true);
            if let Some(mut entity) = commands.get_entity(event.mob) {
                entity.despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently, `TerrainGrid` in GREEN is just a hashset. Use the actual real game grid and integrate with its pathfinding weighting so the tile becomes truly impassable.
- Define what constitutes "force" for the `DisperseMobEvent`. Is it an action from the player, or security pops arriving?
- Mobs should attract nearby pops of the same faction via `UtilityAI`, scaling the severity over time.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Mobs block tiles correctly on the grid preventing pathfinding.
- [ ] Dispersing a mob restores tile passability.

## 7. Technical Guidance
- **Pathfinding Hook:** Ensure you modify the existing `TerrainGrid` (or equivalent grid structure) so the A* pathfinder automatically routes around the mob.
- **Utility AI:** Pops in the mob shouldn't just stand there; their AI state should shift to a `Protesting` action so they ignore normal needs like work (though they might still need to eat).

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
