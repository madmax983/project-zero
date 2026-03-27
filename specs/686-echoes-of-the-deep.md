# Echoes of the Deep

## 1. Overview
Mining too deep doesn't wake a monster; it wakes a memory of something that shouldn't exist. Deep-crust mining operations occasionally uncover "Resonance Chambers." These are not physical structures, but localized areas where the laws of physics are slightly off, causing auditory and visual hallucinations in Pops. Working in these areas provides a massive, unexplained boost to Research output, but causes Pops to develop the "Paranoid" or "Obsessive" traits, eventually leading to violent psychotic breaks.

## 2. Dependencies
- `018` Mining and Resources
- `051` Pop Skills and Experience
- `127` Stress Breakdowns

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::TerrainGrid;
    use crate::layer1::pop::{Pop, Traits};
    use crate::layer1::work::Workstation;
    use crate::layer1::research::ResearchProgress;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (process_resonance_chamber, update_paranoid_pops));
        app.insert_resource(ResearchProgress::default());
        app
    }

    #[test]
    fn test_working_in_resonance_chamber_boosts_research() {
        // Arrange
        let mut app = setup_app();
        let chamber = app.world_mut().spawn((
            ResonanceChamber { intensity: 1.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            Pop::default(),
            Working(chamber),
            Traits::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert - Pop is researching wildly fast
        let progress = app.world().resource::<ResearchProgress>();
        assert!(progress.amount > 10.0, "Research should be massively boosted by chamber");
    }

    #[test]
    fn test_working_in_resonance_chamber_adds_paranoid_trait() {
        // Arrange
        let mut app = setup_app();
        let chamber = app.world_mut().spawn((
            ResonanceChamber { intensity: 100.0 }, // High exposure
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            Pop::default(),
            Working(chamber),
            Traits::default(),
            MentalStrain(0.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        // Act
        for _ in 0..10 { app.update(); } // Accumulate strain

        // Assert
        let traits = app.world().get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::Paranoid), "Pop should become Paranoid from exposure");
    }

    #[test]
    fn test_paranoid_pops_sabotage_machinery() {
        // Arrange
        let mut app = setup_app();
        let pop = app.world_mut().spawn((
            Pop::default(),
            Traits(vec![Trait::Paranoid]),
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        let machine = app.world_mut().spawn((
            Workstation { integrity: 100.0 },
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Act
        app.update(); // Paranoid behavior check

        // Assert
        let station = app.world().get::<Workstation>(machine).unwrap();
        assert!(station.integrity < 100.0, "Paranoid Pop should have sabotaged nearby machine");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Traits, Trait};
use crate::layer1::work::Workstation;
use crate::layer1::research::ResearchProgress;

#[derive(Component)]
pub struct ResonanceChamber {
    pub intensity: f32,
}

#[derive(Component)]
pub struct Working(pub Entity);

#[derive(Component, Default)]
pub struct MentalStrain(pub f32);

pub fn process_resonance_chamber(
    mut research: ResMut<ResearchProgress>,
    chambers: Query<(&ResonanceChamber, Entity)>,
    mut workers: Query<(&mut MentalStrain, &mut Traits, &Working)>,
) {
    for (mut strain, mut traits, working) in workers.iter_mut() {
        if let Ok((chamber, _)) = chambers.get(working.0) {
            // Massive research boost
            research.amount += 15.0 * chamber.intensity;

            // Mental degradation
            strain.0 += 10.0 * chamber.intensity;
            if strain.0 > 100.0 && !traits.has(Trait::Paranoid) {
                traits.add(Trait::Paranoid);
                println!("A Pop has become Paranoid from the Deep Echoes.");
            }
        }
    }
}

pub fn update_paranoid_pops(
    mut machines: Query<(&mut Workstation, &Transform)>,
    pops: Query<(&Traits, &Transform), With<Pop>>,
) {
    for (traits, pop_transform) in pops.iter() {
        if traits.has(Trait::Paranoid) {
            // Paranoid pop sabotages machines
            for (mut station, station_transform) in machines.iter_mut() {
                if pop_transform.translation.distance(station_transform.translation) < 2.0 {
                    station.integrity -= 25.0; // Break the machine
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Utility AI:** Pop needs should reflect their paranoia. They might actively seek to destroy things like Life Support ("The air is lying!") instead of just breaking generic adjacent workstations.
- **Visuals/Audio:** Add shader effects (e.g., slight chromatic aberration) when the player's camera is over a Resonance Chamber, and distorted ambient audio.
- **Curing Paranoia:** Tie this into `038` Medical Care. Can psychiatrists cure Resonance-induced paranoia, or is it permanent?

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Resonance Chambers dramatically boost research output for Pops working in them.
- [ ] Prolonged exposure adds the `Paranoid` trait.
- [ ] Paranoid Pops deal sabotage damage to nearby workstations.

## 7. Technical Guidance
- `ResonanceChamber` should probably be an invisible volume or a tile-property overlaid on deep mining z-levels rather than a physical building.
- Strain accumulation should scale with the Pop's existing Intelligence/Willpower stats.

## 8. Questions
*Builder: add questions here if spec is unclear.*
