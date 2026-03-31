# 770: Temporal Ghost Towns

## 1. Overview
Building your future on top of your past, only to find the past isn't entirely gone. Specific map tiles occasionally "stutter" in time, causing buildings on these tiles to temporarily revert to whatever structure occupied them decades ago before snapping back.

## 2. Dependencies
- Layer 1 Construction / Map System (`GridPosition`, `Building`)
- Layer 1 Resource System (`SimulationTime` or `GlobalTime`)
- Layer 1 Morale System (for Unrest/Sacrilege penalties)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{GridPosition, Building, BuildingType};
    use crate::layer1::time::SimulationTime;

    #[test]
    fn test_temporal_stutter_reverts_building() {
        let mut app = App::new();
        app.add_systems(Update, process_temporal_stutters);

        let position = GridPosition { x: 5, y: 5 };

        let building = app.world_mut().spawn((
            position,
            Building { btype: BuildingType::FusionReactor, ..default() },
            TemporalHistory { past_btype: BuildingType::WoodenHut, active_stutter: false },
            ChronallyUnstableTile,
        )).id();

        // Trigger a temporal anomaly on the tile
        app.world_mut().send_event(TemporalStutterEvent { entity: building });

        app.update();

        // Assert the building has temporarily reverted to its past type
        let b = app.world().get::<Building>(building).unwrap();
        assert_eq!(b.btype, BuildingType::WoodenHut);

        let h = app.world().get::<TemporalHistory>(building).unwrap();
        assert!(h.active_stutter);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::{GridPosition, Building, BuildingType};

#[derive(Component)]
pub struct ChronallyUnstableTile;

#[derive(Component)]
pub struct TemporalHistory {
    pub past_btype: BuildingType,
    pub active_stutter: bool,
}

#[derive(Event)]
pub struct TemporalStutterEvent {
    pub entity: Entity,
}

pub fn process_temporal_stutters(
    mut events: EventReader<TemporalStutterEvent>,
    mut buildings: Query<(&mut Building, &mut TemporalHistory), With<ChronallyUnstableTile>>,
) {
    for event in events.read() {
        if let Ok((mut b, mut h)) = buildings.get_mut(event.entity) {
            if !h.active_stutter {
                b.btype = h.past_btype;
                h.active_stutter = true;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor:** Create a `TemporalRecoveryEvent` or use a timer component to cleanly handle reverting the structure back to its modern state after a set duration.
- **Smell:** Relying entirely on `BuildingType` mutation might bypass specific initialization logic required by the modern building (e.g., losing power output variables when snapping back to Fusion Reactor). Consider hiding/disabling the modern building entity and temporarily spawning a "past ghost" entity instead.
- **Design:** Ensure that pops attempting to work in a temporally stuttered building handle the disruption correctly (e.g., job interruption, returning to base).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Buildings on chronally unstable tiles can temporarily revert to a past state.
- [ ] A temporal stutter correctly flags the active anomaly.

## 7. Technical Guidance
- Map generation should designate specific tiles as chronally unstable, but keep this rare.
- The `TemporalHistory` component should track the earliest structure ever built on that tile.
- Consider emitting Morale events for workers stationed at a modern building when it turns into something sacrilegious (like a graveyard).

## 8. Questions
*Builder: add questions here if spec is unclear.*
