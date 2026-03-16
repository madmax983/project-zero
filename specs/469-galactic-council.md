# 469: The Galactic Council

## 1. Overview
You can win the war but lose the vote. Space bureaucracy is the true endgame boss. Met on Layer 3, Civilizations vote on binding "Resolutions" (e.g., "Prohibit Strip Mining", "Universal Rights"). Vote weight depends on Population and Prestige. If a colony violates an active resolution, they face severe economic sanctions or war from the entire Council.

## 2. Dependencies
- Layer 3 Diplomatic Architecture
- `050` Civil Unrest
- `039` Trade System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    // Assuming these components/resources exist
    // use scale_core::layer3::diplomacy::{GalacticCouncil, Resolution, VoteWeight, CouncilMember};
    // use scale_core::layer1::trade::TradeSanctions;

    #[derive(Resource, Default)]
    struct GalacticCouncil {
        pub active_resolutions: Vec<Resolution>,
    }

    #[derive(PartialEq, Eq, Clone, Debug)]
    enum Resolution {
        BanStripMining,
        UniversalRights,
    }

    #[derive(Component)]
    struct CouncilMember {
        pub in_breach: bool,
    }

    #[derive(Component)]
    struct TradeSanctions {
        pub multiplier: f32, // e.g., 0.5 means half trade value
    }

    fn enforce_resolutions_system(
        council: Res<GalacticCouncil>,
        mut members: Query<(Entity, &mut CouncilMember, Option<&mut TradeSanctions>)>,
        mut commands: Commands,
    ) {
        for (entity, member, mut sanctions) in members.iter_mut() {
            if member.in_breach && !council.active_resolutions.is_empty() {
                // Apply sanctions if in breach of any active resolution
                if let Some(mut s) = sanctions {
                    s.multiplier = 0.5;
                } else {
                    commands.entity(entity).insert(TradeSanctions { multiplier: 0.5 });
                }
            } else if !member.in_breach {
                // Lift sanctions
                if sanctions.is_some() {
                    commands.entity(entity).remove::<TradeSanctions>();
                }
            }
        }
    }

    #[test]
    fn test_in_breach_member_gets_trade_sanctions() {
        let mut app = App::new();
        app.insert_resource(GalacticCouncil {
            active_resolutions: vec![Resolution::BanStripMining],
        });

        let violator = app.world_mut().spawn(CouncilMember { in_breach: true }).id();

        app.add_systems(Update, enforce_resolutions_system);
        app.update();

        let sanctions = app.world().get::<TradeSanctions>(violator);
        assert!(sanctions.is_some(), "Violating member should receive TradeSanctions");
        assert_eq!(sanctions.unwrap().multiplier, 0.5);
    }

    #[test]
    fn test_compliant_member_no_sanctions() {
        let mut app = App::new();
        app.insert_resource(GalacticCouncil {
            active_resolutions: vec![Resolution::BanStripMining],
        });

        let compliant = app.world_mut().spawn(CouncilMember { in_breach: false }).id();
        let formerly_breaching = app.world_mut().spawn((
            CouncilMember { in_breach: false }, // Fixed their breach
            TradeSanctions { multiplier: 0.5 },
        )).id();

        app.add_systems(Update, enforce_resolutions_system);
        app.update();

        assert!(app.world().get::<TradeSanctions>(compliant).is_none(), "Compliant member should not get sanctions");
        assert!(app.world().get::<TradeSanctions>(formerly_breaching).is_none(), "Sanctions should be removed after compliance");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GalacticCouncil {
    pub active_resolutions: Vec<Resolution>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Resolution {
    BanStripMining,
    UniversalRights,
}

#[derive(Component)]
pub struct CouncilMember {
    pub in_breach: bool,
}

#[derive(Component)]
pub struct TradeSanctions {
    pub multiplier: f32, // 1.0 is normal, 0.5 is 50% penalty
}

pub fn enforce_resolutions_system(
    council: Res<GalacticCouncil>,
    mut commands: Commands,
    mut members: Query<(Entity, &CouncilMember, Option<&mut TradeSanctions>)>,
) {
    for (entity, member, sanctions_opt) in members.iter_mut() {
        if member.in_breach && !council.active_resolutions.is_empty() {
            // Apply severe sanctions (50% trade penalty)
            if let Some(mut sanctions) = sanctions_opt {
                sanctions.multiplier = 0.5;
            } else {
                commands.entity(entity).insert(TradeSanctions { multiplier: 0.5 });
            }
        } else {
            // Remove sanctions if compliant
            if sanctions_opt.is_some() {
                commands.entity(entity).remove::<TradeSanctions>();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Resolution Types**: Expand the enum into a richer struct defining the conditions for a breach (e.g., `StripMining` checks for massive extraction rates on Layer 1).
- **Voting Mechanics**: Create a `VoteWeight` component (Pop count + Prestige) and a system that processes a "Resolution Ballot" event every decade.
- **Defiance**: Allow the player an explicit choice to "Defy Council". This instantly drops all diplomatic relations to -100 and spawns a punitive Layer 3 invasion fleet.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Members in breach of active resolutions receive a `TradeSanctions` component reducing trade value by 50%.

## 7. Technical Guidance
- Ensure that the logic to detect a `breach` correctly queries Layer 1 colony state (e.g., checking the number of active Strip Mines) and updates the `CouncilMember.in_breach` boolean before the enforcement system runs.
- Sanctions must multiply the final value of exported cargo items during Trade Ship processing on Layer 2.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
