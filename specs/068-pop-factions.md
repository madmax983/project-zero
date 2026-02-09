# 068: Pop Factions

## Overview

Introduces political factions to the colony. Pops organize into Factions based on their primary skills (Guilds) or traits.
Factions track collective satisfaction and can make demands or go on strike.
This adds a layer of social management, where player decisions (Edicts) have group-level consequences, not just individual ones.

## Dependencies

- `004` — Pop Entity (Pops exist)
- `051` — Pop Skills (Skill-based membership)
- `054` — Colony Edicts (Policy impact)
- `050` — Civil Unrest (Strike mechanics base)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/factions_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use std::collections::HashMap;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::factions::{
        Faction, FactionId, FactionMember, Factions,
        update_faction_membership_system, update_faction_satisfaction_system
    };
    use crate::layer1::edicts::{ColonyPolicies, Policy};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Factions::default());
        world.insert_resource(ColonyPolicies::default());
        world
    }

    #[test]
    fn test_pop_joins_highest_skill_faction() {
        let mut world = setup_world();

        // Spawn a Pop with high Mining skill
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 500.0); // High
        skills.add_xp(SkillType::Farming, 100.0); // Low

        let pop = world.spawn((
            Pop,
            skills,
            FactionMember::default(),
        )).id();

        // Run system
        update_faction_membership_system(&mut world);

        let member = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(member.faction_id, Some(FactionId::MinersGuild));
    }

    #[test]
    fn test_pop_changes_faction_on_skill_change() {
        let mut world = setup_world();

        // Start as Miner
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 200.0);

        let pop = world.spawn((
            Pop,
            skills,
            FactionMember { faction_id: Some(FactionId::MinersGuild) },
        )).id();

        // Update skills to be a Farmer
        let mut skills = world.get_mut::<Skills>(pop).unwrap();
        skills.add_xp(SkillType::Farming, 500.0); // Now higher than Mining

        // Run system
        update_faction_membership_system(&mut world);

        let member = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(member.faction_id, Some(FactionId::FarmersGuild));
    }

    #[test]
    fn test_faction_member_counts() {
        let mut world = setup_world();

        // Spawn 2 Miners, 1 Farmer
        for _ in 0..2 {
            let mut s = Skills::default();
            s.add_xp(SkillType::Mining, 100.0);
            world.spawn((Pop, s, FactionMember::default()));
        }
        let mut s = Skills::default();
        s.add_xp(SkillType::Farming, 100.0);
        world.spawn((Pop, s, FactionMember::default()));

        // Run membership update
        update_faction_membership_system(&mut world);
        // Run satisfaction update (which should also update counts or have a separate system)
        update_faction_satisfaction_system(&mut world);

        let factions = world.resource::<Factions>();

        let miners = factions.get(FactionId::MinersGuild).expect("MinersGuild should exist");
        assert_eq!(miners.members_count, 2);

        let farmers = factions.get(FactionId::FarmersGuild).expect("FarmersGuild should exist");
        assert_eq!(farmers.members_count, 1);
    }

    #[test]
    fn test_policy_affects_satisfaction() {
        let mut world = setup_world();

        // Enable DoubleShifts
        let mut policies = world.resource_mut::<ColonyPolicies>();
        policies.toggle(Policy::DoubleShifts);

        // Run update
        update_faction_satisfaction_system(&mut world);

        let factions = world.resource::<Factions>();
        let miners = factions.get(FactionId::MinersGuild).unwrap();

        // Assume base 1.0, penalty 0.2
        assert!(miners.satisfaction <= 0.8 + f32::EPSILON);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Faction Types and Resource

`src/layer1/factions.rs`

```rust
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::skills::{Skills, SkillType};
use crate::layer1::edicts::{ColonyPolicies, Policy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactionId {
    MinersGuild,
    FarmersGuild,
    MasonsGuild,   // Construction
    LoggersGuild,  // Forestry
    ArtisansGuild, // Crafting
    Unaligned,     // No skills / Balanced
}

#[derive(Debug, Clone)]
pub struct FactionData {
    pub name: String,
    pub satisfaction: f32, // 0.0 to 1.0
    pub members_count: usize,
}

impl Default for FactionData {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            satisfaction: 1.0,
            members_count: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct Factions {
    pub map: HashMap<FactionId, FactionData>,
}

impl Factions {
    pub fn get(&self, id: FactionId) -> Option<&FactionData> {
        self.map.get(&id)
    }

    // Helper to ensure all IDs exist
    pub fn initialize(&mut self) {
        if self.map.is_empty() {
            self.map.insert(FactionId::MinersGuild, FactionData { name: "Miners' Union".into(), ..Default::default() });
            self.map.insert(FactionId::FarmersGuild, FactionData { name: "Growers' Circle".into(), ..Default::default() });
            self.map.insert(FactionId::MasonsGuild, FactionData { name: "Masons' Lodge".into(), ..Default::default() });
            self.map.insert(FactionId::LoggersGuild, FactionData { name: "Foresters' Pact".into(), ..Default::default() });
            self.map.insert(FactionId::ArtisansGuild, FactionData { name: "Artisans' Guild".into(), ..Default::default() });
            self.map.insert(FactionId::Unaligned, FactionData { name: "Unaligned".into(), ..Default::default() });
        }
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct FactionMember {
    pub faction_id: Option<FactionId>,
}
```

### 2. Membership System

```rust
pub fn update_faction_membership_system(
    mut query: Query<(&Skills, &mut FactionMember), Changed<Skills>>,
) {
    for (skills, mut member) in &mut query {
        let mut max_xp = 0.0;
        let mut best_skill = None;

        for (skill, xp) in &skills.xp {
            if *xp > max_xp {
                max_xp = *xp;
                best_skill = Some(*skill);
            }
        }

        let new_faction = match best_skill {
            Some(SkillType::Mining) => FactionId::MinersGuild,
            Some(SkillType::Farming) => FactionId::FarmersGuild,
            Some(SkillType::Construction) => FactionId::MasonsGuild,
            Some(SkillType::Forestry) => FactionId::LoggersGuild,
            Some(SkillType::Crafting) => FactionId::ArtisansGuild,
            None => FactionId::Unaligned,
        };

        // Only switch if XP is significant (e.g. > 100) to avoid jitter?
        // For Green phase, just switch.
        if member.faction_id != Some(new_faction) {
            member.faction_id = Some(new_faction);
        }
    }
}
```

### 3. Satisfaction and Counts System

```rust
pub fn update_faction_satisfaction_system(
    policies: Res<ColonyPolicies>,
    mut factions: ResMut<Factions>,
    member_query: Query<&FactionMember>,
) {
    // 1. Reset counts
    factions.initialize(); // Ensure map exists
    for data in factions.map.values_mut() {
        data.members_count = 0;
        data.satisfaction = 1.0; // Base
    }

    // 2. Count members
    for member in &member_query {
        if let Some(id) = member.faction_id {
            if let Some(data) = factions.map.get_mut(&id) {
                data.members_count += 1;
            }
        }
    }

    // 3. Apply Policy Effects
    // Example: DoubleShifts hurts everyone, but maybe Industrial guilds more?
    // For now, global effect.
    let global_penalty = if policies.is_active(Policy::DoubleShifts) { -0.2 } else { 0.0 };
    let rationing_penalty = if policies.is_active(Policy::Rationing) { -0.1 } else { 0.0 };

    for data in factions.map.values_mut() {
        data.satisfaction = (data.satisfaction + global_penalty + rationing_penalty).clamp(0.0, 1.0);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Iterating all pops to count members every tick is O(N). Optimization: `FactionMember` component change detection triggers a counter update, or run this system infrequently (e.g., once per second).
- **Nuance**: Add "Trait" based factions (e.g., `Traditionalists` for pops with `Trait::Traditional`).
- **Demands**: Implement a `FactionDemands` resource where unhappy factions push for policy changes.
- **UI**: Display Faction status in a new UI panel.
- **Events**: Fire `FactionSatisfactionChanged` event for UI/Narrative.

## Acceptance Criteria

- [ ] `FactionMember` component is assigned based on highest skill.
- [ ] `Factions` resource tracks member counts and satisfaction.
- [ ] Active Policies reduce faction satisfaction.
- [ ] `cargo test` passes.
- [ ] Test coverage for new module > 85%.

## Technical Guidance

- Register `Factions` resource in `main.rs`.
- Add `update_faction_membership_system` to `SimulationSchedule`.
- Add `update_faction_satisfaction_system` to `SimulationSchedule` (maybe `Time::Fixed` or infrequent run).
- Ensure `SkillType` mapping covers all skills.
