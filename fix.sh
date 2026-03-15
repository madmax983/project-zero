sed -i 's/pub(crate) mod ghost_code;/pub mod ghost_code;/g' src/layer1/tech/mod.rs
sed -i 's/pub(crate) mod hypno_learning;/pub mod hypno_learning;/g' src/layer1/tech/mod.rs
sed -i 's/pub(crate) mod infinite_archive;/pub mod infinite_archive;/g' src/layer1/tech/mod.rs
sed -i 's/pub(crate) mod neural_leech;/pub mod neural_leech;/g' src/layer1/tech/mod.rs
sed -i 's/pub(crate) mod legacy_code;/pub mod legacy_code;/g' src/layer1/tech/mod.rs
sed -i 's/pub(crate) mod rhythm;/pub mod rhythm;/g' src/layer1/tech/mod.rs
sed -i 's/pub(crate) mod harmonic;/pub mod harmonic;/g' src/layer1/tech/mod.rs
sed -i 's/pub(crate) mod event_horizon_tap;/pub mod event_horizon_tap;/g' src/layer1/tech/mod.rs
sed -i 's/pub(crate) mod machine_awakening;/pub mod machine_awakening;/g' src/layer1/tech/mod.rs

for f in src/layer1/tech/ghost_code.rs src/layer1/tech/hypno_learning.rs src/layer1/tech/infinite_archive.rs src/layer1/tech/neural_leech.rs src/layer1/tech/legacy_code.rs src/layer1/tech/rhythm.rs src/layer1/tech/harmonic.rs src/layer1/tech/event_horizon_tap.rs src/layer1/tech/machine_awakening.rs; do
  sed -i 's/pub(crate) fn/pub fn/g' "$f"
  sed -i 's/pub(crate) struct/pub struct/g' "$f"
  sed -i 's/pub(crate) enum/pub enum/g' "$f"
done
