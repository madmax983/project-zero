# 156: Xeno-Artifacts

**Layer:** 1 (Colony Simulation)
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

**Fantasy:** Stumbling upon something ancient, powerful, and utterly incomprehensible. The "Monolith" experience.

**Mechanic:**
- **Artifacts** are rare, indestructible entities placed on the map during generation or uncovered by mining.
- They emit **Auras** that apply passive buffs or debuffs to **Pops** and **Buildings** within a specific radius.
- Unlike `Anomalies` (Spec 041), Artifacts are permanent features. You build *around* them to exploit their aura or quarantine them to avoid their curse.

**Examples:**
- **Vitality Crystal:** +Heal Rate, +Hunger (metabolism boost).
- **Humming Monolith:** +Science XP gain, +Stress (mental strain).
- **Glitch Spire:** -Tech Efficiency, +Movement Speed (time dilation).

**Emergence:** Your research lab is built around a "Humming Monolith" to boost science output, but the scientists slowly go insane from the constant psychic hum.

---

## 2. Dependencies

- `002` Terrain Grid (Placement)
- `004` Pop Entity (Target for auras)
- `019` Pop Thoughts & Mood (Target for stress/mood effects)
- `051` Pop Skills & XP (Target for XP buffs)
- `006` Job Assignment (Target for work efficiency)

---

## 3. RED Phase: Tests First

Write these tests in `src/layer1/artifacts_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::artifacts::{Artifact, Aura, AuraEffect, aura_system, ActiveAuras};
    use crate::layer1::stress::Stress; // From Spec 127
    use crate::layer1::skills::SkillType;

    #[test]
    fn test_artifact_aura_application() {
        let mut world = World::new();
        world.init_resource::<crate::shared::time::SimulationTime>();

        // Spawn Artifact at (10, 10) with Radius 5
        world.spawn((
            Artifact,
            Aura {
                radius: 5.0,
                effect: AuraEffect::StressModifier(0.1), // +0.1 Stress/tick
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Spawn Pop at (12, 10) (Distance 2, inside radius)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 12, y: 10 },
            Stress::default(),
            ActiveAuras::default(), // Component to track applied auras
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(aura_system);
        schedule.run(&mut world);

        // Check if Pop has the aura effect applied
        let active_auras = world.get::<ActiveAuras>(pop).unwrap();
        assert!(active_auras.contains_effect(AuraEffect::StressModifier(0.1)));
    }

    #[test]
    fn test_artifact_aura_removal_when_out_of_range() {
        let mut world = World::new();

        // Artifact at (10, 10), Radius 2
        world.spawn((
            Artifact,
            Aura { radius: 2.0, effect: AuraEffect::HealRate(1.5) },
            GridPosition { x: 10, y: 10 },
        ));

        // Pop at (15, 15) (Distance ~7, outside radius)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 15, y: 15 },
            ActiveAuras::default(),
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(aura_system);
        schedule.run(&mut world);

        // Check - should NOT have aura
        let active_auras = world.get::<ActiveAuras>(pop).unwrap();
        assert!(active_auras.is_empty());

        // Move pop inside range
        world.entity_mut(pop).insert(GridPosition { x: 11, y: 10 });
        schedule.run(&mut world);

        let active_auras = world.get::<ActiveAuras>(pop).unwrap();
        assert!(!active_auras.is_empty());

        // Move pop out again
        world.entity_mut(pop).insert(GridPosition { x: 20, y: 20 });
        schedule.run(&mut world);

        let active_auras = world.get::<ActiveAuras>(pop).unwrap();
        assert!(active_auras.is_empty(), "Aura should be removed when leaving range");
    }
}
```

---

## 4. GREEN Phase: Minimal Implementation

### 1. Components

In `src/layer1/artifacts.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;

#[derive(Component, Default)]
pub struct Artifact;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuraEffect {
    StressModifier(f32),
    HealRate(f32),
    SkillXpBoost(crate::layer1::skills::SkillType, f32),
    WorkSpeed(f32),
}

#[derive(Component)]
pub struct Aura {
    pub radius: f32,
    pub effect: AuraEffect,
}

#[derive(Component, Default)]
pub struct ActiveAuras {
    pub effects: Vec<AuraEffect>,
}

impl ActiveAuras {
    pub fn contains_effect(&self, effect: AuraEffect) -> bool {
        self.effects.contains(&effect)
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}
```

### 2. System

```rust
pub fn aura_system(
    artifacts: Query<(&GridPosition, &Aura), With<Artifact>>,
    mut targets: Query<(&GridPosition, &mut ActiveAuras), With<crate::layer1::pop::Pop>>,
) {
    for (pos, mut active_auras) in &mut targets {
        // Clear previous frame's auras (re-calculate each tick for simplicity in MVP)
        // Optimization: In REFACTOR, use differential updates or dirty flags.
        active_auras.effects.clear();

        for (art_pos, aura) in &artifacts {
            let dist_sq = ((pos.x - art_pos.x).pow(2) + (pos.y - art_pos.y).pow(2)) as f32;
            if dist_sq <= aura.radius.powi(2) {
                active_auras.effects.push(aura.effect);
            }
        }
    }
}
```

---

## 5. REFACTOR Phase: Quality & Design

### Refactoring Opportunities

1.  **Spatial Hashing:**
    -   Iterating all Pops x all Artifacts is O(N*M). If N (Pops) is 1000 and M (Artifacts) is 10, it's 10,000 checks. Cheap enough for MVP.
    -   If M grows, use a Spatial Hash or check only "Chunks" near Pops.

2.  **Effect Stacking:**
    -   Currently, multiple artifacts of the same type would stack their effects (pushing to Vec).
    -   Need logic to cap stacks or diminish returns (e.g., max 3.0x Heal Rate).

3.  **Visualization:**
    -   Add a `DrawAura` component to render a faint circle overlay on the map.
    -   Updates `src/ui/map.rs` to render auras when "Inspect Mode" is active.

4.  **Integration Points:**
    -   **Stress System:** Update `stress_system` to read `ActiveAuras` and apply modifiers.
    -   **Health System:** Update `healing_system` to read `HealRate` from `ActiveAuras`.
    -   **Skills System:** Update `xp_gain_system`.

### API Improvements

-   Move `AuraEffect` to a shared `buffs` module if reused by Tech/Edicts.
-   Add `AuraSource` enum to `ActiveAuras` to track *what* is causing the buff (for UI tooltips).

---

## 6. Acceptance Criteria

- [ ] `Artifact` and `Aura` components defined.
- [ ] `aura_system` correctly identifies Pops within radius.
- [ ] `ActiveAuras` component on Pops updates dynamically based on position.
- [ ] Integration with at least one existing system (e.g., Stress or Health) via test verification.
- [ ] Tests pass.

---

## 7. Technical Guidance

-   **Persistence:** Ensure `ActiveAuras` is serialized if using Serde, or marked `#[serde(skip)]` and re-calculated on load.
-   **Indestructibility:** Update `mining_system` or `demolish_system` to prevent targeting entities with `Artifact` component.
-   **Z-Levels:** Artifacts currently only affect the same Z-level (implicit in 2D GridPosition). If Z-levels are added, update distance check.

---

## 8. Questions

-   *Builder: Do artifacts block movement?*
*Architect: Yes, artifacts act as solid obstacles similar to walls or large machines.*
    -   *Architect:* Yes, they occupy the tile like a Wall, unless specified otherwise (e.g. `Passable`).
-   *Builder: Can we move artifacts?*
*Architect: No, artifacts are permanent fixtures of the map. You must build around them.*
    -   *Architect:* No. That's the strategic constraint. You must build around them.
