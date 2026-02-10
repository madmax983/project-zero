# Ludwig's Journal 🎲

## [Colony Balance]
**Friction:** Colonists were starving in 4 seconds (real-time) due to aggressive hunger decay (0.02/tick). This created a panic loop where players couldn't react fast enough.
**Flow:** Slowed hunger decay by 20x (0.001/tick) and balanced food production (0.005/tick) to create a sustainable "Day Cycle" rhythm.

## [Feedback System]
**Friction:** Building actions felt "hollow" because there was no confirmation or error message. Players didn't know why they couldn't place a building.
**Flow:** Implemented a `MessageLog` system that provides colored feedback ("Construction started", "Cannot build on Water"). This closes the feedback loop immediately.

## [Consumption Flow]
**Friction:** Pops were snacking constantly (eating at 70% full), breaking their work rhythm and feeling robotic.
**Flow:** Lowered hunger threshold to 40% to create distinct "Work" and "Eat" phases. Added "Thought" bubbles when eating to provide immediate positive feedback ("Tastes like victory").

## [Resource Loop]
**Friction:** Mining instantly credited resources to the global bank, making the physical act of hauling feel redundant and exploitative (double resource gain bug).
**Flow:** Removed instant resource credit. Resources now spawn as physical items that MUST be hauled to be counted. Added immediate log feedback ("Needs Hauling") to teach the player this new requirement.

## [Combat Feedback]
**Friction:** Combat felt like "math in a vacuum" - damage was applied silently, and players had no idea if they were effective or just swinging at air.
**Flow:** Added `CombatOutcome` system with visual feedback in `MessageLog`.
- **Juice:** Attacks now have variance (0.9x-1.1x), Crits (1.5x, 5% chance), and Misses (10% chance).
- **Feedback:** Messages are color-coded: Red for Crits (Excitement), White for Hits (Clarity), DarkGray for Misses (Frustration/Clarity).
