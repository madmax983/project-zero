# 1089: Nanite Storms

## 1. Overview
Nanite Storms are dynamic weather events that bring both hazards and boons to the colony. A weather event where "Smart Dust" sweeps across the map. "Grey Storms" eat metal structures (damage), "Blue Storms" repair them (heal), and "Red Storms" consume biomass.

## 2. Dependencies
- Layer 1 Grid and Weather/Atmosphere system.
- Entity Component System for buildings (`StructureDurability`, `Health`).
- `ColonyResources` / Biomass structures (e.g. `Crops`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_grey_storm_damages_structures() {
        let mut app = App::new();
        let entity = app.world_mut().spawn((
            StructureDurability { current: 100, max: 100 },
            GridPosition { x: 5, y: 5 }
        )).id();

        app.add_systems(Update, apply_nanite_storm_effects);

        app.world_mut().insert_resource(ActiveNaniteStorm {
            storm_type: NaniteStormType::Grey,
            affected_area: Rect::new(0.0, 0.0, 10.0, 10.0),
            intensity: 10,
        });

        app.update();

        let durability = app.world().get::<StructureDurability>(entity).unwrap();
        assert!(durability.current < 100, "Grey Storm should damage structures within area.");
    }

    #[test]
    fn test_blue_storm_repairs_structures() {
        let mut app = App::new();
        let entity = app.world_mut().spawn((
            StructureDurability { current: 50, max: 100 },
            GridPosition { x: 5, y: 5 }
        )).id();

        app.add_systems(Update, apply_nanite_storm_effects);

        app.world_mut().insert_resource(ActiveNaniteStorm {
            storm_type: NaniteStormType::Blue,
            affected_area: Rect::new(0.0, 0.0, 10.0, 10.0),
            intensity: 20,
        });

        app.update();

        let durability = app.world().get::<StructureDurability>(entity).unwrap();
        assert!(durability.current > 50, "Blue Storm should repair structures within area.");
    }

    #[test]
    fn test_red_storm_consumes_biomass() {
        let mut app = App::new();
        let entity = app.world_mut().spawn((
            BiomassEntity { yield_amount: 100 },
            GridPosition { x: 5, y: 5 }
        )).id();

        app.add_systems(Update, apply_nanite_storm_effects);

        app.world_mut().insert_resource(ActiveNaniteStorm {
            storm_type: NaniteStormType::Red,
            affected_area: Rect::new(0.0, 0.0, 10.0, 10.0),
            intensity: 30,
        });

        app.update();

        let biomass = app.world().get::<BiomassEntity>(entity).unwrap();
        assert!(biomass.yield_amount < 100, "Red Storm should consume biomass within area.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct StructureDurability {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct BiomassEntity {
    pub yield_amount: u32,
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy)]
pub enum NaniteStormType {
    Grey, // Damages structures
    Blue, // Repairs structures
    Red,  // Consumes biomass
}

#[derive(Resource)]
pub struct ActiveNaniteStorm {
    pub storm_type: NaniteStormType,
    pub affected_area: Rect,
    pub intensity: u32,
}

pub fn apply_nanite_storm_effects(
    storm_res: Option<Res<ActiveNaniteStorm>>,
    mut structures: Query<(&mut StructureDurability, &GridPosition)>,
    mut biomasses: Query<(&mut BiomassEntity, &GridPosition)>,
) {
    if let Some(storm) = storm_res {
        for (mut dur, pos) in structures.iter_mut() {
            let p = Vec2::new(pos.x as f32, pos.y as f32);
            if storm.affected_area.contains(p) {
                match storm.storm_type {
                    NaniteStormType::Grey => {
                        dur.current = dur.current.saturating_sub(storm.intensity);
                    }
                    NaniteStormType::Blue => {
                        dur.current = (dur.current + storm.intensity).min(dur.max);
                    }
                    _ => {}
                }
            }
        }

        for (mut bio, pos) in biomasses.iter_mut() {
            let p = Vec2::new(pos.x as f32, pos.y as f32);
            if storm.affected_area.contains(p) {
                if let NaniteStormType::Red = storm.storm_type {
                    bio.yield_amount = bio.yield_amount.saturating_sub(storm.intensity);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathing/Avoidance**: AI logic (`utility_ai`) should factor in the storm. Pops should probably hide during Grey and Red storms, but might want to perform outside maintenance during Blue storms.
- **Visuals/UI**: Needs integration with rendering. The affected area should be clearly visually distinguished.
- **Spawning Logic**: The storm logic right now is just applied via a resource. It needs an event system (e.g. `NaniteStormSpawnEvent`) and a system that moves the storm across the map rather than keeping it stationary.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Active storms correctly modify structure/biomass components based on their type.

## 7. Technical Guidance
- `GridPosition` logic in reality might use the existing `TerrainGrid` coordinates or specific components. Align with existing Layer 1 spatial mapping.
- Consider utilizing a specific `Hazard` component that temporarily gets attached to tiles or entities during the storm, rather than computing collision every frame for all entities, as an optimization if entity count is high.

## 8. Questions
*Builder: Add questions here if the specification is unclear.*
