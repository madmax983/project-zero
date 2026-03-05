# 300 - The Whisper Trade

## 1. Overview
The Whisper Trade implements a mechanic where "Secrets" function as a unique currency. Pops working in Taverns or Comms Centers occasionally discover Secrets about neighbors, the planet, or alien civilizations. A "Broker" building can then sell these Secrets to Layer 3 empires in exchange for unique tech or resources. However, engaging in the Whisper Trade increases global "Paranoia" in the colony, as the populace wonders who is listening, potentially leading to widespread unrest or diplomatic incidents.

## 2. Dependencies
- `004` Pop Entity
- `009` Job System
- `039` Trade System
- `097` Social Tavern (for generating secrets)
- `146` Command Center & System Visibility (for Comms Center generating secrets)
- `050` Civil Unrest (for Paranoia effects)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_secret_generation_in_tavern() {
        // Arrange: Setup world, Pop working in Tavern
        let mut app = App::new();
        app.add_systems(Update, generate_secrets_system);

        let tavern = app.world_mut().spawn(Tavern).id();
        let pop = app.world_mut().spawn((
            Pop,
            JobTenure { job_type: JobType::Bartender, ticks: 1000 },
            AtLocation(tavern),
        )).id();

        // Act: Run the system enough times to trigger a secret discovery
        for _ in 0..100 {
            app.update();
        }

        // Assert: The Pop or the Tavern should now hold a Secret component/resource
        let has_secret = app.world().query::<&Secret>().iter(app.world()).next().is_some()
            || app.world().get_resource::<ColonySecrets>().map_or(0, |res| res.count) > 0;
        assert!(has_secret, "A secret should have been generated.");
    }

    #[test]
    fn test_broker_trade_increases_paranoia() {
        // Arrange: Setup Broker, a Secret, and global Paranoia tracker
        let mut app = App::new();
        app.insert_resource(ColonySecrets { count: 1 });
        app.insert_resource(GlobalParanoia { level: 0.0 });
        app.add_event::<TradeSecretEvent>();
        app.add_systems(Update, execute_whisper_trade_system);

        let broker = app.world_mut().spawn(Broker).id();

        // Act: Trigger a trade event
        app.world_mut().send_event(TradeSecretEvent {
            broker_entity: broker,
            secret_value: 1
        });
        app.update();

        // Assert: Secrets should be consumed, Paranoia should increase
        assert_eq!(app.world().resource::<ColonySecrets>().count, 0);
        assert!(app.world().resource::<GlobalParanoia>().level > 0.0);
    }

    #[test]
    fn test_high_paranoia_causes_unrest() {
        // Arrange: Setup high paranoia, unrest tracking system
        let mut app = App::new();
        app.insert_resource(GlobalParanoia { level: 90.0 });
        app.add_systems(Update, paranoia_unrest_system);

        let pop = app.world_mut().spawn((Pop, UnrestTracker { amount: 0.0 })).id();

        // Act: Apply paranoia effects
        app.update();

        // Assert: Pop unrest should increase due to high global paranoia
        let unrest = app.world().get::<UnrestTracker>(pop).unwrap().amount;
        assert!(unrest > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

// --- Components and Resources ---
#[derive(Component)]
pub struct Tavern;

#[derive(Component)]
pub struct Broker;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct JobTenure {
    pub job_type: JobType,
    pub ticks: u32,
}

#[derive(Component)]
pub struct AtLocation(pub Entity);

#[derive(PartialEq, Eq)]
pub enum JobType {
    Bartender,
    CommsOfficer,
}

#[derive(Component)]
pub struct Secret;

#[derive(Resource, Default)]
pub struct ColonySecrets {
    pub count: u32,
}

#[derive(Resource, Default)]
pub struct GlobalParanoia {
    pub level: f32,
}

#[derive(Event)]
pub struct TradeSecretEvent {
    pub broker_entity: Entity,
    pub secret_value: u32,
}

#[derive(Component)]
pub struct UnrestTracker {
    pub amount: f32,
}

// --- Systems ---
pub fn generate_secrets_system(
    mut secrets: ResMut<ColonySecrets>,
    query: Query<&JobTenure, With<Pop>>,
) {
    let mut rng = rand::thread_rng();
    for tenure in query.iter() {
        if (tenure.job_type == JobType::Bartender || tenure.job_type == JobType::CommsOfficer)
            && rng.gen_bool(0.01) // 1% chance per tick
        {
            secrets.count += 1;
        }
    }
}

pub fn execute_whisper_trade_system(
    mut events: EventReader<TradeSecretEvent>,
    mut secrets: ResMut<ColonySecrets>,
    mut paranoia: ResMut<GlobalParanoia>,
    // In a full implementation, you'd also grant the reward here
) {
    for event in events.read() {
        if secrets.count >= event.secret_value {
            secrets.count -= event.secret_value;
            paranoia.level += 5.0 * (event.secret_value as f32);
        }
    }
}

pub fn paranoia_unrest_system(
    paranoia: Res<GlobalParanoia>,
    mut query: Query<&mut UnrestTracker, With<Pop>>,
) {
    if paranoia.level > 50.0 {
        let unrest_increase = (paranoia.level - 50.0) * 0.1;
        for mut tracker in query.iter_mut() {
            tracker.amount += unrest_increase;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor Opportunity:** Extract the secret generation chance (`0.01`) into a configuration resource (`WhisperTradeConfig`).
- **Refactor Opportunity:** `GlobalParanoia` could decay over time; introduce a `decay_paranoia_system`.
- **Design Improvement:** Differentiate secret "tiers" (e.g., Local Gossip vs. Faction Secret) instead of a simple integer count.
- **Integration:** Hook the `TradeSecretEvent` into the actual Layer 3 diplomacy/trade UI and reward tables.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] Secrets are generated by relevant jobs over time.
- [ ] Trading secrets increases a global paranoia metric.
- [ ] High paranoia correctly translates into pop-level unrest.

## 7. Technical Guidance
- **ECS Pattern:** Use a global `Resource` for simple counting of general "Secrets" and "Global Paranoia", but if secrets become distinct items, consider spawning them as `Entity` with a `SecretType` component.
- **Seams:** Ensure the `TradeSecretEvent` is caught by the UI layer to provide feedback to the player.
- **Balance:** The increase in paranoia should scale non-linearly to discourage rapid-fire selling of secrets, forcing players to manage the tension.

## 8. Questions
*Builder: add questions here if spec is unclear.*
