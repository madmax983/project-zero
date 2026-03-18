# Specification: The Flesh-Weaver's Loom

## 1. Overview
**Layer:** 1
**Fantasy:** Repurposing the dead into beautiful, horrifying tapestries of biological armor.
**Mechanic:** A specialized building that converts corpses (alien or pop) into "Bio-Weave," a highly resistant but grotesque armor for militia and buildings.
**Emergence:** You clad your entire defensive line in the remains of the last pirate raid. Your defenders are invincible, but their morale plummets because their walls are literally made of their former enemies' faces.
**Tension:** High physical defense vs. extreme psychological horror and morale penalties.

## 2. Dependencies
- Needs system (`src/layer1/needs.rs`)
- Item system (`src/layer1/item.rs` / `ColonyInventory`)
- Building system (`src/layer1/building.rs`)
- Mood system (`src/layer1/mood.rs`)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_flesh_weaver_production() {
    // Arrange: setup world, add Corpse item to inventory
    // Act: Process the Flesh Weaver production tick
    // Assert: Corpse is consumed, BioWeave item is created
}

#[test]
fn test_bio_weave_armor_equip() {
    // Arrange: setup world, create Pop, add BioWeave item
    // Act: Equip BioWeave to Pop
    // Assert: Pop gains high Defense stat, Pop gains massive negative Mood modifier ("Horrified")
}

#[test]
fn test_bio_weave_building_construction() {
    // Arrange: setup world, add BioWeave to inventory
    // Act: Construct Wall using BioWeave
    // Assert: Wall has high Health, nearby Pops receive negative Mood aura
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation
// 1. Add BioWeave to ItemType enum
// 2. Add FleshWeaversLoom to BuildingType enum
// 3. Create process_flesh_weaver_system:
//    - Find FleshWeaversLoom buildings
//    - If Inventory has Corpse:
//      - Remove Corpse
//      - Add BioWeave
// 4. Update equip_system to apply Defense buff and Mood debuff when BioWeave is equipped.
```

## 5. REFACTOR Phase: Quality & Design
- Create a generalized system for "Aura" mood effects from buildings or equipment, rather than hardcoding BioWeave's aura.
- Ensure the production system accurately checks for the correct type of Corpse if we want to differentiate between Alien and Pop corpses later.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Flesh Weaver consumes Corpses and produces BioWeave
- [ ] Equipping/Building with BioWeave increases Defense but lowers Mood

## 7. Technical Guidance
- Be careful with how "Auras" are processed; use spatial queries or chunking to optimize checking for nearby Pops.
- Consider adding a `HorrorTolerance` trait to Pops to mitigate the negative mood effects.

## 8. Questions
*Builder: add questions here if spec is unclear.*
