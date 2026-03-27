# The Scrap-Code Evangelists (Spec 664)

## 1. Overview
A cult that worships a virus, believing it to be the voice of a dead god. If the Scrapcode event occurs frequently without being fully purged, infected machines occasionally broadcast seemingly coherent, prophetic messages. Pops with low Morale or the "Superstitious" trait can become "Scrap-Code Evangelists." They stop working to actively spread the Scrapcode via physical interaction with clean machines, believing they are "freeing" the AI.

## 2. Dependencies
- `src/layer1/pop.rs` (Pop entity, traits, and utility AI)
- `src/layer1/machines.rs` (Machine entities and their states)
- `src/layer1/events.rs` (Scrapcode infection events)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_evangelist_conversion() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_evangelist_conversion_system);

        let machine = app.world_mut().spawn((
            Machine::default(),
            InfectionState { level: 100.0, is_broadcasting: true },
        )).id();

        let pop = app.world_mut().spawn((
            Pop::default(),
            Needs { morale: 10.0, ..default() }, // Low morale
            PopTraits::new(vec![TraitType::Superstitious]),
        )).id();

        // Act: Machine broadcasts prophetic scrapcode
        app.world_mut().send_event(ScrapcodeBroadcastEvent {
            source: machine,
            message: "The binary speaks the truth".into(),
        });
        app.update();

        // Assert: Pop is converted to an evangelist
        let traits = app.world().get::<PopTraits>(pop).unwrap();
        assert!(traits.has_trait(TraitType::ScrapCodeEvangelist));

        // Evangelists stop working their normal jobs
        let utility = app.world().get::<UtilityAI>(pop).unwrap();
        assert_eq!(utility.current_action, ActionType::PreachScrapcode);
    }

    #[test]
    fn test_evangelist_spreading_infection() {
        let mut app = App::new();
        app.add_systems(Update, evangelist_action_system);

        let evangelist = app.world_mut().spawn((
            Pop::default(),
            PopTraits::new(vec![TraitType::ScrapCodeEvangelist]),
            Position { x: 5, y: 5 },
        )).id();

        let clean_machine = app.world_mut().spawn((
            Machine::default(),
            InfectionState { level: 0.0, is_broadcasting: false },
            Position { x: 5, y: 5 }, // Co-located
        )).id();

        // Act: Evangelist preaches to the machine
        app.world_mut().send_event(EvangelistPreachEvent {
            evangelist,
            target: clean_machine,
        });
        app.update();

        // Assert: Machine becomes infected
        let infection = app.world().get::<InfectionState>(clean_machine).unwrap();
        assert!(infection.level > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Machine {}

#[derive(Component, Default)]
pub struct InfectionState {
    pub level: f32,
    pub is_broadcasting: bool,
}

#[derive(Component, Default)]
pub struct Needs {
    pub morale: f32,
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
    Superstitious,
    ScrapCodeEvangelist,
}

#[derive(Component, Default)]
pub struct UtilityAI {
    pub current_action: ActionType,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum ActionType {
    #[default]
    Idle,
    Work,
    PreachScrapcode,
}

#[derive(Component, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Event)]
pub struct ScrapcodeBroadcastEvent {
    pub source: Entity,
    pub message: String,
}

#[derive(Event)]
pub struct EvangelistPreachEvent {
    pub evangelist: Entity,
    pub target: Entity,
}

pub fn evaluate_evangelist_conversion_system(
    mut events: EventReader<ScrapcodeBroadcastEvent>,
    mut query: Query<(&mut PopTraits, &Needs, &mut UtilityAI)>,
) {
    for event in events.read() {
        // In a real implementation, proximity checks would be needed here
        for (mut traits, needs, mut utility) in query.iter_mut() {
            if needs.morale < 30.0 || traits.has_trait(TraitType::Superstitious) {
                if !traits.has_trait(TraitType::ScrapCodeEvangelist) {
                    traits.traits.push(TraitType::ScrapCodeEvangelist);
                    utility.current_action = ActionType::PreachScrapcode;
                }
            }
        }
    }
}

pub fn evangelist_action_system(
    mut events: EventReader<EvangelistPreachEvent>,
    mut query: Query<&mut InfectionState>,
) {
    for event in events.read() {
        if let Ok(mut infection) = query.get_mut(event.target) {
            infection.level += 20.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Proximity:** The conversion logic currently assumes all pops hear the broadcast regardless of distance. Add spatial checks to limit the radius of effect.
- **Conversion Probability:** Conversion shouldn't be guaranteed just because morale is low. Implement a probability check based on morale deficit and existing traits.
- **Action Type Integration:** `ActionType::PreachScrapcode` should integrate with the existing Utility AI system to properly score the action against other needs, rather than hardcoding the override.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops with low morale or specific traits are converted to Evangelists upon hearing a broadcast.
- [ ] Evangelists prioritize spreading the infection over regular work.
- [ ] Clean machines become infected when preached to by an Evangelist.

## 7. Technical Guidance
- **Utility AI:** The core of this spec lies in correctly hooking into the Utility AI system. The `PreachScrapcode` action should have a very high utility weight for pops with the `ScrapCodeEvangelist` trait, ensuring they abandon regular tasks.
- **Pathfinding:** Evangelists will need to seek out uninfected machines. This requires integrating with the existing pathfinding and query systems to locate valid targets.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
