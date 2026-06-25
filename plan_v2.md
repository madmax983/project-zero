1. **Claim the task in IN_PROGRESS.md**
   - Use `echo "- [ ] \`INT-289\` Integration: The Mimetic Plague -> Social Interaction — claimed $(date +%Y-%m-%d)" >> design/IN_PROGRESS.md` to claim the integration work.
2. **Verify IN_PROGRESS.md**
   - Run `tail -n 5 design/IN_PROGRESS.md` to ensure the task was added.
3. **Write Integration Test for INT-289 in tests/integration/memetic_plague_transmission.rs**
   - Use `cat > tests/integration/memetic_plague_transmission.rs` with the exact text:
     ```rust
     #[cfg(test)]
     mod integration_tests {
         use bevy::prelude::*;
         use scale::layer1::map::GridPosition;
         use scale::layer1::memetics::memetic_hazards::{
             process_memetic_transmission_system, ConversationEvent,
         };
         use scale::layer1::memetics::parasitic_broadcast::{MemeticInfection, ObsessionType};
         use scale::layer1::pop::Pop;
         use scale::layer1::social::grievances::Ostracized;
         use scale::layer1::social::{proximity_social_system, Relationships};

         #[test]
         fn test_proximity_social_system_emits_conversation_and_spreads_infection() {
             let mut app = App::new();
             app.add_plugins(MinimalPlugins);
             app.init_resource::<Events<ConversationEvent>>();

             app.add_systems(
                 Update,
                 (
                     proximity_social_system,
                     process_memetic_transmission_system,
                 )
                     .chain(),
             );

             // Spawn infected pop
             let infected = app
                 .world_mut()
                 .spawn((
                     Pop,
                     GridPosition { x: 10, y: 10 },
                     Relationships::default(),
                     MemeticInfection {
                         obsession_type: ObsessionType::DigHoles,
                         intensity: 1.0,
                     },
                 ))
                 .id();

             // Spawn healthy pop close enough for proximity interaction
             let healthy = app
                 .world_mut()
                 .spawn((
                     Pop,
                     GridPosition { x: 11, y: 10 },
                     Relationships::default(),
                 ))
                 .id();

             app.update();

             // Assert that healthy pop is now infected
             assert!(
                 app.world().get::<MemeticInfection>(healthy).is_some(),
                 "Healthy pop should have contracted the MemeticInfection via ConversationEvent emitted by proximity_social_system"
             );
         }
     }
     ```
4. **Verify tests/integration/memetic_plague_transmission.rs**
   - Run `cat tests/integration/memetic_plague_transmission.rs` to verify its contents.
5. **Register test in tests/integration.rs**
   - Run `echo "#[path = \"integration/memetic_plague_transmission.rs\"]" >> tests/integration.rs` and `echo "mod memetic_plague_transmission;" >> tests/integration.rs`.
6. **Verify tests/integration.rs**
   - Run `tail -n 5 tests/integration.rs` to confirm registration.
7. **Modify src/layer1/social/mod.rs to emit ConversationEvent**
   - Apply a patch to `src/layer1/social/mod.rs` to add `mut conversation_events: EventWriter<crate::layer1::memetics::memetic_hazards::ConversationEvent>,` to the `proximity_social_system` arguments, and emit the event when `distance <= 2` inside the nested loop (with `entity < other_entity` check).
   - Use a patch file:
     ```diff
     --- src/layer1/social/mod.rs
     +++ src/layer1/social/mod.rs
     @@ -204,6 +204,7 @@
      /// if pop count grows large.
      pub fn proximity_social_system(
          mut commands: Commands,
     +    mut conversation_events: EventWriter<crate::layer1::memetics::memetic_hazards::ConversationEvent>,
          pops: Query<(
              Entity,
              &GridPosition,
     @@ -236,6 +237,13 @@
                  let dy = pos.y.abs_diff(other_pos.y).min(i32::MAX as u32) as i32;
                  let distance = dx.max(dy); // Chebyshev

     +            if distance <= 2 && entity < other_entity {
     +                conversation_events.send(crate::layer1::memetics::memetic_hazards::ConversationEvent {
     +                    initiator: entity,
     +                    receiver: other_entity,
     +                });
     +            }
     +
                  if distance <= 5 {
                      // 5 tile radius
                      let affinity = rel.get_affinity(other_entity);
     ```
8. **Verify src/layer1/social/mod.rs edit**
   - Run `cat src/layer1/social/mod.rs | grep -A 20 "pub fn proximity_social_system"` and `cat src/layer1/social/mod.rs | grep -A 10 "distance <= 2"`.
9. **Update src/layer1/social/mod.rs tests for proximity_social_system**
   - Since we added `EventWriter` to `proximity_social_system`, we need to `app.add_event::<ConversationEvent>()` in `test_proximity_social_system_gives_buff`, `test_proximity_morale_debuff` in `src/layer1/social/mod.rs` or tests won't pass.
   - Use patch:
     ```diff
     --- src/layer1/social/mod.rs
     +++ src/layer1/social/mod.rs
     @@ -512,6 +512,7 @@
          fn test_proximity_social_system_gives_buff() {
              let mut world = World::new();
     +        world.init_resource::<Events<crate::layer1::memetics::memetic_hazards::ConversationEvent>>();

              // Pop 1 and Pop 2 are friends (affinity +50)
              let pop1 = world
     @@ -543,6 +544,7 @@
          #[test]
          fn test_proximity_morale_debuff() {
              let mut world = World::new();
     +        world.init_resource::<Events<crate::layer1::memetics::memetic_hazards::ConversationEvent>>();

              // Pop 1 and Pop 2 are enemies (affinity -50)
              let pop1 = world
     ```
10. **Verify test edits**
    - Run `cat src/layer1/social/mod.rs | grep -A 5 "fn test_proximity_social_system_gives_buff"` and `cat src/layer1/social/mod.rs | grep -A 5 "fn test_proximity_morale_debuff"`.
11. **Run full tests**
    - Run `cargo test` and `cargo clippy -- -D warnings`.
12. **Update SEAM_MAP.md**
    - Run `cat >> design/SEAM_MAP.md` with:
      ```markdown
      ### INT-289: The Mimetic Plague -> Social Interaction
      - **Date:** $(date +%Y-%m-%d)
      - **Systems connected:** `proximity_social_system` -> `ConversationEvent` -> `process_memetic_transmission_system`
      - **Glue added:** Modified `proximity_social_system` in `src/layer1/social/mod.rs` to emit `ConversationEvent` when Pops are within a short distance (<= 2).
      - **Tests:** `tests/integration/memetic_plague_transmission.rs`
      ```
13. **Verify SEAM_MAP.md**
    - Run `tail -n 10 design/SEAM_MAP.md`.
14. **Update COMPLETED.md**
    - Remove from IN_PROGRESS: `sed -i '/INT-289/d' design/IN_PROGRESS.md`.
    - Append to COMPLETED.md: `echo "- [x] \`INT-289\` Integration: The Mimetic Plague -> Social Interaction — completed $(date +%Y-%m-%d)" >> design/COMPLETED.md`.
15. **Verify tracker updates**
    - Run `tail -n 5 design/COMPLETED.md` and `grep INT-289 design/IN_PROGRESS.md || echo "Not found"`.
16. **Pre-commit step**
    - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
