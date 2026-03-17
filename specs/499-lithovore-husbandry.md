# 499 - Lithovore Husbandry

## 1. Overview
Taming deep-crust creatures that eat rock and excrete building materials creates a living, crawling mining operation. You can capture and breed "Lithovores" which autonomously pathfind to and consume raw stone/ore tiles, leaving refined blocks in their wake. They multiply rapidly if overfed but go feral and eat your buildings if starved. This creates a tension between free, autonomous resource refinement and the risk of breeding an uncontrollable swarm that eats your colony's foundations.

## 2. Dependencies
- `018` Mining and Resources
- `075` Animal Husbandry
- `016` Utility AI System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;
    // Assuming existence of these structures
    // use crate::layer1::map::{TerrainGrid, TerrainType};
    // use crate::layer1::fauna::{Fauna, Tame, Hunger};
    // use crate::layer1::resources::{ColonyInventory, ItemType};

    #[derive(Component)]
    struct Lithovore;

    #[derive(Component)]
    struct Hunger {
        pub value: f32, // 0.0 is full, 100.0 is starving
    }

    #[derive(Component)]
    struct Feral;

    #[derive(Resource)]
    struct TerrainGrid {
        pub cells: Vec<TerrainType>,
        pub width: i32,
    }

    #[derive(PartialEq, Clone)]
    enum TerrainType {
        Dirt,
        Rock,
        BuildingWall,
    }

    fn simulate_lithovore_metabolism(
        mut commands: Commands,
        mut query: Query<(Entity, &mut Hunger, Option<&Tame>), With<Lithovore>>,
    ) {
        for (entity, mut hunger, tame_opt) in query.iter_mut() {
            hunger.value += 5.0; // Increase hunger over time
            if hunger.value >= 100.0 {
                if tame_opt.is_some() {
                    commands.entity(entity).remove::<Tame>().insert(Feral);
                }
            }
        }
    }

    #[test]
    fn test_starving_lithovore_goes_feral() {
        let mut app = App::new();

        let lithovore = app.world_mut().spawn((
            Lithovore,
            Hunger { value: 95.0 },
            Tame,
        )).id();

        app.add_systems(Update, simulate_lithovore_metabolism);
        app.update();

        assert!(app.world().entity(lithovore).contains::<Feral>(), "Starving lithovore should lose Tame and become Feral");
        assert!(!app.world().entity(lithovore).contains::<Tame>(), "Starving lithovore should no longer be Tame");
    }

    #[test]
    fn test_lithovore_eats_rock_and_produces_stone_block() {
        // Pseudo-test for the eating mechanic
        // Arrange: A lithovore adjacent to a Rock tile
        // Act: Run the eating system
        // Assert: Rock tile becomes Dirt, a StoneBlock item is spawned, Hunger decreases
    }

    #[test]
    fn test_feral_lithovore_eats_buildings() {
        // Pseudo-test for feral behavior
        // Arrange: A Feral lithovore adjacent to a BuildingWall
        // Act: Run the feral eating system
        // Assert: BuildingWall takes damage or is destroyed, Hunger decreases
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Lithovore;

#[derive(Component)]
pub struct Tame;

#[derive(Component)]
pub struct Feral;

#[derive(Component)]
pub struct Hunger {
    pub value: f32, // 0.0 to 100.0
}

pub fn simulate_lithovore_metabolism(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Hunger, Option<&Tame>), With<Lithovore>>,
) {
    for (entity, mut hunger, tame) in query.iter_mut() {
        hunger.value += 1.0; // Base metabolic rate per tick

        if hunger.value >= 100.0 && tame.is_some() {
            // Starvation causes them to go feral
            commands.entity(entity).remove::<Tame>();
            commands.entity(entity).insert(Feral);
        }
    }
}

// In a full implementation, `lithovore_eating_system` would handle:
// 1. Finding adjacent Rock tiles (if Tame) or Building tiles (if Feral).
// 2. Converting the tile to Dirt.
// 3. Spawning a `StoneBlock` or equivalent resource entity.
// 4. Reducing `Hunger.value`.
// 5. If `Hunger.value` drops to 0.0 frequently, triggering a reproduction system.
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding**: Lithovores need a custom pathfinding cost function that heavily prefers unmined rock over clear paths, treating rock as passable if they are hungry enough to eat it.
- **Utility AI Integration**: Tame Lithovores should be guided by "Ranching" zones or bait stations to keep them away from sensitive infrastructure.
- **Performance**: Ensure that tile conversion (Rock to Dirt) properly updates the `TerrainGrid` and triggers a pathfinding mesh rebuild only when necessary, as a swarm of eating lithovores could cause massive recalculation spikes.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Lithovores consume Rock tiles and produce Refined Stone items.
- [ ] Lithovores lose their `Tame` status and gain `Feral` status when their `Hunger` reaches 100.0.
- [ ] Feral Lithovores can target and damage building tiles.

## 7. Technical Guidance
- The `Feral` component should hook into the existing hostility system (e.g., triggering militia responses).
- Consider adding a `LithovoreBait` item that players can craft to direct the swarm towards specific resource veins.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
