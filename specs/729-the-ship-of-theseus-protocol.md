# Spec 729: The "Ship of Theseus" Protocol

## 1. Overview
**Layer:** 1
**Fantasy:** At what point do you stop being human?
**Mechanic:** Pops with >50% Cybernetic replacements (Limbs, Organs) gain the "Cyborg" trait. They stop consuming Food and start consuming Power (recharging). They gain immunity to disease but vulnerability to EMP.
**Emergence:** A famine strikes. Your biological pops starve. Your cyborgs are fine... until the power plant fails. Then the cyborgs die while the biologicals survive on raw moss.
**Tension:** Biological resilience vs. Mechanical efficiency.

## 2. Dependencies
- Base simulation framework
- Cybernetic Augmentation (`151`)
- Pop Needs (`005`)
- Energy System (`042`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pops::{Pop, Cybernetics, Traits, NeedType};

    #[test]
    fn test_cyborg_trait_granted_at_threshold() {
        // Arrange
        let mut pop = Pop::new();
        let mut cybernetics = Cybernetics::new();

        // Act
        // Add 50%+ cybernetic parts
        cybernetics.add_part(CyberneticPart::Arm);
        cybernetics.add_part(CyberneticPart::Leg);
        cybernetics.add_part(CyberneticPart::Eye);
        cybernetics.add_part(CyberneticPart::Heart); // Total 4/6 major parts = >50%

        update_cyborg_status(&mut pop, &cybernetics);

        // Assert
        assert!(pop.has_trait(Traits::Cyborg));
    }

    #[test]
    fn test_cyborg_need_swap() {
        // Arrange
        let mut pop = Pop::new();
        pop.add_trait(Traits::Cyborg);

        // Act
        let needs = get_pop_needs(&pop);

        // Assert
        assert!(!needs.contains(&NeedType::Food));
        assert!(needs.contains(&NeedType::Power));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/pops.rs
pub enum CyberneticPart {
    Arm, Leg, Eye, Heart, Lung, Spine
}

pub struct Cybernetics {
    pub parts: Vec<CyberneticPart>,
}

impl Cybernetics {
    pub fn new() -> Self { Self { parts: Vec::new() } }
    pub fn add_part(&mut self, part: CyberneticPart) { self.parts.push(part); }
    pub fn percentage(&self) -> f32 {
        self.parts.len() as f32 / 6.0 // Assuming 6 total major parts
    }
}

pub fn update_cyborg_status(pop: &mut Pop, cybernetics: &Cybernetics) {
    if cybernetics.percentage() > 0.5 {
        pop.add_trait(Traits::Cyborg);
    }
}

pub fn get_pop_needs(pop: &Pop) -> Vec<NeedType> {
    if pop.has_trait(Traits::Cyborg) {
        vec![NeedType::Power, NeedType::Rest] // Example
    } else {
        vec![NeedType::Food, NeedType::Rest] // Example
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Use ECS correctly, e.g., `Query<(&Cybernetics, &mut Traits), Changed<Cybernetics>>`.
- Instead of hardcoding 6 major parts, track total replacement value or slots.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code

## 7. Technical Guidance
- Integrate into the `needs_system` so that `Traits::Cyborg` changes which resource is queried when fulfilling the hunger/power need.
- Integrate into medical systems so applying augments triggers the threshold check.

## 8. Questions
*Builder: add questions here if spec is unclear.*
