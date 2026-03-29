# 726 The Propaganda Broadcast Hijack

## 1. Overview
A hostile Layer 3 empire slices into your colony's primary communication network, broadcasting tailored propaganda directly to your Pops. This causes extreme Morale drops, forms seditious factions, or prompts Pops to halt work to build bizarre "Freedom Monuments."

## 2. Dependencies
- 031 Pop Morale
- 068 Pop Factions
- 046 Notifications System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_propaganda_hijack_lowers_morale() {
        let mut world = World::new();
        // Setup communication network and pop
        world.insert_resource(CommsNetwork { hijacked: true });
        let pop = world.spawn((Pop, Morale { current: 100.0 })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_propaganda_effects);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();

        assert!(morale.current < 100.0, "Morale should decrease when comms are hijacked");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CommsNetwork {
    pub hijacked: bool,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale {
    pub current: f32,
}

pub fn apply_propaganda_effects(
    comms: Res<CommsNetwork>,
    mut query: Query<&mut Morale, With<Pop>>,
) {
    if comms.hijacked {
        for mut morale in query.iter_mut() {
            morale.current -= 5.0;
            if morale.current < 0.0 {
                morale.current = 0.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- The morale drop should scale over time rather than dropping a flat amount per tick.
- Different types of propaganda broadcasts should have varying effects (e.g. faction conversion instead of morale drop).

## 6. Acceptance Criteria
- [ ] `apply_propaganda_effects` decreases pop morale when `CommsNetwork.hijacked` is true.
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- Integrate with utility AI to make pops stop work to construct 'Freedom Monuments'.
- Handle the source of the hijack (Layer 3 entities).

## 8. Questions
*Builder: add questions here if spec is unclear.*
