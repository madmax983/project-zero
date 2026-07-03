//! Blacksite Penal Colonies
//!
//! Allows the colony to host dangerous political prisoners from Layer 3 empires
//! for massive payouts, but introduces risks of prison breaks and radicalization.

use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::economy::ideological_contraband::Ethics;
use crate::layer1::rearguard::Enemy;
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::social::social_mimicry::SocialMimicry;
use crate::layer1::culture::artifacts::{ActiveAuras, AuraEffect};

/// Resource tracking the payout rate of a blacksite contract.
#[derive(Resource, Default)]
pub struct BlacksiteContract {
    /// The number of credits paid out per tick per prisoner.
    pub payout_rate: f32,
}

/// Component attached to entities representing a dangerous prisoner.
#[derive(Component)]
pub struct Prisoner {
    /// The current instability level of the prisoner. If it exceeds 10.0, a prison break may occur.
    pub instability: f32,
}

/// System to process payouts from blacksite contracts.
pub fn blacksite_payout_system(
    contract: Option<Res<BlacksiteContract>>,
    mut resources: ResMut<ColonyResources>,
    query: Query<&Prisoner>,
) {
    if let Some(c) = contract {
        for _ in &query {
            // Apply payout scaled down by an assumed tick rate (e.g. 1/60th or just flat small amount).
            resources.credits += c.payout_rate * 0.01;
        }
    }
}

/// System to handle the radicalization of wardens near prisoners.
pub fn prisoner_radicalization_system(
    mut wardens: Query<(&mut Ethics, &ActiveAuras), With<SocialMimicry>>,
) {
    for (mut ethics, auras) in &mut wardens {
        for effect in &auras.effects {
            if let AuraEffect::StressModifier(val) = effect {
                // Scale it down
                ethics.collectivism += (*val * 0.01) as i32;
            }
        }
    }
}

/// System that checks for high instability in prisoners and triggers a prison break event.
pub fn prison_break_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Prisoner), Without<Enemy>>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    for (entity, mut prisoner) in &mut query {
        if prisoner.instability > 10.0 {
            commands.entity(entity).insert(Enemy { action_speed: 1.0 });
            // Reset instability so if it ever loses Enemy, it doesn't immediately break again
            prisoner.instability = 0.0;
            chronicle.send(AddChronicleEvent {
                text: "A prison break has occurred!".to_string(),
                importance: EventImportance::Major,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacksite_contract_payout() {
        let mut world = World::new();
        world.insert_resource(BlacksiteContract { payout_rate: 100.0 });
        world.insert_resource(ColonyResources {
            credits: 0.0,
            ..Default::default()
        });
        world.spawn(Prisoner { instability: 0.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(blacksite_payout_system);
        schedule.run(&mut world);

        assert_eq!(world.resource::<ColonyResources>().credits, 1.0);
    }

    #[test]
    fn test_prisoner_radicalization() {
        let mut world = World::new();
        world.spawn((
            SocialMimicry::default(),
            Ethics { collectivism: 0, elitism: 0 },
            ActiveAuras { effects: vec![AuraEffect::StressModifier(500.0)] }
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(prisoner_radicalization_system);
        schedule.run(&mut world);

        let mut q = world.query::<&Ethics>();
        let ethics = q.single(&world);
        assert_eq!(ethics.collectivism, 5);
    }

    #[test]
    #[allow(deprecated)]
    fn test_prison_break_event() {
        let mut world = World::new();
        world.init_resource::<Events<AddChronicleEvent>>();
        let ent = world.spawn(Prisoner { instability: 11.0 }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(prison_break_system);
        schedule.run(&mut world);

        assert!(world.get::<Enemy>(ent).is_some());
        let events = world.resource::<Events<AddChronicleEvent>>();
        assert_eq!(events.get_reader().len(&events), 1);
    }
}
