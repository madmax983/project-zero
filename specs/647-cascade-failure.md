# Cascade Failure

## 1. Overview
**Layer:** Cross-layer (1 -> 2 -> 3)
**Fantasy:** Watching one small problem snowball into galactic crisis.
**Mechanic:** Resource shortages at Layer 1 reduce colony output. Reduced output strains system logistics. System strain weakens sector defenses. Sector weakness invites invasion.

## 2. Dependencies
- Needs logistics and defense concepts (Layer 2)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_layer1_shortage_reduces_layer2_logistics() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, evaluate_system_logistics);

        let entity = app.world.spawn((
            SystemLogistics { capacity: 100, utilized: 50 },
            ColonyOutput { expected: 100, actual: 50 }, // 50% output due to shortage
        )).id();

        // Act
        app.update();

        // Assert
        let logistics = app.world.get::<SystemLogistics>(entity).unwrap();
        assert_eq!(logistics.capacity, 50); // Capacity drops to match colony output ratio
    }

    #[test]
    fn test_logistics_strain_weakens_defenses() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, update_sector_defenses);

        let entity = app.world.spawn((
            SectorDefense { power: 1000 },
            SystemLogistics { capacity: 50, utilized: 50 }, // 100% utilized/strained
        )).id();

        // Act
        app.update();

        // Assert
        let defense = app.world.get::<SectorDefense>(entity).unwrap();
        assert!(defense.power < 1000); // Defenses should weaken under strain
    }

    #[test]
    fn test_weak_defenses_increase_invasion_threat() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, calculate_invasion_threat);

        let entity = app.world.spawn((
            SectorDefense { power: 200 }, // Weak
            InvasionThreat { level: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let threat = app.world.get::<InvasionThreat>(entity).unwrap();
        assert!(threat.level > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ColonyOutput {
    pub expected: u32,
    pub actual: u32,
}

#[derive(Component)]
pub struct SystemLogistics {
    pub capacity: u32,
    pub utilized: u32,
}

#[derive(Component)]
pub struct SectorDefense {
    pub power: u32,
}

#[derive(Component)]
pub struct InvasionThreat {
    pub level: f32,
}

pub fn evaluate_system_logistics(
    mut query: Query<(&mut SystemLogistics, &ColonyOutput)>
) {
    for (mut logistics, output) in query.iter_mut() {
        if output.expected > 0 {
            let ratio = output.actual as f32 / output.expected as f32;
            logistics.capacity = (100.0 * ratio) as u32; // Assuming base capacity of 100
        }
    }
}

pub fn update_sector_defenses(
    mut query: Query<(&mut SectorDefense, &SystemLogistics)>
) {
    for (mut defense, logistics) in query.iter_mut() {
        if logistics.capacity > 0 {
            let strain = logistics.utilized as f32 / logistics.capacity as f32;
            if strain >= 1.0 {
                defense.power = (defense.power as f32 * 0.9) as u32; // Lose 10% power per tick if strained
            }
        } else {
            defense.power = (defense.power as f32 * 0.5) as u32; // Extreme penalty if no capacity
        }
    }
}

pub fn calculate_invasion_threat(
    mut query: Query<(&SectorDefense, &mut InvasionThreat)>
) {
    for (defense, mut threat) in query.iter_mut() {
        if defense.power < 500 {
            threat.level += 1.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Tweak the penalty rates to feel fair but appropriately punishing for a "cascade" failure.
- Link `ColonyOutput` directly to actual resources (Food, Parts, etc.) rather than an abstract expected/actual value.
- Add descriptive events (`LogisticsStrainedEvent`, `DefenseWeakenedEvent`) to notify the player as the situation escalates.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Layer 1 output deficit cascades to logistics penalties.
- [ ] Logistics penalties cascade to defense penalties.
- [ ] Defense penalties increase invasion threat.

## 7. Technical Guidance
- This is a chain of systems. Ensure they run in the correct order in the Bevy schedule (Output -> Logistics -> Defense -> Threat) so the cascade can happen smoothly.

## 8. Questions
*Builder: add questions here if spec is unclear.*
