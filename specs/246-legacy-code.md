    #[test]
    fn test_bloat_slows_research() {
        // This test assumes a system modifies ResearchRate based on Bloat
        // or we test the helper function `calculate_efficiency`

        let bloat_low = Bloat { current: 10.0, rate: 0.0 };
        assert!(bloat_low.efficiency() > 0.9);

        let bloat_high = Bloat { current: 90.0, rate: 0.0 };
        assert!(bloat_high.efficiency() < 0.2);
    }

    #[test]
    fn test_reformat_clears_bloat_but_disables_system() {
        let mut world = World::new();
        let mainframe = world.spawn((
            Mainframe,
            Bloat { current: 100.0, rate: 0.1 },
            SystemStatus::Online,
        )).id();

        // Trigger Reformat
        // Assume event or component trigger
        crate::layer1::tech::legacy_code::start_reformat(&mut world, mainframe);

        let status = world.get::<SystemStatus>(mainframe).unwrap();
        assert_eq!(*status, SystemStatus::Rebooting(100)); // 100 ticks duration

        // Advance time to finish
        // (Mocking system update for reboot logic)
        crate::layer1::tech::legacy_code::finish_reformat(&mut world, mainframe);

        let bloat = world.get::<Bloat>(mainframe).unwrap();
        assert_eq!(bloat.current, 0.0);

        let status = world.get::<SystemStatus>(mainframe).unwrap();
        assert_eq!(*status, SystemStatus::Online);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/tech/legacy_code.rs

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Mainframe;

#[derive(Component, Default, Debug)]
pub struct Bloat {
    pub current: f32, // 0-100
    pub rate: f32,    // e.g. 0.01 per tick
}

impl Bloat {
    pub fn efficiency(&self) -> f32 {
        // Non-linear decay? Or simple linear?
        // 0 bloat = 1.0
        // 100 bloat = 0.1 (min efficiency)
        (1.0 - (self.current / 110.0)).max(0.1)
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum SystemStatus {
    Online,
    Rebooting(u32), // Ticks remaining
    Offline,
}
```

### 2. Systems

```rust
pub fn update_bloat_system(
    mut query: Query<(&mut Bloat, &mut SystemStatus)>,
) {
    for (mut bloat, mut status) in query.iter_mut() {
        match *status {
            SystemStatus::Online => {
                bloat.current = (bloat.current + bloat.rate).min(100.0);
            }
            SystemStatus::Rebooting(ref mut ticks) => {
                if *ticks > 0 {
                    *ticks -= 1;
                } else {
                    // Done
                    bloat.current = 0.0;
                    *status = SystemStatus::Online;
                }
            }
            _ => {}
        }
    }
}

pub fn start_reformat(world: &mut World, entity: Entity) {
    if let Some(mut status) = world.get_mut::<SystemStatus>(entity) {
        *status = SystemStatus::Rebooting(500); // Constant for now
    }
}

pub fn finish_reformat(world: &mut World, entity: Entity) {
    // Helper for testing
    if let Some(mut status) = world.get_mut::<SystemStatus>(entity) {
        if let SystemStatus::Rebooting(_) = *status {
             *status = SystemStatus::Rebooting(0);
             // Let system handle the switch next tick
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Integration**: `ResearchSystem` must query `Mainframe` bloat to apply efficiency penalty.
- **Integration**: `Turret` system must check `Mainframe` status. If `Rebooting`, turrets default to `Manual` (low accuracy) or `Offline`.
- **UI**: Show "System Efficiency: 85%" and a "Reformat Now" button with warning text.

## Acceptance Criteria

- [ ] `Bloat` accumulates.
- [ ] Efficiency calculation exists.
- [ ] `Reformat` action resets Bloat but imposes `Rebooting` state.
- [ ] Tests pass.

## Technical Guidance

- Ensure `Mainframe` entity is unique (singleton).
- Reboot duration should scale with how much bloat was cleared (longer delay for worse neglect).

## Questions

*Builder: Does bloat affect life support?*
*Architect:* Yes, if life support is linked to the main mainframe. Legacy bloat should increase the power draw of all automated systems, including life support, by a variable percentage.
*Architect: Yes, severe bloat introduces stuttering in life support systems, causing momentary drops in room quality or oxygen flow.*
*Architect: Yes, it introduces latency in automated life support systems (e.g. slower venting, delayed oxygen generation), but it does not completely disable them unless a system shutdown is forced.*
*Architect: Yes, if life support systems are connected to the Mainframe, bloat will reduce their efficiency, potentially causing temporary outages.*
*Architect: No, critical systems are hardwired (analog). Only "Smart" systems lag.*
