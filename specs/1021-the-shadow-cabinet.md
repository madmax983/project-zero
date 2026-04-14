# 1021: The Shadow Cabinet

## 1. Overview
Real power isn't in the throne room; it's in the kitchen and maintenance tunnels. Pops with critical access roles (Cooks, Janitors, Engineers) form hidden alliances known as a "Shadow Cabinet." If the player ignores their specific needs, they engage in "Passive Resistance" (slow doors, cold food, "lost" paperwork) without openly rebelling. This creates emergence where a lowly janitor can become the most powerful person in the colony by creating shortcuts for friends and locking doors for enemies.

## 2. Dependencies
- Layer 1 `Pop` entity and jobs (Access levels).
- Layer 1 `Social` network/relationships.
- Layer 1 `Needs` system.
- Layer 1 `Buildings`/`Doors` (Interaction latency).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Job};
    use crate::layer1::needs::NeedsProfile;
    use crate::layer1::social::Alliance;
    use crate::layer1::buildings::{Door, InteractionLatency};

    #[test]
    fn test_unhappy_critical_workers_form_shadow_cabinet() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_shadow_cabinet_formation_system);

        // Spawn a disgruntled critical worker
        let janitor = app.world_mut().spawn((
            Pop,
            Job { title: "Janitor".to_string(), critical_access: true },
            NeedsProfile { satisfaction: 20.0 }, // Very low
        )).id();

        // Spawn a friend
        let friend = app.world_mut().spawn((
            Pop,
            Job { title: "Miner".to_string(), critical_access: false },
        )).id();

        // Give them an alliance
        app.world_mut().spawn(Alliance { members: vec![janitor, friend] });

        app.update();

        // Verify the alliance became a Shadow Cabinet
        let mut found = false;
        let mut query = app.world_mut().query::<&ShadowCabinet>();
        for _ in query.iter(app.world()) {
            found = true;
        }

        assert!(found, "An alliance containing unhappy critical workers should form a Shadow Cabinet.");
    }

    #[test]
    fn test_shadow_cabinet_sabotages_infrastructure_for_non_members() {
        let mut app = App::new();
        app.add_systems(Update, passive_resistance_system);

        // Setup the Cabinet
        app.world_mut().spawn(ShadowCabinet {
            members: vec![Entity::from_raw(1)],
            resistance_level: 50.0,
        });

        let door = app.world_mut().spawn((
            Door,
            InteractionLatency { delay_ticks: 0 },
        )).id();

        app.update();

        let latency = app.world().get::<InteractionLatency>(door).unwrap();
        assert!(latency.delay_ticks > 0, "Shadow Cabinet should passively increase interaction latency for colony infrastructure.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/shadow_cabinet.rs
use bevy::prelude::*;
use crate::layer1::pop::Job;
use crate::layer1::needs::NeedsProfile;
use crate::layer1::social::Alliance;
use crate::layer1::buildings::InteractionLatency;

#[derive(Component)]
pub struct ShadowCabinet {
    pub members: Vec<Entity>,
    pub resistance_level: f32,
}

pub fn evaluate_shadow_cabinet_formation_system(
    mut commands: Commands,
    query_pops: Query<(&Job, &NeedsProfile)>,
    query_alliances: Query<(Entity, &Alliance), Without<ShadowCabinet>>,
) {
    for (alliance_entity, alliance) in query_alliances.iter() {
        let mut has_unhappy_critical = false;

        for member in &alliance.members {
            if let Ok((job, needs)) = query_pops.get(*member) {
                if job.critical_access && needs.satisfaction < 30.0 {
                    has_unhappy_critical = true;
                    break;
                }
            }
        }

        if has_unhappy_critical {
            commands.entity(alliance_entity).insert(ShadowCabinet {
                members: alliance.members.clone(),
                resistance_level: 10.0, // Base level
            });
        }
    }
}

pub fn passive_resistance_system(
    query_cabinets: Query<&ShadowCabinet>,
    mut query_infrastructure: Query<&mut InteractionLatency>,
) {
    let total_resistance: f32 = query_cabinets.iter().map(|c| c.resistance_level).sum();

    if total_resistance > 0.0 {
        let added_delay = (total_resistance / 10.0) as u32;
        for mut latency in query_infrastructure.iter_mut() {
            latency.delay_ticks = added_delay;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Selective Sabotage:** The `passive_resistance_system` currently punishes everyone globally. It needs to check *who* is using the door/machine. Members of the Shadow Cabinet should have `InteractionLatency` set to 0, while non-members suffer the delay.
- **Needs Integration:** `resistance_level` should scale dynamically with how low the critical workers' satisfaction drops.
- **Discovery:** The player shouldn't know who is in the Shadow Cabinet automatically. It requires an "Investigation" task or "Security" Pops to uncover.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_unhappy_critical_workers_form_shadow_cabinet` passes.
- [ ] Test `test_shadow_cabinet_sabotages_infrastructure_for_non_members` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `critical_access` needs to be defined on the `Job` struct (or via a trait). Jobs like Janitor, Cook, and IT/Network Admin should have this flagged true.
- `InteractionLatency` should be integrated into the pathfinding/movement system so that Pops actually pause at doors rather than just phasing through them.

## 8. Questions
*Builder: add questions here if spec is unclear.*
