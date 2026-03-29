# 753 - The Tethered Guillotine

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** A sword of Damocles hanging over your rebelling planets, threatening to crash the economy to maintain order.
**Mechanic:** Layer 2 orbital defense stations can "tether" to Layer 1 capital buildings. If the tethered colony's Unrest reaches critical mass and a rebellion starts, the station automatically deorbits itself into the capital, instantly crushing the rebellion but utterly annihilating the sector's infrastructure.

## 2. Dependencies
- `152-orbital-stations.md` (Layer 2 orbital stations)
- `233-public-grievances.md` (Unrest mechanic)
- `544-planetary-governance.md` (Rebellions)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::social::unrest::Unrest;
    use crate::layer2::orbital::OrbitalStation;
    use crate::layer1::map::TerrainGrid;
    use bevy::prelude::*;

    #[test]
    fn test_tethered_guillotine_drops() {
        let mut app = App::new();
        app.add_plugins(GuillotinePlugin);

        let capital_id = app.world.spawn((
            CapitalBuilding,
            Unrest { level: 100.0 }, // Rebellion threshold
        )).id();

        let station_id = app.world.spawn((
            OrbitalStation::new("Sword of Damocles"),
            TetheredTo { target: capital_id },
        )).id();

        app.update();

        // The station should be destroyed (de-orbited)
        assert!(app.world.get_entity(station_id).is_none());

        // The capital should be destroyed
        assert!(app.world.get_entity(capital_id).is_none());

        // A crater should remain
        let crater_query = app.world.query::<&Crater>().iter(&app.world).count();
        assert_eq!(crater_query, 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::social::unrest::Unrest;
use crate::layer2::orbital::OrbitalStation;

pub struct GuillotinePlugin;

impl Plugin for GuillotinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tethered_guillotine_system);
    }
}

#[derive(Component)]
pub struct CapitalBuilding;

#[derive(Component)]
pub struct TetheredTo {
    pub target: Entity,
}

#[derive(Component)]
pub struct Crater;

pub fn tethered_guillotine_system(
    mut commands: Commands,
    station_query: Query<(Entity, &TetheredTo), With<OrbitalStation>>,
    capital_query: Query<(Entity, &Unrest), With<CapitalBuilding>>,
) {
    for (station_entity, tether) in station_query.iter() {
        if let Ok((capital_entity, unrest)) = capital_query.get(tether.target) {
            if unrest.level >= 100.0 {
                // De-orbit and destroy
                commands.entity(station_entity).despawn_recursive();
                commands.entity(capital_entity).despawn_recursive();

                // Spawn crater marker (to eventually spawn crater terrain)
                commands.spawn(Crater);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
    - The `Crater` marker should spawn an actual explosion event that replaces `TerrainGrid` tiles with craters, dealing AoE damage to Pops.
    - Firing a Chronicle event when the station drops, to forever record the "Tethered Guillotine" incident.
    - UI indicator: Provide a warning to the player before it drops, like "STATION DE-ORBIT PROTOCOL INITIATED".

## 6. Acceptance Criteria (Testable!)
- [ ] `test_tethered_guillotine_drops` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- **Integration**: Link into the `simulation.rs` execution order after Unrest calculation.
- **Gotchas**: Make sure that despawning the capital building also properly cascades to deselecting it from the UI or ending related tasks.

## 8. Questions
*Builder: add questions here if spec is unclear.*
