# 594: Predictive Policing

## 1. Overview
The Minority Report. Stopping crime before it happens. Security stations with high-tech sensors/AI predict "Mental Breaks" or "Crimes". You can preemptively arrest pops with high probability, even if they haven't done anything yet. You arrest a popular hero because they were having a bad day. The colony riots against your tyranny. Tension: Security (prevention) vs. Liberty (innocence).

## 2. Dependencies
- Requires Layer 1 Social/Needs system for Unrest and Mental Breaks.

## 3. RED Phase: Tests First
```rust
use bevy::prelude::*;
use crate::layer1::needs::Stress;

#[test]
fn test_predictive_policing_flags_high_stress() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, predictive_policing_scan_system);

    // Pop with very high stress (likely to break)
    let risky_pop = app.world_mut().spawn(Stress { value: 95.0 }).id();
    // Normal pop
    let safe_pop = app.world_mut().spawn(Stress { value: 10.0 }).id();

    // Add security station
    app.world_mut().spawn(SecurityStation { predictive_active: true });

    // Act
    app.update();

    // Assert
    assert!(app.world().entity(risky_pop).contains::<PreCrimeSuspect>());
    assert!(!app.world().entity(safe_pop).contains::<PreCrimeSuspect>());
}

#[test]
fn test_preemptive_arrest_causes_unrest() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, execute_preemptive_arrest_system);

    let pop = app.world_mut().spawn((PreCrimeSuspect, Popularity { value: 80 })).id(); // High popularity

    // Act
    // Simulate arresting the pop
    app.world_mut().entity_mut(pop).insert(Arrested);
    app.update();

    // Assert
    // Check that global unrest spiked due to arresting an innocent, popular person
    let unrest = app.world().resource::<GlobalUnrest>();
    assert!(unrest.value > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Stress {
    pub value: f32,
}

#[derive(Component)]
pub struct SecurityStation {
    pub predictive_active: bool,
}

#[derive(Component)]
pub struct PreCrimeSuspect;

#[derive(Component)]
pub struct Popularity {
    pub value: u32,
}

#[derive(Component)]
pub struct Arrested;

#[derive(Resource, Default)]
pub struct GlobalUnrest {
    pub value: f32,
}

pub fn predictive_policing_scan_system(
    mut commands: Commands,
    query: Query<(Entity, &Stress), Without<PreCrimeSuspect>>,
    station_query: Query<&SecurityStation>,
) {
    let is_active = station_query.iter().any(|s| s.predictive_active);
    if is_active {
        for (entity, stress) in query.iter() {
            if stress.value > 90.0 {
                commands.entity(entity).insert(PreCrimeSuspect);
            }
        }
    }
}

pub fn execute_preemptive_arrest_system(
    query: Query<(&Popularity, &PreCrimeSuspect), Added<Arrested>>,
    mut unrest: ResMut<GlobalUnrest>,
) {
    for (popularity, _) in query.iter() {
        // Arresting a popular pre-crime suspect causes massive unrest
        unrest.value += (popularity.value as f32) * 0.5;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Define the `PreCrimeSuspect` component in `layer1/security.rs`.
- Extract the unrest calculation formula into a configurable constant or function to allow balance tweaking without touching the core system.
- Integrate with the existing `Unrest` system rather than a standalone `GlobalUnrest` resource.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] Pops with Stress > 90 are flagged as `PreCrimeSuspect` if predictive policing is active.
- [ ] Arresting a `PreCrimeSuspect` before they commit a crime generates significant Unrest, scaled by their popularity.

## 7. Technical Guidance
- Ensure `PreCrimeSuspect` markers decay or are removed if the pop's stress returns to normal.
- Be careful with `Added<Arrested>` to avoid triggering unrest multiple times for the same arrest.

## 8. Questions
*Builder: add questions here if spec is unclear.*
