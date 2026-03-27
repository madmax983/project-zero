# 670 - Cultural Projection

## 1. Overview
The "Cultural Projection" feature enables a colony to project "Soft Power" across Layer 3 by producing high amounts of Luxury and Art. This cultural output exerts pressure on neighboring factions, causing their pops to demand your goods and reducing their leaders' authority if they attempt to embargo you. If the pressure is high enough, enemy soldiers or citizens may defect to your colony during a conflict, realizing the "Your blue jeans and rock music conquer the galaxy" fantasy.

## 2. Dependencies
- Layer 1 Art and Luxury production (`ColonyResources.art`, `ColonyResources.luxury`)
- Layer 3 Diplomatic Relations and Factions (`Civilization`, `DiplomacyState`)
- Chronicle System for defections and cultural milestones

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer3::factions::{Civilization, DiplomacyState, DiplomaticStance};

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.insert_resource(CulturalInfluenceGrid::default());
        app.add_event::<DefectionEvent>();
        app.add_systems(Update, (
            calculate_cultural_pressure_system,
            apply_cultural_pressure_system,
            process_defections_system,
        ));
        app
    }

    #[test]
    fn test_high_art_luxury_generates_cultural_pressure() {
        let mut app = setup_app();

        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        resources.art = 500.0;
        resources.luxury = 300.0;

        app.update();

        // Assert the cultural influence grid or score has increased
        let influence = app.world().get_resource::<CulturalInfluenceGrid>().unwrap();
        assert!(influence.total_pressure > 100.0, "High art and luxury should generate significant cultural pressure");
    }

    #[test]
    fn test_cultural_pressure_causes_enemy_defections_during_war() {
        let mut app = setup_app();

        // Setup high cultural pressure
        app.world_mut().resource_mut::<CulturalInfluenceGrid>().total_pressure = 1000.0;

        let enemy_civ = app.world_mut().spawn((
            Civilization { name: "The Hegemony".to_string() },
            DiplomacyState { stance: DiplomaticStance::War, opinion: -50.0 },
            CulturalVulnerability { threshold: 500.0 }, // Defect if pressure > threshold
        )).id();

        app.update();

        // Assert defection event was fired
        let defection_events = app.world().get_resource::<Events<DefectionEvent>>().unwrap();
        let mut reader = defection_events.get_reader();
        let ev = reader.read(defection_events).next().unwrap();

        assert_eq!(ev.source_civ, enemy_civ, "Enemy civ should suffer defections due to overwhelming cultural pressure");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer3::factions::{Civilization, DiplomacyState, DiplomaticStance};

#[derive(Resource, Default)]
pub struct CulturalInfluenceGrid {
    pub total_pressure: f32,
}

#[derive(Component)]
pub struct CulturalVulnerability {
    pub threshold: f32,
}

#[derive(Event)]
pub struct DefectionEvent {
    pub source_civ: Entity,
    pub amount: u32,
}

pub fn calculate_cultural_pressure_system(
    resources: Res<ColonyResources>,
    mut influence: ResMut<CulturalInfluenceGrid>,
) {
    // Formula: (Art * 1.5) + Luxury = Pressure
    influence.total_pressure = (resources.art * 1.5) + resources.luxury;
}

pub fn apply_cultural_pressure_system(
    influence: Res<CulturalInfluenceGrid>,
    mut civs: Query<(Entity, &mut DiplomacyState, &CulturalVulnerability), With<Civilization>>,
    mut defection_events: EventWriter<DefectionEvent>,
) {
    for (civ_entity, mut diplomacy, vulnerability) in civs.iter_mut() {
        if influence.total_pressure > vulnerability.threshold {
            // Apply diplomatic penalty if they are hostile
            if diplomacy.stance == DiplomaticStance::War {
                // Defection logic
                defection_events.send(DefectionEvent {
                    source_civ: civ_entity,
                    amount: 10, // Base defection rate
                });

                // Reduce their opinion slightly as they grow resentful of your influence
                diplomacy.opinion -= 1.0;
            } else if diplomacy.stance == DiplomaticStance::Neutral {
                // If neutral, they are drawn closer to alliance
                diplomacy.opinion += 0.5;
            }
        }
    }
}

pub fn process_defections_system(
    mut defection_events: EventReader<DefectionEvent>,
    mut commands: Commands,
) {
    for ev in defection_events.read() {
        // Here, logic would spawn incoming refugee/defector ships on Layer 2
        // For minimal implementation, we log it or add pops to Layer 1
        println!("Received {} defectors from Civ {:?}", ev.amount, ev.source_civ);
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Decay:** `total_pressure` shouldn't instantly mirror resources. It should be a running average or cumulative score that decays over time if `art` and `luxury` drop, simulating "fading trends".
- **Layer 2 Bridge:** In `process_defections_system`, spawn an actual `Fleet` entity on Layer 2 heading towards the player's colony. When it arrives, they spawn as Pops on Layer 1.
- **Chronicle Event:** Send an `AddChronicleEvent` documenting the defection ("Enemy soldiers lay down their arms to defect to our paradise").

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Code coverage is ≥85%.
- [ ] `CulturalInfluenceGrid` increases based on Art and Luxury resources.
- [ ] A `DefectionEvent` is sent when `total_pressure` exceeds an enemy civ's `threshold`.

## 7. Technical Guidance

- **Location:** This spans Layer 1 and Layer 3. Put the calculation in `src/layer1/social/culture.rs` and the application/diplomacy effects in `src/layer3/diplomacy/cultural_pressure.rs`.
- **System Ordering:** Ensure `calculate_cultural_pressure_system` runs after resources are updated, but before `apply_cultural_pressure_system`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
