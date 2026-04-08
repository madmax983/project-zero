# 867: The Mutually Assured Quarantine

## 1. Overview
If a severe contagion breaks out on Layer 1, the planetary governor (Layer 2) automatically initiates a quarantine, halting all orbital trade to protect the wider empire. However, desperate Pops on Layer 1 can storm the local orbital defense cannons and hold the planet's hyperlane junction hostage, threatening to shoot down *all* passing commercial traffic unless the quarantine is lifted and a cure is delivered immediately.

## 2. Dependencies
- Layer 1 `ContagionTracker` or illness system
- Layer 2 `OrbitalTrade` and `GovernorDecision` logic
- `Pop` unrest / militant behavior system
- Planetary Defense Cannon entities / interactions

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_quarantine_activation_on_severe_contagion() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(QuarantinePlugin);

        let planet = app.world_mut().spawn((
            Planet,
            ContagionTracker { severity: 90.0, ..default() },
            TradeStatus::Open,
        )).id();

        // Act
        app.update();

        // Assert: Quarantine is enacted and trade is halted
        let trade_status = app.world().get::<TradeStatus>(planet).unwrap();
        assert_eq!(trade_status, &TradeStatus::Quarantined);
    }

    #[test]
    fn test_desperate_pops_hijack_defense_cannons() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(QuarantinePlugin);

        let planet = app.world_mut().spawn((
            Planet,
            ContagionTracker { severity: 95.0, duration: 1000, ..default() },
            TradeStatus::Quarantined,
        )).id();

        let cannon = app.world_mut().spawn((
            DefenseCannon,
            Location(planet),
            Status::Online,
            Control::Governor,
        )).id();

        // Setup high unrest pop
        let pop = app.world_mut().spawn((
            Pop,
            Location(planet),
            UnrestTracker { level: 90.0, ..default() },
        )).id();

        // Act
        app.update();

        // Assert: Pops take over cannons
        let cannon_control = app.world().get::<Control>(cannon).unwrap();
        assert_eq!(cannon_control, &Control::Mutineers);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Planet;

#[derive(Component, Default)]
pub struct ContagionTracker {
    pub severity: f32,
    pub duration: u32,
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum TradeStatus {
    Open,
    Quarantined,
    Blockaded,
}

#[derive(Component)]
pub struct DefenseCannon;

#[derive(Component)]
pub struct Location(pub Entity);

#[derive(Component, Debug, PartialEq, Eq)]
pub enum Status {
    Online,
    Offline,
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum Control {
    Governor,
    Mutineers,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component, Default)]
pub struct UnrestTracker {
    pub level: f32,
}

pub struct QuarantinePlugin;

impl Plugin for QuarantinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            governor_quarantine_system,
            cannon_hijack_system,
        ));
    }
}

fn governor_quarantine_system(
    mut planets: Query<(&ContagionTracker, &mut TradeStatus), With<Planet>>,
) {
    for (contagion, mut trade) in planets.iter_mut() {
        if contagion.severity > 80.0 {
            *trade = TradeStatus::Quarantined;
        }
    }
}

fn cannon_hijack_system(
    mut planets: Query<(Entity, &ContagionTracker, &TradeStatus), With<Planet>>,
    pops: Query<(&Location, &UnrestTracker), With<Pop>>,
    mut cannons: Query<(&Location, &mut Control), With<DefenseCannon>>,
) {
    for (planet_entity, contagion, trade) in planets.iter_mut() {
        if *trade == TradeStatus::Quarantined && contagion.duration >= 1000 {
            // Check if there are highly unrestful pops on this planet
            let has_unrest = pops.iter().any(|(loc, unrest)| loc.0 == planet_entity && unrest.level > 80.0);

            if has_unrest {
                // Hijack cannons
                for (cannon_loc, mut control) in cannons.iter_mut() {
                    if cannon_loc.0 == planet_entity {
                        *control = Control::Mutineers;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Magic Numbers:** Extract contagion severity and duration thresholds to a configuration resource `QuarantineConfig`.
- **System Decoupling:** Consider splitting `cannon_hijack_system` by emitting a `CannonHijackEvent` instead of directly mutating components. This allows for better tracing and audio/visual event triggers.
- **Consequences:** The hijack should actively block commercial traffic (e.g. modify a `HyperlaneStatus` component on adjacent routes).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the quarantine system.
- [ ] Quarantines trigger automatically on high contagion.
- [ ] Defense cannons are properly seized by mutineering Pops when conditions are met.

## 7. Technical Guidance
- Integrate with existing `Unrest` / `Rebellion` mechanics if they exist on Layer 1.
- Make sure to add this plugin to the appropriate Layer 2 / Layer 1 bridging schedule.

## 8. Questions
*Builder: add questions here if spec is unclear.*
