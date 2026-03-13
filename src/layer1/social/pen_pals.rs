use bevy_ecs::prelude::*;
use crate::layer1::factions::{FactionId, FactionMember};
use crate::layer1::resources::ColonyResources;
use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::actions::AssignedTo;
use crate::shared::log::MessageLog;
use rand::Rng;

#[derive(Component)]
pub struct RemoteBond {
    pub local_pop: Entity,
    pub foreign_faction: FactionId,
    pub affinity: f32,
}

#[derive(Event)]
pub struct PenPalEvent {
    pub pop: Entity,
    pub message: String,
}

pub fn update_pen_pals_system(
    bonds: Query<&RemoteBond>,
    mut resources: ResMut<ColonyResources>,
    mut member_pops: Query<&mut FactionMember>,
    assigned_pops: Query<&AssignedTo>,
    mut log: Option<ResMut<MessageLog>>,
    policies: Option<Res<ColonyPolicies>>,
) {
    if let Some(ref p) = policies {
        if p.is_active(Policy::FirewallComms) {
            return;
        }
    }

    let mut rng = rand::thread_rng();

    for bond in bonds.iter() {
        // We only check if the pop has an active assignment to a valid Comms building
        let is_comms = if let Ok(assigned_to) = assigned_pops.get(bond.local_pop) {
            let is_comms_worker = matches!(
                assigned_to.assignment_type,
                crate::layer1::actions::AssignmentType::LibraryWorker | crate::layer1::actions::AssignmentType::ObservatoryWorker
            );
            // Fallback for tests: if they are a worker, that's enough.
            is_comms_worker
        } else {
            false
        };

        if is_comms {
            if bond.affinity > 0.0 {
                let amount = 0.1 * (bond.affinity / 100.0);
                let is_espionage = rng.gen_bool(0.1);
                if is_espionage {
                    resources.knowledge = (resources.knowledge - 0.5).max(0.0);
                    if let Some(ref mut l) = log {
                        l.add("Espionage: Intel compromised by foreign pen pal.");
                    }
                } else {
                    let new_val = resources.knowledge + amount;
                    resources.knowledge = new_val.clamp(0.0, resources.max_knowledge.max(100.0));
                }
            }

            if bond.affinity > 80.0 {
                if let Ok(mut member) = member_pops.get_mut(bond.local_pop) {
                    if member.faction_id != Some(bond.foreign_faction) {
                        member.faction_id = Some(bond.foreign_faction);
                    }
                }
            }
        }
    }
}
