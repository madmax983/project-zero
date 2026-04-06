# Specification: Memetic Hazards (Feature 811)

## 1. Overview
The **Memetic Hazards** feature simulates dangerous knowledge that acts like a cognitive virus. High-level research or deciphering alien artifacts yields massive XP but risks giving a Researcher a "Memetic Virus." Affected researchers become "Carriers" who, when communicating with other Pops, spread strange obsessions (like drawing symbols or refusing to sleep). This poses a tension between quarantining intellectual assets (which slows research) versus risking colony-wide infection for rapid scientific breakthroughs.

## 2. Dependencies
- `Layer 1 System`
- `Research/Science System`
- `Pop Interaction/Communication System`
- `Trait/Affliction System`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_researching_alien_artifact_grants_memetic_virus() {
        let mut app = App::new();
        app.add_systems(Update, process_artifact_research_system);

        // Arrange
        let entity = app.world_mut().spawn((
            Researcher { xp: 0 },
            ActiveResearch { artifact_type: ArtifactType::Hazardous, progress: 100.0 }
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<MemeticCarrier>(entity).is_some(), "Completing hazardous research should apply the MemeticCarrier trait");
    }

    #[test]
    fn test_memetic_virus_spreads_via_conversation() {
        let mut app = App::new();
        app.add_systems(Update, spread_memetic_hazard_system);

        // Arrange
        let carrier = app.world_mut().spawn((Pop, MemeticCarrier)).id();
        let target = app.world_mut().spawn((Pop, VulnerableMind)).id();

        app.world_mut().spawn(ConversationEvent { initiator: carrier, receiver: target });

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<MemeticCarrier>(target).is_some(), "Target pop should contract the virus after conversation with a carrier");
    }

    #[test]
    fn test_memetic_carrier_exhibits_obsessive_behavior() {
        let mut app = App::new();
        app.add_systems(Update, apply_obsession_penalty_system);

        // Arrange
        let carrier = app.world_mut().spawn((
            Pop,
            MemeticCarrier,
            TaskEfficiency { value: 1.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let efficiency = app.world().get::<TaskEfficiency>(carrier).unwrap();
        assert!(efficiency.value < 1.0, "Memetic carriers should suffer task efficiency penalties due to obsession");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Researcher {
    pub xp: u32,
}

#[derive(PartialEq, Debug)]
pub enum ArtifactType {
    Safe,
    Hazardous,
}

#[derive(Component)]
pub struct ActiveResearch {
    pub artifact_type: ArtifactType,
    pub progress: f32,
}

#[derive(Component)]
pub struct MemeticCarrier;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct VulnerableMind;

#[derive(Event)]
pub struct ConversationEvent {
    pub initiator: Entity,
    pub receiver: Entity,
}

#[derive(Component)]
pub struct TaskEfficiency {
    pub value: f32,
}

pub fn process_artifact_research_system(
    mut commands: Commands,
    mut query: Query<(Entity, &ActiveResearch, &mut Researcher)>
) {
    for (entity, research, mut researcher) in query.iter_mut() {
        if research.progress >= 100.0 {
            researcher.xp += 1000;
            if research.artifact_type == ArtifactType::Hazardous {
                commands.entity(entity).insert(MemeticCarrier);
            }
            // Assuming we remove the research after completion
            commands.entity(entity).remove::<ActiveResearch>();
        }
    }
}

pub fn spread_memetic_hazard_system(
    mut commands: Commands,
    mut events: EventReader<ConversationEvent>,
    carrier_query: Query<&MemeticCarrier>,
    vulnerable_query: Query<&VulnerableMind>
) {
    for event in events.read() {
        if carrier_query.get(event.initiator).is_ok() && vulnerable_query.get(event.receiver).is_ok() {
            commands.entity(event.receiver).insert(MemeticCarrier);
        }
    }
}

pub fn apply_obsession_penalty_system(mut query: Query<&mut TaskEfficiency, With<MemeticCarrier>>) {
    for mut efficiency in query.iter_mut() {
        efficiency.value = 0.5; // Simulate distraction
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an enum for `MemeticStrain` so that multiple different viruses can exist simultaneously (e.g., `Sleeplessness`, `CompulsiveDrawing`), each with distinct behavioral overrides.
- In `spread_memetic_hazard_system`, add a probability check based on the target Pop's `Willpower` or `Intellect` stat instead of an automatic infection.
- Add an `IsQuarantined` component that specifically disables `ConversationEvent` generation, allowing the player to mitigate spread at the cost of the researcher's productivity and mood.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Completing hazardous research consistently triggers carrier status.
- [ ] Interaction properly delegates the affliction between connected entities.

## 7. Technical Guidance
- The `ConversationEvent` is a strong Integration Seam. Ensure other ambient systems (like `idle_chatter_system`) correctly fire this event.
- If you implement `MemeticStrain`, keep it modular to allow the `Lore Master` agent to populate procedural strings describing the obsessions later.

## 8. Questions
*Builder: add questions here if spec is unclear.*
