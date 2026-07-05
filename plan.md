1. **Setup Test Environment**:
   - Write integration tests using a heredoc: `cat << 'EOF' > tests/integration/feral_admin_chronicle.rs`
   ```rust
   use bevy::prelude::*;
   use scale::layer1::administration::feral_administration::{FeralColony, AdministrativeBuilding, UnprocessedForms};
   use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};

   fn verify_event(mut events: EventReader<AddChronicleEvent>) {
       let emitted: Vec<_> = events.read().collect();
       assert_eq!(emitted.len(), 1);
       assert!(emitted[0].text.contains("A mountain of Unprocessed Forms has collapsed"));
   }

   #[test]
   fn test_feral_admin_chronicle_bridge() {
       let mut app = App::new();
       app.add_plugins(MinimalPlugins);
       app.add_event::<AddChronicleEvent>();
       app.add_systems(Update, (
           scale::layer1::administration::feral_administration::feral_admin_chronicle_bridge,
           verify_event
       ).chain());

       // Act: Spawn an UnprocessedForm that crosses the impassable threshold
       app.world_mut().spawn((
           UnprocessedForms { stack_size: 15 },
       ));

       app.update();
   }
   ```
   - Update `tests/integration.rs` to include the file using: `echo '#[path = "integration/feral_admin_chronicle.rs"]\nmod feral_admin_chronicle;' >> tests/integration.rs`.

2. **Write the Glue**:
   - Add the new system directly to `src/layer1/administration/feral_administration.rs` by using python string replacement:
   ```bash
   cat << 'EOF' > update_glue.py
   with open("src/layer1/administration/feral_administration.rs", "r") as f:
       content = f.read()

   new_system = """pub fn feral_admin_chronicle_bridge(
       query: Query<(Entity, &UnprocessedForms), Changed<UnprocessedForms>>,
       mut recorded: Local<bevy_utils::HashSet<Entity>>,
       mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
   ) {
       for (entity, forms) in query.iter() {
           if forms.stack_size >= 10 && !recorded.contains(&entity) {
               recorded.insert(entity);
               chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                   text: "A mountain of Unprocessed Forms has collapsed, rendering a section of the colony impassable!".to_string(),
                   importance: crate::layer1::core::chronicle::EventImportance::Major,
               });
           }
       }
   }

   pub fn process_impassable_terrain_system(
   """
   content = content.replace("pub fn process_impassable_terrain_system(", new_system)

   with open("src/layer1/administration/feral_administration.rs", "w") as f:
       f.write(content)
   EOF
   python3 update_glue.py
   rm update_glue.py
   ```

   - Register the system in `src/layer1/systems/economy.rs` using a targeted `sed` command:
   ```bash
   sed -i '/crate::layer1::administration::feral_administration::process_impassable_terrain_system,/a \            crate::layer1::administration::feral_administration::feral_admin_chronicle_bridge,' src/layer1/systems/economy.rs
   ```

3. **Verify the Replacements**:
   - Check the file contents to ensure they were written successfully.
   ```bash
   grep -C 5 "pub fn feral_admin_chronicle_bridge" src/layer1/administration/feral_administration.rs
   grep -C 2 "feral_admin_chronicle_bridge" src/layer1/systems/economy.rs
   ```

4. **Verify Tests**:
   - Run `cargo test --test integration`
   - Run `cargo clippy -- -D warnings` to verify

5. **Update State Files**:
   - Append to `design/SEAM_MAP.md`:
   ```bash
   cat << 'EOF' >> design/SEAM_MAP.md

   ### INT-1310: The Feral Administration -> Chronicle
   - **Date:** 2024-05-31
   - **Systems connected:** `UnprocessedForms` -> `feral_admin_chronicle_bridge` -> `AddChronicleEvent`
   - **Glue added:** `feral_admin_chronicle_bridge` in `src/layer1/administration/feral_administration.rs`.
   - **Schedule:** Registered in Layer 1 Economy schedule (`src/layer1/systems/economy.rs`).
   - **Tests:** `tests/integration/feral_admin_chronicle.rs`
   EOF
   ```
   - Append to `design/COMPLETED.md`:
   ```bash
   echo '- [x] `INT-1310` Integration: The Feral Administration -> Chronicle — completed 2024-05-31' >> design/COMPLETED.md
   ```

6. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**

7. **Submit Changes** with commit title `feat(integration): connect feral administration to chronicle`.
