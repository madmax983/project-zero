1. **Create the Tectonic Prophets Feature (`src/experimental/tectonic_prophets.rs`)**:
   - Command:
     ```bash
     cat << 'EOF' > src/experimental/tectonic_prophets.rs
     //! Tectonic Prophets (Nova Feature)
     //!
     //! # The Spark
     //! We have `TectonicStress` in the geology layer which builds towards a `MegaQuake`.
     //! We also have `Trait::Prophet` and `Trait::Anxious` in the psychology system.
     //!
     //! # The Feature
     //! When `TectonicStress` exceeds 80% of its threshold, Pops with `Trait::Prophet`
     //! feel the rumbling and gain a massive morale boost ("Vibrations of the Deep"),
     //! while Pops with `Trait::Anxious` suffer a severe morale penalty ("Impending Doom").
     //!
     //! # The Potential
     //! Connects the deep crust physical simulation directly to the socio-psychological
     //! state of the colonists, providing dynamic narrative tension before disaster strikes.

     use bevy_ecs::prelude::*;
     use crate::layer1::geology::tectonic::TectonicStress;
     use crate::layer1::psychology::traits::{Trait, Traits};
     use crate::layer1::social::morale::{MoodModifier, Morale};
     use crate::layer1::entities::pop::Pop;

     pub fn tectonic_prophets_system(
         stress: Option<Res<TectonicStress>>,
         mut pops: Query<(&Traits, &mut Morale), With<Pop>>,
     ) {
         let Some(stress) = stress else { return; };

         // Trigger when stress is >= 80% of threshold
         if stress.current >= stress.threshold * 0.8 {
             for (traits, mut morale) in pops.iter_mut() {
                 if traits.has(Trait::Prophet) {
                     morale.add_modifier(MoodModifier {
                         label: "Vibrations of the Deep".to_string(),
                         value: 0.2,
                         duration: 10,
                     });
                 }
                 if traits.has(Trait::Anxious) {
                     morale.add_modifier(MoodModifier {
                         label: "Impending Doom".to_string(),
                         value: -0.2,
                         duration: 10,
                     });
                 }
             }
         }
     }

     pub fn register(schedule: &mut Schedule) {
         schedule.add_systems(tectonic_prophets_system);
     }

     #[cfg(test)]
     mod tests {
         use super::*;
         use bevy_ecs::system::RunSystemOnce;

         #[test]
         fn test_tectonic_prophets_apply_morale_effects() {
             let mut world = World::new();

             world.insert_resource(TectonicStress {
                 current: 85.0,
                 threshold: 100.0,
                 dissipation_rate: 0.1,
             });

             let mut prophet_traits = Traits::default();
             prophet_traits.add(Trait::Prophet);

             let mut anxious_traits = Traits::default();
             anxious_traits.add(Trait::Anxious);

             let prophet_pop = world.spawn((Pop, prophet_traits, Morale::default())).id();
             let anxious_pop = world.spawn((Pop, anxious_traits, Morale::default())).id();

             world.run_system_once(tectonic_prophets_system).unwrap();

             let prophet_morale = world.get::<Morale>(prophet_pop).unwrap();
             assert!(prophet_morale.modifiers.iter().any(|m| m.label == "Vibrations of the Deep"));

             let anxious_morale = world.get::<Morale>(anxious_pop).unwrap();
             assert!(anxious_morale.modifiers.iter().any(|m| m.label == "Impending Doom"));
         }

         #[test]
         fn test_tectonic_prophets_no_effect_below_threshold() {
             let mut world = World::new();

             world.insert_resource(TectonicStress {
                 current: 50.0, // Below 80%
                 threshold: 100.0,
                 dissipation_rate: 0.1,
             });

             let mut prophet_traits = Traits::default();
             prophet_traits.add(Trait::Prophet);

             let prophet_pop = world.spawn((Pop, prophet_traits, Morale::default())).id();

             world.run_system_once(tectonic_prophets_system).unwrap();

             let prophet_morale = world.get::<Morale>(prophet_pop).unwrap();
             assert!(prophet_morale.modifiers.is_empty());
         }
     }
     EOF
     ```
   - Command: `ls -l src/experimental/tectonic_prophets.rs` to verify the file was created.

2. **Register the Module (`src/experimental/mod.rs` and `src/simulation.rs`)**:
   - Command:
     ```bash
     cat << 'EOF' >> src/experimental/mod.rs
     #[cfg(feature = "nova")]
     pub mod tectonic_prophets;
     EOF
     ```
   - Command:
     ```bash
     sed -i '/crate::experimental::tavern_brawls::register(schedule);/a \
     #[cfg(feature = "nova")]\
     crate::experimental::tectonic_prophets::register(schedule);' src/simulation.rs
     ```
   - Command: `git diff src/experimental/mod.rs src/simulation.rs` to verify the code was injected correctly.

3. **Update Journal (`.jules/nova.md`)**:
   - Command:
     ```bash
     cat << 'EOF' >> .jules/nova.md

     ## [Tectonic Prophets]
     **Concept:** Added `tectonic_prophets_system` to `src/experimental/tectonic_prophets.rs`. It reads the `TectonicStress` resource, and when it nears the `MegaQuake` threshold (>80%), Pops with `Trait::Prophet` gain a large morale boost ("Vibrations of the Deep"), while Pops with `Trait::Anxious` gain a severe penalty ("Impending Doom").
     **Fate:** Submitted
     **Lesson:** Connecting geological layer mechanics directly to the personality traits of individual pops creates strong tension before disasters even strike, rewarding or punishing colony trait compositions dynamically.
     EOF
     ```
   - Command: `tail -n 10 .jules/nova.md` to verify the journal update.

4. **Verify Implementation**:
   - Command: `cargo test --features nova`
   - Command: `cargo clippy --all-targets --all-features -- -D warnings`
   - Command: `cargo fmt --all`

5. **Complete pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit PR**:
   - Commit the changes with the required specific format:
     ```bash
     git add -A
     git commit -m "🌟 Nova: Tectonic Prophets

     💡 The Spark: We have `TectonicStress` which builds towards a `MegaQuake`, and `Trait::Prophet`/`Trait::Anxious` in the psychology system.
     🚀 The Feature: When `TectonicStress` hits 80%, Prophets get a massive morale boost ('Vibrations of the Deep'), while Anxious pops get a massive penalty ('Impending Doom').
     🔮 The Potential: Connects deep crust physical simulation to the socio-psychological state, creating dynamic tension before disaster strikes.
     ⚠️ Risk: Low. Isolated in `src/experimental/tectonic_prophets.rs` behind the `nova` feature flag.

     Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>"
     ```
