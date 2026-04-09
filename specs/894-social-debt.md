# Social Debt

## 1. Overview
The colony operates on an informal economy of favors: "You owe me one." If a pop performs a life-saving or highly significant action for another (like healing them in medical or saving them in combat), a "Social Debt" is created. Pops will support the factions or requests of those they owe, even if it contradicts their own ethics or preferred faction. This introduces deep social simulation where unpopular leaders might stay in power solely because they hold the debts of key individuals.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- Entities representing Pops
- Events representing significant acts (e.g., `LifeSavedEvent`)
- Faction/Ethics preference system (so we can see support overridden)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_life_saving_act_creates_social_debt() {
    // Arrange
    let mut app = App::new();
    app.add_event::<LifeSavedEvent>();
    app.add_systems(Update, process_life_saved_system);

    let savior = app.world_mut().spawn_empty().id();
    let saved = app.world_mut().spawn_empty().id();

    // Act
    app.world_mut().send_event(LifeSavedEvent {
        savior,
        saved,
    });
    app.update();

    // Assert: The 'saved' pop should now have a SocialDebt component tracking 'savior'
    let debt = app.world().get::<SocialDebts>(saved).unwrap();
    assert_eq!(debt.owed_to.get(&savior), Some(&1));
}

#[test]
fn test_debt_overrides_faction_support() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, evaluate_faction_support_system);

    let savior = app.world_mut().spawn(FactionMember { faction_id: 1 }).id();

    let mut debts = SocialDebts::default();
    debts.owed_to.insert(savior, 1);

    let saved = app.world_mut().spawn((
        FactionMember { faction_id: 2 }, // Naturally prefers faction 2
        debts,
        ActiveSupport { supported_faction: 2 }, // Currently supporting their own
    )).id();

    // Act
    app.update();

    // Assert: The saved pop should now support faction 1 because they owe the savior
    let support = app.world().get::<ActiveSupport>(saved).unwrap();
    assert_eq!(support.supported_faction, 1);
}

#[test]
fn test_debt_consumed_when_called_in() {
    // Arrange
    let mut app = App::new();
    app.add_event::<CallInFavorEvent>();
    app.add_systems(Update, process_favors_system);

    let savior = app.world_mut().spawn_empty().id();

    let mut debts = SocialDebts::default();
    debts.owed_to.insert(savior, 1);
    let saved = app.world_mut().spawn(debts).id();

    // Act
    app.world_mut().send_event(CallInFavorEvent {
        caller: savior,
        target: saved,
    });
    app.update();

    // Assert: The debt is consumed
    let debt = app.world().get::<SocialDebts>(saved).unwrap();
    assert_eq!(debt.owed_to.get(&savior).copied().unwrap_or(0), 0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Event)]
pub struct LifeSavedEvent {
    pub savior: Entity,
    pub saved: Entity,
}

#[derive(Event)]
pub struct CallInFavorEvent {
    pub caller: Entity,
    pub target: Entity,
}

#[derive(Component, Default)]
pub struct SocialDebts {
    pub owed_to: HashMap<Entity, u32>,
}

#[derive(Component)]
pub struct FactionMember {
    pub faction_id: u32,
}

#[derive(Component)]
pub struct ActiveSupport {
    pub supported_faction: u32,
}

pub fn process_life_saved_system(
    mut events: EventReader<LifeSavedEvent>,
    mut query: Query<&mut SocialDebts>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut debts) = query.get_mut(event.saved) {
            *debts.owed_to.entry(event.savior).or_insert(0) += 1;
        } else {
            let mut debts = SocialDebts::default();
            debts.owed_to.insert(event.savior, 1);
            commands.entity(event.saved).insert(debts);
        }
    }
}

pub fn evaluate_faction_support_system(
    mut supporters: Query<(&FactionMember, &SocialDebts, &mut ActiveSupport)>,
    faction_members: Query<&FactionMember, Without<SocialDebts>>,
) {
    for (own_faction, debts, mut support) in supporters.iter_mut() {
        let mut overridden = false;

        // Find if they owe someone in a different faction
        for (creditor, amount) in debts.owed_to.iter() {
            if *amount > 0 {
                if let Ok(creditor_faction) = faction_members.get(*creditor) {
                    support.supported_faction = creditor_faction.faction_id;
                    overridden = true;
                    break;
                }
            }
        }

        // Default back to own faction if no active debts force otherwise
        if !overridden {
            support.supported_faction = own_faction.faction_id;
        }
    }
}

pub fn process_favors_system(
    mut events: EventReader<CallInFavorEvent>,
    mut debtors: Query<&mut SocialDebts>,
) {
    for event in events.read() {
        if let Ok(mut debts) = debtors.get_mut(event.target) {
            if let Some(amount) = debts.owed_to.get_mut(&event.caller) {
                if *amount > 0 {
                    *amount -= 1;
                    // Trigger actual favor action here in future
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **HashMap Usage**: Used `bevy::utils::HashMap` (AHash) instead of `std::collections::HashMap` for performance, as recommended by `bevy` for component data.
- **Query Optimization**: `evaluate_faction_support_system` currently uses `.get()` inside a loop. If social debts grow large, resolving faction IDs might become expensive. Consider tracking the creditor's faction ID directly in the debt entry, or caching it.
- **Dead Entities**: Ensure `SocialDebts` cleans up keys when the `savior` entity is despawned (e.g., they die), otherwise it's a memory leak of dead entity IDs.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for new code is >= 85%.
- [ ] Dead entity cleanup logic is tested and implemented.

## 7. Technical Guidance
- Bevy's `Entity` is safe to use as a hash map key.
- To handle despawned entities, you can add a system that iterates over `SocialDebts` and removes keys if `commands.get_entity(*key)` returns `None`, or listen for `PopDeathEvent`s.

## 8. Questions
*Builder: Add questions here regarding how "Calling in a favor" visually presents to the player, or if it's purely an under-the-hood mechanic.*
