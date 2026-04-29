use bevy_ecs::prelude::*;

use crate::layer1::mind::utility_types::ActionType;
use crate::layer1::social::factions::{FactionMember, Factions};

/// Evaluates if a pop should join a protest mob.
/// Pops with low faction satisfaction are more likely to join a protest.
pub fn evaluate_protest(
    entity: Entity,
    faction_member_q: &Query<&FactionMember>,
    factions: Option<&Factions>,
) -> f32 {
    let mut score = 0.0;

    if let (Some(factions), Ok(membership)) = (factions, faction_member_q.get(entity)) {
        if let Some(faction_id) = membership.faction_id {
            if let Some(faction) = factions.get(faction_id) {
                // Lower satisfaction means higher chance to protest
                if faction.satisfaction < 40.0 {
                    score = 80.0 - faction.satisfaction; // Peaks at 80
                }
            }
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use crate::layer1::social::factions::{FactionData, FactionId};

    #[test]
    fn test_evaluate_protest_high_satisfaction() {
        let mut app = App::new();

        let mut factions = Factions::default();
        factions.insert(FactionId::NostalgiaCult, FactionData { satisfaction: 90.0, ..Default::default() });
        app.insert_resource(factions);
        let pop = app.world_mut().spawn(FactionMember { faction_id: Some(FactionId::NostalgiaCult) }).id();

        let faction_member_q = app.world_mut().query::<&FactionMember>();
        let factions_res = app.world().resource::<Factions>();

        let score = evaluate_protest(pop, &faction_member_q, Some(factions_res));
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_evaluate_protest_low_satisfaction() {
        let mut app = App::new();

        let mut factions = Factions::default();
        factions.insert(FactionId::NostalgiaCult, FactionData { satisfaction: 10.0, ..Default::default() });
        app.insert_resource(factions);
        let pop = app.world_mut().spawn(FactionMember { faction_id: Some(FactionId::NostalgiaCult) }).id();

        let faction_member_q = app.world_mut().query::<&FactionMember>();
        let factions_res = app.world().resource::<Factions>();

        let score = evaluate_protest(pop, &faction_member_q, Some(factions_res));
        assert_eq!(score, 70.0);
    }

    #[test]
    fn test_evaluate_protest_no_faction() {
        let mut app = App::new();

        let pop = app.world_mut().spawn_empty().id();

        let faction_member_q = app.world_mut().query::<&FactionMember>();
        let factions_res = Factions::default();

        let score = evaluate_protest(pop, &faction_member_q, Some(&factions_res));
        assert_eq!(score, 0.0);
    }
}
