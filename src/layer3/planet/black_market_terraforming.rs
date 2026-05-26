//! Black Market Terraforming
//!
//! Simulates illegal and highly dangerous planetary engineering.
//! Syndicates offer cheap terraforming that often results in catastrophic ecological backlash.

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Faction {
    pub wealth: f32,
    pub sector: u32,
}

#[derive(Component)]
pub struct CorporateGreed {
    pub active: bool,
}

#[derive(Component)]
pub struct SectorClimate {
    pub sector: u32,
    pub humidity: f32,
}

#[derive(Event)]
pub struct RogueTerraformEvent {
    pub target_sector: u32,
}

pub fn trigger_rogue_terraforming(
    mut factions: Query<(&mut Faction, &CorporateGreed)>,
    mut events: EventWriter<RogueTerraformEvent>,
) {
    for (mut faction, greed) in factions.iter_mut() {
        if greed.active && faction.wealth > 5000.0 {
            // Secretly buy atmospheric seeders
            faction.wealth -= 5000.0;

            events.send(RogueTerraformEvent {
                target_sector: faction.sector,
            });
        }
    }
}

pub fn apply_rogue_terraforming_events(
    mut events: EventReader<RogueTerraformEvent>,
    mut climates: Query<&mut SectorClimate>,
) {
    for event in events.read() {
        let target_sector = event.target_sector;

        for mut climate in climates.iter_mut() {
            if climate.sector == target_sector {
                climate.humidity += 50.0; // Boosts their local farms
            } else if climate.sector == target_sector + 1
                || target_sector > 0 && climate.sector == target_sector - 1
            {
                climate.humidity += 30.0; // Floods neighbors inadvertently
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_wealthy_faction_triggers_rogue_terraforming() {
        let mut app = App::new();
        app.add_event::<RogueTerraformEvent>();
        app.add_systems(
            Update,
            (trigger_rogue_terraforming, apply_rogue_terraforming_events),
        );

        let faction_entity = app
            .world_mut()
            .spawn((
                Faction {
                    wealth: 10_000.0,
                    sector: 1,
                },
                CorporateGreed { active: true },
            ))
            .id();

        app.world_mut().spawn((SectorClimate {
            sector: 1,
            humidity: 10.0,
        },));

        // Neighboring sector that will get ruined
        app.world_mut().spawn((SectorClimate {
            sector: 2,
            humidity: 10.0,
        },));

        app.update(); // Triggers event
        app.update(); // Processes event

        // Faction spent money
        let faction = app.world().get::<Faction>(faction_entity).unwrap();
        assert!(
            faction.wealth < 10_000.0,
            "Faction should spend wealth to terraform"
        );

        // Their sector improved (increased humidity for farming)
        let mut climates = app.world_mut().query::<&SectorClimate>();
        let mut found_sec_1 = false;
        let mut found_sec_2 = false;
        for c in climates.iter(app.world()) {
            if c.sector == 1 {
                assert!(c.humidity > 10.0, "Target sector humidity should rise");
                found_sec_1 = true;
            }
            if c.sector == 2 {
                assert!(c.humidity > 10.0, "Adjacent sector flooded (ruined)");
                found_sec_2 = true;
            }
        }
        assert!(found_sec_1 && found_sec_2);
    }
}
