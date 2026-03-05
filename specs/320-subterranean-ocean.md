# 320: The Subterranean Ocean

## 1. Overview

Beneath the rock layers lies a massive, pressurized ocean. Breaching it floods the lower levels of your mine with water. The ocean contains unique, blind flora and fauna (some hostile) and rare "Deep Pearls" (high-value trade goods). This mechanic balances the risk of catastrophic flooding against access to a completely new biome and valuable resources.

## 2. Dependencies

- `018` Mining Resources (for digging mechanics)
- `119` Airlock & Pressure (for fluid dynamics)
- `164` Modular Fauna (for deep ocean life)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ocean_breach_floods_mine() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_mining_breach);

        let ocean = app.world_mut().spawn((
            GridPosition { x: 5, y: -10 },
            SubterraneanOcean { pressure: 1000.0 },
        )).id();

        let mine_tile = app.world_mut().spawn((
            GridPosition { x: 5, y: -9 },
            Terrain { is_solid: false },
            FluidLevel { amount: 0.0 },
        )).id();

        // Act - Simulate digging into the ocean
        app.world_mut().insert_resource(MiningEvent { target: ocean });
        app.update();

        // Assert
        let fluid = app.world().get::<FluidLevel>(mine_tile).unwrap();
        assert!(fluid.amount > 0.0, "Mine tile should be flooded after breaching the ocean");

        let breach_event = app.world_mut().query::<&OceanBreachEvent>().iter(&app.world()).next();
        assert!(breach_event.is_some(), "An ocean breach event should have been dispatched");
    }

    #[test]
    fn test_deep_pearl_harvest() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_pearl_harvesting);

        let pearl = app.world_mut().spawn((
            GridPosition { x: 5, y: -10 },
            DeepPearl { value: 500 },
        )).id();

        app.world_mut().insert_resource(HarvestEvent { target: pearl });
        app.world_mut().insert_resource(ColonyWealth { credits: 0 });

        // Act
        app.update();

        // Assert
        let wealth = app.world().get_resource::<ColonyWealth>().unwrap();
        assert_eq!(wealth.credits, 500, "Harvesting a deep pearl should add to colony wealth");

        let pearl_exists = app.world().get::<DeepPearl>(pearl).is_some();
        assert!(!pearl_exists, "Harvested pearl should be removed");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct SubterraneanOcean {
    pub pressure: f32,
}

#[derive(Component)]
pub struct Terrain {
    pub is_solid: bool,
}

#[derive(Component)]
pub struct FluidLevel {
    pub amount: f32,
}

#[derive(Resource)]
pub struct MiningEvent {
    pub target: Entity,
}

#[derive(Component)]
pub struct OceanBreachEvent {
    pub location: GridPosition,
}

#[derive(Component)]
pub struct DeepPearl {
    pub value: u32,
}

#[derive(Resource)]
pub struct HarvestEvent {
    pub target: Entity,
}

#[derive(Resource)]
pub struct ColonyWealth {
    pub credits: u32,
}

pub fn process_mining_breach(
    mut commands: Commands,
    mut mining_event: Option<ResMut<MiningEvent>>,
    ocean_query: Query<(&GridPosition, &SubterraneanOcean)>,
    mut mine_query: Query<(&GridPosition, &mut FluidLevel, &Terrain)>,
) {
    if let Some(event) = mining_event.take() {
        if let Ok((ocean_pos, ocean)) = ocean_query.get(event.target) {
            // Breach occurred
            commands.spawn(OceanBreachEvent {
                location: GridPosition { x: ocean_pos.x, y: ocean_pos.y },
            });

            // Flood adjacent non-solid tiles
            for (mine_pos, mut fluid, terrain) in mine_query.iter_mut() {
                if !terrain.is_solid {
                    let dx = (ocean_pos.x - mine_pos.x).abs();
                    let dy = (ocean_pos.y - mine_pos.y).abs();
                    if dx <= 1 && dy <= 1 {
                        fluid.amount += ocean.pressure * 0.1; // Initial flood surge
                    }
                }
            }
        }
    }
}

pub fn process_pearl_harvesting(
    mut commands: Commands,
    mut harvest_event: Option<ResMut<HarvestEvent>>,
    pearl_query: Query<&DeepPearl>,
    mut wealth: ResMut<ColonyWealth>,
) {
    if let Some(event) = harvest_event.take() {
        if let Ok(pearl) = pearl_query.get(event.target) {
            wealth.credits += pearl.value;
            commands.entity(event.target).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Event Despawn/Removal:** Use Bevy's built-in `EventReader` and `EventWriter` for `MiningEvent`, `OceanBreachEvent`, and `HarvestEvent` instead of resources that need to be manually `.take()`n or entity spawns.
- **Fluid Dynamics:** The flooding shouldn't be instant; it should be handled by a fluid simulation system over multiple ticks, using the `PressureGrid` or `AtmosphereGrid` systems.
- **Wealth Resource:** `ColonyWealth` should be part of the global `ColonyResources` or `TradeSystem` inventory rather than an isolated resource.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Mining an ocean tile floods adjacent open tiles based on pressure.
- [ ] Harvesting a Deep Pearl despawns the pearl and adds its value to colony resources.
- [ ] An `OceanBreachEvent` is properly emitted when breaching.

## 7. Technical Guidance

- Implement `SubterraneanOcean` and `DeepPearl` mechanics in `src/layer1/terrain/ocean.rs`.
- Tie the `OceanBreachEvent` into the UI notifications and Chronicle system.
- Integrate the flood spreading logic into the existing fluid simulation loop if one exists, otherwise create a simplified grid-based cellular automaton for water spread.

## 8. Questions

*Builder: add questions here if spec is unclear.*
