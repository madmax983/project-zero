# 145: Prototyping Phase

**Layer:** 1
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

**Fantasy:** The first time you build a fusion reactor, it shouldn't work perfectly. You are figuring it out as you go.

**Mechanic:**
- The first time a complex building is constructed, it is a "Prototype".
- Prototypes have reduced efficiency (e.g., lower power output, slower production) and higher breakdown chance.
- As the Prototype operates, it generates "Mastery" for that building type.
- Once Mastery reaches a threshold (100%), the design is "Mastered".
- Future buildings of that type are constructed as "Standard" (full stats).
- Existing Prototypes remain Prototypes unless deconstructed and rebuilt (Retrograde Engineering style loop).

**Why:** Adds pacing to the tech progression. Prevents "unlock and spam" meta. Encourages building early, imperfect versions to learn.

---

## 2. Dependencies

- `007` Building: Housing (Basic building system)
- `029` Knowledge System (For tracking global state, though this is a separate resource)
- `045` Structure Durability (For breakdown logic)
- `112` Maintenance Debt (Ideally, but can work with simple breakdown chance)

---

## 3. RED Phase: Tests First

These tests define the behavior. They should fail until the Green phase is implemented.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_construction_is_prototype() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BuildingMastery>();
        // ... setup building system ...

        // Act
        // Spawn a FusionReactor for the first time
        let entity = app.world.spawn(BuildingBundle::new(BuildingType::FusionReactor)).id();
        app.update();

        // Assert
        // Should have Prototype component
        assert!(app.world.entity(entity).contains::<Prototype>());

        // Should have reduced efficiency
        let prototype = app.world.entity(entity).get::<Prototype>().unwrap();
        assert_eq!(prototype.efficiency_modifier, 0.5); // 50% efficiency
    }

    #[test]
    fn test_mastery_accumulation() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<BuildingMastery>();

        let building_type = BuildingType::FusionReactor;
        let entity = app.world.spawn((
            BuildingBundle::new(building_type),
            Prototype { efficiency_modifier: 0.5, breakdown_chance_modifier: 2.0 },
            PowerProducer { current: 10.0, max: 20.0 } // Active
        )).id();

        // Act
        // Run simulation for 1 tick (e.g. 1 hour)
        app.update();

        // Assert
        let mastery = app.world.resource::<BuildingMastery>();
        let progress = mastery.get_progress(building_type);
        assert!(progress > 0.0);
        assert!(progress < 1.0);
    }

    #[test]
    fn test_mastery_unlock_standard_builds() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<BuildingMastery>();
        let building_type = BuildingType::FusionReactor;

        // Set mastery to complete
        app.world.resource_mut::<BuildingMastery>().set_mastered(building_type);

        // Act
        // Build a NEW reactor
        let entity = app.world.spawn(BuildingBundle::new(building_type)).id();
        app.update();

        // Assert
        // Should NOT be a prototype
        assert!(!app.world.entity(entity).contains::<Prototype>());
    }

    #[test]
    fn test_prototype_efficiency_affects_production() {
        // Arrange
        let mut app = App::new();
        // Setup a system that calculates production based on efficiency
        app.add_systems(Update, production_system);

        let entity = app.world.spawn((
            BuildingType::FusionReactor,
            Prototype { efficiency_modifier: 0.5, ..default() },
            Production { base_rate: 100.0, current_rate: 0.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let production = app.world.entity(entity).get::<Production>().unwrap();
        assert_eq!(production.current_rate, 50.0); // 100 * 0.5
    }

    #[test]
    fn test_idle_prototypes_do_not_generate_mastery() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<BuildingMastery>();

        let building_type = BuildingType::FusionReactor;
        // Spawn INACTIVE prototype (e.g. disabled or no fuel)
        let entity = app.world.spawn((
            BuildingBundle::new(building_type),
            Prototype::default(),
            PowerProducer { current: 0.0, max: 20.0 } // Inactive
        )).id();

        // Act
        app.update();

        // Assert
        let mastery = app.world.resource::<BuildingMastery>();
        assert_eq!(mastery.get_progress(building_type), 0.0);
    }
}
```

---

## 4. GREEN Phase: Minimal Implementation

The simplest code to pass the tests.

### 1. Resources and Components

```rust
// src/layer1/prototyping.rs

#[derive(Resource, Default)]
pub struct BuildingMastery {
    // Map of BuildingType -> Progress (0.0 to 1.0)
    // If >= 1.0, it is Mastered.
    progress: HashMap<BuildingType, f32>,
}

impl BuildingMastery {
    pub fn get_progress(&self, building: BuildingType) -> f32 {
        *self.progress.get(&building).unwrap_or(&0.0)
    }

    pub fn is_mastered(&self, building: BuildingType) -> bool {
        self.get_progress(building) >= 1.0
    }

    pub fn add_progress(&mut self, building: BuildingType, amount: f32) {
        let current = self.progress.entry(building).or_insert(0.0);
        *current = (*current + amount).min(1.0);
    }

    #[cfg(test)]
    pub fn set_mastered(&mut self, building: BuildingType) {
        self.progress.insert(building, 1.0);
    }
}

#[derive(Component, Default)]
pub struct Prototype {
    pub efficiency_modifier: f32, // e.g. 0.5
    pub breakdown_chance_modifier: f32, // e.g. 2.0
}
```

### 2. Systems

```rust
// src/layer1/prototyping.rs

pub fn mastery_accumulation_system(
    mut mastery: ResMut<BuildingMastery>,
    // Query active prototypes.
    // Requires some way to know if active (e.g. PowerConsumer, PowerProducer, or Working state)
    prototypes: Query<(&BuildingType, &Prototype), With<Active>>,
    time: Res<Time>,
) {
    const MASTERY_RATE_PER_SECOND: f32 = 0.01; // Takes 100 seconds to master

    for (building_type, _) in prototypes.iter() {
        if !mastery.is_mastered(*building_type) {
            mastery.add_progress(*building_type, MASTERY_RATE_PER_SECOND * time.delta_seconds());
        }
    }
}

// Hook into building construction
pub fn apply_prototype_status_command(
    commands: &mut Commands,
    entity: Entity,
    building_type: BuildingType,
    mastery: &BuildingMastery,
) {
    if !mastery.is_mastered(building_type) {
        commands.entity(entity).insert(Prototype {
            efficiency_modifier: 0.5,
            breakdown_chance_modifier: 2.0,
        });
    }
}
```

### 3. Integration with Production

```rust
// In existing production systems (e.g. power, crafting)
pub fn production_efficiency_system(
    mut query: Query<(&mut Production, Option<&Prototype>)>,
) {
    for (mut production, prototype) in query.iter_mut() {
        let efficiency = prototype.map_or(1.0, |p| p.efficiency_modifier);
        production.current_rate = production.base_rate * efficiency;
    }
}
```

---

## 5. REFACTOR Phase: Quality & Design

### Refactoring Opportunities

1.  **Centralize Efficiency Logic:**
    -   Instead of checking `Option<&Prototype>` everywhere, create a helper `get_building_efficiency(entity)` or a derived component `Efficiency` that is updated when `Prototype` changes.
    -   Or make `Prototype` a factor in a centralized `Stats` component if one exists.

2.  **Configurable Rates:**
    -   Move `MASTERY_RATE_PER_SECOND` to a config file or `BuildingType` data. Some complex buildings should take longer to master.

3.  **UI Feedback:**
    -   Add a UI indicator for "Prototype" status on selected buildings.
    -   Show Mastery progress bar in the Tech/Building menu.

4.  **Events:**
    -   Fire `Event::MasteryAchieved(BuildingType)` when hitting 100%. Use this to spawn a notification ("Fusion Reactor design mastered!").

5.  **Persistence:**
    -   Ensure `BuildingMastery` is serialized in save games.

### API Improvements

-   `BuildingMastery::progress` should be private, exposed only via methods to ensure clamping.
-   `Prototype` values could depend on `BuildingType` (some are jankier than others).

---

## 6. Acceptance Criteria

- [ ] `Prototype` component exists and can be queried.
- [ ] `BuildingMastery` resource tracks progress correctly.
- [ ] New builds check Mastery; if low, they get `Prototype`.
- [ ] `Prototype` buildings generate Mastery over time *only when active*.
- [ ] Mastered buildings do NOT get `Prototype` component.
- [ ] Production systems respect `Prototype.efficiency_modifier`.
- [ ] Tests pass.

---

## 7. Technical Guidance

-   **Integration Point:** The `spawn_building` function (likely in `src/layer1/building.rs`) is the critical hook. You must inject the check there.
-   **Active State:** "Active" is ambiguous. Use `PowerConsumer::active` or `Job::working` or similar depending on the building type. If a building is just a passive box (Stockpile), maybe it gains mastery just by existing? (Probably not, focus on machines).
-   **Breakdowns:** If `112` isn't fully implemented, just stub the `breakdown_chance_modifier` logic or hook it into `Structure.durability` decay.

---

## 8. Questions

-   *Does deconstructing a Prototype refund full cost?* (Probably not, standard logic applies).
-   *Architect:* No, prototyping is inherently wasteful. Deconstruction yields 50% of the standard refund rate.
-   *Can you speed up mastery with Research?* (Future feature).
-   *Do multiple prototypes speed up mastery?* (Yes, the loop iterates all active prototypes. 2 reactors = 2x speed. This is intentional emergent behavior).

*Architect:* Standard deconstruction refunds apply. Multiple prototypes stack their learning loops.
