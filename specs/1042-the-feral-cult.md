# 1042: The Feral Cult

## 1. Overview
Pops whose morale or needs drop to critically low levels for an extended period have a chance to "break" and join the Feral Cult. They abandon their normal jobs, refuse to use standard housing, and instead gather around high-energy or industrial buildings, claiming them as holy sites. They will forcefully defend these sites from non-cultist pops but will occasionally over-boost the building's output.

## 2. Dependencies
- Layer 1 `Morale` system and `Pop` component.
- Target building system (e.g., `EnergyPlant`, `MiningDrill`).
- Action evaluation system to override normal utility AI.

## 3. RED Phase: Tests First
```rust
#[test]
fn test_pop_joins_feral_cult_on_prolonged_low_morale() {
    // Arrange
    let mut app = setup_test_app();
    let pop = app.world.spawn((
        Pop,
        Morale { value: 10.0, modifiers: vec![] }, // Critically low
    )).id();

    // Act
    app.update();

    // Assert
    assert!(app.world.entity(pop).contains::<FeralCultist>());
}

#[test]
fn test_feral_cultist_boosts_building_output() {
    // Arrange
    let mut app = setup_test_app();
    let building = app.world.spawn((
        Building,
        EnergyPlant { output: 100 },
        HolySite { cultists_present: 5 },
    )).id();

    // Act
    app.update(); // Run cultist boost system

    // Assert
    let plant = app.world.entity(building).get::<EnergyPlant>().unwrap();
    assert!(plant.output > 100, "Cultists should over-boost output");
}

#[test]
fn test_feral_cultist_ignores_normal_jobs() {
    // Arrange
    let mut app = setup_test_app();
    let pop = app.world.spawn((
        Pop,
        FeralCultist,
        Job { workplace: Entity::PLACEHOLDER, job_type: AssignmentType::Miner },
    )).id();

    // Act
    app.update(); // Run utility AI / job assignment

    // Assert
    let job = app.world.entity(pop).get::<Job>();
    assert!(job.is_none() || job.unwrap().job_type == AssignmentType::CultActivity);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// 1. Define FeralCultist component
#[derive(Component)]
pub struct FeralCultist;

// 2. System to convert low-morale pops
pub fn cult_conversion_system(
    mut commands: Commands,
    query: Query<(Entity, &Morale), (With<Pop>, Without<FeralCultist>)>
) {
    for (entity, morale) in query.iter() {
        if morale.value < 20.0 { // Threshold
            commands.entity(entity).insert(FeralCultist);
            commands.entity(entity).remove::<Job>(); // Abandon jobs
        }
    }
}

// 3. Define HolySite and boosting logic
#[derive(Component)]
pub struct HolySite {
    pub cultists_present: u32,
}

pub fn cult_boosting_system(
    mut query: Query<(&HolySite, &mut EnergyPlant)>
) {
    for (site, mut plant) in query.iter_mut() {
        if site.cultists_present > 0 {
            plant.output = plant.output * 2; // Simple 200% boost
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Configurable Thresholds:** Move the `20.0` morale threshold into a `Res<SimulationSettings>`.
- **Timer/Duration:** A pop shouldn't instantly convert on a dip; require prolonged low morale using a timer or accumulator.
- **Action System Integration:** Instead of just removing `Job`, properly integrate `FeralCultist` into the `utility_ai.rs` so they naturally score "Worship at Holy Site" higher than anything else.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] `FeralCultist` component is successfully applied to Pops with low morale.
- [ ] Buildings targeted by cultists exhibit increased output.

## 7. Technical Guidance
- **Integration Point:** `src/layer1/systems/morale.rs` or `utility_ai.rs` for the conversion logic.
- **Gotcha:** Ensure the output boost isn't applied recursively every tick (e.g., store base output and calculate boosted output, or use a buff system).
- **Performance:** When cultists search for a Holy Site, cache the locations or use spatial queries to avoid O(N^2) searches.

## 8. Questions
*Builder: add questions here if spec is unclear.*
