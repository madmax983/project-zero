# 774: Architectural Grafting

## 1. Overview
**Layer:** 1
**Fantasy:** A sprawling, messy colony where new tech is just bolted onto old rusted frames, creating a Frankenstein city.
**Mechanic:** "Graft" high-tech modules directly onto obsolete structures to save resources. The grafted building operates at higher efficiency but inherits "Quirks" and higher maintenance debt of the base structure.

## 2. Dependencies
- Building/Infrastructure system
- Maintenance debt mechanics

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_grafting_inherits_quirks_and_debt() {
        let mut app = App::new();
        app.add_event::<GraftBuildingEvent>();
        app.add_systems(Update, process_grafting);

        let base_building = app.world_mut().spawn((
            Building { tech_level: 1 },
            MaintenanceDebt { amount: 50.0 },
            Quirks { list: vec!["Leaky Vents".to_string()] },
        )).id();

        app.world_mut().send_event(GraftBuildingEvent {
            target: base_building,
            new_module_tech_level: 3,
            efficiency_bonus: 2.0,
        });

        app.update();

        // Target should now have grafted component and increased tech level, retaining old quirks and debt
        let building = app.world().get::<Building>(base_building).unwrap();
        assert_eq!(building.tech_level, 3);

        let graft = app.world().get::<GraftedModule>(base_building).unwrap();
        assert_eq!(graft.efficiency_bonus, 2.0);

        let quirks = app.world().get::<Quirks>(base_building).unwrap();
        assert!(quirks.list.contains(&"Leaky Vents".to_string()));
        assert!(quirks.list.contains(&"Frankenstein Architecture".to_string()), "Should add grafting quirk");

        let debt = app.world().get::<MaintenanceDebt>(base_building).unwrap();
        assert!(debt.amount > 50.0, "Grafting should increase maintenance debt baseline");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Building {
    pub tech_level: u32,
}

#[derive(Component)]
pub struct MaintenanceDebt {
    pub amount: f32,
}

#[derive(Component)]
pub struct Quirks {
    pub list: Vec<String>,
}

#[derive(Component)]
pub struct GraftedModule {
    pub efficiency_bonus: f32,
}

#[derive(Event)]
pub struct GraftBuildingEvent {
    pub target: Entity,
    pub new_module_tech_level: u32,
    pub efficiency_bonus: f32,
}

pub fn process_grafting(
    mut commands: Commands,
    mut events: EventReader<GraftBuildingEvent>,
    mut buildings: Query<(&mut Building, &mut MaintenanceDebt, &mut Quirks)>,
) {
    for event in events.read() {
        if let Ok((mut building, mut debt, mut quirks)) = buildings.get_mut(event.target) {
            building.tech_level = event.new_module_tech_level;
            debt.amount += 20.0; // Grafting penalty
            quirks.list.push("Frankenstein Architecture".to_string());

            commands.entity(event.target).insert(GraftedModule {
                efficiency_bonus: event.efficiency_bonus,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: Creating a robust `Quirk` enum or specific component markers instead of string vectors would be more strongly typed.
- **Performance**: Event driven so only evaluated when grafts happen, meaning negligible overhead.
- **API Improvements**: Maybe add a system that randomly triggers events based on `Quirks` (e.g. `Leaky Vents` causes localized pollution).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Upgrades tech, adds `GraftedModule`, inherits/expands quirks, increases debt.

## 7. Technical Guidance
- String-based quirks are okay for the minimal implementation, but consider an enum for the long term if quirks have mechanical effects.
- Ensure UI can display a building as "Grafted".

## 8. Questions
*Builder: add questions here if spec is unclear.*
