# The Symbiotic Parasite

## 1. Overview
The Symbiotic Parasite feature introduces a rare alien spore that infects Pops. Infected Pops become hyper-efficient: they no longer need to sleep or eat and work at 150% efficiency. However, this comes at a terrible cost: they slowly drain the health of adjacent healthy Pops during work cycles. This forces the player into a moral dilemma: isolate the infected to create a super-productive "leper colony" of immortal, sleepless workers, or risk them draining the rest of the healthy population. This mechanic applies primarily to the Colony Layer (Layer 1).

## 2. Dependencies
- `src/layer1/needs.rs`: Modifying or freezing hunger and rest needs.
- `src/layer1/health.rs`: For applying damage to healthy Pops.
- `src/layer1/components.rs`: The `Pop` component and introducing new components for the infection state.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components for testing
    #[derive(Component)]
    struct Pop;

    #[derive(Component)]
    struct Health(f32);

    #[derive(Component)]
    struct WorkEfficiency(f32);

    #[derive(Component)]
    struct Hunger(f32);

    #[derive(Component)]
    struct Rest(f32);

    #[derive(Component)]
    struct SymbioticParasite {
        drain_amount: f32,
        drain_radius: f32,
    }

    #[derive(Component)]
    struct Transform(Vec3);

    // 1. Test that infected pops do not increase hunger or rest needs and have high efficiency
    #[test]
    fn test_infected_pop_needs_frozen_and_efficiency_boosted() {
        let mut app = App::new();
        app.add_systems(Update, apply_parasite_buffs_system);

        let pop = app.world_mut().spawn((
            Pop,
            Hunger(50.0),
            Rest(50.0),
            WorkEfficiency(1.0),
            SymbioticParasite {
                drain_amount: 5.0,
                drain_radius: 5.0,
            },
        )).id();

        app.update();

        // Needs should be effectively frozen or handled such that they don't impact the pop negatively.
        // For simplicity in this test, we might just set them to a "full" or "frozen" state.
        // Let's test efficiency first.
        let efficiency = app.world().get::<WorkEfficiency>(pop).unwrap().0;
        assert_eq!(efficiency, 1.5, "Infected pop should have 150% work efficiency.");
    }

    // 2. Test that infected pops drain health from nearby healthy pops
    #[test]
    fn test_infected_pop_drains_nearby_health() {
        let mut app = App::new();
        app.add_systems(Update, apply_parasite_health_drain_system);

        // Spawn infected pop
        app.world_mut().spawn((
            Pop,
            SymbioticParasite {
                drain_amount: 10.0,
                drain_radius: 5.0,
            },
            Transform(Vec3::new(0.0, 0.0, 0.0)),
        ));

        // Spawn healthy pop near
        let healthy_near = app.world_mut().spawn((
            Pop,
            Health(100.0),
            Transform(Vec3::new(3.0, 0.0, 0.0)),
        )).id();

        // Spawn healthy pop far
        let healthy_far = app.world_mut().spawn((
            Pop,
            Health(100.0),
            Transform(Vec3::new(10.0, 0.0, 0.0)),
        )).id();

        // Spawn another infected pop near (should not drain each other ideally, or at least we test healthy ones)
        let infected_near = app.world_mut().spawn((
            Pop,
            Health(100.0),
            SymbioticParasite { drain_amount: 5.0, drain_radius: 5.0 },
            Transform(Vec3::new(-3.0, 0.0, 0.0)),
        )).id();

        app.update();

        let hp_near = app.world().get::<Health>(healthy_near).unwrap().0;
        let hp_far = app.world().get::<Health>(healthy_far).unwrap().0;
        let hp_infected = app.world().get::<Health>(infected_near).unwrap().0;

        assert_eq!(hp_near, 90.0, "Healthy pop near infected should lose health.");
        assert_eq!(hp_far, 100.0, "Healthy pop far away should not lose health.");
        assert_eq!(hp_infected, 100.0, "Infected pops should not drain health from each other.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components
#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct WorkEfficiency(pub f32);

#[derive(Component)]
pub struct Hunger(pub f32);

#[derive(Component)]
pub struct Rest(pub f32);

#[derive(Component)]
pub struct SymbioticParasite {
    pub drain_amount: f32,
    pub drain_radius: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Transform(pub Vec3);

// Systems
pub fn apply_parasite_buffs_system(
    mut query: Query<(&mut WorkEfficiency, &mut Hunger, &mut Rest), With<SymbioticParasite>>,
) {
    for (mut efficiency, mut hunger, mut rest) in query.iter_mut() {
        efficiency.0 = 1.5; // 150% efficiency

        // "Freeze" needs by keeping them full (assuming 100 is full)
        hunger.0 = 100.0;
        rest.0 = 100.0;
    }
}

pub fn apply_parasite_health_drain_system(
    parasites: Query<(&SymbioticParasite, &Transform)>,
    mut healthy_pops: Query<(&mut Health, &Transform), (With<Pop>, Without<SymbioticParasite>)>,
) {
    for (parasite, p_transform) in parasites.iter() {
        for (mut health, h_transform) in healthy_pops.iter_mut() {
            let distance = p_transform.0.distance(h_transform.0);
            if distance <= parasite.drain_radius {
                health.0 -= parasite.drain_amount;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactor 1: Need Management.** Instead of forcing `Hunger` and `Rest` to `100.0` every tick, it might be cleaner to add an `IgnoresNeeds` component or flag that the global metabolism systems check before decaying needs, preventing the value from oscillating or interfering with UI logic that expects natural decay.
- **Refactor 2: Spatial Optimization.** The `apply_parasite_health_drain_system` has O(Parasites * HealthyPops) complexity. If the colony grows large, this needs to be optimized using spatial hashing or a grid lookup system to only check pops in adjacent or nearby tiles.
- **Refactor 3: Infection Spread (Optional but thematic).** Currently, the spec handles the *effects* of the parasite. A future iteration or secondary spec should handle how the parasite *spreads* (e.g., a small chance to infect the drained pop if their health gets too low).

## 6. Acceptance Criteria

- [ ] Infected pops operate at 1.5x efficiency.
- [ ] Infected pops do not suffer from hunger or rest decay.
- [ ] Infected pops drain a specific amount of health from non-infected pops within a defined radius.
- [ ] Infected pops do NOT drain health from other infected pops.
- [ ] All RED phase tests pass (`cargo test` returns 0 failures).
- [ ] Clippy warnings resolved.
- [ ] Test coverage ≥ 85% for new implementation.

## 7. Technical Guidance

- Pay attention to the queries in `apply_parasite_health_drain_system`. The `Without<SymbioticParasite>` filter is crucial to ensure parasites don't kill each other, allowing the player to actually form a functional "leper colony" if they manage to isolate them.
- Consider what happens if a Pop's health reaches 0 due to draining. Ensure the standard death systems trigger correctly.

## 8. Questions
*Builder: add questions here if spec is unclear.*

- **Architectural Contradictions:** The RED Phase tests define `Health(f32)` and `Hunger(f32)` as tuple components. However, in the current architecture, `Health` is a full struct (`pub struct Health { pub current: f32, pub max: f32, pub has_rust_lung: bool }`), and `Hunger` is not a standalone component but a field inside the `Needs` component (`needs.hunger`). This makes the RED phase impossible to implement as written. I'm moving on to a different task.
- **Architectural Contradictions:** `Health` is not a tuple struct (e.g. `Health(f32)`), it is a full struct (`Health { current: f32, max: f32, ... }`). Additionally, `Hunger` and `Rest` are not standalone components but fields within the `Needs` component. The RED phase tests and GREEN phase logic cannot be implemented as written due to these mismatching types. I'm picking a different task.
