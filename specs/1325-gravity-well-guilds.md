# 1325: Gravity Well Guilds

## 1. Overview
Navigating the politics and monopolies of powerful organizations that control the logistics of getting on and off planets. Specialized logistics guilds form around high-gravity worlds. They control the orbital elevators and heavy lifters. If a player relies too much on a single world for exports, the local guild gains influence and can demand higher tariffs or political concessions.

## 2. Dependencies
- `099` Fleet Movement (for logistics)
- `1030` Gravity Engineering / Planet Gravity types
- `039` Trade System (for tariffs)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::planet::{Planet, GravityType};
    use crate::layer2::trade::{TradeRoute, Tariff};
    use crate::layer2::guilds::{LogisticsGuild, GuildInfluence, update_guild_influence};

    #[test]
    fn test_guild_forms_on_high_gravity() {
        let mut world = World::new();
        let planet = world.spawn((Planet { gravity: GravityType::High }, LogisticsGuild::default())).id();

        assert!(world.get::<LogisticsGuild>(planet).is_some());
    }

    #[test]
    fn test_export_volume_increases_influence() {
        let mut world = World::new();
        let mut guild = LogisticsGuild { influence: 10.0, ..Default::default() };
        let export_volume = 1000.0;

        update_guild_influence(&mut guild, export_volume);

        assert!(guild.influence > 10.0);
    }

    #[test]
    fn test_high_influence_raises_tariffs() {
        let mut world = World::new();
        let planet = world.spawn((Planet { gravity: GravityType::High }, LogisticsGuild { influence: 90.0, ..Default::default() })).id();
        let route = TradeRoute { origin: planet, tariff: Tariff(0.05) };

        let new_tariff = crate::layer2::guilds::calculate_guild_tariff(route.tariff, world.get::<LogisticsGuild>(planet).unwrap());

        assert!(new_tariff.0 > 0.05);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation in src/layer2/guilds.rs
use bevy_ecs::prelude::*;
use crate::layer2::trade::Tariff;

#[derive(Component, Default)]
pub struct LogisticsGuild {
    pub influence: f32,
    pub base_tariff_multiplier: f32,
}

pub fn update_guild_influence(guild: &mut LogisticsGuild, export_volume: f32) {
    // Simple linear increase based on volume
    guild.influence += export_volume * 0.001;
    if guild.influence > 100.0 {
        guild.influence = 100.0;
    }
}

pub fn calculate_guild_tariff(base_tariff: Tariff, guild: &LogisticsGuild) -> Tariff {
    let increase = (guild.influence / 100.0) * 0.10; // Up to +10% tariff
    Tariff(base_tariff.0 + increase)
}
```

## 5. REFACTOR Phase: Quality & Design
- Make the influence gain curve non-linear so it's harder to max out.
- Add an "Appease Guild" action to spend resources to lower their influence/tariffs.
- Hook this into the Layer 2 UI so players can see the Guild's current grip on the planet.

## 6. Acceptance Criteria (Testable!)
- [ ] Tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Guilds increase influence when large export volumes are processed
- [ ] Tariffs increase proportionally with guild influence

## 7. Technical Guidance
- Ensure `LogisticsGuild` is only applied to planets with `GravityType::High` or `GravityType::Extreme` during generation or via an event.
- Tariff calculation should intercept the standard trade pricing logic.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
