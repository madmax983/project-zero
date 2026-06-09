use crate::layer3::market::GalacticMarketStatus;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct RepoFleetArrivalEvent {
    pub debt_amount: f32,
}

#[derive(Event)]
pub struct AttackRepoFleetEvent;

#[derive(Resource)]
pub struct ActiveCollectionProtocol {
    pub remaining_debt: f32,
}

pub fn handle_repo_fleet_arrival(
    mut commands: Commands,
    mut events: EventReader<RepoFleetArrivalEvent>,
) {
    for event in events.read() {
        commands.insert_resource(ActiveCollectionProtocol {
            remaining_debt: event.debt_amount,
        });
    }
}

pub fn handle_repo_fleet_attack(
    mut events: EventReader<AttackRepoFleetEvent>,
    mut market_status: Option<ResMut<GalacticMarketStatus>>,
) {
    for _ in events.read() {
        if let Some(ref mut status) = market_status {
            status.in_default = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_fleet_arrival_triggers_collection() {
        let mut app = App::new();
        app.add_event::<RepoFleetArrivalEvent>();
        app.add_systems(Update, handle_repo_fleet_arrival);

        app.world_mut().send_event(RepoFleetArrivalEvent {
            debt_amount: 1000000.0,
        });
        app.update();

        // Ensure a Collection Protocol is active
        assert!(
            app.world()
                .get_resource::<ActiveCollectionProtocol>()
                .is_some(),
            "Collection protocol should be active"
        );
    }

    #[test]
    fn test_attacking_repo_fleet_causes_default() {
        let mut app = App::new();
        app.insert_resource(GalacticMarketStatus { in_default: false });
        app.add_event::<AttackRepoFleetEvent>();
        app.add_systems(Update, handle_repo_fleet_attack);

        app.world_mut().send_event(AttackRepoFleetEvent);
        app.update();

        // Verify the colony is in default
        let market_status = app.world().resource::<GalacticMarketStatus>();
        assert!(
            market_status.in_default,
            "Attacking the Repo Fleet should trigger market default"
        );
    }
}

pub struct GenerationalDebtPlugin;

impl Plugin for GenerationalDebtPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RepoFleetArrivalEvent>()
            .add_event::<AttackRepoFleetEvent>()
            .add_systems(
                Update,
                (handle_repo_fleet_arrival, handle_repo_fleet_attack),
            );
    }
}
