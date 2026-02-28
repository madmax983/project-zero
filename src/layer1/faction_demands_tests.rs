#[cfg(test)]
mod tests {
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::factions::{
        is_pop_striking, update_faction_demands_system, update_faction_strikes_system,
        FactionDemand, FactionId, FactionMember, FactionState, Factions,
    };
    use crate::layer1::unrest::MentalState;
    use crate::layer1::utility_ai::{ActionType, UtilityWeights};
    use crate::layer1::{GridPosition, Pop};
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut factions = Factions::default();
        factions.initialize();
        world.insert_resource(factions);
        world.insert_resource(ColonyPolicies::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_faction_state_defaults_to_loyal() {
        let world = setup_world();
        let factions = world.resource::<Factions>();
        let data = factions.get(FactionId::MinersGuild).unwrap();
        assert_eq!(data.state, FactionState::Loyal);
    }

    #[test]
    fn test_generate_demand_on_low_satisfaction() {
        let mut world = setup_world();

        {
            let mut factions = world.resource_mut::<Factions>();
            // Lower satisfaction to threshold (e.g., 0.4)
            if let Some(data) = factions.map.get_mut(&FactionId::MinersGuild) {
                data.satisfaction = 0.3;
            }
        }

        // Run demand generation system
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_demands_system,
        );

        let factions = world.resource::<Factions>();
        let data = factions.get(FactionId::MinersGuild).unwrap();

        // Should have a demand
        assert!(data.active_demand.is_some());
        // State should be Unhappy
        assert_eq!(data.state, FactionState::Unhappy);
    }

    #[test]
    fn test_demand_timeout_triggers_strike() {
        let mut world = setup_world();

        // Setup existing demand with near-expiry timeout
        {
            let mut factions = world.resource_mut::<Factions>();
            let data = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            data.satisfaction = 0.3;
            data.state = FactionState::Unhappy;
            data.active_demand = Some(FactionDemand {
                policy: Some(Policy::Rationing),
                remaining_time: 1.0, // 1 tick remaining
                ..Default::default()
            });
        }

        // Run strike update system
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_strikes_system,
        );

        let factions = world.resource::<Factions>();
        let data = factions.get(FactionId::MinersGuild).unwrap();

        // Should transition to Strike
        assert_eq!(data.state, FactionState::Striking);
    }

    #[test]
    fn test_strike_overrides_work_utility() {
        // This test ensures striking pops don't work
        let mut world = setup_world();

        // Set MinersGuild to Strike
        {
            let mut factions = world.resource_mut::<Factions>();
            let data = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            data.state = FactionState::Striking;
        }

        // Spawn a Pop in MinersGuild
        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                GridPosition { x: 0, y: 0 },
                MentalState::Normal,
            ))
            .id();

        let is_striking = is_pop_striking(&world, pop);
        assert!(is_striking);
    }

    #[test]
    fn test_meeting_demand_resolves_strike() {
        let mut world = setup_world();

        // Set MinersGuild to Strike with a specific demand
        {
            let mut factions = world.resource_mut::<Factions>();
            let data = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            data.state = FactionState::Striking;
            data.active_demand = Some(FactionDemand {
                policy: Some(Policy::DoubleShifts), // Demand: Toggle DoubleShifts
                ..Default::default()
            });
        }

        // Assume DoubleShifts is currently ACTIVE, so they want it ENDED.
        world
            .resource_mut::<ColonyPolicies>()
            .active_policies
            .insert(Policy::DoubleShifts);

        // Player action: Toggle DoubleShifts OFF
        world
            .resource_mut::<ColonyPolicies>()
            .toggle(Policy::DoubleShifts);

        // Run update system
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            update_faction_demands_system,
        );

        let factions = world.resource::<Factions>();
        let data = factions.get(FactionId::MinersGuild).unwrap();

        // Should resolve
        assert!(data.active_demand.is_none());
        assert_eq!(data.state, FactionState::Loyal);
    }

    #[test]
    fn test_evaluate_actions_skips_work_when_striking() {
        use crate::layer1::day_night::DayNightCycle;
        use crate::layer1::designation::{Designation, DesignationType};
        use crate::layer1::needs::Needs;
        use crate::layer1::resources::ColonyResources;
        use crate::layer1::taboo::TabooState;
        use crate::layer1::utility_ai::{evaluate_actions_system, PopAction, UtilityConfig};

        let mut world = setup_world();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(DayNightCycle::default());
        world.insert_resource(TabooState::default());

        // Set MinersGuild to Strike
        {
            let mut factions = world.resource_mut::<Factions>();
            let data = factions.map.get_mut(&FactionId::MinersGuild).unwrap();
            data.state = FactionState::Striking;
        }

        // Spawn a Pop in MinersGuild with work need/opportunity
        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                UtilityWeights::default(),
                PopAction {
                    current: ActionType::Idle,
                    current_utility: 0.1,
                    ticks_committed: 100, // Ready to switch
                },
            ))
            .id();

        // Spawn a designation to work on
        world.spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 1, y: 0 },
        ));

        // Run evaluation
        evaluate_actions_system(&mut world);

        // Pop should NOT be working. Should still be Idle (or other action).
        let action = world.get::<PopAction>(pop).unwrap();
        assert_ne!(
            action.current,
            ActionType::Work,
            "Striking pop should not work"
        );
    }
}
