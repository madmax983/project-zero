#!/bin/bash

sed -i 's/pub struct PolicyState {/\/\/\/ Stores the current state of a policy, including its active status and duration.\n\/\/\/\n\/\/\/ # Examples\n\/\/\/\n\/\/\/ ```\n\/\/\/ use scale::layer1::administration::edicts::PolicyState;\n\/\/\/\n\/\/\/ let state = PolicyState { active: true, duration: 10, is_tradition: false };\n\/\/\/ ```\npub struct PolicyState {/' src/layer1/administration/edicts.rs

sed -i 's/    pub active: bool,/    \/\/\/ Whether the policy is currently active.\n    pub active: bool,/' src/layer1/administration/edicts.rs
sed -i 's/    pub duration: u32,/    \/\/\/ The remaining duration of the policy in ticks.\n    pub duration: u32,/' src/layer1/administration/edicts.rs
sed -i 's/    pub is_tradition: bool,/    \/\/\/ Whether this policy has been entrenched as a cultural tradition.\n    pub is_tradition: bool,/' src/layer1/administration/edicts.rs

sed -i 's/    pub policy_states: std::collections::HashMap<Policy, PolicyState>,/    \/\/\/ A map of all policies and their current state information.\n    pub policy_states: std::collections::HashMap<Policy, PolicyState>,/' src/layer1/administration/edicts.rs

sed -i 's/    MartialLaw,/    \/\/\/ Declares martial law, restricting movement and increasing security.\n    MartialLaw,/' src/layer1/administration/edicts.rs

sed -i 's/pub struct TogglePolicyEvent(pub Policy);/\/\/\/ Event triggered to toggle the active status of a policy.\npub struct TogglePolicyEvent(pub Policy);/' src/layer1/administration/edicts.rs

sed -i 's/pub struct AccessDeniedEvent {/\/\/\/ Event triggered when a policy action is denied due to insufficient access.\npub struct AccessDeniedEvent {/' src/layer1/administration/edicts.rs
sed -i 's/    pub reason: String,/    \/\/\/ The reason why access was denied.\n    pub reason: String,/' src/layer1/administration/edicts.rs

sed -i 's/pub struct RevokePolicyEvent {/\/\/\/ Event triggered to forcefully revoke an active policy.\npub struct RevokePolicyEvent {/' src/layer1/administration/edicts.rs
sed -i 's/    pub policy: Policy,/    \/\/\/ The policy to revoke.\n    pub policy: Policy,/' src/layer1/administration/edicts.rs

sed -i 's/pub struct HackCentralHubEvent {/\/\/\/ Event triggered when attempting to hack the central hub to force a policy change.\npub struct HackCentralHubEvent {/' src/layer1/administration/edicts.rs
sed -i 's/    pub target_policy: Policy,/    \/\/\/ The policy targeted by the hack.\n    pub target_policy: Policy,/' src/layer1/administration/edicts.rs

sed -i 's/pub struct ColonyPolicies {/\/\/\/ Resource tracking active colony policies\/edicts and their states.\n\/\/\/\n\/\/\/ # Examples\n\/\/\/\n\/\/\/ ```\n\/\/\/ use scale::layer1::administration::edicts::{ColonyPolicies, Policy};\n\/\/\/\n\/\/\/ let mut policies = ColonyPolicies::default();\n\/\/\/ policies.toggle(Policy::Rationing);\n\/\/\/ assert!(policies.is_active(Policy::Rationing));\n\/\/\/ ```\npub struct ColonyPolicies {/' src/layer1/administration/edicts.rs

sed -i 's/pub fn handle_revoke_policy_system(/\/\/\/ Handles events to forcefully revoke an active policy.\n\/\/\/\n\/\/\/ If the policy is a tradition, revoking it will cause unrest.\n\/\/\/\n\/\/\/ # Examples\n\/\/\/\n\/\/\/ ```\n\/\/\/ use bevy_ecs::prelude::*;\n\/\/\/ use scale::layer1::administration::edicts::{handle_revoke_policy_system, RevokePolicyEvent, ColonyPolicies, Policy, PolicyState};\n\/\/\/ use scale::layer1::social::unrest::Unrest;\n\/\/\/\n\/\/\/ let mut world = World::new();\n\/\/\/ let mut policies = ColonyPolicies::default();\n\/\/\/ policies.active_policies.insert(Policy::Rationing);\n\/\/\/ policies.policy_states.insert(Policy::Rationing, PolicyState { active: true, duration: 0, is_tradition: false });\n\/\/\/ world.insert_resource(policies);\n\/\/\/ world.init_resource::<Unrest>();\n\/\/\/ world.init_resource::<Events<RevokePolicyEvent>>();\n\/\/\/ world.send_event(RevokePolicyEvent { policy: Policy::Rationing });\n\/\/\/\n\/\/\/ let mut schedule = Schedule::default();\n\/\/\/ schedule.add_systems(handle_revoke_policy_system);\n\/\/\/ schedule.run(\&mut world);\n\/\/\/ ```\npub fn handle_revoke_policy_system(/' src/layer1/administration/edicts.rs

sed -i 's/pub fn handle_policy_toggle_system(/\/\/\/ Handles events to toggle the active status of a policy.\n\/\/\/\n\/\/\/ If a policy is orphaned, it cannot be toggled normally and will emit an `AccessDeniedEvent`.\n\/\/\/\n\/\/\/ # Examples\n\/\/\/\n\/\/\/ ```\n\/\/\/ use bevy_ecs::prelude::*;\n\/\/\/ use scale::layer1::administration::edicts::{handle_policy_toggle_system, TogglePolicyEvent, AccessDeniedEvent, ColonyPolicies, Policy};\n\/\/\/\n\/\/\/ let mut world = World::new();\n\/\/\/ world.init_resource::<ColonyPolicies>();\n\/\/\/ world.init_resource::<Events<TogglePolicyEvent>>();\n\/\/\/ world.init_resource::<Events<AccessDeniedEvent>>();\n\/\/\/ world.send_event(TogglePolicyEvent(Policy::DoubleShifts));\n\/\/\/\n\/\/\/ let mut schedule = Schedule::default();\n\/\/\/ schedule.add_systems(handle_policy_toggle_system);\n\/\/\/ schedule.run(\&mut world);\n\/\/\/ ```\npub fn handle_policy_toggle_system(/' src/layer1/administration/edicts.rs

sed -i 's/pub fn handle_hack_hub_system(/\/\/\/ Handles events to hack the central hub and clear orphaned policies.\n\/\/\/\n\/\/\/ # Examples\n\/\/\/\n\/\/\/ ```\n\/\/\/ use bevy_ecs::prelude::*;\n\/\/\/ use scale::layer1::administration::edicts::{handle_hack_hub_system, HackCentralHubEvent, ColonyPolicies, Policy};\n\/\/\/\n\/\/\/ let mut world = World::new();\n\/\/\/ let mut policies = ColonyPolicies::default();\n\/\/\/ policies.orphaned_policies.insert(Policy::Rationing);\n\/\/\/ world.insert_resource(policies);\n\/\/\/ world.init_resource::<Events<HackCentralHubEvent>>();\n\/\/\/ world.send_event(HackCentralHubEvent { target_policy: Policy::Rationing });\n\/\/\/\n\/\/\/ let mut schedule = Schedule::default();\n\/\/\/ schedule.add_systems(handle_hack_hub_system);\n\/\/\/ schedule.run(\&mut world);\n\/\/\/ ```\npub fn handle_hack_hub_system(/' src/layer1/administration/edicts.rs

sed -i 's/pub fn update_policy_tradition_system(/\/\/\/ Periodically updates policy durations and entrenches long-running ones as traditions.\n\/\/\/\n\/\/\/ # Examples\n\/\/\/\n\/\/\/ ```\n\/\/\/ use bevy_ecs::prelude::*;\n\/\/\/ use scale::layer1::administration::edicts::{update_policy_tradition_system, ColonyPolicies, Policy, PolicyState};\n\/\/\/\n\/\/\/ let mut world = World::new();\n\/\/\/ let mut policies = ColonyPolicies::default();\n\/\/\/ policies.active_policies.insert(Policy::Rationing);\n\/\/\/ policies.policy_states.insert(Policy::Rationing, PolicyState { active: true, duration: 9999, is_tradition: false });\n\/\/\/ world.insert_resource(policies);\n\/\/\/\n\/\/\/ let mut schedule = Schedule::default();\n\/\/\/ schedule.add_systems(update_policy_tradition_system);\n\/\/\/ schedule.run(\&mut world);\n\/\/\/\n\/\/\/ let p = world.resource::<ColonyPolicies>();\n\/\/\/ assert!(p.policy_states.get(\&Policy::Rationing).unwrap().is_tradition);\n\/\/\/ ```\npub fn update_policy_tradition_system(/' src/layer1/administration/edicts.rs

sed -i 's/crate::layer1::edicts::/crate::layer1::administration::edicts::/g' src/layer1/administration/edicts.rs

sed -i 's/\/\/\/ Returns the modifier for hunger decay rate./\/\/\/ Calculates the global hunger decay modifier based on active policies.\n\/\/\/\n\/\/\/ * `Rationing`: 0.5x decay.\n\/\/\/\n\/\/\/ # Examples\n\/\/\/ ```\n\/\/\/ use scale::layer1::administration::edicts::{ColonyPolicies, Policy, get_hunger_decay_modifier};\n\/\/\/ let mut policies = ColonyPolicies::default();\n\/\/\/ policies.toggle(Policy::Rationing);\n\/\/\/ assert_eq!(get_hunger_decay_modifier(\&policies), 0.5);\n\/\/\/ ```/' src/layer1/administration/edicts.rs

sed -i 's/\/\/\/ Returns the modifier for work speed./\/\/\/ Calculates the global work speed modifier based on active policies.\n\/\/\/\n\/\/\/ * `DoubleShifts`: 1.2x speed.\n\/\/\/ * `PestControl`: 0.95x speed.\n\/\/\/\n\/\/\/ # Examples\n\/\/\/ ```\n\/\/\/ use scale::layer1::administration::edicts::{ColonyPolicies, Policy, get_work_speed_modifier};\n\/\/\/ let mut policies = ColonyPolicies::default();\n\/\/\/ policies.toggle(Policy::DoubleShifts);\n\/\/\/ assert_eq!(get_work_speed_modifier(\&policies), 1.2);\n\/\/\/ ```/' src/layer1/administration/edicts.rs

sed -i 's/\/\/\/ Returns the flat modifier for morale./\/\/\/ Calculates the global morale modifier based on active policies.\n\/\/\/\n\/\/\/ * `Rationing`: -0.1 morale.\n\/\/\/ * `DoubleShifts`: -0.1 morale.\n\/\/\/\n\/\/\/ # Examples\n\/\/\/ ```\n\/\/\/ use scale::layer1::administration::edicts::{ColonyPolicies, Policy, get_morale_modifier};\n\/\/\/ let mut policies = ColonyPolicies::default();\n\/\/\/ policies.toggle(Policy::DoubleShifts);\n\/\/\/ assert_eq!(get_morale_modifier(\&policies), -0.1);\n\/\/\/ ```/' src/layer1/administration/edicts.rs

sed -i '/\/\/\/ \* `Rationing`: 0.5x decay./d' src/layer1/administration/edicts.rs
sed -i '/\/\/\/ \* `DoubleShifts`: 1.2x speed./d' src/layer1/administration/edicts.rs
sed -i '/\/\/\/ \* `PestControl`: 0.95x speed./d' src/layer1/administration/edicts.rs
sed -i '/\/\/\/ \* `Rationing`: -0.1 morale./d' src/layer1/administration/edicts.rs
sed -i '/\/\/\/ \* `DoubleShifts`: -0.1 morale./d' src/layer1/administration/edicts.rs

cargo test --doc
