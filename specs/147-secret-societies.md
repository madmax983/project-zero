# 147: Secret Societies

**Layer:** 1 (Colony Simulation)
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

**Fantasy:** The colony has a life you don't control. Cults, unions, and clubs form in the shadows.

**Mechanic:**
- Pops with shared **Traits** (e.g., `Pyromaniac`, `Traditionalist`) or low **Mood** form hidden **Secret Societies**.
- Societies have **Secrecy** (0.0-1.0) and **Power** (0.0-1.0).
- Members meet secretly to increase Power.
- High Power triggers **Society Events** (Sabotage, Rituals, Theft).
- The **Sheriff** (Justice System) can investigate to lower Secrecy/Power or reveal members.
- Player can **Suppress** (arrest leader) or **Tolerate** (accept demands).

**Why:** Adds social depth and internal conflict. Utilizes existing Trait/Mood systems. Gives the Justice System a proactive role.

---

## 2. Dependencies

- `004` Pop Entity (Pops exist)
- `084` Pop Traits (Basis for membership)
- `050` Civil Unrest (Consequence of suppression)
- `072` Justice System (Investigation mechanic)

---

## 3. RED Phase: Tests First

These tests define the behavior. They should fail until the Green phase is implemented.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Traits, Trait};
    use crate::layer1::society::{SecretSociety, SocietyMember, SocietyPower, Secrecy, form_societies_system, society_meeting_system};

    #[test]
    fn test_society_formation_based_on_traits() {
        let mut world = World::new();
        world.init_resource::<SecretSocieties>();

        // Spawn Pops with Pyromaniac trait
        for _ in 0..3 {
            let mut traits = Traits::default();
            traits.add(Trait::Pyromaniac);
            world.spawn((Pop, traits));
        }

        // Run formation system
        let mut schedule = Schedule::default();
        schedule.add_systems(form_societies_system);
        schedule.run(&mut world);

        // Check if "Order of the Flame" society exists
        let societies = world.resource::<SecretSocieties>();
        assert!(societies.has_society("Order of the Flame"));
        assert_eq!(societies.get_member_count("Order of the Flame"), 3);
    }

    #[test]
    fn test_society_power_increase_on_meeting() {
        let mut world = World::new();
        let mut societies = SecretSocieties::default();
        societies.add_society("Cult of the Machine", 0.1, 0.8); // Low power, high secrecy
        world.insert_resource(societies);

        // Run meeting system
        let mut schedule = Schedule::default();
        schedule.add_systems(society_meeting_system);
        schedule.run(&mut world);

        let societies = world.resource::<SecretSocieties>();
        let cult = societies.get("Cult of the Machine").unwrap();
        assert!(cult.power > 0.1); // Power increased
    }

    #[test]
    fn test_investigation_reveals_members() {
        let mut world = World::new();
        let mut societies = SecretSocieties::default();
        societies.add_society("Thieves Guild", 0.5, 0.5);
        world.insert_resource(societies);

        let pop = world.spawn((
            Pop,
            SocietyMember { society_id: "Thieves Guild".to_string(), known: false },
        )).id();

        // Simulate Sheriff investigation success
        world.send_event(InvestigationEvent { target: pop, success: true });

        // Run investigation handler system (assumed existing or new)
        // ... (implementation of handler system call)

        let member = world.get::<SocietyMember>(pop).unwrap();
        assert!(member.known); // Member is now revealed
    }

    #[test]
    fn test_suppression_causes_unrest() {
        let mut world = World::new();
        world.init_resource::<Unrest>();
        let mut societies = SecretSocieties::default();
        societies.add_society("Rebels", 0.8, 0.2); // High power
        world.insert_resource(societies);

        // Player suppresses the society
        world.send_event(SuppressSocietyEvent { society_id: "Rebels".to_string() });

        // Run suppression system
        // ... (system call)

        let unrest = world.resource::<Unrest>();
        assert!(unrest.level > 0.0); // Unrest increased
        let societies = world.resource::<SecretSocieties>();
        let rebels = societies.get("Rebels").unwrap();
        assert!(rebels.power < 0.8); // Power decreased
    }
}
```

---

## 4. GREEN Phase: Minimal Implementation

### 1. Data Structures

```rust
// src/layer1/society.rs

use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SocietyData {
    pub name: String,
    pub power: f32,   // 0.0 to 1.0
    pub secrecy: f32, // 0.0 to 1.0 (1.0 = completely hidden)
    pub members: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct SecretSocieties {
    pub map: HashMap<String, SocietyData>,
}

impl SecretSocieties {
    pub fn add_society(&mut self, name: &str, power: f32, secrecy: f32) {
        self.map.insert(name.to_string(), SocietyData {
            name: name.to_string(),
            power,
            secrecy,
            members: Vec::new(),
        });
    }

    pub fn get(&self, name: &str) -> Option<&SocietyData> {
        self.map.get(name)
    }

    pub fn has_society(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub fn get_member_count(&self, name: &str) -> usize {
        self.map.get(name).map_or(0, |s| s.members.len())
    }
}

#[derive(Component, Debug, Clone)]
pub struct SocietyMember {
    pub society_id: String,
    pub known: bool, // Visible to player?
}
```

### 2. Formation System

```rust
pub fn form_societies_system(
    mut commands: Commands,
    mut societies: ResMut<SecretSocieties>,
    query: Query<(Entity, &Traits), Without<SocietyMember>>,
) {
    // Define trait mappings
    let pyro_trait = Trait::Pyromaniac;
    let machine_trait = Trait::Technophobe; // Or Technophile if it exists

    for (entity, traits) in &query {
        if traits.has(pyro_trait) {
            let name = "Order of the Flame";
            if !societies.has_society(name) {
                societies.add_society(name, 0.1, 0.9);
            }
            societies.map.get_mut(name).unwrap().members.push(entity);
            commands.entity(entity).insert(SocietyMember {
                society_id: name.to_string(),
                known: false,
            });
        }
        // Add more mappings
    }
}
```

### 3. Meeting System

```rust
pub fn society_meeting_system(
    mut societies: ResMut<SecretSocieties>,
    // Optional: Query for time/night cycle
) {
    for society in societies.map.values_mut() {
        // Simple accumulation for MVP
        // In reality, this should depend on members actually meeting
        if society.members.len() > 0 {
            society.power = (society.power + 0.001 * society.members.len() as f32).min(1.0);
        }
    }
}
```

---

## 5. REFACTOR Phase: Quality & Design

### Refactoring Opportunities

1.  **Event Integration:**
    -   When `SocietyPower` hits thresholds (0.5, 0.8, 1.0), trigger `SocietyEvent` (e.g., "Arson" for Order of the Flame).
    -   Use the `Chronicle` system to log these events if they are "Public".

2.  **Meeting Logic:**
    -   Instead of passive gain, make members actually pathfind to a hidden location at night.
    -   Sheriffs patrolling nearby can spot them, reducing Secrecy.

3.  **UI:**
    -   "Intelligence Report" UI panel to show known Societies and their estimated power.
    -   "Investigate" button on Pops to assign Sheriff priority.

### API Improvements

-   `SocietyTrait` enum instead of string IDs for type safety.
-   `SocietyGoal` (e.g., `Sabotage`, `Worship`, `Theft`) to drive AI behavior.

---

## 6. Acceptance Criteria

- [ ] `SecretSocieties` resource exists and tracks power/secrecy.
- [ ] Pops with specific traits form/join societies automatically.
- [ ] Society Power increases over time (representing meetings).
- [ ] Investigation reveals `SocietyMember` status.
- [ ] Suppression affects Power and Unrest.
- [ ] Tests pass.

---

## 7. Technical Guidance

-   **Traits:** Use `src/layer1/traits.rs`.
-   **Unrest:** Use `src/layer1/unrest.rs` (if exists) or create basic resource.
-   **Sheriff:** Hook into `src/layer1/justice.rs`. Sheriff jobs should have a `detect_chance` based on their `Observant` trait or skill.

---

## 8. Questions

-   *Builder: Should societies compete with each other?*
*Architect:* Not directly in V1. They simply compete for Pop membership. If a Pop is drawn to multiple societies, they choose the one matching their highest trait affinity.
    *Architect: Yes, opposing secret societies should generate negative social standing and conflicts between members.*
*Architect: Yes, members of rival societies should experience friction (negative relationship modifiers) and potentially sabotage each other.*
    -   *Architect:* Yes, but for MVP, they are independent.
-   *Builder: Can a Pop belong to multiple societies?*
*Architect:* No, society membership is mutually exclusive to ensure distinct factional blocks form within the colony.
    *Architect: For MVP, limit Pops to one secret society at a time to keep membership tracking simple.*
*Architect: No, exclusivity forces harder choices and cleaner faction boundaries.*
    -   *Architect:* No. One secret allegiance per Pop for simplicity.
