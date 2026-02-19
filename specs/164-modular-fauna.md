# 164: Modular Fauna

## Overview

Transition from hardcoded `FaunaType` enums (e.g., `Wolf`, `SpaceRat`) to a procedural **Modular Fauna** system. Animals are composed of distinct parts (Head, Body, Limbs, Tail, Integument) that determine their stats, behaviors, and resource drops.

This allows for the procedural generation of alien wildlife ("Wolf-Headed Crab with Scales") and lays the foundation for `163` Vermin Evolution and `165` Gene-Banks.

## Dependencies

- `048` — Hostile Fauna (Base AI and Entity structure)
- `075` — Animal Husbandry (Taming/Harvesting logic)

## RED Phase: Tests First

Write these tests in `src/layer1/fauna_modular_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::fauna::{Fauna, FaunaBody, FaunaPart, BodyPartType, FaunaStats};
    use crate::layer1::fauna_gen::{FaunaGenerator, FaunaSeed};

    #[test]
    fn test_fauna_body_generation_from_seed() {
        let seed = FaunaSeed::new(12345);
        let body = FaunaGenerator::generate(seed);

        assert!(body.parts.contains_key(&BodyPartType::Head));
        assert!(body.parts.contains_key(&BodyPartType::Body));
        assert!(body.parts.contains_key(&BodyPartType::Limbs));

        // Deterministic check
        let seed2 = FaunaSeed::new(12345);
        let body2 = FaunaGenerator::generate(seed2);
        assert_eq!(body.name, body2.name);
    }

    #[test]
    fn test_fauna_stats_aggregation() {
        // Create a body with specific parts
        let mut body = FaunaBody::default();

        // Head: Sharp Teeth (+5 Damage)
        body.add_part(FaunaPart {
            part_type: BodyPartType::Head,
            name: "Wolf Head".to_string(),
            stats: FaunaStats { attack: 5.0, ..Default::default() }
        });

        // Body: Thick Hide (+10 Health)
        body.add_part(FaunaPart {
            part_type: BodyPartType::Body,
            name: "Bear Torso".to_string(),
            stats: FaunaStats { health_max: 10.0, ..Default::default() }
        });

        let stats = body.aggregate_stats();

        assert_eq!(stats.attack, 5.0);
        assert_eq!(stats.health_max, 10.0);
    }

    #[test]
    fn test_legacy_fauna_type_mapping() {
        // Ensure old enum still works or converts
        let wolf_seed = FaunaSeed::from_preset("Wolf");
        let body = FaunaGenerator::generate(wolf_seed);

        let stats = body.aggregate_stats();
        assert!(stats.attack > 0.0);
        assert!(body.name.contains("Wolf"));
    }

    #[test]
    fn test_husbandry_resource_check() {
        // Cow-like creature should produce Milk
        let mut body = FaunaBody::default();
        body.add_part(FaunaPart {
            part_type: BodyPartType::Body,
            name: "Udder-Body".to_string(),
            resource_drop: Some("Milk".to_string()),
            ..Default::default()
        });

        assert!(body.can_produce("Milk"));
        assert!(!body.can_produce("Wool"));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Data Structures

```rust
// src/layer1/fauna/modular.rs

use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum BodyPartType {
    Head,
    Body,
    Limbs,
    Tail,
    Integument, // Skin/Fur/Scales
}

#[derive(Debug, Clone, Default)]
pub struct FaunaStats {
    pub health_max: f32,
    pub attack: f32,
    pub speed: f32,
    pub defense: f32,
}

impl std::ops::Add for FaunaStats {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            health_max: self.health_max + other.health_max,
            attack: self.attack + other.attack,
            speed: self.speed + other.speed,
            defense: self.defense + other.defense,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FaunaPart {
    pub part_type: BodyPartType,
    pub name: String,
    pub stats: FaunaStats,
    pub resource_drop: Option<String>, // e.g., "Milk", "Venom"
}

#[derive(Component, Debug, Default, Clone)]
pub struct FaunaBody {
    pub name: String,
    pub parts: HashMap<BodyPartType, FaunaPart>,
}

impl FaunaBody {
    pub fn add_part(&mut self, part: FaunaPart) {
        self.parts.insert(part.part_type, part);
    }

    pub fn aggregate_stats(&self) -> FaunaStats {
        self.parts.values().fold(FaunaStats::default(), |acc, part| acc + part.stats.clone())
    }

    pub fn can_produce(&self, resource: &str) -> bool {
        self.parts.values().any(|p| p.resource_drop.as_deref() == Some(resource))
    }
}
```

### 2. Implement Generator

```rust
// src/layer1/fauna_gen.rs

use super::modular::{FaunaBody, FaunaPart, BodyPartType, FaunaStats};
use rand::{Rng, SeedableRng, rngs::StdRng};

pub struct FaunaSeed {
    pub value: u64,
    pub preset: Option<String>,
}

impl FaunaSeed {
    pub fn new(value: u64) -> Self { Self { value, preset: None } }
    pub fn from_preset(name: &str) -> Self {
        Self { value: 0, preset: Some(name.to_string()) }
    }
}

pub struct FaunaGenerator;

impl FaunaGenerator {
    pub fn generate(seed: FaunaSeed) -> FaunaBody {
        if let Some(preset) = seed.preset {
            return Self::generate_preset(&preset);
        }

        let mut rng = StdRng::seed_from_u64(seed.value);
        let mut body = FaunaBody::default();

        // Procedural generation logic
        // Pick random head
        let head = FaunaPart {
            part_type: BodyPartType::Head,
            name: "Wolf Head".to_string(), // Simplified for Green
            stats: FaunaStats { attack: 5.0, ..Default::default() },
            resource_drop: None,
        };
        body.add_part(head);

        // Pick random body
        let torso = FaunaPart {
            part_type: BodyPartType::Body,
            name: "Bear Torso".to_string(),
            stats: FaunaStats { health_max: 20.0, ..Default::default() },
            resource_drop: Some("Meat".to_string()),
        };
        body.add_part(torso);

        // Pick limbs
        let limbs = FaunaPart {
            part_type: BodyPartType::Limbs,
            name: "Insect Legs".to_string(),
            stats: FaunaStats { speed: 1.2, ..Default::default() },
            resource_drop: None,
        };
        body.add_part(limbs);

        body.name = format!("Generated Beast {}", seed.value % 100);
        body
    }

    fn generate_preset(name: &str) -> FaunaBody {
        let mut body = FaunaBody::default();
        if name == "Wolf" {
            body.name = "Grey Wolf".to_string();
            // Add wolf parts...
             body.add_part(FaunaPart {
                part_type: BodyPartType::Head,
                name: "Wolf Head".to_string(),
                stats: FaunaStats { attack: 8.0, ..Default::default() },
                resource_drop: None,
            });
            // ...
        }
        body
    }
}
```

## REFACTOR Phase: Quality & Design

- **Integration**: Update `fauna_behavior_system` to read `FaunaBody` stats instead of static lookup.
- **Migration**: Deprecate `FaunaType` enum or make it a wrapper around `FaunaSeed::from_preset`.
- **Rendering**: Procedural rendering is hard. For now, map `Head` part to a glyph ('w' for Wolf Head, 'c' for Crab Head) or use the `Body` color.
- **Descriptions**: Generate description strings: "A [Integument] [Head]-headed beast with [Limbs]." -> "A Furry Wolf-headed beast with Insect Legs."

## Acceptance Criteria

- [ ] `FaunaBody` component exists.
- [ ] Random generation produces valid bodies with stats.
- [ ] Presets ("Wolf", "Rat") generate equivalent stats to legacy system.
- [ ] `Husbandry` system correctly identifies produceable resources from parts.
- [ ] Existing `048` behaviors work with the new `FaunaBody` stats.

## Technical Guidance

- Use `bevy_ecs` Component for `FaunaBody`.
- Don't over-engineer the "Parts Database" yet. Hardcoded lists in `FaunaGenerator` are fine for now.
- Keep `Fauna` component for state (Wander/Chase) but remove `attack_damage` field, redirecting it to `FaunaBody.stats().attack`.

## Questions

- Should we store DNA strings? (Yes, for Spec 165 Gene-Banks, `FaunaSeed` value IS the DNA).
