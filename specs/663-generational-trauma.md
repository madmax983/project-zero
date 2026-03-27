# Generational Trauma (Spec 663)

## 1. Overview
The wounds of the past don't heal; they fester and infect the children. Major disasters or atrocities (Layer 1 famines, Layer 2 orbital bombardments, Layer 3 betrayals) generate "Trauma" tokens for surviving Pops. When these Pops reproduce, a percentage of that Trauma is passed down as a permanent "Generational Trauma" trait, making descendants inherently distrustful of the original source of trauma or triggering adverse reactions to related stimuli.

## 2. Dependencies
- `src/layer1/pop.rs` (Pop entity and traits)
- `src/layer1/memory.rs` (Memory system for tracking past events)
- `src/layer1/reproduction.rs` (Pop reproduction mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_trauma_token_acquisition() {
        let mut app = App::new();
        app.add_systems(Update, process_trauma_events_system);

        let survivor = app.world_mut().spawn((
            Pop::default(),
            TraumaState::default(),
        )).id();

        // Act: Trigger a famine event
        app.world_mut().send_event(DisasterEvent {
            disaster_type: DisasterType::Famine,
            severity: 5.0,
        });
        app.update();

        // Assert: Survivor gains a trauma token related to Famine
        let trauma = app.world().get::<TraumaState>(survivor).unwrap();
        assert!(trauma.tokens.iter().any(|t| t.source == TraumaSource::Famine));
        assert_eq!(trauma.tokens[0].intensity, 5.0);
    }

    #[test]
    fn test_generational_trauma_inheritance() {
        let mut app = App::new();
        app.add_systems(Update, apply_generational_trauma_system);

        // Arrange: Parent with high trauma
        let parent = app.world_mut().spawn((
            Pop::default(),
            TraumaState {
                tokens: vec![TraumaToken {
                    source: TraumaSource::OrbitalBombardment,
                    intensity: 10.0,
                }],
            },
        )).id();

        let child = app.world_mut().spawn(Pop::default()).id();

        // Act: Reproduction event
        app.world_mut().send_event(ReproductionEvent {
            parent,
            child,
        });
        app.update();

        // Assert: Child inherits a fraction of the trauma as a permanent trait
        let child_traits = app.world().get::<PopTraits>(child).unwrap();
        assert!(child_traits.has_trait(TraitType::GenerationalTrauma(TraumaSource::OrbitalBombardment)));

        let child_trauma = app.world().get::<TraumaState>(child).unwrap();
        assert_eq!(child_trauma.tokens[0].intensity, 5.0); // 50% inheritance
    }

    #[test]
    fn test_trauma_trigger_reaction() {
        let mut app = App::new();
        app.add_systems(Update, trigger_trauma_reactions_system);

        let traumatized_pop = app.world_mut().spawn((
            Pop::default(),
            PopTraits::new(vec![TraitType::GenerationalTrauma(TraumaSource::AlienSpecies(AlienId(1)))]),
            Needs { morale: 100.0, ..default() },
        )).id();

        // Act: Alien presence detected
        app.world_mut().send_event(AlienPresenceEvent {
            alien_id: AlienId(1),
        });
        app.update();

        // Assert: Morale plummets due to trauma trigger
        let needs = app.world().get::<Needs>(traumatized_pop).unwrap();
        assert!(needs.morale < 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct TraumaState {
    pub tokens: Vec<TraumaToken>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TraumaToken {
    pub source: TraumaSource,
    pub intensity: f32,
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum TraumaSource {
    Famine,
    OrbitalBombardment,
    AlienSpecies(AlienId),
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct AlienId(pub u32);

#[derive(Event)]
pub struct DisasterEvent {
    pub disaster_type: DisasterType,
    pub severity: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DisasterType {
    Famine,
    Bombardment,
}

#[derive(Event)]
pub struct ReproductionEvent {
    pub parent: Entity,
    pub child: Entity,
}

#[derive(Event)]
pub struct AlienPresenceEvent {
    pub alien_id: AlienId,
}

#[derive(Component, Default)]
pub struct PopTraits {
    pub traits: Vec<TraitType>,
}

impl PopTraits {
    pub fn new(traits: Vec<TraitType>) -> Self {
        Self { traits }
    }

    pub fn has_trait(&self, trait_type: TraitType) -> bool {
        self.traits.contains(&trait_type)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TraitType {
    GenerationalTrauma(TraumaSource),
}

#[derive(Component, Default)]
pub struct Needs {
    pub morale: f32,
}

pub fn process_trauma_events_system(
    mut events: EventReader<DisasterEvent>,
    mut query: Query<&mut TraumaState>,
) {
    for event in events.read() {
        let source = match event.disaster_type {
            DisasterType::Famine => TraumaSource::Famine,
            DisasterType::Bombardment => TraumaSource::OrbitalBombardment,
        };

        for mut trauma_state in query.iter_mut() {
            trauma_state.tokens.push(TraumaToken {
                source: source.clone(),
                intensity: event.severity,
            });
        }
    }
}

pub fn apply_generational_trauma_system(
    mut events: EventReader<ReproductionEvent>,
    mut commands: Commands,
    parent_query: Query<&TraumaState>,
) {
    for event in events.read() {
        if let Ok(parent_trauma) = parent_query.get(event.parent) {
            let mut inherited_tokens = Vec::new();
            let mut inherited_traits = Vec::new();

            for token in &parent_trauma.tokens {
                let inherited_intensity = token.intensity * 0.5;
                if inherited_intensity > 0.0 {
                    inherited_tokens.push(TraumaToken {
                        source: token.source.clone(),
                        intensity: inherited_intensity,
                    });
                    inherited_traits.push(TraitType::GenerationalTrauma(token.source.clone()));
                }
            }

            commands.entity(event.child).insert((
                TraumaState { tokens: inherited_tokens },
                PopTraits::new(inherited_traits),
            ));
        }
    }
}

pub fn trigger_trauma_reactions_system(
    mut events: EventReader<AlienPresenceEvent>,
    mut query: Query<(&PopTraits, &mut Needs)>,
) {
    for event in events.read() {
        for (traits, mut needs) in query.iter_mut() {
            if traits.has_trait(TraitType::GenerationalTrauma(TraumaSource::AlienSpecies(event.alien_id.clone()))) {
                needs.morale = (needs.morale - 60.0).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Data Normalization:** `TraumaState` tokens might accumulate indefinitely. Implement a mechanism to decay trauma over time or cap the number of tokens.
- **Event Filtering:** `process_trauma_events_system` currently applies trauma to *all* pops with `TraumaState`. We need a way to filter based on proximity to the disaster or actual exposure.
- **Component Design:** `PopTraits` might overlap with existing trait systems. Ensure this integrates cleanly with existing Bevy ECS patterns for traits in the colony layer.
- **Inheritance Logic:** The 50% inheritance is hardcoded. This should ideally be a configurable parameter, potentially modified by cultural or medical technologies.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops acquire trauma tokens during specific disaster events.
- [ ] Child entities inherit a portion of the parent's trauma as traits.
- [ ] Traumatized pops suffer specific penalties (e.g., morale loss) when exposed to related triggers.

## 7. Technical Guidance
- **Integration Seam:** This system acts as a bridge between Layer 1 memory/needs and higher-level events. Ensure the `DisasterEvent` triggers properly translate from Layer 2/3 mechanics to Layer 1 components.
- **Performance:** Iterating through all traits and trauma tokens during every trigger event could be slow. Consider using specific marker components (e.g., `TraumatizedBy(AlienId)`) to speed up queries if the number of pops gets very large.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
