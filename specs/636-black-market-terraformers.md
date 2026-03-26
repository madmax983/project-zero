# 636: Black-Market Terraformers

## 1. Overview

Unsanctioned environmental manipulation that fixes one problem while creating a terrifying unknown. If a colony's environment is highly hostile (e.g., toxic atmosphere, extreme cold) and the player hasn't invested in official terraforming, wealthy Pops might hire "Black-Market Terraformers" from a passing Layer 2 smuggler fleet. These rogue engineers deploy cheap, unregulated geo-engineering devices. They rapidly solve the immediate environmental hazard but introduce a permanent, unstable "Wildcard" trait to the planet (e.g., sentient weather patterns, hyper-aggressive localized flora).

## 2. Dependencies

- `063` Atmospheric Simulation
- `107` Biocompatibility
- `348` The Black Market

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_black_market_terraformer_contract() {
        // Arrange
        let mut app = App::new();
        app.add_event::<EnvironmentalHazardEvent>()
           .add_event::<BlackMarketTerraformEvent>()
           .add_systems(Update, trigger_black_market_terraforming);

        app.world_mut().insert_resource(TerraformState { has_official_tech: false });
        let colony = app.world_mut().spawn(Colony {
            wealth: 5000.0,
            hazard_level: 100.0,
        }).id();

        // Act
        app.world_mut().send_event(EnvironmentalHazardEvent {
            colony,
            severity: 100.0,
            type_: HazardType::ToxicAtmosphere,
        });
        app.update();

        // Assert
        let events = app.world().resource::<Events<BlackMarketTerraformEvent>>();
        assert_eq!(events.iter().count(), 1, "High hazard without tech should trigger black market terraform event");

        let colony_data = app.world().get::<Colony>(colony).unwrap();
        assert!(colony_data.wealth < 5000.0, "Colony wealth should decrease to pay for rogue terraformers");
    }

    #[test]
    fn test_terraformer_wildcard_application() {
        // Arrange
        let mut app = App::new();
        app.add_event::<BlackMarketTerraformEvent>()
           .add_systems(Update, apply_terraforming_effects);

        let planet = app.world_mut().spawn(Planet {
            hazard_level: 100.0,
            wildcard_traits: vec![],
        }).id();

        // Act
        app.world_mut().send_event(BlackMarketTerraformEvent { planet, cost: 1000.0 });
        app.update();

        // Assert
        let planet_data = app.world().get::<Planet>(planet).unwrap();
        assert_eq!(planet_data.hazard_level, 0.0, "Hazard should be completely cleared");
        assert!(planet_data.wildcard_traits.len() > 0, "Planet must receive an unstable wildcard trait");
    }

    #[test]
    fn test_official_tech_prevents_rogue_action() {
        // Arrange
        let mut app = App::new();
        app.add_event::<EnvironmentalHazardEvent>()
           .add_systems(Update, trigger_black_market_terraforming);

        app.world_mut().insert_resource(TerraformState { has_official_tech: true });
        let colony = app.world_mut().spawn(Colony {
            wealth: 5000.0,
            hazard_level: 100.0,
        }).id();

        // Act
        app.world_mut().send_event(EnvironmentalHazardEvent {
            colony,
            severity: 100.0,
            type_: HazardType::ToxicAtmosphere,
        });
        app.update();

        // Assert
        let events = app.world().resource::<Events<BlackMarketTerraformEvent>>();
        assert_eq!(events.iter().count(), 0, "Official tech prevents Pops from hiring rogue terraformers");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::seq::SliceRandom;

#[derive(Component)]
pub struct Colony {
    pub wealth: f32,
    pub hazard_level: f32,
}

#[derive(Component)]
pub struct Planet {
    pub hazard_level: f32,
    pub wildcard_traits: Vec<WildcardTrait>,
}

#[derive(PartialEq, Clone)]
pub enum HazardType {
    ToxicAtmosphere,
    ExtremeCold,
}

#[derive(PartialEq, Clone, Debug)]
pub enum WildcardTrait {
    AcidTsunamis,
    SentientWeather,
    AggressiveFlora,
}

#[derive(Resource)]
pub struct TerraformState {
    pub has_official_tech: bool,
}

#[derive(Event)]
pub struct EnvironmentalHazardEvent {
    pub colony: Entity,
    pub severity: f32,
    pub type_: HazardType,
}

#[derive(Event)]
pub struct BlackMarketTerraformEvent {
    pub planet: Entity,
    pub cost: f32,
}

pub fn trigger_black_market_terraforming(
    mut events: EventReader<EnvironmentalHazardEvent>,
    mut terraform_events: EventWriter<BlackMarketTerraformEvent>,
    mut colonies: Query<&mut Colony>,
    state: Res<TerraformState>,
) {
    if state.has_official_tech {
        return;
    }

    for event in events.read() {
        if event.severity >= 80.0 {
            if let Ok(mut colony) = colonies.get_mut(event.colony) {
                if colony.wealth >= 1000.0 {
                    colony.wealth -= 1000.0;
                    terraform_events.send(BlackMarketTerraformEvent {
                        planet: event.colony, // Assuming colony and planet entity mapping is simplified here
                        cost: 1000.0,
                    });
                }
            }
        }
    }
}

pub fn apply_terraforming_effects(
    mut events: EventReader<BlackMarketTerraformEvent>,
    mut planets: Query<&mut Planet>,
) {
    let possible_traits = vec![
        WildcardTrait::AcidTsunamis,
        WildcardTrait::SentientWeather,
        WildcardTrait::AggressiveFlora,
    ];
    let mut rng = rand::thread_rng();

    for event in events.read() {
        if let Ok(mut planet) = planets.get_mut(event.planet) {
            planet.hazard_level = 0.0;
            if let Some(trait_to_add) = possible_traits.choose(&mut rng) {
                planet.wildcard_traits.push(trait_to_add.clone());
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Wealth Source:** Pops themselves, specifically the wealthy caste (if social stratification exists), should initiate the contract and pay for it, not a generic "Colony Wealth" pool. This creates more specific internal economic shifts.
- **Rogue Fleet Presence:** The event should only trigger if a smuggler fleet or black market entity is actively present in Layer 2 orbit, integrating with `348 The Black Market`.
- **Wildcard Implementation:** The `WildcardTrait` enum needs to tie into actual environmental mechanics, such as registering new localized hazards (e.g., `AcidTsunamis` scheduling periodic destruction of coastal tiles).
- **Delayed Gratification:** The terraforming shouldn't be instantaneous. A temporary geo-engineering structure should spawn and complete the process over time.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] High hazard environments without official tech trigger rogue terraforming contracts if wealth is sufficient.
- [ ] Rogue terraforming successfully clears the primary hazard.
- [ ] A permanent, unstable wildcard trait is applied to the planet after rogue terraforming.

## 7. Technical Guidance

- Utilize random trait generation from a predefined set of unstable environmental effects.
- Implement a cooldown or a flag to ensure the black market event doesn't trigger multiple times for the same hazard.
- Display a notification indicating that rogue elements have altered the planet's ecosystem.

## 8. Questions

*Builder: add questions here if spec is unclear.*
