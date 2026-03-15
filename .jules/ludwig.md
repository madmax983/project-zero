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
