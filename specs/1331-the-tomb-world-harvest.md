# 1331: The Tomb World Harvest

## 1. Overview
A desperate colony cannibalizes ancient ruins to survive, slowly replacing their culture with the ghosts of the past. Pops can mine ancient ruins for advanced materials and synthetic rations. Doing so generates a "Desecration" memory. Over time, pops who rely on these rations begin adopting the traits, names, and needs of the ruined civilization.

## 2. Dependencies
- `004` Pop Entity
- `010` Chronicle System

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_mining_ruins_generates_desecration_memory() {
        let mut app = App::new();
        app.add_systems(Update, mine_ancient_ruins_system);

        let pop = app.world_mut().spawn((
            Pop,
            Name::new("Bob"),
            DesecrationMemoryCount(0),
        )).id();

        let ruin = app.world_mut().spawn((
            AncientRuin,
            ResourceYield(ItemType::SyntheticRation, 5),
        )).id();

        app.world_mut().send_event(MineRuinEvent { pop, ruin });
        app.update();

        let memory_count = app.world().get::<DesecrationMemoryCount>(pop).unwrap();
        assert_eq!(memory_count.0, 1, "Mining a ruin must generate a Desecration memory count");
    }

    #[test]
    fn test_desecration_memory_triggers_cultural_shift() {
        let mut app = App::new();
        app.add_systems(Update, cultural_shift_system);

        let pop = app.world_mut().spawn((
            Pop,
            Culture(CultureType::Original),
            DesecrationMemoryCount(5),
        )).id();

        app.update();

        let culture = app.world().get::<Culture>(pop).unwrap();
        assert_eq!(*culture, Culture(CultureType::TombWorld), "High desecration count must shift culture to Tomb World");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct AncientRuin;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ItemType {
    SyntheticRation,
}

#[derive(Component)]
pub struct ResourceYield(pub ItemType, pub u32);

#[derive(Component)]
pub struct DesecrationMemoryCount(pub u32);

#[derive(Event)]
pub struct MineRuinEvent {
    pub pop: Entity,
    pub ruin: Entity,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CultureType {
    Original,
    TombWorld,
}

#[derive(Component, PartialEq, Eq, Debug)]
pub struct Culture(pub CultureType);

pub fn mine_ancient_ruins_system(
    mut events: EventReader<MineRuinEvent>,
    mut query: Query<&mut DesecrationMemoryCount>,
) {
    for event in events.read() {
        if let Ok(mut memory) = query.get_mut(event.pop) {
            memory.0 += 1;
        }
    }
}

pub fn cultural_shift_system(mut query: Query<(&mut Culture, &DesecrationMemoryCount)>) {
    for (mut culture, memory) in query.iter_mut() {
        if memory.0 >= 5 {
            culture.0 = CultureType::TombWorld;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently hardcoded the desecration memory threshold.
- Improve naming logic so pops get renamed to "ghost names".
- Consider integrating with Needs so Tomb World pops demand ancient artifacts.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops correctly gain Desecration memory when mining ruins.
- [ ] Pops successfully shift to Tomb World culture at the threshold.

## 7. Technical Guidance
- Integrate with the Chronicle system so the shift in culture is logged.
- The `mine_ancient_ruins_system` could plug into `work_execution_system`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
