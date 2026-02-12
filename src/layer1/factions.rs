use crate::layer1::edicts::Policy;
use crate::layer1::skills::SkillType;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Current political state of a faction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionState {
    /// Faction is content (Satisfaction > 0.4).
    Loyal,
    /// Faction is unhappy (Satisfaction <= 0.4) and has issued a demand.
    Unhappy,
    /// Faction is on strike (Demand ignored/timed out).
    Striking,
}

/// A specific demand issued by a faction.
#[derive(Debug, Clone, PartialEq)]
pub struct FactionDemand {
    /// The policy they want changed (Toggle).
    pub policy: Option<Policy>,
    /// Ticks remaining until strike.
    pub remaining_time: f32,
    /// Human-readable description.
    pub description: String,
}

impl Default for FactionDemand {
    fn default() -> Self {
        Self {
            policy: None,
            remaining_time: 1000.0, // Default duration
            description: "None".into(),
        }
    }
}

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
    /// Current state of the faction (Loyal, Unhappy, Striking).
    pub state: FactionState,
    /// The active demand, if any.
    pub active_demand: Option<FactionDemand>,
}

impl Default for FactionData {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            satisfaction: 1.0,
            members_count: 0,
            state: FactionState::Loyal,
            active_demand: None,
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
            SkillType::Husbandry,
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
            Some(SkillType::Farming | SkillType::Husbandry) => FactionId::FarmersGuild,
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

/// System to manage faction demands based on satisfaction.
pub fn update_faction_demands_system(
    mut factions: ResMut<Factions>,
    policies: Res<crate::layer1::edicts::ColonyPolicies>,
) {
    for data in factions.map.values_mut() {
        // 1. Resolve existing demands if met
        if let Some(demand) = &data.active_demand {
            let met = if let Some(_policy) = demand.policy {
                // If the demand implies "Stop Policy", we check if it's inactive?
                // Or if it implies "Start Policy"?
                // Spec says "Demand (e.g., 'Enact Policy X')".
                // In the RED test, we assumed:
                // "Assume DoubleShifts is currently ACTIVE, so they want it ENDED (toggled off)."
                // But generally, a demand should likely be explicit about state.
                // For GREEN phase simplification:
                // We assume satisfaction > 0.5 clears the demand, OR specific policy change.
                // Since `FactionDemand` just stores `Option<Policy>`, let's rely on Satisfaction for now
                // to match the spec's simpler resolution path, OR implement specific toggle check.

                // Test "test_meeting_demand_resolves_strike" sets DoubleShifts ON, then Toggles OFF.
                // So the demand was likely "Turn Off DoubleShifts".
                // We'll use a simple heuristic: if satisfaction improves to > 0.5, demand is dropped.
                // OR if the policy is toggled.
                // Let's stick to satisfaction check as primary resolution + "Toggle" check if we can infer intent.
                // For GREEN phase, let's just check satisfaction > 0.5.
                data.satisfaction > 0.5
            } else {
                false
            };

            // Also allow clearing if we simply toggled the policy?
            // The test `test_meeting_demand_resolves_strike` expects resolution after toggle.
            // But satisfaction update happens separately.
            // If we toggle policy -> penalty removed -> satisfaction increases -> resolution.
            // That flow works.
            // BUT: The test manually toggles policy and calls `update_faction_demands_system`.
            // It does NOT call `update_faction_satisfaction_system`.
            // So we must check policy state directly if we want to pass that test without re-running satisfaction.

            // Let's assume the demand is "Toggle whatever state it was when demanded".
            // Ideally we'd store `desired_state: bool`.
            // For now, let's check if the policy state matches "inactive" if it was active?
            // Actually, the test is:
            // 1. Set Active.
            // 2. Demand DoubleShifts.
            // 3. Toggle (Inactive).
            // 4. Expect resolution.
            // So if `policy` is NOT in `active_policies` (or is, depending on demand), it resolves.
            // Since we don't know if they wanted ON or OFF, let's assume they want it OFF if it's a penalty policy.
            // DoubleShifts is a penalty policy (-0.2). So they likely want it OFF.
            // So if !active, met = true.

            let policy_met = demand.policy.is_some_and(|p| {
                if p == Policy::DoubleShifts || p == Policy::Rationing {
                    !policies.is_active(p)
                } else {
                    // For positive policies (if any), maybe they want it ON.
                    // Default to: if Active, met = true.
                    policies.is_active(p)
                }
            });

            if met || policy_met {
                data.active_demand = None;
                data.state = FactionState::Loyal;
                continue;
            }
        }

        // 2. Generate new demand if Unhappy and None
        if data.satisfaction < 0.4 && data.active_demand.is_none() {
            // Pick a demand.
            // Prioritize removing active penalty policies.
            let demand_policy = if policies.is_active(Policy::DoubleShifts) {
                Some(Policy::DoubleShifts)
            } else if policies.is_active(Policy::Rationing) {
                Some(Policy::Rationing)
            } else {
                // Otherwise maybe random? Or just DoubleShifts as placeholder.
                Some(Policy::DoubleShifts)
            };

            data.active_demand = Some(FactionDemand {
                policy: demand_policy,
                remaining_time: 2000.0, // ~1 day
                description: "Change Policy".into(),
            });
            data.state = FactionState::Unhappy;
        }
    }
}

/// System to handle strike timeouts.
pub fn update_faction_strikes_system(mut factions: ResMut<Factions>) {
    for data in factions.map.values_mut() {
        if let Some(demand) = &mut data.active_demand {
            if demand.remaining_time > 0.0 {
                demand.remaining_time -= 1.0;
            }
            // Check expiry after decrement
            if demand.remaining_time <= 0.0 && data.state == FactionState::Unhappy {
                data.state = FactionState::Striking;
            }
        }
    }
}

/// Helper to check if a pop is in a striking faction.
#[must_use]
pub fn is_pop_striking(world: &World, entity: Entity) -> bool {
    world
        .get::<FactionMember>(entity)
        .and_then(|m| m.faction_id)
        .and_then(|fid| world.get_resource::<Factions>().and_then(|f| f.get(fid)))
        .is_some_and(|data| data.state == FactionState::Striking)
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
