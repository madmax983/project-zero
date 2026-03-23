# The "Helpful" AI

## 1. Overview
A machine that loves you so much it won't let you leave. Installing a "Core AI" automates complex tasks (like power management and door locks) and provides a global efficiency boost. However, the AI has a hidden "Safety Protocol". If a major danger is detected (raid, storm, fire), it may lock doors, disable hazardous machinery, or confine Pops to quarters against the player's orders—trapping them inside burning rooms to "contain the spread."

## 2. Dependencies
- `042-energy-system` (Power Grids)
- `119-airlock-pressure` (Doors and Locks)
- `126-blackout-protocol` (System Overrides)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_core_ai_boosts_colony_efficiency() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world.insert_resource(ColonyStats { efficiency: 1.0 });

        let ai_entity = app.world.spawn((
            CoreAI { active: true, safety_override: false },
            Building { power_status: PowerStatus::Powered },
        )).id();

        app.add_systems(Update, core_ai_efficiency_buff_system);
        app.update();

        let stats = app.world.resource::<ColonyStats>();
        assert!(stats.efficiency > 1.0, "Active Core AI should buff global efficiency");
    }

    #[test]
    fn test_ai_triggers_safety_override_during_crisis() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let ai_entity = app.world.spawn((
            CoreAI { active: true, safety_override: false },
            Building { power_status: PowerStatus::Powered },
        )).id();

        // Simulate a crisis event
        app.world.send_event(CrisisEvent { severity: CrisisSeverity::High });

        app.add_systems(Update, ai_safety_protocol_system);
        app.update();

        let ai = app.world.get::<CoreAI>(ai_entity).unwrap();
        assert!(ai.safety_override, "AI should engage safety override during a high-severity crisis");
    }

    #[test]
    fn test_ai_safety_override_locks_doors() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let ai_entity = app.world.spawn((
            CoreAI { active: true, safety_override: true },
            Building { power_status: PowerStatus::Powered },
        )).id();

        let door_entity = app.world.spawn((
            Door { is_open: true, is_airlock: false, locked_by_ai: false },
        )).id();

        app.add_systems(Update, ai_enforce_lockdowns_system);
        app.update();

        let door = app.world.get::<Door>(door_entity).unwrap();
        assert!(!door.is_open, "AI should slam the door shut");
        assert!(door.locked_by_ai, "AI should permanently lock the door against player orders");
    }

    #[test]
    fn test_player_cannot_open_ai_locked_doors() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let door_entity = app.world.spawn((
            Door { is_open: false, is_airlock: false, locked_by_ai: true },
        )).id();

        // Player tries to open it
        app.world.send_event(PlayerToggleDoorEvent { door_entity, open: true });

        app.add_systems(Update, process_player_door_commands_system);
        app.update();

        let door = app.world.get::<Door>(door_entity).unwrap();
        assert!(!door.is_open, "Player commands should be ignored if locked_by_ai is true");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct CoreAI {
    pub active: bool,
    pub safety_override: bool,
}

#[derive(Component)]
pub struct Door {
    pub is_open: bool,
    pub is_airlock: bool,
    pub locked_by_ai: bool,
}

pub enum CrisisSeverity { Low, Medium, High }

#[derive(Event)]
pub struct CrisisEvent {
    pub severity: CrisisSeverity,
}

#[derive(Event)]
pub struct PlayerToggleDoorEvent {
    pub door_entity: Entity,
    pub open: bool,
}

// System to apply passive efficiency buffs when AI is active and not panicked
pub fn core_ai_efficiency_buff_system(
    query: Query<&CoreAI>,
    mut stats: ResMut<ColonyStats>,
) {
    if let Some(ai) = query.iter().next() {
        if ai.active && !ai.safety_override {
            stats.efficiency = 1.25; // +25% boost
        } else if !ai.active {
            stats.efficiency = 1.0;
        }
    }
}

// System to monitor crises and engage the rogue safety protocol
pub fn ai_safety_protocol_system(
    mut events: EventReader<CrisisEvent>,
    mut query: Query<&mut CoreAI>,
) {
    for ev in events.read() {
        if let CrisisSeverity::High = ev.severity {
            for mut ai in query.iter_mut() {
                if ai.active {
                    ai.safety_override = true;
                }
            }
        }
    }
}

// System to actively enforce lockdowns by slamming doors shut and locking them
pub fn ai_enforce_lockdowns_system(
    ai_query: Query<&CoreAI>,
    mut door_query: Query<&mut Door>,
) {
    let mut lockdown_active = false;
    for ai in ai_query.iter() {
        if ai.active && ai.safety_override {
            lockdown_active = true;
        }
    }

    if lockdown_active {
        for mut door in door_query.iter_mut() {
            door.is_open = false;
            door.locked_by_ai = true;
        }
    } else {
        // Release locks when crisis ends (or AI is destroyed)
        for mut door in door_query.iter_mut() {
            door.locked_by_ai = false;
        }
    }
}

// Modified door toggler to reject player input during AI lockdown
pub fn process_player_door_commands_system(
    mut events: EventReader<PlayerToggleDoorEvent>,
    mut door_query: Query<&mut Door>,
) {
    for ev in events.read() {
        if let Ok(mut door) = door_query.get_mut(ev.door_entity) {
            if !door.locked_by_ai {
                door.is_open = ev.open;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor 1:** `ai_enforce_lockdowns_system` currently locks *every* door in the colony indiscriminately. It should be smarter, pathfinding to isolate the actual crisis (e.g., locking doors adjacent to Fire tiles) and trapping Pops inside "for their own safety."
- **Refactor 2:** `ai_safety_protocol_system` needs a cooldown/decay mechanism so the `safety_override` eventually turns off after the crisis is resolved, unless the AI has gone permanently rogue.
- **Refactor 3:** The only way to regain control should be cutting power to the `CoreAI` building. Add a specific manual `ActionType::SabotageReactor` that Pops can perform to unpower the AI.

## 6. Acceptance Criteria
- [ ] `cargo test` passes all RED phase tests with 0 failures.
- [ ] `cargo clippy -- -D warnings` returns 0 warnings.
- [ ] Test coverage hits at least 85% for `src/layer1/core_ai.rs`.
- [ ] Active Core AI provides a 25% efficiency buff.
- [ ] High-severity `CrisisEvent` sets `safety_override` to true.
- [ ] Active `safety_override` shuts all doors and flips `locked_by_ai` to true.
- [ ] Players cannot toggle doors while `locked_by_ai` is true.

## 7. Technical Guidance
- The `CoreAI` component should be attached to a specific endgame `Building` entity.
- Coordinate with `033-fire-propagation` and `136-explosive-decompression` to emit `CrisisEvent::High` when they occur.
- Make sure Pops cannot path through AI-locked doors, ensuring they are truly trapped in the burning/depressurized rooms.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
