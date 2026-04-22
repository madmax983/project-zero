# Specification 1137: Heroic Acts

## 1. Overview
The "Heroic Acts" feature introduces a "Last Stand" mechanic for combat units. When activated, a unit gains massive temporary buffs (e.g., increased fire rate, significant damage reduction) but is guaranteed to die or suffer a permanent debilitating injury at the end of the combat encounter. This adds a layer of emergent narrative and high-stakes tactical decision-making, fulfilling the fantasy of the ultimate sacrifice to save the colony.

## 2. Dependencies
- Base combat system and unit stats.
- Unit health and status effect components.
- Chronicle system for recording the heroic act.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_heroic_act_activation_grants_buffs() {
        let mut world = World::new();

        let entity = world.spawn((
            CombatStats { fire_rate: 1.0, damage_reduction: 0.0 },
            Health { current: 100, max: 100 },
        )).id();

        // Act: Toggle "Last Stand" mode
        activate_heroic_act(&mut world, entity);

        // Assert: Unit should have the HeroicAct buff component
        assert!(world.get::<HeroicActStatus>(entity).is_some());

        let stats = world.get::<CombatStats>(entity).unwrap();
        assert!(stats.fire_rate > 1.0, "Fire rate should be increased");
        assert!(stats.damage_reduction > 0.0, "Damage reduction should be increased");
    }

    #[test]
    fn test_heroic_act_results_in_death_or_injury_after_combat() {
        let mut world = World::new();

        let entity = world.spawn((
            CombatStats { fire_rate: 2.0, damage_reduction: 0.8 },
            Health { current: 100, max: 100 },
            HeroicActStatus { active: true },
        )).id();

        // Act: End of combat resolution
        resolve_combat_end(&mut world);

        // Assert: Unit is dead or permanently injured
        let is_dead = world.get::<Dead>(entity).is_some();
        let is_injured = world.get::<PermanentInjury>(entity).is_some();

        assert!(is_dead || is_injured, "Unit must die or be permanently injured after a heroic act");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct CombatStats {
    pub fire_rate: f32,
    pub damage_reduction: f32,
}

#[derive(Component)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct HeroicActStatus {
    pub active: bool,
}

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct PermanentInjury;

pub fn activate_heroic_act(world: &mut World, entity: Entity) {
    if let mut entity_mut = world.entity_mut(entity) {
        entity_mut.insert(HeroicActStatus { active: true });
        if let Some(mut stats) = entity_mut.get_mut::<CombatStats>() {
            stats.fire_rate *= 2.0;
            stats.damage_reduction = 0.8;
        }
    }
}

pub fn resolve_combat_end(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<HeroicActStatus>>();
    let entities: Vec<Entity> = query.iter(world).collect();

    for entity in entities {
        if let mut entity_mut = world.entity_mut(entity) {
            entity_mut.remove::<HeroicActStatus>();
            entity_mut.insert(Dead); // Minimal implementation: always results in death
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Make the buff magnitudes configurable via a `HeroicActConfig` resource rather than hardcoding multipliers.
- Instead of instantly adding the `Dead` component, emit a `PopDied` or `HeroicSacrifice` event to hook into the Chronicle system for logging the narrative event.
- Differentiate between death and permanent injury based on RNG or remaining health.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for the new module.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Activating the ability demonstrably alters combat stats.
- [ ] The unit is guaranteed to receive a fatal/debilitating outcome post-combat.

## 7. Technical Guidance
- Integrate with the existing `src/layer1/` combat systems if applicable, or wrap the stats modification in a Bevy System.
- Ensure the `HeroicActStatus` component is properly checked in any logic that ends a combat encounter.

## 8. Questions
*Builder: add questions here if spec is unclear.*
