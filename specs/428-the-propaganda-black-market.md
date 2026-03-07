# 428: The Propaganda Black Market

## 1. Overview
In regimes with high Ideological Control (censorship, state media), an underground network emerges known as the "Propaganda Black Market." Shady "Truth-Dealers" spawn in the colony's unobserved or low-security zones, selling smuggled "Uncensored Data" from Layer 3 empires. Pops who purchase or interact with this data gain massive temporary Morale (relief of deep-seated stress) but lose their State Ideology alignment, increasing overall Unrest.

This mechanic introduces a tension where players must balance the temptation of relieving colony-wide stress with the risk of sparking a rebellion fueled by outside ideas.

## 2. Dependencies
- `src/layer1/edicts.rs`: Colony-wide active edicts/policies (e.g., Censorship Level).
- `src/layer1/unrest.rs`: Managing colony unrest and ideological divergence.
- `src/layer1/morale.rs`: Morale boosting systems.
- `src/layer1/economy.rs`: Pop currency/black market transactions.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::edicts::{EdictManager, IdeologyControl};
    use crate::layer1::unrest::UnrestLevel;
    use crate::layer1::morale::MoodModifier;
    use crate::layer1::economy::PopWealth;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(EdictManager { control_level: 100.0 }); // High censorship
        world.insert_resource(UnrestLevel { value: 0.0 });
        world
    }

    #[test]
    fn test_truth_dealer_spawns_under_high_censorship() {
        let mut world = setup_world();

        // Assume system runs periodically to check spawn conditions
        update_truth_dealer_spawning(&mut world);

        let mut query = world.query::<&TruthDealer>();
        let dealer_count = query.iter(&world).count();
        assert!(dealer_count > 0, "Truth Dealers must spawn when censorship is high");
    }

    #[test]
    fn test_no_dealers_under_low_censorship() {
        let mut world = setup_world();
        world.resource_mut::<EdictManager>().control_level = 10.0; // Free speech

        update_truth_dealer_spawning(&mut world);

        let mut query = world.query::<&TruthDealer>();
        let dealer_count = query.iter(&world).count();
        assert_eq!(dealer_count, 0, "Truth Dealers shouldn't spawn under low censorship");
    }

    #[test]
    fn test_pop_buys_uncensored_data() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            PopWealth { credits: 50.0 },
            MoodModifier { current: 50.0, label: "Sad".to_string() },
        )).id();

        let dealer = world.spawn((TruthDealer,)).id();

        // Trigger the interaction explicitly
        trigger_black_market_transaction(&mut world, pop, dealer);

        // Assertions:
        // 1. Credits reduced
        let wealth = world.get::<PopWealth>(pop).unwrap();
        assert!(wealth.credits < 50.0, "Pop must spend credits for data");

        // 2. Morale increases
        let mood = world.get::<MoodModifier>(pop).unwrap();
        assert!(mood.current > 50.0, "Mood must increase from Uncensored Data");
        assert_eq!(mood.label, "Enlightened by Truth");

        // 3. Unrest increases globally due to ideological shift
        let unrest = world.resource::<UnrestLevel>();
        assert!(unrest.value > 0.0, "Global unrest should increase as Pops diverge");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer1::edicts::EdictManager;
use crate::layer1::unrest::UnrestLevel;
use crate::layer1::morale::MoodModifier;
use crate::layer1::economy::PopWealth;

#[derive(Component)]
pub struct TruthDealer;

#[derive(Component)]
pub struct Pop;

pub fn update_truth_dealer_spawning(world: &mut World) {
    let edicts = world.resource::<EdictManager>();

    // Spawn threshold: Control level > 80.0
    if edicts.control_level > 80.0 {
        let mut rng = rand::thread_rng();
        // 10% chance to spawn a dealer per tick (for simplicity)
        if rng.gen_bool(0.1) {
            world.spawn(TruthDealer);
        }
    }
}

pub fn trigger_black_market_transaction(world: &mut World, pop_entity: Entity, dealer_entity: Entity) {
    let data_cost = 20.0;

    // Check if pop can afford it
    let mut can_afford = false;
    if let Some(wealth) = world.get::<PopWealth>(pop_entity) {
        if wealth.credits >= data_cost {
            can_afford = true;
        }
    }

    if can_afford {
        // Deduct cost
        if let Some(mut wealth) = world.get_mut::<PopWealth>(pop_entity) {
            wealth.credits -= data_cost;
        }

        // Boost Morale
        if let Some(mut mood) = world.get_mut::<MoodModifier>(pop_entity) {
            mood.current += 30.0; // Massive morale boost
            mood.label = "Enlightened by Truth".to_string();
        }

        // Increase Unrest
        if let Some(mut unrest) = world.get_resource_mut::<UnrestLevel>() {
            unrest.value += 5.0; // Ideological divergence increases unrest
        }

        // Despawn dealer after transaction (or cooldown)
        world.despawn(dealer_entity);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding & AI:** The interaction shouldn't be a forced `trigger_black_market_transaction`. Pops with low morale and high credits should dynamically pathfind toward the `TruthDealer` using utility AI.
- **Despawn Logic:** Instead of despawning the dealer immediately, give the dealer a limited "stock" of data, or have them disappear if security guards approach.
- **Unrest Scaling:** Ensure unrest increases don't instantly trigger game-over. Provide players with tools (like police raids) to lower unrest or clear out Truth-Dealers.

## 6. Acceptance Criteria (Testable!)
- [ ] RED phase tests compile and pass.
- [ ] Truth Dealers only spawn when `control_level` > 80.0.
- [ ] Transactions correctly reduce Pop credits, increase Morale, and add the "Enlightened by Truth" label.
- [ ] Global `UnrestLevel` increases upon transaction.
- [ ] Test coverage >= 85%.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- Truth-Dealers should probably spawn as hidden or camouflaged entities so the player has to actively use a "Scan" or "Police Raid" command to find them.
- Hook into `src/layer1/utility_ai.rs` to allow unhappy Pops to prioritize finding a dealer. Give the action a high weight if censorship is maxed out and the Pop is stressed.

## 8. Questions
*Builder: add questions here if spec is unclear.*
