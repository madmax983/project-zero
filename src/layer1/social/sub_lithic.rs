use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::factions::{FactionId, FactionMember};
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::pop::Job;
use crate::layer1::utility_types::AssignmentType;
use crate::layer1::building::BuildingType;
use crate::layer1::health::Health;

#[derive(Component, Default)]
pub struct DeepMiningExposure(pub f32);

// Process exposure to deep mining environments
pub fn process_deep_mining_exposure(
    mut query: Query<(&Job, &mut DeepMiningExposure, &mut Traits)>,
) {
    for (job, mut exposure, mut traits) in query.iter_mut() {
        if job.job_type == AssignmentType::DeepMining {
            exposure.0 += 0.2; // Arbitrary increment
            if exposure.0 > 1.0 {
                traits.add(Trait::Agoraphobic);
            }
        }
    }
}

// Logic to group agoraphobic pops into the Sub-Lithic Cult faction
pub fn evaluate_cult_formation(
    mut query: Query<(&Traits, &mut FactionMember), With<Pop>>,
) {
    let mut agoraphobic_count = 0;
    for (traits, _) in query.iter() {
        if traits.has(Trait::Agoraphobic) {
            agoraphobic_count += 1;
        }
    }

    if agoraphobic_count >= 5 {
        for (traits, mut faction_member) in query.iter_mut() {
            if traits.has(Trait::Agoraphobic) {
                faction_member.faction_id = Some(FactionId::SubLithic);
            }
        }
    }
}

// Logic for cult members to target surface structures
#[derive(Event, Debug, Clone)]
pub struct SabotageEvent {
    pub target: Entity,
}

pub fn process_cult_sabotage(
    cult_query: Query<&FactionMember, With<Pop>>,
    target_query: Query<(Entity, &crate::layer1::building::Building), With<Health>>,
    mut sabotage_events: EventWriter<SabotageEvent>,
) {
    for member in cult_query.iter() {
        if member.faction_id == Some(FactionId::SubLithic) {
            for (target_entity, building) in target_query.iter() {
                if building.building_type == BuildingType::TradeDepot {
                    sabotage_events.send(SabotageEvent { target: target_entity });
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agoraphobia_development() {
        // Arrange: Setup world with a pop assigned to deep mining
        let mut app = World::new();
        let workplace = app.spawn(()).id();
        let pop = app.spawn((
            Pop,
            Job {
                workplace,
                job_type: AssignmentType::DeepMining,
            },
            DeepMiningExposure(0.0),
            Traits::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_deep_mining_exposure);

        // Act: Run simulation ticks to increase exposure
        for _ in 0..10 {
            schedule.run(&mut app);
        }

        // Assert: Verify pop developed Agoraphobia trait
        let exposure = app.get::<DeepMiningExposure>(pop).unwrap();
        assert!(exposure.0 > 1.0); // Threshold reached

        let traits = app.get::<Traits>(pop).unwrap();
        let has_agoraphobia = traits.has(Trait::Agoraphobic);
        assert!(has_agoraphobia, "Pop should develop Agoraphobia after prolonged deep mining");
    }

    #[test]
    fn test_sub_lithic_cult_formation() {
        // Arrange: Multiple pops with Agoraphobia
        let mut app = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_cult_formation);

        for _ in 0..5 {
            let mut traits = Traits::default();
            traits.add(Trait::Agoraphobic);
            app.spawn((Pop, traits, FactionMember::default()));
        }

        // Act: Run cult formation system
        schedule.run(&mut app);

        // Assert: A new Faction with Sub-Lithic Ideology should be created
        // We will check if the FactionMember has FactionId::SubLithic
        let mut query = app.query::<&FactionMember>();
        let cult_exists = query.iter(&app).any(|f| f.faction_id == Some(FactionId::SubLithic));
        assert!(cult_exists, "Sub-Lithic Cult faction should form when enough pops have Agoraphobia");
    }

    #[test]
    fn test_spaceport_sabotage() {
        // Arrange: A cult member and a spaceport building
        let mut app = World::new();
        app.insert_resource(bevy_ecs::event::Events::<SabotageEvent>::default());
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::systems::update_event_buffer::<SabotageEvent>);
        schedule.add_systems(process_cult_sabotage.after(crate::layer1::systems::update_event_buffer::<SabotageEvent>));

        let _spaceport = app.spawn((crate::layer1::building::Building { building_type: BuildingType::TradeDepot, ..Default::default() }, Health { current: 100.0, max: 100.0 })).id();
        let mut traits = Traits::default();
        traits.add(Trait::Agoraphobic);
        app.spawn((
            Pop,
            traits,
            FactionMember { faction_id: Some(FactionId::SubLithic) },
        ));

        app.resource_mut::<Events<SabotageEvent>>().clear();

        // Act: Run sabotage logic
        schedule.run(&mut app);

        // Assert: Spaceport is sabotaged
        let sabotage_events = app.resource::<Events<SabotageEvent>>();
        assert!(!sabotage_events.is_empty(), "Cult member should trigger a sabotage event on the spaceport");
    }
}
