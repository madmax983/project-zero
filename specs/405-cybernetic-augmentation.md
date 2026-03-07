# 405: Cybernetic Augmentation

## 1. Overview
The frail human form can only endure so much. "Cybernetic Augmentation" introduces the ability to craft and install mechanical prosthetics on Pops.

Augmentations significantly boost work speed or combat stats but introduce a new logistical overhead: "Maintenance". Cyborgs consume Energy or require dedicated repair jobs. Furthermore, extreme augmentation triggers the "Ship of Theseus" effect, drastically lowering the Pop's "Social" stats as they become alienated from unaugmented colonists.

## 2. Dependencies
- `004-pop-entity.md` (Base Pop entity)
- `051-pop-skills-xp.md` (To provide bonuses to skills)
- `038-medical-care.md` (For the surgical application logic)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::PopSkills;

    #[test]
    fn test_augmentation_applies_stat_boosts() {
        let mut app = App::new();
        app.add_systems(Update, apply_cybernetic_modifiers_system);

        let pop_ent = app.world_mut().spawn((
            Pop,
            PopSkills { work_speed: 1.0, social_affinity: 1.0, ..Default::default() },
            CyberneticImplants {
                implants: vec![
                    ImplantType::MotorServo, // Boosts speed
                    ImplantType::NeuralLink // Lowers social
                ]
            }
        )).id();

        app.update();

        let skills = app.world().get::<PopSkills>(pop_ent).unwrap();
        // Base is 1.0, MotorServo adds 0.5
        assert_eq!(skills.work_speed, 1.5, "MotorServo should boost work speed");
        // Base is 1.0, NeuralLink subtracts 0.3
        assert_eq!(skills.social_affinity, 0.7, "NeuralLink should reduce social affinity");
    }

    #[test]
    fn test_augmentation_requires_maintenance() {
        let mut app = App::new();
        app.add_systems(Update, cybernetic_maintenance_system);

        let pop_ent = app.world_mut().spawn((
            Pop,
            CyberneticImplants { implants: vec![ImplantType::MotorServo] },
            CyberneticMaintenance { current_charge: 100.0, max_charge: 100.0 },
        )).id();

        app.update(); // Tick

        let maint = app.world().get::<CyberneticMaintenance>(pop_ent).unwrap();
        assert!(maint.current_charge < 100.0, "Implants should drain maintenance charge over time");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::skills::PopSkills;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplantType {
    MotorServo,
    NeuralLink,
    SubdermalArmor,
}

#[derive(Component)]
pub struct CyberneticImplants {
    pub implants: Vec<ImplantType>,
}

#[derive(Component)]
pub struct CyberneticMaintenance {
    pub current_charge: f32,
    pub max_charge: f32,
}

pub fn apply_cybernetic_modifiers_system(
    mut query: Query<(&CyberneticImplants, &mut PopSkills), Changed<CyberneticImplants>>,
) {
    for (cyber, mut skills) in query.iter_mut() {
        // Reset to base before applying to avoid compounding on repeated ticks
        // Assuming base is 1.0 for MVP
        skills.work_speed = 1.0;
        skills.social_affinity = 1.0;

        for implant in &cyber.implants {
            match implant {
                ImplantType::MotorServo => skills.work_speed += 0.5,
                ImplantType::NeuralLink => skills.social_affinity -= 0.3,
                ImplantType::SubdermalArmor => {} // Affects health/combat elsewhere
            }
        }
    }
}

pub fn cybernetic_maintenance_system(
    mut query: Query<(&CyberneticImplants, &mut CyberneticMaintenance)>,
) {
    for (cyber, mut maint) in query.iter_mut() {
        let drain_rate = cyber.implants.len() as f32 * 0.1;
        maint.current_charge = (maint.current_charge - drain_rate).max(0.0);

        // If charge hits 0, there should be consequences (e.g. disabling the stat boosts or causing damage)
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Base Stats vs Modifiers:** Mutating `PopSkills` directly and resetting it to a hardcoded `1.0` is fragile. Implement a proper `StatModifier` system where base skills and temporary/permanent modifiers are tracked separately.
- **Maintenance Needs:** Integrate `CyberneticMaintenance` into the Utility AI so Pops will actively seek out a "Recharge Station" or consume "Energy Cells" from their inventory when low.
- **Surgical Operation:** Add a `CraftingRecipe` for the implants and a medical Utility AI job for the installation surgery.

## 6. Acceptance Criteria (Testable!)
- [ ] `CyberneticImplants` component applies specific buffs and debuffs to `PopSkills`.
- [ ] Implants drain `CyberneticMaintenance` charge over time.
- [ ] System properly iterates over all applied implants on a Pop.
- [ ] Tests pass successfully.

## 7. Technical Guidance
- For the `Changed<CyberneticImplants>` query to work accurately without resetting unrelated skill XP, consider having `PopSkills` store a `base` value and a `current` value.

## 8. Questions
*Builder: add questions here if spec is unclear.*
