# 1169: The Last Archive

## 1. Overview
A late-game expedition to the galactic center reveals "The Last Archive"—the physical server running the simulation of your reality. Interacting with it reveals the universe's hardware is failing, creating "Glitch Zones" where physics breaks down. To fix it, the player must harvest an impossible amount of energy and exotic matter to physically patch the server before the simulation terminates, likely uniting all other factions against them.

## 2. Dependencies
- Galactic Map/Layer 3 exploration
- Crisis/Endgame Event System
- Resource pooling (Dyson Swarms/Megastructures)
- "Glitch" mechanics (Layer 1 and Layer 2 physics alterations)

## 3. RED Phase: Tests First

```rust
// src/layer3/last_archive_tests.rs
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_discovering_archive_triggers_glitch_zones() {
        let mut app = App::new();
        app.add_systems(Update, process_archive_discovery);

        let archive_node = app.world_mut().spawn((
            GalacticNode,
            TheLastArchive { discovered: false, integrity: 100.0 },
        )).id();

        // Simulate discovery
        app.world_mut().send_event(ArchiveDiscoveredEvent { node_id: archive_node });
        app.update();

        let archive = app.world().get::<TheLastArchive>(archive_node).unwrap();
        assert!(archive.discovered);

        // Check if a glitch zone was spawned
        let mut query = app.world_mut().query::<&GlitchZone>();
        assert!(query.iter(app.world()).count() > 0, "Discovering the archive should spawn Glitch Zones");
    }

    #[test]
    fn test_patching_archive_requires_massive_energy() {
        let mut app = App::new();
        app.add_systems(Update, patch_archive_system);

        let player_empire = app.world_mut().spawn(EmpireResources { energy: 5_000_000.0 }).id();
        let archive_node = app.world_mut().spawn((
            TheLastArchive { discovered: true, integrity: 50.0 },
        )).id();

        // Attempt patch
        app.world_mut().send_event(PatchArchiveEvent {
            empire_id: player_empire,
            archive_id: archive_node,
            energy_invested: 1_000_000.0,
        });

        app.update();

        let resources = app.world().get::<EmpireResources>(player_empire).unwrap();
        assert_eq!(resources.energy, 4_000_000.0, "Energy should be consumed");

        let archive = app.world().get::<TheLastArchive>(archive_node).unwrap();
        assert!(archive.integrity > 50.0, "Archive integrity should increase after patching");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- **Glitch Mechanics**: Glitch zones should randomly alter localized physics constants (like movement speed, gravity, or rendering layers) on both Layer 1 maps and Layer 2 systems.
- **Diplomatic Fallout**: Investing massive resources into the Archive should steadily trigger massive threat generation across all other empires, leading to a "Crisis" state where the rest of the galaxy unites against the player.
- **Victory Condition**: Fully restoring the Archive's integrity triggers the ultimate "Simulation Maintained" victory condition.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer3/last_archive.rs`.
- [ ] Discovering the Archive spawns `GlitchZone` entities.
- [ ] Players can invest massive amounts of energy to repair the Archive's integrity.

## 7. Technical Guidance
- The "Archive" is effectively the final endgame crisis boss, but instead of fighting it, the player must feed it resources while fighting everyone else.
- The energy cost should be astronomically high, requiring Dyson Swarm-level infrastructure.

## 8. Questions
*Builder: add questions here if spec is unclear.*
