# The Sunk-Cost Monument

## 1. Overview
The Sunk-Cost Monument feature introduces late-game vanity projects that demand an ever-increasing amount of resources to construct. If construction is halted or canceled, the partially completed structure leaves behind a permanent "ruin" that significantly penalizes the morale of nearby Pops. This forces a brutal decision: starve the colony to finish the monument, or cut your losses and live with a permanent scar on the landscape. This mechanic applies primarily to the Colony Layer (Layer 1).

## 2. Dependencies
- `src/layer1/building.rs`: Core building system and construction progress mechanics.
- `src/layer1/needs.rs`: For Morale penalties.
- `src/layer1/components.rs`: `ConstructionProgress`, `Ruin`, and related building state components.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components for testing
    #[derive(Component)]
    struct MonumentMarker;

    #[derive(Component)]
    struct SunkCostUpkeep {
        base_cost: f32,
        multiplier: f32,
        ticks_building: u32,
    }

    #[derive(Component)]
    struct SunkCostRuin {
        morale_penalty_radius: f32,
        penalty_amount: f32,
    }

    #[derive(Component)]
    struct Pop;

    #[derive(Component)]
    struct Morale(f32);

    #[derive(Component)]
    struct Transform(Vec3);

    // 1. Test that the upkeep cost of a Sunk-Cost Monument increases over time
    #[test]
    fn test_sunk_cost_upkeep_increases_over_time() {
        let mut app = App::new();
        app.add_systems(Update, calculate_sunk_cost_upkeep_system);

        let monument = app.world_mut().spawn(SunkCostUpkeep {
            base_cost: 10.0,
            multiplier: 1.1,
            ticks_building: 0,
        }).id();

        app.update(); // Tick 1

        let upkeep_1 = app.world().get::<SunkCostUpkeep>(monument).unwrap();
        assert_eq!(upkeep_1.ticks_building, 1);
        let cost_1 = upkeep_1.base_cost * upkeep_1.multiplier.powf(1.0);

        app.update(); // Tick 2

        let upkeep_2 = app.world().get::<SunkCostUpkeep>(monument).unwrap();
        assert_eq!(upkeep_2.ticks_building, 2);
        let cost_2 = upkeep_2.base_cost * upkeep_2.multiplier.powf(2.0);

        assert!(cost_2 > cost_1);
    }

    // 2. Test that canceling a Sunk-Cost Monument spawns a Ruin with a morale penalty
    #[test]
    fn test_canceling_monument_creates_ruin() {
        let mut app = App::new();
        app.add_event::<CancelConstructionEvent>();
        app.add_systems(Update, handle_monument_cancellation_system);

        let monument_id = app.world_mut().spawn((
            MonumentMarker,
            Transform(Vec3::new(0.0, 0.0, 0.0)),
        )).id();

        app.world_mut().send_event(CancelConstructionEvent(monument_id));
        app.update();

        // The original monument should be despawned or converted
        assert!(app.world().get::<MonumentMarker>(monument_id).is_none());

        // A ruin should exist at the same location
        let mut ruin_query = app.world_mut().query::<(&SunkCostRuin, &Transform)>();
        let ruin_exists = ruin_query.iter(app.world()).any(|(_, transform)| {
            transform.0 == Vec3::new(0.0, 0.0, 0.0)
        });

        assert!(ruin_exists, "A SunkCostRuin should be spawned when construction is canceled.");
    }

    // 3. Test that the Ruin applies a morale penalty to nearby Pops
    #[test]
    fn test_ruin_applies_morale_penalty() {
        let mut app = App::new();
        app.add_systems(Update, apply_ruin_morale_penalty_system);

        // Spawn a ruin
        app.world_mut().spawn((
            SunkCostRuin {
                morale_penalty_radius: 10.0,
                penalty_amount: -5.0,
            },
            Transform(Vec3::new(0.0, 0.0, 0.0)),
        ));

        // Spawn a Pop within radius
        let pop_near = app.world_mut().spawn((
            Pop,
            Morale(50.0),
            Transform(Vec3::new(5.0, 0.0, 0.0)), // Distance 5 < 10
        )).id();

        // Spawn a Pop outside radius
        let pop_far = app.world_mut().spawn((
            Pop,
            Morale(50.0),
            Transform(Vec3::new(20.0, 0.0, 0.0)), // Distance 20 > 10
        )).id();

        app.update();

        let morale_near = app.world().get::<Morale>(pop_near).unwrap().0;
        let morale_far = app.world().get::<Morale>(pop_far).unwrap().0;

        assert_eq!(morale_near, 45.0, "Pop near the ruin should receive a morale penalty.");
        assert_eq!(morale_far, 50.0, "Pop far from the ruin should not be affected.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components
#[derive(Component)]
pub struct SunkCostUpkeep {
    pub base_cost: f32,
    pub multiplier: f32,
    pub ticks_building: u32,
}

#[derive(Component)]
pub struct SunkCostRuin {
    pub morale_penalty_radius: f32,
    pub penalty_amount: f32,
}

#[derive(Component)]
pub struct MonumentMarker;

// Dummy events and components for compilation
#[derive(Event)]
pub struct CancelConstructionEvent(pub Entity);

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale(pub f32);

#[derive(Component, Clone, Copy)]
pub struct Transform(pub Vec3);

// Systems
pub fn calculate_sunk_cost_upkeep_system(
    mut query: Query<&mut SunkCostUpkeep>,
) {
    for mut upkeep in query.iter_mut() {
        upkeep.ticks_building += 1;
        // Cost calculation logic is implicit in the component state,
        // to be utilized by resource deduction systems.
    }
}

pub fn handle_monument_cancellation_system(
    mut commands: Commands,
    mut events: EventReader<CancelConstructionEvent>,
    query: Query<&Transform, With<MonumentMarker>>,
) {
    for event in events.read() {
        if let Ok(transform) = query.get(event.0) {
            // Despawn the incomplete monument
            commands.entity(event.0).despawn();

            // Spawn the Ruin
            commands.spawn((
                SunkCostRuin {
                    morale_penalty_radius: 10.0, // Fixed radius for now
                    penalty_amount: -5.0,        // Fixed penalty for now
                },
                *transform,
            ));
        }
    }
}

pub fn apply_ruin_morale_penalty_system(
    ruins: Query<(&SunkCostRuin, &Transform)>,
    mut pops: Query<(&mut Morale, &Transform), With<Pop>>,
) {
    for (ruin, ruin_transform) in ruins.iter() {
        for (mut morale, pop_transform) in pops.iter_mut() {
            let distance = ruin_transform.0.distance(pop_transform.0);
            if distance <= ruin.morale_penalty_radius {
                morale.0 += ruin.penalty_amount;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactor 1: Integrate with Actual Resource System.** The `calculate_sunk_cost_upkeep_system` needs to be wired into the actual `ColonyResources` to actively drain materials per tick. The current test just validates the state change.
- **Refactor 2: Dynamic Ruin Stats.** The `morale_penalty_radius` and `penalty_amount` spawned by the cancellation system should probably scale based on the `ticks_building` of the canceled monument (e.g., a monument canceled at 90% completion leaves a much worse ruin than one canceled at 10%).
- **Refactor 3: Performance.** Iterating over all ruins and all pops every frame in `apply_ruin_morale_penalty_system` is O(R * P). This should be optimized, perhaps using an spatial partition grid or applying the debuff as a lingering effect rather than calculating it continuously every tick. Or use Bevy's `SpatialQuery`.

## 6. Acceptance Criteria

- [ ] `calculate_sunk_cost_upkeep_system` correctly increments the internal ticker.
- [ ] Canceling a monument despawns it and creates a `SunkCostRuin` at the exact same transform.
- [ ] `apply_ruin_morale_penalty_system` correctly applies negative morale only to Pops within the defined radius.
- [ ] All RED phase tests pass (`cargo test` returns 0 failures).
- [ ] Clippy warnings resolved.
- [ ] Test coverage ≥ 85% for new implementation.

## 7. Technical Guidance

- Bevy's ECS makes spawning and despawning entities straightforward. When handling the cancellation event, make sure the `Entity` ID passed in the event actually exists and has the `MonumentMarker` before trying to operate on it to prevent panics.
- For calculating the distance in the spatial penalty check, `Vec3::distance()` is perfectly fine for small numbers of entities, but if the pop count gets high, consider square distance comparisons (`distance_squared`) to save the square root operation.

## 8. Questions
*Builder: add questions here if spec is unclear.*
