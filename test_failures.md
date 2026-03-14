I need to fix the following tests:

1. `layer1::execution::tests::combat_tests::test_combat_execution_system_attacks_in_range`:
`execute_attack` uses `rand::thread_rng().gen_bool(CRIT_CHANCE)`. In the test we check `(dmg - 10.0).abs() < f32::EPSILON || (dmg - 20.0).abs() < f32::EPSILON`, but `CRIT_MULTIPLIER` is 3.0, and base damage is 10.0, so crit damage is 30.0. The test is asserting for 20.0.

2. `layer1::geodetic_tests::tests::test_living_stone_heat_attraction`:
The stone moves from `(15, 15)` to heat at `(20, 20)`. The test currently fails because `nx > 15 || ny > 15` is not satisfied. Wait, actually, let's look at `update_living_stone_system`. The score calculation:
```rust
let attraction_score = -min_dist * 2.0; // Weight attraction heavily
```
If there are no other stones, `stone_positions` is empty, so `min_dist` stays 1000.0. `attraction_score` becomes -2000.0. This happens for all neighbors AND the current position.
The current position score:
```rust
let mut min_dist = 1000.0;
for (other_e, other_pos) in &stone_positions {
    ...
}
let current_score = -min_dist * 2.0; // heat diff is 0
if current_score >= best_score {
    best_move = None;
}
```
If there are no other stones, both `best_score` and `current_score` are -2000.0, but `current_score >= best_score` causes `best_move = None`, so it doesn't move.

3. `layer1::memory::tests::test_calculate_effective_morale` and related tests:
`needs.morale()` returns `(0.5 + 0.5 + 0.5 + 0.8) / 4.0 = 2.3 / 4 = 0.575` instead of `0.5`. Tests expect `needs.morale()` to be `0.5`, because they assume 3 needs or `hygiene` was added later but not properly adjusted in the test expectation. Wait, `needs` has 4 fields. `0.5 + 0.5 + 0.5 + 0.8 = 2.3`. Average is 0.575.
If we set `hygiene = 0.5` in tests, it will be `0.5`.

4. `layer1::nature::weather_tests::tests::test_inversion_halts_diffusion`:
Test calls `simulate_diffusion_system` directly, but the grid's `diffusion_rate` is not updated based on weather because `update_weather_diffusion_system` is not scheduled before it in the test. The default `diffusion_rate` of `AtmosphereGrid` is `1.0`. Wait, if it's 1.0, why did it fall below 99.0?
Because `simulate_diffusion_system` is called, and `config.rate` is 0.0, but maybe there's a problem with edge vacuum? "Vacuum edge sucks pollution away". The test is at 10,10 in a 20x20 grid, so it's not at the edge. But let's check diffusion again.
Oh! The test sets `horizontal_rate` to 0.0 (via `DiffusionConfig`), but the diffusion logic has `self.scratch[idx] *= self.diffusion_rate`. Wait, if `AtmosphereGrid::new()` sets `diffusion_rate` to 0.95 (which is 1.0 - 0.05). Let's check `AtmosphereGrid::new()`.

5. `layer1::politics::tests::test_voting_logic`:
```rust
            if let Some(member) = faction_member {
                if let Some(faction_id) = member.faction_id {
                    if let Some(&candidate_idx) = faction_candidate_map.get(&faction_id) {
                        manager.candidates[candidate_idx].votes += 1;
                    }
                }
            }
```
Wait, the Voter 1 is spawned with `MinersGuild` faction. The candidate A has `MinersGuild`. So Candidate A should get 1 vote. Wait, Candidate A is also a `Pop` with `FactionMember`, so does Candidate A vote for themselves? Yes! Candidate A is a Pop, so Candidate A + Voter 1 = 2 votes for Candidate A! Test expects 1 vote.

6. `layer1::rumor::tests::test_rumor_generation_negative_morale`:
Morale is `0.1 + 0.1 + 0.1 + 0.8 = 1.1 / 4 = 0.275`. Test checks `needs.morale() < 0.2`. So it fails to generate a rumor because `0.275` is not `< 0.2`.

7. `layer1::stress::tests::test_stress_accumulation`:
Morale is `0.05 + 0.05 + 0.05 + 0.8 = 0.95 / 4 = 0.2375`. Breakdown system checks `needs.morale() < LOW_MORALE_THRESHOLD` (which is `0.15`). It doesn't accumulate stress because `0.2375 > 0.15`.

8. `layer1::void_stare::tests::test_void_exposure_gain`:
```rust
        // 4. Calculate Delta
        let effective_void = (void_intensity - occlusion * 0.5).max(0.0);

        let gain = effective_void * 0.1;
        let loss = visible_life * 0.05;

        // Apply
        let delta = (gain - loss) * exposure.susceptibility;
        exposure.current = (exposure.current + delta).clamp(0.0, 100.0);
```
In the test, `pos.x = 5, pos.y = 5`. `void_intensity = 1.0`. `visible_life`: 8 neighbors. TerrainGrid is all `Grass`. So 8 neighbors have `Grass`. `visible_life += 0.5` per neighbor = `4.0`. `occlusion = 0`.
`gain = 1.0 * 0.1 = 0.1`.
`loss = 4.0 * 0.05 = 0.2`.
`delta = 0.1 - 0.2 = -0.1`.
`exposure.current` stays 0.0. Test expects it to be > 0.0. We should change the terrain around the pop in the test to be empty or void, so `visible_life` is 0.
