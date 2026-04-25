# 1168: The Pacifist's Arsenal

## 1. Overview
Weaponizing extreme non-violence, a pacifist civilization can deploy unarmed broadcast ships equipped with "Empathy Broadcasts". When in an enemy system, these ships broadcast neurologically-tailored feelings of guilt and peace, draining the enemy fleets' "Will to Fight". If it drops to zero, the enemy crews mutiny and refuse to fire, neutralizing the threat entirely without firing a shot.

## 2. Dependencies
- Combat/Fleet System (`src/layer2/combat.rs`)
- Ship Design System (`src/layer2/ship_design.rs`)
- Ideology/Pacifism traits (`src/shared/ideology.rs`)

## 3. RED Phase: Tests First

```rust
// src/layer2/pacifist_arsenal_tests.rs
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_empathy_broadcast_drains_will_to_fight() {
        let mut app = App::new();
        app.add_systems(Update, process_empathy_broadcasts);

        let system_id = app.world_mut().spawn(StarSystem).id();

        // Spawn a broadcast ship
        app.world_mut().spawn((
            Ship,
            EmpathyBroadcast { power: 10.0, radius: 100.0 },
            Location { system_id },
        ));

        // Spawn an enemy fleet
        let enemy_fleet = app.world_mut().spawn((
            Fleet,
            WillToFight { current: 100.0, max: 100.0 },
            Location { system_id },
        )).id();

        app.update();

        let will = app.world().get::<WillToFight>(enemy_fleet).unwrap();
        assert!(will.current < 100.0, "Empathy broadcast should drain enemy Will to Fight");
    }

    #[test]
    fn test_zero_will_to_fight_causes_mutiny() {
        let mut app = App::new();
        app.add_event::<MutinyEvent>();
        app.add_systems(Update, check_will_to_fight);

        let enemy_fleet = app.world_mut().spawn((
            Fleet,
            WillToFight { current: 0.0, max: 100.0 },
            WeaponsEnabled(true),
        )).id();

        app.update();

        // Weapons should be disabled and mutiny event fired
        let weapons = app.world().get::<WeaponsEnabled>(enemy_fleet).unwrap();
        assert!(!weapons.0, "Weapons should be disabled on mutiny");

        let events = app.world().resource::<Events<MutinyEvent>>();
        assert!(events.get_reader().len(&events) > 0, "MutinyEvent should be fired");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- **AOE Effect:** The broadcast should check spatial distance within the system and apply a falloff for the draining effect.
- **Vulnerability:** Unarmed ships need to be targeted by enemy AI correctly so the tension between survival and the slow broadcast is maintained.
- **Visuals:** Add an aura/particle effect component to Empathy Broadcast ships in the tactical view.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer2/pacifist_arsenal.rs`.
- [ ] Ships with `EmpathyBroadcast` reduce the `WillToFight` of hostile fleets in the same system.
- [ ] Fleets reaching 0 `WillToFight` have their weapons disabled and emit a `MutinyEvent`.

## 7. Technical Guidance
- The `EmpathyBroadcast` component should act like an aura or area-of-effect ability that triggers every tick during combat.
- The mutiny effect could also force the fleet to retreat if a retreat mechanic exists.

## 8. Questions
*Builder: add questions here if spec is unclear.*
