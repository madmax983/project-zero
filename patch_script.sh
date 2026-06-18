#!/bin/bash
set -e

patch src/layer1/psychology/needs.rs << 'PATCH'
@@ -271,6 +271,15 @@ pub fn decay_needs_system(
             // Hygiene is decayed separately in hygiene.rs
         });
 }
+
+/// System to despawn pops that have reached 0.0 hunger.
+pub fn kill_starving_pops_system(mut commands: Commands, query: Query<(Entity, &Needs), With<crate::layer1::entities::pop::Pop>>) {
+    for (entity, needs) in query.iter() {
+        if needs.hunger <= 0.0 {
+            commands.entity(entity).despawn();
+        }
+    }
+}

 #[cfg(test)]
 mod tests {
@@ -621,4 +630,90 @@ mod tests {
         // Rest should not decay at all due to Insomnia Drive
         assert_eq!(needs.rest, 1.0);
     }
+
+    #[test]
+    fn test_kill_starving_pops_system() {
+        let mut world = setup();
+
+        // Spawn healthy pop
+        world.spawn((Pop, Needs { hunger: 0.5, rest: 0.5, leisure: 0.5, hygiene: 0.5 }));
+
+        // Spawn starving pop
+        world.spawn((Pop, Needs { hunger: 0.0, rest: 0.5, leisure: 0.5, hygiene: 0.5 }));
+
+        world.run_system_once(kill_starving_pops_system).unwrap();
+
+        let count = world.query::<&Pop>().iter(&world).count();
+        assert_eq!(count, 1, "Only healthy pop should survive");
+    }
+
+    #[test]
+    fn test_kill_only_when_hunger_zero() {
+        let mut world = setup();
+
+        // Pop with very low hunger but not zero
+        world.spawn((Pop, Needs { hunger: 0.01, rest: 0.0, leisure: 0.0, hygiene: 0.0 }));
+
+        world.run_system_once(kill_starving_pops_system).unwrap();
+
+        let count = world.query::<&Pop>().iter(&world).count();
+        assert_eq!(count, 1, "Pop with 0.01 hunger should survive");
+    }
+
+    #[test]
+    fn test_starve_from_full() {
+        let mut world = setup();
+        world.spawn((Pop, Needs::default()));
+
+        // Run until pop dies
+        let mut ticks = 0;
+        while world.query::<&Pop>().iter(&world).count() > 0 && ticks < 1000 {
+            world.run_system_once(decay_needs_system).unwrap();
+            world.run_system_once(kill_starving_pops_system).unwrap();
+            ticks += 1;
+        }
+
+        assert!(ticks < 850, "Pop should die within ~850 ticks from full (0.8)");
+        assert!(ticks > 750, "Pop should survive at least 750 ticks");
+    }
+
+    // Notice: `pop_display` logic tests as specified by 005.
+    // The actual function `get_pop_display` is located in `src/ui/map.rs`.
+    // The spec requires testing the display logic, which outputs "☺", "☻", "☹".
+    #[test]
+    fn test_pop_display_basic_healthy() {
+        let needs = Needs { hunger: 0.8, rest: 0.8, leisure: 0.8, hygiene: 0.8 };
+        let (ch, color) = crate::ui::map::get_pop_display(&needs);
+
+        assert_eq!(ch, "☺");
+        assert_eq!(color, ratatui::style::Color::Yellow);
+    }
+
+    #[test]
+    fn test_pop_display_basic_warning() {
+        let needs = Needs { hunger: 0.5, rest: 0.8, leisure: 0.8, hygiene: 0.8 };
+        let (ch, color) = crate::ui::map::get_pop_display(&needs);
+
+        assert_eq!(ch, "☻");
+        assert_eq!(color, ratatui::style::Color::Rgb(255, 165, 0)); // Orange
+    }
+
+    #[test]
+    fn test_pop_display_basic_critical() {
+        let needs = Needs { hunger: 0.2, rest: 0.8, leisure: 0.8, hygiene: 0.8 };
+        let (ch, color) = crate::ui::map::get_pop_display(&needs);
+
+        assert_eq!(ch, "☹");
+        assert_eq!(color, ratatui::style::Color::Red);
+    }
+
+    #[test]
+    fn test_pop_display_basic_uses_worst_need() {
+        // Even if hunger is high, low rest should trigger warning
+        let needs = Needs { hunger: 0.9, rest: 0.4, leisure: 0.8, hygiene: 0.8 };
+        let (ch, color) = crate::ui::map::get_pop_display(&needs);
+
+        assert_eq!(ch, "☻"); // Warning state
+        assert_eq!(color, ratatui::style::Color::Rgb(255, 165, 0));
+    }
 }
PATCH

patch src/layer1/systems/consumption.rs << 'PATCH'
@@ -124,6 +124,7 @@ pub fn register(schedule: &mut Schedule) {
             pressure_damage_system.after(decay_needs_system),
             consume_artifacts_during_famine_system.after(decay_needs_system),
             crate::layer1::needs::starvation_damage_system.after(decay_needs_system),
+            crate::layer1::needs::kill_starving_pops_system.after(decay_needs_system),
             crate::layer1::biology::rust_lung::rust_lung_degradation_system
                 .after(decay_needs_system),
             crate::layer1::atmosphere::apply_smog_damage_system.after(decay_needs_system),
PATCH

patch src/ui/map.rs << 'PATCH'
@@ -996,15 +996,11 @@
 /// ```
 #[must_use]
 pub fn get_pop_display(needs: &Needs) -> (&'static str, Color) {
-    if needs.hunger > HEALTHY_THRESHOLD {
-        // Well-fed: check other needs for mood
-        if needs.worst() > WARNING_THRESHOLD {
-            ("☺", Color::Yellow)
-        } else {
-            ("☻", Color::Rgb(255, 165, 0)) // fed but tired/bored
-        }
-    } else if needs.hunger > WARNING_THRESHOLD {
-        ("☻", Color::Rgb(255, 165, 0)) // getting hungry
+    let worst = needs.worst();
+    if worst > HEALTHY_THRESHOLD {
+        ("☺", Color::Yellow)
+    } else if worst > WARNING_THRESHOLD {
+        ("☻", Color::Rgb(255, 165, 0)) // fed but tired/bored
     } else {
         ("☹", Color::Red) // starving
     }
PATCH
