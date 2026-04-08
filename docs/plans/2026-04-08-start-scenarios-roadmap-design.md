# Start Scenarios Roadmap Design

Date: 2026-04-08
Status: Validated

## Summary

SCALE should stop treating the opening as one fixed colony plus different flavor text. The current startup path generates prehistory and then drops the player into the same physical landing shell with the same core opening cadence. That makes the world feel varied in prose but flat in play.

The first roadmap pass should introduce a scenario system that changes the actual first-hour game while keeping scope controlled. For now, all scenarios share the same landing footprint and starter shell. Variation comes from three scenario-defined payloads:

- population profile
- material profile
- opening pressure

This keeps implementation practical while still delivering meaningful replayability. The player should be able to tell which scenario they chose from behavior alone, not by re-reading the intro paragraph.

## Design Constraints

- Keep the current landing layout for the first pass.
- Ship exactly three curated scenarios first.
- Use a clear difficulty curve instead of pretending all starts are equally sharp.
- Reuse existing simulation systems wherever possible instead of inventing isolated one-off rules.
- Make the scenario framework data-driven enough that later starts can vary map footprint, but do not build that extra surface area yet.

The intended first curve is:

- `Ground Survival`: hardest
- `Social Drama`: medium
- `Layer 2 Ready`: easiest opener

That curve gives each start a job. `Ground Survival` teaches survival triage. `Social Drama` teaches that people are the real disaster. `Layer 2 Ready` teaches that the colony is a launchpad into system play, not just a dirt-side ant farm.

## Scenario Architecture

Startup should be refactored around a `StartScenario` model selected before world creation. The shared landing shell remains intact, but scenario data mutates what gets inserted into that shell and what immediate pressures start running on tick 0.

The first version should expose four scenario-facing structures:

- `StartScenarioId`: stable enum or identifier used by menu selection, setup, saves, and tests
- `ScenarioPopulationProfile`: pop count, founder or immigrant mix, trait weighting, role bias, and social baggage
- `ScenarioLoadoutProfile`: starting resources, intact or damaged core systems, tool and supply bonuses, and optional early unlock posture
- `ScenarioPressureProfile`: opening crisis flags, timed penalties, initial event injections, and chronicle intro text

This should be implemented as scenario application layered over the shared setup sequence rather than three independent setup functions. One shell, different poison.

The framework should preserve deterministic testing. Given the same seed and selected scenario, startup should produce reproducible population state, inventories, and opening pressures.

## Initial Scenario Set

### `Ground Survival`

This is the harsh start. The landing shell exists, but the colony is materially weak and immediately under strain. Food, tools, air, or environmental readiness should all feel tight enough that the player has to triage survival rather than optimize elegance.

The scenario should not rely on raw stat nerfs alone. The best version is one where the player sees concrete problems: damaged systems, thinner supplies, fewer skilled pops, hostile surface conditions, or immediate maintenance debt. The opening fantasy is not "numbers are smaller." It is "this landing was barely controlled and now you have to keep the can alive."

Success condition for the design: the player spends the first stretch solving real physical problems and feels relief when the colony stabilizes.

### `Social Drama`

This is the medium start. The colony should be materially viable enough that starvation is not the main story, but socially unstable enough that the player feels factional and generational stress early.

This scenario should lean hard on existing founder and immigrant friction, mood modifiers, factional behavior, and any available social systems instead of inventing bespoke drama code. The colony begins with a composition problem: who arrived, who belongs, who resents whom, and who thinks this place is already theirs.

The opening fantasy is that the infrastructure mostly works but the people do not. The player should feel the colony pulling against itself through behavior, productivity drag, mood penalties, public grievances, or micro-crises that arise from distrust rather than hunger.

Success condition for the design: if the player describes the start as "stable base, unstable society," it worked.

### `Layer 2 Ready`

This is the easiest early survival start and the start most explicitly aimed at broader-scale play. The colony should begin with a stronger command, logistics, and systems posture so the player can move into planetary or orbital interaction sooner.

This does not mean it is risk-free. It means the opening pressure should come from responsibility rather than scarcity. Trade obligations, sensor dependence, debt exposure, launch preparation, or system visibility constraints are better fits than food panic.

This scenario should be compatible with system-view and command-center progression rather than bypassing it. The player starts closer to Layer 2 competence, but still inside the same game rules.

Success condition for the design: the player feels invited outward instead of pinned inward.

## Roadmap

### Phase 1: Scenario Framework

Extract the current hardcoded start sequence into a scenario-driven startup pipeline. Keep one shared landing layout. Add scenario selection to the main menu flow. Ensure there is a default scenario that reproduces current startup closely enough to avoid accidental regressions.

Deliverables:

- startup accepts a selected `StartScenarioId`
- scenario application modifies pops, resources, state, and intro events
- scenario selection is visible in the menu flow
- save and headless setup paths can specify a scenario

### Phase 2: Curated Scenario Content

Implement the three starts as real packages rather than copy-pasted setup branches.

Deliverables:

- `Ground Survival` tunes environmental and supply pressure
- `Social Drama` tunes composition and social volatility
- `Layer 2 Ready` tunes command and outward-facing readiness
- each scenario has distinct chronicle intro text and setup messaging

### Phase 3: Balancing And Verification

Prove that the scenarios are mechanically distinct and that the intended difficulty curve is real rather than imagined.

Deliverables:

- deterministic startup tests for all scenarios
- headless smoke tests covering the first few hundred ticks
- assertions around founder mix, resources, damaged systems, and opening events
- balancing pass that confirms the difficulty order: hard -> medium -> easy

## Spec Slices

The roadmap should be implemented as small, atomic specs:

1. `Scenario Framework`
2. `Scenario Selection`
3. `Ground Survival`
4. `Social Drama`
5. `Layer 2 Ready`
6. `Scenario Verification`

Each spec should remain narrow enough to complete in one focused implementation session. The framework spec is plumbing. The three scenario specs are content and tuning. The verification spec is where we catch the usual "different tooltip, same coffin" failure mode.

## Testing Strategy

The minimum verification bar for the feature set:

- scenario selection changes startup state in observable ways
- all three starts share the same landing footprint in this first pass
- all three starts produce different population, loadout, and pressure profiles
- the default scenario remains backward-compatible enough to avoid collateral breakage
- scenario-specific intro text matches actual mechanics

Recommended test coverage:

- unit tests for scenario application helpers
- setup tests proving scenario-specific world state
- headless integration tests for early-tick survival and stability behavior
- balance assertions that compare baseline stockpiles, damages, or stressors across scenarios

## Future Extensions

This roadmap intentionally leaves several obvious expansions for later:

- unique landing footprints per scenario
- light procedural modifiers inside curated scenarios
- scenario-specific map generation constraints
- additional starts beyond the first three
- unlockable or faction-specific scenario packs

That future work becomes much easier once the first pass proves the core idea: starting scenario must alter the opening game state, not just the opening paragraph.
