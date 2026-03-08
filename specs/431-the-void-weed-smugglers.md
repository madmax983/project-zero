# Spec 431: The Void-Weed Smugglers

## 1. Overview
A harmless, slightly relaxing native plant ("Void-Weed") is discovered. Pops cultivate it privately for a tiny mood boost. However, passing merchants discover it's highly addictive to alien species. They start paying exorbitant Credits for it, turning the colony into an accidental cartel.

## 2. Dependencies
- `039-trade-system`
- `044-horticulture-beauty`
- `203-prohibition-contraband`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_void_weed_smuggling_profits() {
        let mut app = App::new();
        app.insert_resource(TradeNetwork { credits: 0.0 });
        app.add_systems(Update, process_void_weed_trade_system);

        let entity = app.world_mut().spawn((
            VoidWeedStash { amount: 10.0 },
        )).id();

        app.world_mut().send_event(MerchantArrivalEvent {
            merchant_type: MerchantType::Smuggler,
        });

        app.update();

        let network = app.world().resource::<TradeNetwork>();
        assert_eq!(network.credits, 5000.0); // 10.0 * 500.0 credits
        let stash = app.world().get::<VoidWeedStash>(entity).unwrap();
        assert_eq!(stash.amount, 0.0);
    }

    #[test]
    fn test_void_weed_smuggling_attracts_pirates() {
        let mut app = App::new();
        app.insert_resource(SmugglingHeat { level: 0.0 });
        app.add_event::<PirateRaidEvent>();
        app.add_systems(Update, evaluate_smuggling_heat_system);

        app.world_mut().insert_resource(SmugglingHeat { level: 100.0 });

        app.update();

        let raid_events = app.world().resource::<Events<PirateRaidEvent>>();
        let mut reader = raid_events.get_reader();
        let events: Vec<_> = reader.read(raid_events).collect();

        assert_eq!(events.len(), 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct VoidWeedStash {
    pub amount: f32,
}

#[derive(Resource)]
pub struct TradeNetwork {
    pub credits: f32,
}

#[derive(Resource)]
pub struct SmugglingHeat {
    pub level: f32,
}

#[derive(Event)]
pub struct MerchantArrivalEvent {
    pub merchant_type: MerchantType,
}

#[derive(PartialEq)]
pub enum MerchantType {
    Smuggler,
    Legitimate,
}

#[derive(Event)]
pub struct PirateRaidEvent;

pub fn process_void_weed_trade_system(
    mut events: EventReader<MerchantArrivalEvent>,
    mut network: ResMut<TradeNetwork>,
    mut stashes: Query<&mut VoidWeedStash>,
    mut heat: ResMut<SmugglingHeat>,
) {
    for event in events.read() {
        if event.merchant_type == MerchantType::Smuggler {
            for mut stash in stashes.iter_mut() {
                if stash.amount > 0.0 {
                    let profit = stash.amount * 500.0;
                    network.credits += profit;
                    heat.level += stash.amount * 2.0;
                    stash.amount = 0.0;
                }
            }
        }
    }
}

pub fn evaluate_smuggling_heat_system(
    mut heat: ResMut<SmugglingHeat>,
    mut raid_events: EventWriter<PirateRaidEvent>,
) {
    if heat.level >= 100.0 {
        raid_events.send(PirateRaidEvent);
        heat.level = 0.0; // Reset heat after raid
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Connect `SmugglingHeat` to a diplomatic penalty with the Galactic Council.
- Add an `AddictionLevel` component to alien species passing through the trade network.
- Have pops secretly grow Void-Weed in hidden `PrivateStashEntity` containers.

## 6. Acceptance Criteria
- [ ] Smugglers buying Void-Weed generates massive amounts of Credits.
- [ ] Selling Void-Weed increases `SmugglingHeat`.
- [ ] High `SmugglingHeat` triggers a `PirateRaidEvent`.
- [ ] Tests pass with >= 85% coverage.

## 7. Technical Guidance
- Hook into the existing Trade Network to handle merchant arrivals and cargo exchanges.
- Make sure Void-Weed grows organically, perhaps as a weed in standard Hydroponics bays.

## 8. Questions
- Can the player actively crack down on Void-Weed cultivation, or is it an inevitable weed?
- *Architect:* The player can actively crack down by assigning security jobs to raid the stashes, but this creates significant localized unrest.
