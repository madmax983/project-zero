# 399: Prison Labor

## 1. Overview
The "Prison Labor" mechanic expands on the Justice System. Instead of merely locking arrested Pops in a Jail zone (where they consume food but produce nothing), players can assign them to "Penal Zones".

In a Penal Zone, Inmates work at extremely high efficiency, their happiness/morale needs are entirely ignored by the Utility AI, but they accumulate a hidden "Revolt Risk" modifier. If Revolt Risk gets too high, they trigger a violent breakout.

## 2. Dependencies
- `072-justice-system.md` (for the Inmate component and Jail zones)
- `118-penal-labor.md` (Refining and extending the base penal labor spec to formalize the revolt/efficiency mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::justice::Inmate;
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (apply_penal_labor_buffs_system, handle_prison_revolt_system));
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Penal);
        app.insert_resource(zone_grid);
        app
    }

    #[test]
    fn test_inmate_in_penal_zone_gains_revolt_risk() {
        let mut app = setup_app();

        let inmate = app.world_mut().spawn((
            Inmate { sentence_ticks: 1000 },
            GridPosition { x: 5, y: 5 }, // In Penal zone
            Needs { hunger: 0.5, rest: 0.5, leisure: 0.5 },
        )).id();

        app.update();

        // Should have gained PenalLabor and RevoltRisk components
        assert!(app.world().get::<PenalLabor>(inmate).is_some());

        let risk = app.world().get::<RevoltRisk>(inmate).unwrap();
        assert!(risk.current > 0.0, "Revolt risk should accumulate while working in a penal zone");
    }

    #[test]
    fn test_revolt_triggers_at_threshold() {
        let mut app = setup_app();

        let inmate = app.world_mut().spawn((
            Inmate { sentence_ticks: 1000 },
            GridPosition { x: 5, y: 5 },
            PenalLabor { efficiency_multiplier: 2.0 },
            RevoltRisk { current: 100.0, threshold: 100.0 }, // Ready to revolt
        )).id();

        app.update();

        // Inmate component should be removed (they escaped)
        assert!(app.world().get::<Inmate>(inmate).is_none(), "Inmate should lose Inmate status on revolt");

        // Should gain a Hostile/Rioter component (mocked here as generic `Hostile`)
        assert!(app.world().get::<Rioter>(inmate).is_some(), "Inmate should become a Rioter");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::justice::Inmate;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::map::GridPosition;

#[derive(Component, Debug)]
pub struct PenalLabor {
    pub efficiency_multiplier: f32,
}

#[derive(Component, Debug)]
pub struct RevoltRisk {
    pub current: f32,
    pub threshold: f32,
}

#[derive(Component, Debug)]
pub struct Rioter; // Represents an escaped, hostile prisoner

pub fn apply_penal_labor_buffs_system(
    mut commands: Commands,
    zone_grid: Res<ZoneGrid>,
    mut query: Query<(Entity, &GridPosition, Option<&mut RevoltRisk>), With<Inmate>>,
) {
    for (entity, pos, mut opt_risk) in query.iter_mut() {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Penal {
            if let Some(ref mut risk) = opt_risk {
                risk.current += 0.5; // Accumulate risk
            } else {
                // Just entered the zone, attach components
                commands.entity(entity).insert((
                    PenalLabor { efficiency_multiplier: 1.5 },
                    RevoltRisk { current: 0.0, threshold: 100.0 },
                ));
            }
        }
    }
}

pub fn handle_prison_revolt_system(
    mut commands: Commands,
    query: Query<(Entity, &RevoltRisk), With<Inmate>>,
) {
    for (entity, risk) in query.iter() {
        if risk.current >= risk.threshold {
            // Revolt!
            commands.entity(entity)
                .remove::<Inmate>()
                .remove::<PenalLabor>()
                .remove::<RevoltRisk>()
                .insert(Rioter);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with 118:** Make sure this seamlessly overlays or supersedes any stub logic placed by `118-penal-labor.md`.
- **Warden presence:** Having a Pop with a `Guard` or `Warden` job near a `PenalZone` should actively reduce the `RevoltRisk` accumulation per tick.
- **Utility AI overrides:** Ensure `PenalLabor` explicitly overrides standard Needs evaluation in `evaluate_actions_system`, so they don't stop working to go to the Tavern.

## 6. Acceptance Criteria (Testable!)
- [ ] `PenalLabor` grants a work speed multiplier.
- [ ] `RevoltRisk` increases every tick an Inmate is in a `Penal` zone.
- [ ] Inmate transforms into a `Rioter` when `RevoltRisk` reaches its threshold.
- [ ] Tests pass locally.
- [ ] Coverage meets 85%.

## 7. Technical Guidance
- `Rioter` should hook into the standard combat/faction system, essentially placing them into a hostile sub-faction on Layer 1.

## 8. Questions
*Builder: add questions here if spec is unclear.*
