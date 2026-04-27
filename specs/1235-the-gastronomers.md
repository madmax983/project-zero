# The Gastronomers

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** A society obsessed with achieving the perfect dish inadvertently creates a galaxy-spanning crisis.
**Mechanic:** As your empire advances, a faction called "The Gastronomers" emerges. They demand increasingly exotic and dangerous ingredients from across the galaxy to achieve the "Culinary Singularity"—a meal so perfect it induces a state of permanent enlightenment. Fulfilling their requests requires diverting military fleets to hunt Leviathans or mining unstable stars for rare isotopes.

## 2. Dependencies
- Layer 1 Resource System
- Faction System (for the emergence of the "Gastronomers" faction)
- Layer 3 Fleet and Resource System (for retrieving the exotic ingredients)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_gastronomer_faction_emergence() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(EmpireAdvancement { level: 5 }); // Advanced empire
        app.init_resource::<Factions>();

        // Act
        app.add_systems(Update, spawn_gastronomer_faction_system);
        app.update();

        // Assert
        let factions = app.world().resource::<Factions>();
        assert!(factions.get(FactionId::Gastronomers).is_some(), "The Gastronomers faction should emerge in an advanced empire");
    }

    #[test]
    fn test_culinary_singularity_buff() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<ColonyResources>();
        app.world_mut().resource_mut::<ColonyResources>().exotic_ingredients = 100.0;

        let pop = app.world_mut().spawn((Pop, Needs { morale: 0.5, ..Default::default() })).id();
        app.add_event::<CulinarySingularityEvent>();

        // Act
        app.world_mut().send_event(CulinarySingularityEvent);
        app.add_systems(Update, apply_culinary_singularity_buff_system);
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert_eq!(needs.morale, 1.0, "The Culinary Singularity should grant maximum morale");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct EmpireAdvancement {
    pub level: u32,
}

#[derive(Event)]
pub struct CulinarySingularityEvent;

pub fn spawn_gastronomer_faction_system(
    advancement: Res<EmpireAdvancement>,
    mut factions: ResMut<Factions>,
) {
    if advancement.level >= 5 && factions.get(FactionId::Gastronomers).is_none() {
        factions.map.insert(
            FactionId::Gastronomers,
            FactionData {
                name: "The Gastronomers".to_string(),
                satisfaction: 1.0,
                members_count: 0,
                active_demand: None,
                state: FactionState::Loyal,
            },
        );
    }
}

pub fn apply_culinary_singularity_buff_system(
    mut events: EventReader<CulinarySingularityEvent>,
    mut query: Query<&mut Needs, With<Pop>>,
) {
    for _ in events.read() {
        for mut needs in query.iter_mut() {
            // Apply permanent max morale buff (simplified for MVP)
            needs.morale = 1.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an actual `ExoticIngredient` enum or resource tracking system for the different stages of the singularity.
- Connect the emergence of the faction directly to the layer 3 advancement metrics rather than a mocked `EmpireAdvancement` resource.
- Handle the downside and risks (e.g., supernova event) properly by integrating with Layer 3 systems.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The Gastronomers faction emerges when empire advancement criteria are met.
- [ ] Fulfilling the singularity grants a major colony-wide buff.

## 7. Technical Guidance
- **Faction Integration:** Use the existing `Factions` map and define `FactionId::Gastronomers` in the main enum.
- **Buffs:** Instead of manually setting `morale = 1.0` continuously, consider a `PermanentBuff` component that the morale system respects.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
