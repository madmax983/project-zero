use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::skills::{SkillType, Skills};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{SkillType, Skills};

    fn setup_world() -> World {
        let mut world = World::new();
        let mut factions = Factions::default();
        factions.initialize();
        world.insert_resource(factions);
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
        let mut schedule = Schedule::default();
        schedule.add_systems(update_faction_membership_system);
        schedule.run(&mut world);

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
        let mut schedule = Schedule::default();
        schedule.add_systems(update_faction_membership_system);
        schedule.run(&mut world);

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
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                update_faction_membership_system,
                update_faction_satisfaction_system,
            )
                .chain(),
        );
        schedule.run(&mut world);

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
        let mut schedule = Schedule::default();
        schedule.add_systems(update_faction_satisfaction_system);
        schedule.run(&mut world);

        let factions = world.resource::<Factions>();
        // We need to ensure the faction exists first
        // Usually update_faction_satisfaction_system should initialize factions if missing, or we do it manually.
        // Based on spec, update_faction_satisfaction_system calls initialize().
        let miners = factions.get(FactionId::MinersGuild).unwrap();

        // Assume base 1.0, penalty 0.2
        assert!(miners.satisfaction <= 0.8 + f32::EPSILON);
    }
}

/// Unique identifiers for factions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactionId {
    /// Faction for miners.
    MinersGuild,
    /// Faction for farmers.
    FarmersGuild,
    /// Faction for builders/masons.
    MasonsGuild,
    /// Faction for loggers.
    LoggersGuild,
    /// Faction for crafters.
    ArtisansGuild,
    /// Pops with no dominant skill or balanced skills.
    Unaligned,
}

/// Data stored for each faction.
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

/// Global resource tracking all factions.
#[derive(Resource, Default)]
pub struct Factions {
    /// Map of faction ID to faction data.
    pub map: HashMap<FactionId, FactionData>,
}

impl Factions {
    /// Retrieves faction data for a given ID.
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

/// Component attaching a pop to a faction.
#[derive(Component, Default, Debug, Clone)]
pub struct FactionMember {
    /// The faction this pop belongs to, if any.
    pub faction_id: Option<FactionId>,
}

/// System to update pop faction membership based on skills.
pub fn update_faction_membership_system(
    mut query: Query<(&Skills, &mut FactionMember), Changed<Skills>>,
) {
    for (skills, mut member) in &mut query {
        let mut max_xp = 0.0;
        let mut best_skill = None;

        // Iterate in a fixed order to ensure determinism
        let all_skills = [
            SkillType::Mining,
            SkillType::Forestry,
            SkillType::Farming,
            SkillType::Construction,
            SkillType::Crafting,
        ];

        for skill in all_skills {
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
            _ => FactionId::Unaligned,
        };

        if member.faction_id != Some(new_faction) {
            member.faction_id = Some(new_faction);
        }
    }
}

/// System to update faction satisfaction and member counts.
pub fn update_faction_satisfaction_system(
    policies: Res<ColonyPolicies>,
    mut factions: ResMut<Factions>,
    member_query: Query<&FactionMember>,
) {
    // 1. Ensure map exists
    factions.initialize();

    // Reset counts
    for data in factions.map.values_mut() {
        data.members_count = 0;
        data.satisfaction = 1.0; // Base satisfaction
    }

    // 2. Count members
    for member in &member_query {
        if let Some(data) = member.faction_id.and_then(|id| factions.map.get_mut(&id)) {
            data.members_count += 1;
        }
    }

    // 3. Apply Policy Effects
    // Example: DoubleShifts hurts everyone.
    let global_penalty = if policies.is_active(Policy::DoubleShifts) {
        -0.2
    } else {
        0.0
    };
    let rationing_penalty = if policies.is_active(Policy::Rationing) {
        -0.1
    } else {
        0.0
    };

    for data in factions.map.values_mut() {
        data.satisfaction =
            (data.satisfaction + global_penalty + rationing_penalty).clamp(0.0, 1.0);
    }
}
