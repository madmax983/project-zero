# Spec 606: Biocompatibility

## 1. Overview
**Layer:** 1
**Fantasy:** The planet's biology rejects you.
**Mechanic:** Pops have a "Biocompatibility" rating with the local flora/atmosphere. Low rating = sickness/slower work in unsealed areas. Can be improved via gene-modding or drugs.
**Emergence:** Your best miner is violently allergic to the planet and has to work a desk job or live in a suit.
**Tension:** Modify the planet (Terraforming) or modify the people (Adaptation)?

## 2. Dependencies
- `src/layer1/pop.rs` (Pop component)
- `src/layer1/health.rs` (Condition tracking, specifically allergies/sickness)
- `src/layer1/atmosphere.rs` or `src/layer1/environment.rs` (Tracking if an area is sealed/unsealed)
- `src/layer1/genetics.rs` or `src/layer1/medical.rs` (Systems for gene-modding or drugs to improve rating)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod biocompatibility_tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_biocompatibility_initialization() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, initialize_biocompatibility_system);

        // Act
        let entity = app.world_mut().spawn(Pop).id();
        app.update();

        // Assert
        let rating = app.world().get::<Biocompatibility>(entity);
        assert!(rating.is_some(), "Pop should have a Biocompatibility rating assigned.");
    }

    #[test]
    fn test_low_biocompatibility_in_unsealed_area_causes_sickness() {
        // Arrange
        let mut app = App::new();
        app.add_event::<AllergicReactionEvent>();
        app.add_systems(Update, process_environmental_allergies_system);

        // Act: Spawn a pop with low compatibility in an unsealed environment.
        let entity = app.world_mut().spawn((
            Pop,
            Biocompatibility { level: 10.0 }, // Low rating out of 100
            UnsealedEnvironment,
        )).id();

        app.update();

        // Assert
        let events = app.world().resource::<Events<AllergicReactionEvent>>();
        let mut reader = events.get_reader();
        let reactions = reader.read(events).collect::<Vec<_>>();
        assert_eq!(reactions.len(), 1, "An allergic reaction event should be triggered.");
        assert_eq!(reactions[0].entity, entity);
    }

    #[test]
    fn test_high_biocompatibility_prevents_sickness() {
        // Arrange
        let mut app = App::new();
        app.add_event::<AllergicReactionEvent>();
        app.add_systems(Update, process_environmental_allergies_system);

        // Act: Spawn a pop with high compatibility in an unsealed environment.
        let entity = app.world_mut().spawn((
            Pop,
            Biocompatibility { level: 90.0 }, // High rating out of 100
            UnsealedEnvironment,
        )).id();

        app.update();

        // Assert
        let events = app.world().resource::<Events<AllergicReactionEvent>>();
        assert!(events.is_empty(), "No allergic reaction should occur for high compatibility pops.");
    }

    #[test]
    fn test_sealed_area_prevents_sickness() {
        // Arrange
        let mut app = App::new();
        app.add_event::<AllergicReactionEvent>();
        app.add_systems(Update, process_environmental_allergies_system);

        // Act: Spawn a pop with low compatibility in a sealed environment.
        let entity = app.world_mut().spawn((
            Pop,
            Biocompatibility { level: 10.0 },
            SealedEnvironment, // They are safe here
        )).id();

        app.update();

        // Assert
        let events = app.world().resource::<Events<AllergicReactionEvent>>();
        assert!(events.is_empty(), "No allergic reaction should occur in a sealed area.");
    }

    #[test]
    fn test_biocompatibility_improvement_via_drugs() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_medical_treatments_system);

        let entity = app.world_mut().spawn((
            Pop,
            Biocompatibility { level: 10.0 },
            ActiveTreatment { drug: DrugType::Antihistamine, duration: 10 },
        )).id();

        // Act
        app.update();

        // Assert
        let rating = app.world().get::<Biocompatibility>(entity).unwrap();
        assert!(rating.level > 10.0, "Drug treatment should improve the biocompatibility rating.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Biocompatibility {
    pub level: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct UnsealedEnvironment;

#[derive(Component)]
pub struct SealedEnvironment;

#[derive(Event)]
pub struct AllergicReactionEvent {
    pub entity: Entity,
}

pub enum DrugType {
    Antihistamine,
}

#[derive(Component)]
pub struct ActiveTreatment {
    pub drug: DrugType,
    pub duration: u32,
}

pub fn initialize_biocompatibility_system(
    mut commands: Commands,
    query: Query<Entity, (With<Pop>, Without<Biocompatibility>)>,
) {
    for entity in query.iter() {
        // Base randomization for compatibility.
        commands.entity(entity).insert(Biocompatibility { level: 50.0 });
    }
}

pub fn process_environmental_allergies_system(
    query: Query<(Entity, &Biocompatibility), With<UnsealedEnvironment>>,
    mut events: EventWriter<AllergicReactionEvent>,
) {
    for (entity, compatibility) in query.iter() {
        if compatibility.level < 40.0 {
            events.send(AllergicReactionEvent { entity });
        }
    }
}

pub fn process_medical_treatments_system(
    mut query: Query<(&mut Biocompatibility, &ActiveTreatment)>,
) {
    for (mut compatibility, treatment) in query.iter_mut() {
        if matches!(treatment.drug, DrugType::Antihistamine) {
            compatibility.level += 5.0; // Minimal implementation of stat boost.
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor Opportunity:** Replace `UnsealedEnvironment` and `SealedEnvironment` marker components with a spatial query against a unified `AtmosphereGrid` or `RoomSystem` to dynamically determine if a pop is exposed to local flora/atmosphere based on their world position.
- **Refactor Opportunity:** Extract the specific threshold (`40.0`) for allergic reactions into a configurable tuning parameter or resource (`PlanetBiomeConstants`), allowing different planets to have harsher default penalty thresholds.
- **Performance:** Ensure the `process_environmental_allergies_system` only runs periodically (e.g., using a `Time<Fixed>` schedule or custom timer) rather than every tick, to reduce ECS overhead.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `Biocompatibility` component accurately triggers negative health events when exposed to unsealed environments.

## 7. Technical Guidance
- Implement this module in `src/layer1/biocompatibility.rs`.
- Ensure integration with the existing `Needs` and `Health` systems so that `AllergicReactionEvent` translates into actual mechanical penalties (like slower movement speed or increased stress).
- Consider making the base compatibility rating dependent on the starting scenario or initial colonist gene templates.

## 8. Questions
*Builder: add questions here if spec is unclear.*
