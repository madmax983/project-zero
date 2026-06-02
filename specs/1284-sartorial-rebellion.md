# 1284: Sartorial Rebellion

## 1. Overview
**Layer:** 1

**Fantasy:** The uniform is the first casualty of war.

**Mechanic:** Factions develop "Visual Signifiers" (e.g., rolled sleeves, backwards caps, specific colors). Pops modify their appearance to signal allegiance. Banning these mods increases Unrest but restores "Order".

## 2. Dependencies
- Base Layer 1 Population System
- Faction/Social System
- Unrest System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::faction::Faction;
    use crate::layer1::social::unrest::Unrest;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            adopt_visual_signifier_system,
            enforce_dress_code_system,
        ));
        app
    }

    #[test]
    fn test_pop_adopts_faction_signifier() {
        let mut app = setup_app();

        // Spawn a faction with a visual signifier
        let faction = app.world_mut().spawn((
            Faction { name: "The Rust Towners".to_string() },
            VisualSignifier { description: "Red Bandana".to_string() },
        )).id();

        // Spawn a pop belonging to that faction, without the signifier initially
        let pop = app.world_mut().spawn((
            Pop,
            FactionMember { faction_entity: faction },
            Appearance { signifiers: vec![] },
        )).id();

        app.update();

        // The pop should have adopted the signifier
        let appearance = app.world().get::<Appearance>(pop).unwrap();
        assert!(appearance.signifiers.contains(&"Red Bandana".to_string()));
    }

    #[test]
    fn test_banning_signifier_increases_unrest() {
        let mut app = setup_app();

        // Insert global dress code policy
        app.insert_resource(DressCodePolicy { banned_signifiers: vec!["Red Bandana".to_string()] });

        let faction = app.world_mut().spawn((
            Faction { name: "The Rust Towners".to_string() },
            VisualSignifier { description: "Red Bandana".to_string() },
        )).id();

        // Spawn a pop with the banned signifier
        let pop = app.world_mut().spawn((
            Pop,
            FactionMember { faction_entity: faction },
            Appearance { signifiers: vec!["Red Bandana".to_string()] },
            Unrest { level: 10.0 },
        )).id();

        app.update();

        // The signifier should be removed, but unrest should increase
        let appearance = app.world().get::<Appearance>(pop).unwrap();
        assert!(!appearance.signifiers.contains(&"Red Bandana".to_string()));

        let unrest = app.world().get::<Unrest>(pop).unwrap();
        assert!(unrest.level > 10.0, "Unrest should increase when signifier is banned");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::social::faction::Faction;
use crate::layer1::social::unrest::Unrest;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct FactionMember {
    pub faction_entity: Entity,
}

#[derive(Component)]
pub struct VisualSignifier {
    pub description: String,
}

#[derive(Component, Default)]
pub struct Appearance {
    pub signifiers: Vec<String>,
}

#[derive(Resource, Default)]
pub struct DressCodePolicy {
    pub banned_signifiers: Vec<String>,
}

pub fn adopt_visual_signifier_system(
    factions: Query<&VisualSignifier, With<Faction>>,
    mut pops: Query<(&FactionMember, &mut Appearance), With<Pop>>,
) {
    for (member, mut appearance) in pops.iter_mut() {
        if let Ok(faction_signifier) = factions.get(member.faction_entity) {
            if !appearance.signifiers.contains(&faction_signifier.description) {
                appearance.signifiers.push(faction_signifier.description.clone());
            }
        }
    }
}

pub fn enforce_dress_code_system(
    policy: Option<Res<DressCodePolicy>>,
    mut pops: Query<(&mut Appearance, &mut Unrest), With<Pop>>,
) {
    if let Some(policy) = policy {
        for (mut appearance, mut unrest) in pops.iter_mut() {
            let mut removed = false;
            appearance.signifiers.retain(|s| {
                if policy.banned_signifiers.contains(s) {
                    removed = true;
                    false
                } else {
                    true
                }
            });

            if removed {
                unrest.level += 5.0; // Penalty for suppressing expression
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Enums over Strings**: Replace `String` descriptions with an enum `SignifierType` to avoid typos and improve performance.
- **Gradual Unrest**: Instead of an instant unrest spike, have the ban add a `SuppressedExpression` trait that slowly generates unrest over time.
- **Enforcement Cost**: Banning a signifier should require `Admin` or `Security` resources, rather than happening instantly and globally for free.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure `Appearance` component correctly hooks into the UI/Rendering systems later so players can see the visual changes.
- Add `DressCodePolicy` to the colony management UI so players can toggle bans on/off.

## 8. Questions
*Builder: add questions here if spec is unclear.*
