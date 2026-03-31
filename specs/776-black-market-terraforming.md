# 776: Black Market Terraforming

## 1. Overview
**Layer:** Cross-layer (1 & 3)
**Fantasy:** Rogue billionaires playing god with a planet's climate to boost their own profit margins, consequences be damned.
**Mechanic:** Wealthy Pops/factions can secretly fund localized terraforming projects (like atmospheric seeders to increase rain for their farms). This inadvertently ruins other sectors (e.g. flooding an industrial sector).

## 2. Dependencies
- Faction/Pop wealth mechanics
- Planetary Climate / Terraforming systems
- Grid/Sector system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_wealthy_faction_triggers_rogue_terraforming() {
        let mut app = App::new();
        app.add_systems(Update, trigger_rogue_terraforming);

        let faction_entity = app.world_mut().spawn((
            Faction { wealth: 10_000.0, sector: 1 },
            CorporateGreed { active: true },
        )).id();

        app.world_mut().spawn((
            SectorClimate { sector: 1, humidity: 10.0 },
        ));

        // Neighboring sector that will get ruined
        app.world_mut().spawn((
            SectorClimate { sector: 2, humidity: 10.0 },
        ));

        app.update();

        // Faction spent money
        let faction = app.world().get::<Faction>(faction_entity).unwrap();
        assert!(faction.wealth < 10_000.0, "Faction should spend wealth to terraform");

        // Their sector improved (increased humidity for farming)
        let climates = app.world().query::<&SectorClimate>();
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
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

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

pub fn trigger_rogue_terraforming(
    mut factions: Query<(&mut Faction, &CorporateGreed)>,
    mut climates: Query<&mut SectorClimate>,
) {
    for (mut faction, greed) in factions.iter_mut() {
        if greed.active && faction.wealth > 5000.0 {
            // Secretly buy atmospheric seeders
            faction.wealth -= 5000.0;

            let target_sector = faction.sector;

            for mut climate in climates.iter_mut() {
                if climate.sector == target_sector {
                    climate.humidity += 50.0; // Boosts their local farms
                } else if climate.sector == target_sector + 1 || target_sector > 0 && climate.sector == target_sector - 1 {
                    climate.humidity += 30.0; // Floods neighbors inadvertently
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The spatial relation logic `sector == target_sector + 1` is brittle. A graph or spatial grid structure should be used to find neighboring sectors.
- **Performance**: Instead of iterating all climates per greedy faction, use spatial partitioning if the grid is large.
- **API Improvements**: Use a `RogueTerraformEvent` so that UI can notify the player that "Unseasonal rains are flooding Sector 2" without explicitly revealing the culprit.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Wealth drops, target sector climate changes, neighbor sector climate changes collaterally.

## 7. Technical Guidance
- Integrate with the planet's atmospheric model so that local humidity spikes cause actual "Flooding" status effects on industrial buildings in those sectors.

## 8. Questions
*Builder: add questions here if spec is unclear.*
