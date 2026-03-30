## [Direct Link Wall Sliding]
**Friction:** Diagonal movement against a blocked corner instantly stopped all movement, feeling rigid, clunky, and unresponsive. Players had to perfectly align their inputs to move past corners, leading to frustration and lost momentum.
**Flow:** Implemented wall sliding to preserve movement intent. If a diagonal move is blocked, the system now evaluates single-axis movement (X-only or Y-only). If one axis is free, the character smoothly slides along the wall, maintaining flow and "Juice" (particle effects) without feeling penalized for imprecise input.

## [Combat Cooldown]
**Friction:** Characters were only able to attack once because their `CombatState::cooldown` was set but never decremented. After the first attack, they stood still and never attacked again, feeling broken and unresponsive.
**Flow:** Implemented `combat_cooldown_system` to decrement `CombatState::cooldown` every tick, allowing characters to attack repeatedly and making combat functional and responsive again.

## [Critical Hits]
**Friction:** Combat felt unrewarding and predictable due to low critical hit chance (5%) and a standard multiplier (2x), making encounters feel like a slow grind against spongy enemies.
**Flow:** Increased critical hit chance to 15% and multiplier to 3x. This injects more excitement and "Juice" into combat, allowing for sudden, impactful bursts of damage that keep the player engaged and make hits feel significantly more powerful.

## [Direct Link Diagonal Buffering]
**Friction:** Players trying to move diagonally right as the movement cooldown ends often only moved in one direction. This happened because the input buffer only stored a single `KeyCode`, discarding the other key press and making quick diagonal movements feel dropped or rigid.
**Flow:** Changed the input buffer from a single `Option<KeyCode>` to `buffered_dx` and `buffered_dy` to store movement intent across both axes. This allows diagonal inputs to be perfectly buffered and executed seamlessly, maintaining "Flow" and responsiveness.

## [Notification Slide-In]
**Friction:** Notifications felt stiff and abrupt when appearing on screen because they used a short, linear padding reduction (teleporting 1 character per tick). It lacked the polish expected of modern UI.
**Flow:** Replaced the linear interpolation with a Cubic Ease-Out curve and increased the animation duration. Now notifications "slide" in quickly and settle smoothly into place, adding visual "Juice" and Delight without distracting the player.

## [Direct Link Input Grace Period]
**Friction:** Input buffering in Direct Link mode was unbounded. If a player pressed a key early in the cooldown, the character would execute the move much later, leading to unexpected, sluggish, and "stuck" movements that felt completely disconnected from the player's intent.
**Flow:** Implemented a Grace Period (0.2s) for input buffering. Now, inputs are only buffered if pressed slightly before the action is ready. Stale inputs are safely discarded, ensuring movement feels tight, predictable, and responsive to the player's immediate commands.

## [Particle Gravity]
**Friction:** Particles floated weightlessly and slid along axes linearly, which felt "floaty" and lacked physical weight or impact. It did not communicate the grittiness of the world.
**Flow:** Added a `GRAVITY` constant (0.05) to `particle_physics_system` that pulls particles downwards (increasing `dy`) every tick. This simple tweak gives particles a satisfying parabolic arc, adding immediate visual "Juice" and grounding the effects in the physical world without modifying the engine's core physics loop.

## [Global Hit Stop]
**Friction:** Critical hits and heavy attacks applied a local hit stop to the combatants, but the rest of the game kept moving. The impact felt isolated and didn't convey the immense weight of the blow to the player.
**Flow:** Triggered the `GlobalHitStop` resource on critical hits and heavy damage. This briefly freezes the entire game logic (while particles and screen shake continue), simulating the dramatic "Hit Stop" pause found in fighting games and adding massive "Juice" to heavy combat impacts.

## [Combat Feel & Crits]
**Friction:** Crits were too rare (15%) and when they did hit, the hit stop (12 ticks) felt like it didn't fully sell the "massive impact" against heavy targets.
**Flow:** Increased base CRIT_CHANCE to 20% to make combat feel more rewarding and juicy, slightly reduced CRIT_MULTIPLIER to 2.5 to maintain balance. Increased HIT_STOP_CRIT to 15 ticks to heavily emphasize those big, crunchy blows.

## [Movement Flow (Coyote Speed)]
**Friction:** The movement system felt slightly clunky when pops were encumbered, leading to a "just missing the bus" feeling where they had to wait an extra tick to move because they were a fraction of a speed point short.
**Flow:** Increased COYOTE_THRESHOLD from 0.25 to 0.35. This provides a slightly larger grace period for movement cost evaluation, allowing for smoother, more continuous movement even when dealing with wind or terrain penalties.

## [Camera Smoothing (Lerp)]
**Friction:** Camera panning felt slightly jarring and rigid when moving large distances or switching targets. The hardcoded linear interpolation factor (0.2) snapped too quickly, causing mild visual discomfort.
**Flow:** Extracted the lerp multiplier into a `CAMERA_LERP_FACTOR` constant and decreased it from `0.2` to `0.15`. This slightly elongates the interpolation curve, providing a more elegant, cinematic "Ease-Out" effect that feels significantly more polished and "Juicy".
