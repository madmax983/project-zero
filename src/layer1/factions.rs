use crate::layer1::skills::SkillType;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Unique identifier for each faction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactionId {
    /// Faction for miners.
    MinersGuild,
    /// Faction for farmers.
    FarmersGuild,
    /// Faction for construction workers.
    MasonsGuild,
    /// Faction for foresters.
    LoggersGuild,
    /// Faction for crafters.
    ArtisansGuild,
    /// Faction for those with no specific skill focus.
    Unaligned,
}

/// Data associated with a faction.
#[derive(Debug, Clone)]
pub struct FactionData {
    /// Display name of the faction.
    pub name: String,
    /// Current satisfaction level (0.0 to 1.0).
    pub satisfaction: f32,
    /// Number of members in the faction.
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

/// Resource containing all factions.
#[derive(Resource, Default)]
pub struct Factions {
    /// Map of `FactionId` to `FactionData`.
    pub map: HashMap<FactionId, FactionData>,
}

impl Factions {
    /// Retrieves data for a specific faction.
    #[must_use]
    pub fn get(&self, id: FactionId) -> Option<&FactionData> {
        self.map.get(&id)
    }

    /// Initializes default factions if the map is empty.
    pub fn initialize(&mut self) {
        if self.map.is_empty() {
            self.map.insert(
                FactionId::MinersGuild,
                FactionData {
                    name: "Miners' Union".into(),
                    ..Default::default()
                },
            );
            self.map.insert(
                FactionId::FarmersGuild,
                FactionData {
                    name: "Growers' Circle".into(),
                    ..Default::default()
                },
            );
            self.map.insert(
                FactionId::MasonsGuild,
                FactionData {
                    name: "Masons' Lodge".into(),
                    ..Default::default()
                },
            );
            self.map.insert(
                FactionId::LoggersGuild,
                FactionData {
                    name: "Foresters' Pact".into(),
                    ..Default::default()
                },
            );
            self.map.insert(
                FactionId::ArtisansGuild,
                FactionData {
                    name: "Artisans' Guild".into(),
                    ..Default::default()
                },
            );
            self.map.insert(
                FactionId::Unaligned,
                FactionData {
                    name: "Unaligned".into(),
                    ..Default::default()
                },
            );
        }
    }
}

/// Component indicating faction membership.
#[derive(Component, Default, Debug, Clone)]
pub struct FactionMember {
    /// The ID of the faction this entity belongs to.
    pub faction_id: Option<FactionId>,
}

/// System to update faction membership based on highest skill.
pub fn update_faction_membership_system(
    mut query: Query<
        (&crate::layer1::skills::Skills, &mut FactionMember),
        Changed<crate::layer1::skills::Skills>,
    >,
) {
    for (skills, mut member) in &mut query {
        let mut max_xp = 0.0;
        let mut best_skill = None;

        // Iterate in fixed order for determinism
        for &skill in &[
            SkillType::Mining,
            SkillType::Forestry,
            SkillType::Farming,
            SkillType::Construction,
            SkillType::Crafting,
        ] {
            let xp = skills.get_xp(skill);
            if xp > max_xp {
                max_xp = xp;
                best_skill = Some(skill);
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

        if member.faction_id != Some(new_faction) {
            member.faction_id = Some(new_faction);
        }
    }
}

/// System to update faction satisfaction and member counts.
pub fn update_faction_satisfaction_system(
    policies: Res<crate::layer1::edicts::ColonyPolicies>,
    mut factions: ResMut<Factions>,
    member_query: Query<&FactionMember>,
) {
    // 1. Reset counts/Initialize
    factions.initialize();
    for data in factions.map.values_mut() {
        data.members_count = 0;
        data.satisfaction = 1.0;
    }

    // 2. Count members
    for member in &member_query {
        if let Some(data) = member.faction_id.and_then(|id| factions.map.get_mut(&id)) {
            data.members_count += 1;
        }
    }

    // 3. Apply Policy Effects
    let double_shifts_penalty = if policies.is_active(crate::layer1::edicts::Policy::DoubleShifts) {
        -0.2
    } else {
        0.0
    };
    let rationing_penalty = if policies.is_active(crate::layer1::edicts::Policy::Rationing) {
        -0.1
    } else {
        0.0
    };

    let total_penalty = double_shifts_penalty + rationing_penalty;

    for data in factions.map.values_mut() {
        data.satisfaction = (data.satisfaction + total_penalty).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{SkillType, Skills};

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

        let pop = world.spawn((Pop, skills, FactionMember::default())).id();

        // Run system
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_membership_system,
        );

        let member = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(member.faction_id, Some(FactionId::MinersGuild));
    }

    #[test]
    fn test_pop_changes_faction_on_skill_change() {
        let mut world = setup_world();

        // Start as Miner
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 200.0);

        let pop = world
            .spawn((
                Pop,
                skills,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
            ))
            .id();

        // Update skills to be a Farmer
        let mut skills = world.get_mut::<Skills>(pop).unwrap();
        skills.add_xp(SkillType::Farming, 500.0); // Now higher than Mining

        // Run system
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_membership_system,
        );

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
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_membership_system,
        );
        // Run satisfaction update (which should also update counts or have a separate system)
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_satisfaction_system,
        );

        let factions = world.resource::<Factions>();

        let miners = factions
            .get(FactionId::MinersGuild)
            .expect("MinersGuild should exist");
        assert_eq!(miners.members_count, 2);

        let farmers = factions
            .get(FactionId::FarmersGuild)
            .expect("FarmersGuild should exist");
        assert_eq!(farmers.members_count, 1);
    }

    #[test]
    fn test_policy_affects_satisfaction() {
        let mut world = setup_world();

        // Enable DoubleShifts
        let mut policies = world.resource_mut::<ColonyPolicies>();
        policies.toggle(Policy::DoubleShifts);

        // Run update
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_satisfaction_system,
        );

        let factions = world.resource::<Factions>();
        let miners = factions.get(FactionId::MinersGuild).unwrap();

        // Assume base 1.0, penalty 0.2
        assert!(miners.satisfaction <= 0.8 + f32::EPSILON);
    }

    #[test]
    fn test_unaligned_faction() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                Pop,
                Skills::default(), // No XP
                FactionMember::default(),
            ))
            .id();

        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_membership_system,
        );

        let member = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(member.faction_id, Some(FactionId::Unaligned));
    }

    #[test]
    fn test_tie_breaking_deterministic() {
        let mut world = setup_world();
        let mut skills = Skills::default();
        // Mining and Farming same XP
        skills.add_xp(SkillType::Mining, 100.0);
        skills.add_xp(SkillType::Farming, 100.0);

        let pop = world.spawn((Pop, skills, FactionMember::default())).id();

        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_membership_system,
        );

        let member = world.get::<FactionMember>(pop).unwrap();
        // Mining is checked before Farming in the loop, so it wins.
        assert_eq!(member.faction_id, Some(FactionId::MinersGuild));
    }
}
