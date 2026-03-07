# 400: Lagrange Point Hubs

## 1. Overview
In a star system (Layer 2), planetary orbits require constant fuel upkeep (Station Keeping) to avoid orbital decay. However, Gravity-neutral zones (Lagrange points, L-points) allow stations to exist without this upkeep.

Because they are "free" to occupy, they naturally attract high traffic, becoming trade hubs or, if ignored by the player, pirate dens. Neglecting the L-points forces the player to siege heavily fortified pirate bases in empty space later.

## 2. Dependencies
- `094-system-view.md` (System nodes)
- `152-orbital-stations.md` (Constructing stations)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::system::SystemNode;
    use crate::layer2::station::{OrbitalStation, StationFuelUpkeep};

    #[test]
    fn test_lagrange_point_has_zero_upkeep() {
        let mut app = App::new();
        app.add_systems(Update, calculate_station_upkeep_system);

        let lagrange_node = app.world_mut().spawn((
            SystemNode,
            LagrangePoint,
        )).id();

        let station_ent = app.world_mut().spawn((
            OrbitalStation { parent_node: lagrange_node },
            StationFuelUpkeep { base_cost: 10, current_cost: 10 },
        )).id();

        app.update();

        let upkeep = app.world().get::<StationFuelUpkeep>(station_ent).unwrap();
        assert_eq!(upkeep.current_cost, 0, "Stations at Lagrange points should have 0 upkeep");
    }

    #[test]
    fn test_pirates_spawn_in_empty_lagrange_points() {
        let mut app = App::new();
        app.insert_resource(crate::shared::time::SimulationTime { tick: 10000, speed: Default::default() });
        app.add_systems(Update, pirate_l_point_spawn_system);

        // Empty Lagrange Point
        let l_point = app.world_mut().spawn((
            SystemNode,
            LagrangePoint,
        )).id();

        app.update();

        // A pirate station/fleet should have spawned attached to this node
        let mut query = app.world_mut().query::<&PirateBase>();
        assert!(query.iter(app.world()).count() > 0, "Pirates should spawn in ignored L-Points");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::system::SystemNode;
use crate::layer2::station::{OrbitalStation, StationFuelUpkeep};
use crate::shared::time::SimulationTime;

#[derive(Component)]
pub struct LagrangePoint;

#[derive(Component)]
pub struct PirateBase {
    pub strength: u32,
}

pub fn calculate_station_upkeep_system(
    mut station_query: Query<(&OrbitalStation, &mut StationFuelUpkeep)>,
    l_point_query: Query<&LagrangePoint>,
) {
    for (station, mut upkeep) in station_query.iter_mut() {
        if l_point_query.get(station.parent_node).is_ok() {
            upkeep.current_cost = 0;
        } else {
            upkeep.current_cost = upkeep.base_cost;
        }
    }
}

pub fn pirate_l_point_spawn_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    l_point_query: Query<(Entity, &LagrangePoint)>,
    station_query: Query<&OrbitalStation>,
    pirate_query: Query<&PirateBase>,
) {
    // Mock condition: If tick > 5000, chance to spawn pirate base if empty
    if time.tick % 5000 == 0 {
        for (l_node, _) in l_point_query.iter() {
            // Check if occupied by player station
            let mut is_occupied = false;
            for station in station_query.iter() {
                if station.parent_node == l_node {
                    is_occupied = true;
                    break;
                }
            }

            // Check if already a pirate base
            // (Assuming pirate bases are linked to the node somehow, or we query them directly)
            // Simplified for MVP
            if !is_occupied {
                 commands.spawn(PirateBase { strength: 100 })
                         // In a real implementation, link this to the l_node via a Parent/Child or custom relational component
                         .insert(crate::layer2::movement::InOrbit { parent: l_node });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Lore Context:** Add a chronicle event when a Pirate Base takes root in an L-Point.
- **Trade Hubs:** If the player builds a station there, it should attract significantly more neutral merchant traffic due to the ease of docking.
- **Spatial Representation:** Ensure L-points are spawned at mathematically appropriate locations relative to planets on the System Map.

## 6. Acceptance Criteria (Testable!)
- [ ] `LagrangePoint` component removes fuel upkeep for stations orbiting it.
- [ ] Empty L-Points periodically spawn `PirateBase` entities.
- [ ] Tests pass.

## 7. Technical Guidance
- `pirate_l_point_spawn_system` should probably use a deterministic RNG based on the `SimulationTime` tick and a game seed rather than just modulo math, to prevent save-scumming the spawn.

## 8. Questions
*Builder: add questions here if spec is unclear.*
