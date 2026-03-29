# 739 - Vermin Evolution

## 1. Overview
The pests are part of the ecosystem, and they adapt to your industry. Vermin populations inherit traits from the resources they consume. Rats eating from the "Uranium Stockpile" become "Rad-Rats" (emit radiation). Rats eating "Glow-Moss" become bioluminescent. This forces the player to choose between secure containment (hassle) and easy storage (risk).

## 2. Dependencies
- Layer 1 Pest/Vermin spawning and tracking.
- Resource stockpiles and consumption logic.
- Trait/Modifier system for entities.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_vermin_eating_uranium_gains_irradiated_trait() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<VerminConsumedResourceEvent>();
        app.add_systems(Update, handle_vermin_evolution_system);

        let vermin = app.world_mut().spawn((
            Vermin,
            BaseStats { health: 10.0 },
        )).id();

        // Act
        app.world_mut().send_event(VerminConsumedResourceEvent {
            vermin_entity: vermin,
            resource_type: ResourceType::Uranium,
        });
        app.update();

        // Assert
        assert!(app.world().get::<IrradiatedTrait>(vermin).is_some(), "Vermin should gain IrradiatedTrait after eating Uranium");
    }

    #[test]
    fn test_vermin_eating_glow_moss_gains_bioluminescent_trait() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<VerminConsumedResourceEvent>();
        app.add_systems(Update, handle_vermin_evolution_system);

        let vermin = app.world_mut().spawn((
            Vermin,
            BaseStats { health: 10.0 },
        )).id();

        // Act
        app.world_mut().send_event(VerminConsumedResourceEvent {
            vermin_entity: vermin,
            resource_type: ResourceType::GlowMoss,
        });
        app.update();

        // Assert
        assert!(app.world().get::<BioluminescentTrait>(vermin).is_some(), "Vermin should gain BioluminescentTrait after eating Glow Moss");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Vermin;

#[derive(Component)]
pub struct BaseStats {
    pub health: f32,
}

#[derive(Component)]
pub struct IrradiatedTrait;

#[derive(Component)]
pub struct BioluminescentTrait;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Uranium,
    GlowMoss,
    StandardRations,
}

#[derive(Event)]
pub struct VerminConsumedResourceEvent {
    pub vermin_entity: Entity,
    pub resource_type: ResourceType,
}

pub fn handle_vermin_evolution_system(
    mut commands: Commands,
    mut events: EventReader<VerminConsumedResourceEvent>,
) {
    for event in events.read() {
        match event.resource_type {
            ResourceType::Uranium => {
                commands.entity(event.vermin_entity).insert(IrradiatedTrait);
            }
            ResourceType::GlowMoss => {
                commands.entity(event.vermin_entity).insert(BioluminescentTrait);
            }
            _ => {} // Standard resources do not trigger evolution in this MVP
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Abstract the hardcoded `match` statement into a `ResourceEvolutionMap` resource that maps `ResourceType` to a component or trait ID, so new resource interactions can be added without changing the code.
- Ensure the `IrradiatedTrait` actually hooks into the hazard system to emit radiation into adjacent tiles.
- Evolved traits should pass down to vermin offspring if vermin reproduce in the simulation.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Vermin gain specific component traits when consuming specific resources.

## 7. Technical Guidance
- Integrate with the logistics/stockpile systems. Vermin need to actually path to and target stockpiles of these resources.
- The `IrradiatedTrait` will need a system to occasionally apply damage or negative morale to nearby Pops.

## 8. Questions
*Builder: add questions here if spec is unclear.*
