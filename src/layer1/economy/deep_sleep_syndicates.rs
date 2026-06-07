use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
use crate::layer1::psychology::stress::TraumaTracker;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct ProductionModifier {
    pub multiplier: f32,
}

impl Default for ProductionModifier {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SyndicateType {
    Industrial,
    Logistical,
    Military,
}

#[derive(Event)]
pub struct ThawSyndicateEvent {
    pub syndicate_type: SyndicateType,
}

pub fn process_thaw_syndicate_event(
    mut events: EventReader<ThawSyndicateEvent>,
    mut production_modifier: ResMut<ProductionModifier>,
    mut trauma_tracker: ResMut<TraumaTracker>,
    mut policies: ResMut<ColonyPolicies>,
) {
    for event in events.read() {
        // Apply the immediate boost based on the type
        match event.syndicate_type {
            SyndicateType::Industrial => {
                production_modifier.multiplier += 2.0; // Massive +200% boost
            }
            _ => {
                production_modifier.multiplier += 1.0;
            }
        }

        // Add the severe drawback: Trauma and Unrest
        trauma_tracker.recent_deaths += 50;

        // Force an archaic policy
        policies.active_policies.insert(Policy::DoubleShifts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_thawing_syndicate_grants_boost_and_adds_trauma() {
        let mut app = App::new();
        app.add_systems(Update, process_thaw_syndicate_event);
        app.add_event::<ThawSyndicateEvent>();

        // Setup initial TraumaTracker
        app.insert_resource(TraumaTracker {
            recent_deaths: 0,
            famine_ticks: 0,
        });

        app.insert_resource(ColonyPolicies::default());

        // Setup a global production modifier
        app.insert_resource(ProductionModifier { multiplier: 1.0 });

        // Fire the event to thaw the syndicate
        app.world_mut()
            .resource_mut::<Events<ThawSyndicateEvent>>()
            .send(ThawSyndicateEvent {
                syndicate_type: SyndicateType::Industrial,
            });

        app.update();

        // Verify production was boosted
        let production = app.world().resource::<ProductionModifier>();
        assert!(
            production.multiplier > 1.0,
            "Thawing a syndicate should provide a massive production boost."
        );

        // Verify trauma was added to the TraumaTracker
        let trauma = app.world().resource::<TraumaTracker>();
        assert!(
            trauma.recent_deaths > 0,
            "Thawing a syndicate must add significant trauma to the colony."
        );

        // Verify policy was enacted
        let policies = app.world().resource::<ColonyPolicies>();
        assert!(
            policies.is_active(Policy::DoubleShifts),
            "Thawing a syndicate should force an archaic policy to be enacted."
        );
    }
}
