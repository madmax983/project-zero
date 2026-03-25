# 586: Doppelgangers

## 1. Overview
Paranoia. Who is that working next to you? Rare event where a Pop is replaced by a mimic. They look identical but sabotage jobs (consume double resources, work backwards). Revealed by medical scan or strange behavior. The tension is paranoia (frequent scanning, lost time) vs. Trust (risk of sabotage). Layer: 1.

## 2. Dependencies
- Pop component and Job system
- Medical scanning system
- Event generation for replacement

## 3. RED Phase: Tests First
```rust
#[test]
fn test_doppelganger_sabotages_job() {
    let mut world = World::new();
    let job = world.spawn(Job { progress: 10.0 }).id();
    world.spawn((Pop, Doppelganger, AssignedJob(job)));

    doppelganger_sabotage_system(&mut world);

    let job_data = world.get::<Job>(job).unwrap();
    assert!(job_data.progress < 10.0);
}

#[test]
fn test_medical_scan_reveals_doppelganger() {
    let mut world = World::new();
    let pop = world.spawn((Pop, Doppelganger)).id();
    world.spawn((MedicalScanner, Target(pop)));

    medical_scan_system(&mut world);

    assert!(world.get::<RevealedDoppelganger>(pop).is_some());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
pub struct Doppelganger;
pub struct RevealedDoppelganger;
pub fn doppelganger_sabotage_system(world: &mut World) { ... }
pub fn medical_scan_system(world: &mut World) { ... }
```

## 5. REFACTOR Phase: Quality & Design
- Ensure job sabotage logic shares code with regular job progression but inverted.
- Allow tuning the frequency and discovery rate of doppelgangers via constants.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Doppelgangers decrease job progress and can be revealed by medical scans.

## 7. Technical Guidance
- Integrate doppelgangers as an invisible component added to existing pops.
- Visuals shouldn't change until `RevealedDoppelganger` is added.

## 8. Questions
*Builder: add questions here if spec is unclear.*
